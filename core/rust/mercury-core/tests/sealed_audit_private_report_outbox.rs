use mercury_core::{
    PrototypeSealedAuditPrivateReportOutbox, SealedAuditPrivateReportOutboxDecision,
    SealedAuditPrivateReportOutboxReason, SealedAuditPrivateReportOutboxWrite,
    SealedAuditPrivateReportTransportDecision, SealedAuditPrivateReportTransportReason,
    put_sealed_audit_private_report_outbox_record,
};

const REPORT_ID: [u8; 32] = [0xA1; 32];
const NEXT_REPORT_ID: [u8; 32] = [0xA2; 32];
const DIGEST: [u8; 32] = [0xB1; 32];
const SHORT_DIGEST: [u8; 16] = [0xB2; 16];

#[test]
fn private_report_outbox_persists_only_accepted_digest_records() {
    let mut store = PrototypeSealedAuditPrivateReportOutbox::default();

    let decision = put_sealed_audit_private_report_outbox_record(&mut store, valid_write())
        .expect("prototype private report outbox cannot fail");

    assert!(decision.accepted);
    assert_eq!(
        decision.reason,
        SealedAuditPrivateReportOutboxReason::Accepted
    );
    assert_eq!(decision.record_count, 1);
    assert!(decision.persisted_record);
    assert!(decision.can_enqueue_report);
    assert!(decision.can_submit_now);
    assert!(decision.can_retry_safely);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store
        .get_by_id(&REPORT_ID)
        .expect("accepted report should be persisted");
    assert_eq!(record.report_id, REPORT_ID);
    assert_eq!(record.report_payload_digest.len(), 32);
    assert_eq!(record.report_sequence, 1);
    assert!(record.can_submit_now);
    assert!(record.can_retry_safely);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn private_report_outbox_rejects_transport_failures_and_bad_shapes_without_mutation() {
    let mut store = PrototypeSealedAuditPrivateReportOutbox::default();

    let transport_rejected = SealedAuditPrivateReportOutboxWrite {
        transport_decision: rejected_transport_decision(false),
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_private_report_outbox_record(&mut store, transport_rejected)
            .expect("prototype private report outbox cannot fail"),
        SealedAuditPrivateReportOutboxReason::PrivateReportTransportRequired,
    );
    assert!(store.is_empty());

    let bad_digest = SealedAuditPrivateReportOutboxWrite {
        report_payload_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_private_report_outbox_record(&mut store, bad_digest)
            .expect("prototype private report outbox cannot fail"),
        SealedAuditPrivateReportOutboxReason::BadRecordShape,
    );
    assert!(store.is_empty());
}

#[test]
fn private_report_outbox_rejects_replay_rate_route_stale_and_plaintext_state() {
    let replay = SealedAuditPrivateReportOutboxWrite {
        replay_window_bound: false,
        ..valid_write()
    };
    let replay_decision = evaluate(replay);
    assert_rejected(
        replay_decision,
        SealedAuditPrivateReportOutboxReason::ReplayGuardRequired,
    );
    assert!(replay_decision.requires_replay_guard);

    let rate_limit = SealedAuditPrivateReportOutboxWrite {
        privacy_pass_token_present: false,
        ..valid_write()
    };
    let rate_limit_decision = evaluate(rate_limit);
    assert_rejected(
        rate_limit_decision,
        SealedAuditPrivateReportOutboxReason::RateLimitTokenRequired,
    );
    assert!(rate_limit_decision.requires_rate_limit_token);

    let route = SealedAuditPrivateReportOutboxWrite {
        relay_gateway_separated: false,
        ..valid_write()
    };
    let route_decision = evaluate(route);
    assert_rejected(
        route_decision,
        SealedAuditPrivateReportOutboxReason::RoutePrivacyRequired,
    );
    assert!(route_decision.requires_route_privacy);

    let stale = SealedAuditPrivateReportOutboxWrite {
        created_at_s: 1_769_994_000,
        ..valid_write()
    };
    let stale_decision = evaluate(stale);
    assert_rejected(
        stale_decision,
        SealedAuditPrivateReportOutboxReason::StalePolicySnapshot,
    );
    assert!(stale_decision.requires_policy_refresh);

    let plaintext = SealedAuditPrivateReportOutboxWrite {
        plaintext_selector_count: 1,
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let plaintext_decision = evaluate(plaintext);
    assert_rejected(
        plaintext_decision,
        SealedAuditPrivateReportOutboxReason::PlaintextMetadataForbidden,
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn private_report_outbox_store_rejects_duplicate_and_rollback_sequences() {
    let mut store = PrototypeSealedAuditPrivateReportOutbox::default();

    let first = put_sealed_audit_private_report_outbox_record(&mut store, valid_write())
        .expect("prototype private report outbox cannot fail");
    assert!(first.accepted);

    let duplicate = put_sealed_audit_private_report_outbox_record(&mut store, valid_write())
        .expect("prototype private report outbox cannot fail");
    assert_rejected(
        duplicate,
        SealedAuditPrivateReportOutboxReason::ReplayGuardRequired,
    );
    assert_eq!(store.len(), 1);

    let rollback = SealedAuditPrivateReportOutboxWrite {
        report_id: &NEXT_REPORT_ID,
        report_sequence: 1,
        previous_report_sequence: 1,
        previous_report_id: &REPORT_ID,
        ..valid_write()
    };
    let rollback_decision = put_sealed_audit_private_report_outbox_record(&mut store, rollback)
        .expect("prototype private report outbox cannot fail");
    assert_rejected(
        rollback_decision,
        SealedAuditPrivateReportOutboxReason::ReplayGuardRequired,
    );
    assert_eq!(store.len(), 1);

    let next = SealedAuditPrivateReportOutboxWrite {
        report_id: &NEXT_REPORT_ID,
        report_sequence: 2,
        previous_report_sequence: 1,
        previous_report_id: &REPORT_ID,
        ..valid_write()
    };
    let next_decision = put_sealed_audit_private_report_outbox_record(&mut store, next)
        .expect("prototype private report outbox cannot fail");
    assert!(next_decision.accepted);
    assert_eq!(store.len(), 2);
    assert_eq!(store.latest().expect("latest exists").report_sequence, 2);
}

#[test]
fn private_report_outbox_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (
            SealedAuditPrivateReportOutboxReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            SealedAuditPrivateReportOutboxReason::PrivateReportTransportRequired,
            1,
            "PRIVATE_REPORT_TRANSPORT_REQUIRED",
        ),
        (
            SealedAuditPrivateReportOutboxReason::ReplayGuardRequired,
            2,
            "REPLAY_GUARD_REQUIRED",
        ),
        (
            SealedAuditPrivateReportOutboxReason::RateLimitTokenRequired,
            3,
            "RATE_LIMIT_TOKEN_REQUIRED",
        ),
        (
            SealedAuditPrivateReportOutboxReason::StalePolicySnapshot,
            4,
            "STALE_POLICY_SNAPSHOT",
        ),
        (
            SealedAuditPrivateReportOutboxReason::RoutePrivacyRequired,
            5,
            "ROUTE_PRIVACY_REQUIRED",
        ),
        (
            SealedAuditPrivateReportOutboxReason::PlaintextMetadataForbidden,
            6,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditPrivateReportOutboxReason::BadRecordShape,
            7,
            "BAD_RECORD_SHAPE",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn evaluate(
    write: SealedAuditPrivateReportOutboxWrite<'_>,
) -> SealedAuditPrivateReportOutboxDecision {
    mercury_core::evaluate_sealed_audit_private_report_outbox(write)
}

fn valid_write() -> SealedAuditPrivateReportOutboxWrite<'static> {
    SealedAuditPrivateReportOutboxWrite {
        transport_decision: accepted_transport_decision(),
        report_format_version: 1,
        report_id: &REPORT_ID,
        previous_report_id: &[],
        incident_id: &DIGEST,
        report_payload_digest: &DIGEST,
        report_schema_digest: &DIGEST,
        ohttp_gateway_key_digest: &DIGEST,
        ohttp_relay_policy_digest: &DIGEST,
        privacy_pass_token_digest: &DIGEST,
        rate_limit_bucket_digest: &DIGEST,
        replay_window_digest: &DIGEST,
        retry_backoff_digest: &DIGEST,
        request_transcript_digest: &DIGEST,
        response_transcript_digest: &DIGEST,
        audit_checkpoint_digest: &DIGEST,
        report_sequence: 1,
        previous_report_sequence: 0,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        created_at_s: 1_769_991_000,
        expires_at_s: 1_769_994_000,
        next_retry_after_s: 1_769_991_060,
        send_attempt_count: 0,
        max_send_attempts: 3,
        report_window_s: 3600,
        max_reports_per_window: 3,
        ohttp_request_encapsulated: true,
        gateway_response_encapsulated: true,
        gateway_response_authenticated: true,
        relay_gateway_separated: true,
        no_cookie_or_auth_state: true,
        private_route_selected: true,
        privacy_pass_token_present: true,
        privacy_pass_token_bound: true,
        privacy_pass_token_spent_once: true,
        anonymous_rate_limit_enforced: true,
        replay_window_bound: true,
        duplicate_report_rejected: true,
        retry_backoff_persisted: true,
        report_payload_encrypted: true,
        outbox_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

const fn accepted_transport_decision() -> SealedAuditPrivateReportTransportDecision {
    SealedAuditPrivateReportTransportDecision {
        accepted: true,
        reason: SealedAuditPrivateReportTransportReason::Accepted,
        can_submit_private_report: true,
        can_retry_safely: true,
        requires_private_transport: false,
        requires_replay_guard: false,
        requires_rate_limit_token: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
    }
}

const fn rejected_transport_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditPrivateReportTransportDecision {
    SealedAuditPrivateReportTransportDecision {
        accepted: false,
        reason: SealedAuditPrivateReportTransportReason::PlaintextMetadataForbidden,
        can_submit_private_report: false,
        can_retry_safely: false,
        requires_private_transport: true,
        requires_replay_guard: false,
        requires_rate_limit_token: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
    }
}

fn assert_rejected(
    decision: SealedAuditPrivateReportOutboxDecision,
    reason: SealedAuditPrivateReportOutboxReason,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.persisted_record);
    assert!(!decision.can_enqueue_report);
    assert!(!decision.can_submit_now);
    assert!(!decision.can_retry_safely);
    assert!(decision.keeps_digest_only);
}
