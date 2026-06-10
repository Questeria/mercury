use mercury_core::{
    PrototypeSealedAuditPrivateReportGatewayEvidenceStore,
    SealedAuditPrivateReportGatewayEvidenceDecision, SealedAuditPrivateReportGatewayEvidenceReason,
    SealedAuditPrivateReportGatewayEvidenceWrite, SealedAuditPrivateReportReconciliationDecision,
    SealedAuditPrivateReportReconciliationReason,
    put_sealed_audit_private_report_gateway_evidence_record,
};

const EVIDENCE_ID: [u8; 32] = [0xAD; 32];
const NEXT_EVIDENCE_ID: [u8; 32] = [0xAE; 32];
const RECONCILIATION_ID: [u8; 32] = [0xA5; 32];
const REPORT_ID: [u8; 32] = [0xA1; 32];
const NEXT_REPORT_ID: [u8; 32] = [0xA2; 32];
const RECEIPT_ID: [u8; 32] = [0xA3; 32];
const NEXT_RECEIPT_ID: [u8; 32] = [0xA4; 32];
const DIGEST: [u8; 32] = [0xB1; 32];
const SHORT_DIGEST: [u8; 16] = [0xB2; 16];

#[test]
fn private_report_gateway_evidence_persists_only_accepted_digest_records() {
    let mut store = PrototypeSealedAuditPrivateReportGatewayEvidenceStore::default();

    let decision =
        put_sealed_audit_private_report_gateway_evidence_record(&mut store, valid_write())
            .expect("prototype private report gateway evidence store cannot fail");

    assert!(decision.accepted);
    assert_eq!(
        decision.reason,
        SealedAuditPrivateReportGatewayEvidenceReason::Accepted
    );
    assert_eq!(decision.record_count, 1);
    assert!(decision.persisted_record);
    assert!(decision.can_raise_gateway_incident);
    assert!(decision.can_notify_operator);
    assert!(decision.can_show_unavailable_status);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store
        .get_by_id(&EVIDENCE_ID)
        .expect("accepted gateway evidence should be persisted");
    assert_eq!(record.evidence_id, EVIDENCE_ID);
    assert_eq!(record.report_id, REPORT_ID);
    assert_eq!(record.receipt_id, RECEIPT_ID);
    assert_eq!(record.unavailable_evidence_digest.len(), 32);
    assert_eq!(record.retry_exhaustion_digest.len(), 32);
    assert_eq!(record.gateway_status_code, 503);
    assert_eq!(record.evidence_sequence, 1);
    assert!(record.can_raise_gateway_incident);
    assert!(record.can_notify_operator);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn private_report_gateway_evidence_rejects_reconciliation_failures_and_bad_shapes_without_mutation()
{
    let mut store = PrototypeSealedAuditPrivateReportGatewayEvidenceStore::default();

    let reconciliation_rejected = SealedAuditPrivateReportGatewayEvidenceWrite {
        reconciliation_decision: rejected_reconciliation_decision(false),
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_private_report_gateway_evidence_record(
            &mut store,
            reconciliation_rejected,
        )
        .expect("prototype private report gateway evidence store cannot fail"),
        SealedAuditPrivateReportGatewayEvidenceReason::PrivateReportReconciliationRequired,
    );
    assert!(store.is_empty());

    let bad_digest = SealedAuditPrivateReportGatewayEvidenceWrite {
        unavailable_evidence_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_private_report_gateway_evidence_record(&mut store, bad_digest)
            .expect("prototype private report gateway evidence store cannot fail"),
        SealedAuditPrivateReportGatewayEvidenceReason::BadRecordShape,
    );
    assert!(store.is_empty());
}

#[test]
fn private_report_gateway_evidence_rejects_forged_unavailable_retry_accountability_and_plaintext() {
    let forged_unavailable = SealedAuditPrivateReportGatewayEvidenceWrite {
        no_client_asserted_unavailability: false,
        ..valid_write()
    };
    let forged_decision = evaluate(forged_unavailable);
    assert_rejected(
        forged_decision,
        SealedAuditPrivateReportGatewayEvidenceReason::UnavailableEvidenceRequired,
    );
    assert!(forged_decision.requires_unavailable_evidence);

    let retry_not_exhausted = SealedAuditPrivateReportGatewayEvidenceWrite {
        retry_attempt_count: 2,
        ..valid_write()
    };
    let retry_decision = evaluate(retry_not_exhausted);
    assert_rejected(
        retry_decision,
        SealedAuditPrivateReportGatewayEvidenceReason::RetryExhaustionRequired,
    );
    assert!(retry_decision.requires_retry_exhaustion);

    let accountability = SealedAuditPrivateReportGatewayEvidenceWrite {
        accountability_route_bound: false,
        ..valid_write()
    };
    let accountability_decision = evaluate(accountability);
    assert_rejected(
        accountability_decision,
        SealedAuditPrivateReportGatewayEvidenceReason::AccountabilityRouteRequired,
    );
    assert!(accountability_decision.requires_accountability_route);

    let plaintext = SealedAuditPrivateReportGatewayEvidenceWrite {
        plaintext_selector_count: 1,
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let plaintext_decision = evaluate(plaintext);
    assert_rejected(
        plaintext_decision,
        SealedAuditPrivateReportGatewayEvidenceReason::PlaintextMetadataForbidden,
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn private_report_gateway_evidence_store_rejects_duplicate_and_rollback_sequences() {
    let mut store = PrototypeSealedAuditPrivateReportGatewayEvidenceStore::default();

    let first = put_sealed_audit_private_report_gateway_evidence_record(&mut store, valid_write())
        .expect("prototype private report gateway evidence store cannot fail");
    assert!(first.accepted);

    let duplicate =
        put_sealed_audit_private_report_gateway_evidence_record(&mut store, valid_write())
            .expect("prototype private report gateway evidence store cannot fail");
    assert_rejected(
        duplicate,
        SealedAuditPrivateReportGatewayEvidenceReason::UnavailableEvidenceRequired,
    );
    assert_eq!(store.len(), 1);

    let rollback = SealedAuditPrivateReportGatewayEvidenceWrite {
        evidence_id: &NEXT_EVIDENCE_ID,
        report_id: &NEXT_REPORT_ID,
        receipt_id: &NEXT_RECEIPT_ID,
        evidence_sequence: 1,
        previous_evidence_sequence: 0,
        previous_evidence_id: &[],
        reconciliation_sequence: 2,
        report_sequence: 2,
        receipt_sequence: 2,
        reconciliation_decision: accepted_reconciliation_decision_for_sequences(2, 2, 2),
        ..valid_write()
    };
    let rollback_decision =
        put_sealed_audit_private_report_gateway_evidence_record(&mut store, rollback)
            .expect("prototype private report gateway evidence store cannot fail");
    assert_rejected(
        rollback_decision,
        SealedAuditPrivateReportGatewayEvidenceReason::UnavailableEvidenceRequired,
    );
    assert_eq!(store.len(), 1);

    let next = SealedAuditPrivateReportGatewayEvidenceWrite {
        evidence_id: &NEXT_EVIDENCE_ID,
        previous_evidence_id: &EVIDENCE_ID,
        report_id: &NEXT_REPORT_ID,
        receipt_id: &NEXT_RECEIPT_ID,
        evidence_sequence: 2,
        previous_evidence_sequence: 1,
        reconciliation_sequence: 2,
        report_sequence: 2,
        receipt_sequence: 2,
        reconciliation_decision: accepted_reconciliation_decision_for_sequences(2, 2, 2),
        ..valid_write()
    };
    let next_decision = put_sealed_audit_private_report_gateway_evidence_record(&mut store, next)
        .expect("prototype private report gateway evidence store cannot fail");
    assert!(next_decision.accepted);
    assert_eq!(store.len(), 2);
    assert_eq!(store.latest().expect("latest exists").evidence_sequence, 2);
}

#[test]
fn private_report_gateway_evidence_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (
            SealedAuditPrivateReportGatewayEvidenceReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            SealedAuditPrivateReportGatewayEvidenceReason::PrivateReportReconciliationRequired,
            1,
            "PRIVATE_REPORT_RECONCILIATION_REQUIRED",
        ),
        (
            SealedAuditPrivateReportGatewayEvidenceReason::UnavailableEvidenceRequired,
            2,
            "UNAVAILABLE_EVIDENCE_REQUIRED",
        ),
        (
            SealedAuditPrivateReportGatewayEvidenceReason::AccountabilityRouteRequired,
            3,
            "ACCOUNTABILITY_ROUTE_REQUIRED",
        ),
        (
            SealedAuditPrivateReportGatewayEvidenceReason::RetryExhaustionRequired,
            4,
            "RETRY_EXHAUSTION_REQUIRED",
        ),
        (
            SealedAuditPrivateReportGatewayEvidenceReason::PlaintextMetadataForbidden,
            5,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditPrivateReportGatewayEvidenceReason::BadRecordShape,
            6,
            "BAD_RECORD_SHAPE",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn evaluate(
    write: SealedAuditPrivateReportGatewayEvidenceWrite<'_>,
) -> SealedAuditPrivateReportGatewayEvidenceDecision {
    mercury_core::evaluate_sealed_audit_private_report_gateway_evidence(write)
}

fn valid_write() -> SealedAuditPrivateReportGatewayEvidenceWrite<'static> {
    SealedAuditPrivateReportGatewayEvidenceWrite {
        reconciliation_decision: accepted_reconciliation_decision(),
        evidence_format_version: 1,
        evidence_id: &EVIDENCE_ID,
        previous_evidence_id: &[],
        reconciliation_id: &RECONCILIATION_ID,
        report_id: &REPORT_ID,
        receipt_id: &RECEIPT_ID,
        unavailable_evidence_digest: &DIGEST,
        relay_observation_digest: &DIGEST,
        gateway_error_digest: &DIGEST,
        target_absence_digest: &DIGEST,
        retry_exhaustion_digest: &DIGEST,
        rate_limit_state_digest: &DIGEST,
        gateway_key_state_digest: &DIGEST,
        accountability_route_digest: &DIGEST,
        blinded_failure_bucket_digest: &DIGEST,
        monitor_submission_digest: &DIGEST,
        audit_checkpoint_digest: &DIGEST,
        evidence_sequence: 1,
        previous_evidence_sequence: 0,
        reconciliation_sequence: 1,
        report_sequence: 1,
        receipt_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        created_at_s: 1_769_991_090,
        expires_at_s: 1_769_994_000,
        retry_attempt_count: 3,
        max_retry_attempts: 3,
        gateway_status_code: 503,
        reconciliation_bound: true,
        unavailable_evidence_gateway_authenticated: true,
        relay_observation_signed: true,
        target_absence_proof_bound: true,
        gateway_timeout_or_5xx_classified: true,
        no_client_asserted_unavailability: true,
        retry_exhaustion_bound: true,
        rate_limit_continuity_bound: true,
        gateway_key_state_bound: true,
        accountability_route_bound: true,
        operator_escalation_bound: true,
        blinded_failure_bucket_only: true,
        monitor_route_private: true,
        incident_visible_only_after_policy: true,
        evidence_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

const fn accepted_reconciliation_decision() -> SealedAuditPrivateReportReconciliationDecision {
    accepted_reconciliation_decision_for_sequences(1, 1, 1)
}

const fn accepted_reconciliation_decision_for_sequences(
    evidence_sequence: i64,
    report_sequence: i64,
    receipt_sequence: i64,
) -> SealedAuditPrivateReportReconciliationDecision {
    SealedAuditPrivateReportReconciliationDecision {
        accepted: true,
        reason: SealedAuditPrivateReportReconciliationReason::Accepted,
        persisted_record: true,
        record_count: evidence_sequence as usize,
        reconciliation_sequence: evidence_sequence,
        report_sequence,
        receipt_sequence,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_reconcile_delivery: true,
        can_schedule_retry: false,
        can_show_retry_status: true,
        requires_private_report_receipt: false,
        requires_retry_schedule: false,
        requires_rate_limit_continuity: false,
        rejects_false_delivery: true,
        requires_operator_accountability: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

const fn rejected_reconciliation_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditPrivateReportReconciliationDecision {
    SealedAuditPrivateReportReconciliationDecision {
        accepted: false,
        reason: SealedAuditPrivateReportReconciliationReason::PlaintextMetadataForbidden,
        persisted_record: false,
        record_count: 0,
        reconciliation_sequence: 1,
        report_sequence: 1,
        receipt_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_reconcile_delivery: false,
        can_schedule_retry: false,
        can_show_retry_status: false,
        requires_private_report_receipt: false,
        requires_retry_schedule: false,
        requires_rate_limit_continuity: false,
        rejects_false_delivery: false,
        requires_operator_accountability: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
    }
}

fn assert_rejected(
    decision: SealedAuditPrivateReportGatewayEvidenceDecision,
    reason: SealedAuditPrivateReportGatewayEvidenceReason,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.persisted_record);
    assert!(!decision.can_raise_gateway_incident);
    assert!(!decision.can_notify_operator);
    assert!(!decision.can_show_unavailable_status);
    assert!(decision.keeps_digest_only);
}
