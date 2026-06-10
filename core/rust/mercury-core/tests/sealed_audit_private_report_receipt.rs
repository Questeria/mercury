use mercury_core::{
    PrototypeSealedAuditPrivateReportReceiptStore, SealedAuditPrivateReportOutboxDecision,
    SealedAuditPrivateReportOutboxReason, SealedAuditPrivateReportReceiptDecision,
    SealedAuditPrivateReportReceiptReason, SealedAuditPrivateReportReceiptWrite,
    put_sealed_audit_private_report_receipt_record,
};

const RECEIPT_ID: [u8; 32] = [0xA3; 32];
const NEXT_RECEIPT_ID: [u8; 32] = [0xA4; 32];
const REPORT_ID: [u8; 32] = [0xA1; 32];
const NEXT_REPORT_ID: [u8; 32] = [0xA2; 32];
const DIGEST: [u8; 32] = [0xB1; 32];
const SHORT_DIGEST: [u8; 16] = [0xB2; 16];

#[test]
fn private_report_receipt_persists_only_accepted_digest_records() {
    let mut store = PrototypeSealedAuditPrivateReportReceiptStore::default();

    let decision = put_sealed_audit_private_report_receipt_record(&mut store, valid_write())
        .expect("prototype private report receipt store cannot fail");

    assert!(decision.accepted);
    assert_eq!(
        decision.reason,
        SealedAuditPrivateReportReceiptReason::Accepted
    );
    assert_eq!(decision.record_count, 1);
    assert!(decision.persisted_record);
    assert!(decision.can_mark_delivered);
    assert!(decision.can_stop_retrying);
    assert!(decision.can_show_delivery_status);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store
        .get_by_id(&RECEIPT_ID)
        .expect("accepted receipt should be persisted");
    assert_eq!(record.receipt_id, RECEIPT_ID);
    assert_eq!(record.report_id, REPORT_ID);
    assert_eq!(record.gateway_receipt_digest.len(), 32);
    assert_eq!(record.receipt_sequence, 1);
    assert!(record.can_mark_delivered);
    assert!(record.can_stop_retrying);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn private_report_receipt_rejects_outbox_failures_and_bad_shapes_without_mutation() {
    let mut store = PrototypeSealedAuditPrivateReportReceiptStore::default();

    let outbox_rejected = SealedAuditPrivateReportReceiptWrite {
        outbox_decision: rejected_outbox_decision(false),
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_private_report_receipt_record(&mut store, outbox_rejected)
            .expect("prototype private report receipt store cannot fail"),
        SealedAuditPrivateReportReceiptReason::PrivateReportOutboxRequired,
    );
    assert!(store.is_empty());

    let bad_digest = SealedAuditPrivateReportReceiptWrite {
        gateway_receipt_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_private_report_receipt_record(&mut store, bad_digest)
            .expect("prototype private report receipt store cannot fail"),
        SealedAuditPrivateReportReceiptReason::BadRecordShape,
    );
    assert!(store.is_empty());
}

#[test]
fn private_report_receipt_rejects_receipt_transparency_monitor_replay_and_plaintext_state() {
    let missing_receipt = SealedAuditPrivateReportReceiptWrite {
        gateway_receipt_signature_verified: false,
        ..valid_write()
    };
    let missing_receipt_decision = evaluate(missing_receipt);
    assert_rejected(
        missing_receipt_decision,
        SealedAuditPrivateReportReceiptReason::ReceiptRequired,
    );
    assert!(missing_receipt_decision.requires_receipt);

    let transparency = SealedAuditPrivateReportReceiptWrite {
        gateway_key_consistency_verified: false,
        ..valid_write()
    };
    let transparency_decision = evaluate(transparency);
    assert_rejected(
        transparency_decision,
        SealedAuditPrivateReportReceiptReason::GatewayTransparencyRequired,
    );
    assert!(transparency_decision.requires_gateway_transparency);

    let replay = SealedAuditPrivateReportReceiptWrite {
        completion_state_monotonic: false,
        ..valid_write()
    };
    let replay_decision = evaluate(replay);
    assert_rejected(
        replay_decision,
        SealedAuditPrivateReportReceiptReason::DeliveryReplayRejected,
    );
    assert!(replay_decision.requires_delivery_replay_guard);

    let monitor = SealedAuditPrivateReportReceiptWrite {
        monitor_submission_proof_verified: false,
        ..valid_write()
    };
    let monitor_decision = evaluate(monitor);
    assert_rejected(
        monitor_decision,
        SealedAuditPrivateReportReceiptReason::MonitorProofRequired,
    );
    assert!(monitor_decision.requires_monitor_proof);

    let plaintext = SealedAuditPrivateReportReceiptWrite {
        plaintext_selector_count: 1,
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let plaintext_decision = evaluate(plaintext);
    assert_rejected(
        plaintext_decision,
        SealedAuditPrivateReportReceiptReason::PlaintextMetadataForbidden,
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn private_report_receipt_store_rejects_duplicate_and_rollback_sequences() {
    let mut store = PrototypeSealedAuditPrivateReportReceiptStore::default();

    let first = put_sealed_audit_private_report_receipt_record(&mut store, valid_write())
        .expect("prototype private report receipt store cannot fail");
    assert!(first.accepted);

    let duplicate = put_sealed_audit_private_report_receipt_record(&mut store, valid_write())
        .expect("prototype private report receipt store cannot fail");
    assert_rejected(
        duplicate,
        SealedAuditPrivateReportReceiptReason::DeliveryReplayRejected,
    );
    assert_eq!(store.len(), 1);

    let rollback = SealedAuditPrivateReportReceiptWrite {
        receipt_id: &NEXT_RECEIPT_ID,
        report_id: &NEXT_REPORT_ID,
        receipt_sequence: 1,
        previous_receipt_sequence: 1,
        previous_receipt_id: &RECEIPT_ID,
        report_sequence: 2,
        outbox_decision: accepted_outbox_decision_for_sequence(2),
        ..valid_write()
    };
    let rollback_decision = put_sealed_audit_private_report_receipt_record(&mut store, rollback)
        .expect("prototype private report receipt store cannot fail");
    assert_rejected(
        rollback_decision,
        SealedAuditPrivateReportReceiptReason::DeliveryReplayRejected,
    );
    assert_eq!(store.len(), 1);

    let next = SealedAuditPrivateReportReceiptWrite {
        receipt_id: &NEXT_RECEIPT_ID,
        report_id: &NEXT_REPORT_ID,
        receipt_sequence: 2,
        previous_receipt_sequence: 1,
        previous_receipt_id: &RECEIPT_ID,
        report_sequence: 2,
        outbox_decision: accepted_outbox_decision_for_sequence(2),
        ..valid_write()
    };
    let next_decision = put_sealed_audit_private_report_receipt_record(&mut store, next)
        .expect("prototype private report receipt store cannot fail");
    assert!(next_decision.accepted);
    assert_eq!(store.len(), 2);
    assert_eq!(store.latest().expect("latest exists").receipt_sequence, 2);
}

#[test]
fn private_report_receipt_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (
            SealedAuditPrivateReportReceiptReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            SealedAuditPrivateReportReceiptReason::PrivateReportOutboxRequired,
            1,
            "PRIVATE_REPORT_OUTBOX_REQUIRED",
        ),
        (
            SealedAuditPrivateReportReceiptReason::ReceiptRequired,
            2,
            "RECEIPT_REQUIRED",
        ),
        (
            SealedAuditPrivateReportReceiptReason::GatewayTransparencyRequired,
            3,
            "GATEWAY_TRANSPARENCY_REQUIRED",
        ),
        (
            SealedAuditPrivateReportReceiptReason::DeliveryReplayRejected,
            4,
            "DELIVERY_REPLAY_REJECTED",
        ),
        (
            SealedAuditPrivateReportReceiptReason::MonitorProofRequired,
            5,
            "MONITOR_PROOF_REQUIRED",
        ),
        (
            SealedAuditPrivateReportReceiptReason::PlaintextMetadataForbidden,
            6,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditPrivateReportReceiptReason::BadRecordShape,
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
    write: SealedAuditPrivateReportReceiptWrite<'_>,
) -> SealedAuditPrivateReportReceiptDecision {
    mercury_core::evaluate_sealed_audit_private_report_receipt(write)
}

fn valid_write() -> SealedAuditPrivateReportReceiptWrite<'static> {
    SealedAuditPrivateReportReceiptWrite {
        outbox_decision: accepted_outbox_decision(),
        receipt_format_version: 1,
        receipt_id: &RECEIPT_ID,
        previous_receipt_id: &[],
        report_id: &REPORT_ID,
        gateway_receipt_digest: &DIGEST,
        gateway_signature_key_digest: &DIGEST,
        gateway_key_transparency_checkpoint_digest: &DIGEST,
        gateway_key_consistency_proof_digest: &DIGEST,
        gateway_key_rotation_digest: &DIGEST,
        relay_policy_digest: &DIGEST,
        response_transcript_digest: &DIGEST,
        monitor_submission_proof_digest: &DIGEST,
        blinded_failure_class_digest: &DIGEST,
        retry_completion_digest: &DIGEST,
        audit_checkpoint_digest: &DIGEST,
        receipt_sequence: 1,
        previous_receipt_sequence: 0,
        report_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        submitted_at_s: 1_769_991_000,
        acknowledged_at_s: 1_769_991_030,
        expires_at_s: 1_769_994_000,
        gateway_log_tree_size: 51,
        previous_gateway_log_tree_size: 50,
        delivery_attempt_count: 1,
        max_delivery_attempts: 3,
        gateway_receipt_signature_verified: true,
        receipt_binds_report_id: true,
        receipt_binds_response_transcript: true,
        receipt_binds_gateway_key: true,
        gateway_key_transparency_verified: true,
        gateway_key_consistency_verified: true,
        gateway_key_not_stale: true,
        gateway_key_rotation_authenticated: true,
        relay_policy_bound: true,
        monitor_submission_proof_verified: true,
        monitor_route_private: true,
        completion_state_monotonic: true,
        delivery_replay_rejected: true,
        duplicate_receipt_rejected: true,
        blinded_failure_classification: true,
        retry_completion_persisted: true,
        report_marked_delivered_only_after_receipt: true,
        receipt_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

const fn accepted_outbox_decision() -> SealedAuditPrivateReportOutboxDecision {
    accepted_outbox_decision_for_sequence(1)
}

const fn accepted_outbox_decision_for_sequence(
    report_sequence: i64,
) -> SealedAuditPrivateReportOutboxDecision {
    SealedAuditPrivateReportOutboxDecision {
        accepted: true,
        reason: SealedAuditPrivateReportOutboxReason::Accepted,
        persisted_record: true,
        record_count: report_sequence as usize,
        report_sequence,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_enqueue_report: true,
        can_submit_now: true,
        can_retry_safely: true,
        requires_private_transport: false,
        requires_replay_guard: false,
        requires_rate_limit_token: false,
        requires_policy_refresh: false,
        requires_route_privacy: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

const fn rejected_outbox_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditPrivateReportOutboxDecision {
    SealedAuditPrivateReportOutboxDecision {
        accepted: false,
        reason: SealedAuditPrivateReportOutboxReason::PlaintextMetadataForbidden,
        persisted_record: false,
        record_count: 0,
        report_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_enqueue_report: false,
        can_submit_now: false,
        can_retry_safely: false,
        requires_private_transport: false,
        requires_replay_guard: false,
        requires_rate_limit_token: false,
        requires_policy_refresh: false,
        requires_route_privacy: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
    }
}

fn assert_rejected(
    decision: SealedAuditPrivateReportReceiptDecision,
    reason: SealedAuditPrivateReportReceiptReason,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.persisted_record);
    assert!(!decision.can_mark_delivered);
    assert!(!decision.can_stop_retrying);
    assert!(!decision.can_show_delivery_status);
    assert!(decision.keeps_digest_only);
}
