use mercury_core::{
    PrototypeSealedAuditPrivateReportReconciliationStore, SealedAuditPrivateReportReceiptDecision,
    SealedAuditPrivateReportReceiptReason, SealedAuditPrivateReportReconciliationDecision,
    SealedAuditPrivateReportReconciliationReason, SealedAuditPrivateReportReconciliationWrite,
    put_sealed_audit_private_report_reconciliation_record,
};

const RECONCILIATION_ID: [u8; 32] = [0xA5; 32];
const NEXT_RECONCILIATION_ID: [u8; 32] = [0xA6; 32];
const REPORT_ID: [u8; 32] = [0xA1; 32];
const NEXT_REPORT_ID: [u8; 32] = [0xA2; 32];
const RECEIPT_ID: [u8; 32] = [0xA3; 32];
const NEXT_RECEIPT_ID: [u8; 32] = [0xA4; 32];
const DIGEST: [u8; 32] = [0xB1; 32];
const SHORT_DIGEST: [u8; 16] = [0xB2; 16];

#[test]
fn private_report_reconciliation_persists_only_accepted_digest_records() {
    let mut store = PrototypeSealedAuditPrivateReportReconciliationStore::default();

    let decision = put_sealed_audit_private_report_reconciliation_record(&mut store, valid_write())
        .expect("prototype private report reconciliation store cannot fail");

    assert!(decision.accepted);
    assert_eq!(
        decision.reason,
        SealedAuditPrivateReportReconciliationReason::Accepted
    );
    assert_eq!(decision.record_count, 1);
    assert!(decision.persisted_record);
    assert!(decision.can_reconcile_delivery);
    assert!(!decision.can_schedule_retry);
    assert!(decision.can_show_retry_status);
    assert!(decision.rejects_false_delivery);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store
        .get_by_id(&RECONCILIATION_ID)
        .expect("accepted reconciliation should be persisted");
    assert_eq!(record.reconciliation_id, RECONCILIATION_ID);
    assert_eq!(record.report_id, REPORT_ID);
    assert_eq!(record.receipt_id, RECEIPT_ID);
    assert_eq!(record.retry_schedule_digest.len(), 32);
    assert_eq!(record.rate_limit_state_digest.len(), 32);
    assert_eq!(record.reconciliation_sequence, 1);
    assert!(record.can_reconcile_delivery);
    assert!(!record.can_schedule_retry);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn private_report_reconciliation_rejects_receipt_failures_and_bad_shapes_without_mutation() {
    let mut store = PrototypeSealedAuditPrivateReportReconciliationStore::default();

    let receipt_rejected = SealedAuditPrivateReportReconciliationWrite {
        receipt_decision: rejected_receipt_decision(false),
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_private_report_reconciliation_record(&mut store, receipt_rejected)
            .expect("prototype private report reconciliation store cannot fail"),
        SealedAuditPrivateReportReconciliationReason::PrivateReportReceiptRequired,
    );
    assert!(store.is_empty());

    let bad_digest = SealedAuditPrivateReportReconciliationWrite {
        retry_schedule_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_private_report_reconciliation_record(&mut store, bad_digest)
            .expect("prototype private report reconciliation store cannot fail"),
        SealedAuditPrivateReportReconciliationReason::BadRecordShape,
    );
    assert!(store.is_empty());
}

#[test]
fn private_report_reconciliation_rejects_retry_rate_false_delivery_operator_and_plaintext_state() {
    let retry = SealedAuditPrivateReportReconciliationWrite {
        retry_schedule_bound: false,
        ..valid_write()
    };
    let retry_decision = evaluate(retry);
    assert_rejected(
        retry_decision,
        SealedAuditPrivateReportReconciliationReason::RetryScheduleRequired,
    );
    assert!(retry_decision.requires_retry_schedule);

    let rate_limit = SealedAuditPrivateReportReconciliationWrite {
        rate_limit_token_spend_preserved: false,
        ..valid_write()
    };
    let rate_limit_decision = evaluate(rate_limit);
    assert_rejected(
        rate_limit_decision,
        SealedAuditPrivateReportReconciliationReason::RateLimitContinuityRequired,
    );
    assert!(rate_limit_decision.requires_rate_limit_continuity);

    let false_delivery = SealedAuditPrivateReportReconciliationWrite {
        receipt_present: false,
        ..valid_write()
    };
    let false_delivery_decision = evaluate(false_delivery);
    assert_rejected(
        false_delivery_decision,
        SealedAuditPrivateReportReconciliationReason::FalseDeliveryRejected,
    );
    assert!(false_delivery_decision.rejects_false_delivery);

    let operator = SealedAuditPrivateReportReconciliationWrite {
        operator_accountability_route_bound: false,
        ..valid_write()
    };
    let operator_decision = evaluate(operator);
    assert_rejected(
        operator_decision,
        SealedAuditPrivateReportReconciliationReason::OperatorAccountabilityRequired,
    );
    assert!(operator_decision.requires_operator_accountability);

    let plaintext = SealedAuditPrivateReportReconciliationWrite {
        plaintext_selector_count: 1,
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let plaintext_decision = evaluate(plaintext);
    assert_rejected(
        plaintext_decision,
        SealedAuditPrivateReportReconciliationReason::PlaintextMetadataForbidden,
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn private_report_reconciliation_store_rejects_duplicate_and_rollback_sequences() {
    let mut store = PrototypeSealedAuditPrivateReportReconciliationStore::default();

    let first = put_sealed_audit_private_report_reconciliation_record(&mut store, valid_write())
        .expect("prototype private report reconciliation store cannot fail");
    assert!(first.accepted);

    let duplicate =
        put_sealed_audit_private_report_reconciliation_record(&mut store, valid_write())
            .expect("prototype private report reconciliation store cannot fail");
    assert_rejected(
        duplicate,
        SealedAuditPrivateReportReconciliationReason::FalseDeliveryRejected,
    );
    assert_eq!(store.len(), 1);

    let rollback = SealedAuditPrivateReportReconciliationWrite {
        reconciliation_id: &NEXT_RECONCILIATION_ID,
        report_id: &NEXT_REPORT_ID,
        receipt_id: &NEXT_RECEIPT_ID,
        reconciliation_sequence: 1,
        previous_reconciliation_sequence: 0,
        previous_reconciliation_id: &[],
        report_sequence: 2,
        receipt_sequence: 2,
        receipt_decision: accepted_receipt_decision_for_sequences(2, 2),
        ..valid_write()
    };
    let rollback_decision =
        put_sealed_audit_private_report_reconciliation_record(&mut store, rollback)
            .expect("prototype private report reconciliation store cannot fail");
    assert_rejected(
        rollback_decision,
        SealedAuditPrivateReportReconciliationReason::FalseDeliveryRejected,
    );
    assert_eq!(store.len(), 1);

    let next = SealedAuditPrivateReportReconciliationWrite {
        reconciliation_id: &NEXT_RECONCILIATION_ID,
        report_id: &NEXT_REPORT_ID,
        receipt_id: &NEXT_RECEIPT_ID,
        reconciliation_sequence: 2,
        previous_reconciliation_sequence: 1,
        previous_reconciliation_id: &RECONCILIATION_ID,
        report_sequence: 2,
        receipt_sequence: 2,
        receipt_decision: accepted_receipt_decision_for_sequences(2, 2),
        ..valid_write()
    };
    let next_decision = put_sealed_audit_private_report_reconciliation_record(&mut store, next)
        .expect("prototype private report reconciliation store cannot fail");
    assert!(next_decision.accepted);
    assert_eq!(store.len(), 2);
    assert_eq!(
        store
            .latest()
            .expect("latest exists")
            .reconciliation_sequence,
        2
    );
}

#[test]
fn private_report_reconciliation_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (
            SealedAuditPrivateReportReconciliationReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            SealedAuditPrivateReportReconciliationReason::PrivateReportReceiptRequired,
            1,
            "PRIVATE_REPORT_RECEIPT_REQUIRED",
        ),
        (
            SealedAuditPrivateReportReconciliationReason::RetryScheduleRequired,
            2,
            "RETRY_SCHEDULE_REQUIRED",
        ),
        (
            SealedAuditPrivateReportReconciliationReason::RateLimitContinuityRequired,
            3,
            "RATE_LIMIT_CONTINUITY_REQUIRED",
        ),
        (
            SealedAuditPrivateReportReconciliationReason::FalseDeliveryRejected,
            4,
            "FALSE_DELIVERY_REJECTED",
        ),
        (
            SealedAuditPrivateReportReconciliationReason::OperatorAccountabilityRequired,
            5,
            "OPERATOR_ACCOUNTABILITY_REQUIRED",
        ),
        (
            SealedAuditPrivateReportReconciliationReason::PlaintextMetadataForbidden,
            6,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditPrivateReportReconciliationReason::BadRecordShape,
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
    write: SealedAuditPrivateReportReconciliationWrite<'_>,
) -> SealedAuditPrivateReportReconciliationDecision {
    mercury_core::evaluate_sealed_audit_private_report_reconciliation(write)
}

fn valid_write() -> SealedAuditPrivateReportReconciliationWrite<'static> {
    SealedAuditPrivateReportReconciliationWrite {
        receipt_decision: accepted_receipt_decision(),
        reconciliation_format_version: 1,
        reconciliation_id: &RECONCILIATION_ID,
        previous_reconciliation_id: &[],
        report_id: &REPORT_ID,
        receipt_id: &RECEIPT_ID,
        pending_outbox_digest: &DIGEST,
        retry_schedule_digest: &DIGEST,
        rate_limit_state_digest: &DIGEST,
        delivered_state_digest: &DIGEST,
        failure_bucket_digest: &DIGEST,
        operator_accountability_route_digest: &DIGEST,
        crash_recovery_cursor_digest: &DIGEST,
        audit_checkpoint_digest: &DIGEST,
        reconciliation_sequence: 1,
        previous_reconciliation_sequence: 0,
        report_sequence: 1,
        receipt_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        created_at_s: 1_769_991_035,
        next_retry_after_s: 1_769_991_060,
        expires_at_s: 1_769_994_000,
        retry_attempt_count: 1,
        max_retry_attempts: 3,
        reports_remaining_in_window: 2,
        window_resets_at_s: 1_769_994_600,
        receipt_present: true,
        pending_outbox_bound: true,
        delivered_state_requires_receipt: true,
        retry_schedule_bound: true,
        retry_after_monotonic: true,
        duplicate_retry_rejected: true,
        retry_idempotency_key_bound: true,
        no_retry_after_delivered: true,
        rate_limit_window_bound: true,
        rate_limit_token_spend_preserved: true,
        retry_does_not_mint_new_report: true,
        crash_recovery_cursor_bound: true,
        resumes_pending_only: true,
        operator_accountability_route_bound: true,
        missing_receipt_escalates: true,
        blinded_failure_bucket_only: true,
        reconciliation_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

const fn accepted_receipt_decision() -> SealedAuditPrivateReportReceiptDecision {
    accepted_receipt_decision_for_sequences(1, 1)
}

const fn accepted_receipt_decision_for_sequences(
    report_sequence: i64,
    receipt_sequence: i64,
) -> SealedAuditPrivateReportReceiptDecision {
    SealedAuditPrivateReportReceiptDecision {
        accepted: true,
        reason: SealedAuditPrivateReportReceiptReason::Accepted,
        persisted_record: true,
        record_count: receipt_sequence as usize,
        receipt_sequence,
        report_sequence,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_mark_delivered: true,
        can_stop_retrying: true,
        can_show_delivery_status: true,
        requires_private_report_outbox: false,
        requires_receipt: false,
        requires_gateway_transparency: false,
        requires_delivery_replay_guard: false,
        requires_monitor_proof: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

const fn rejected_receipt_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditPrivateReportReceiptDecision {
    SealedAuditPrivateReportReceiptDecision {
        accepted: false,
        reason: SealedAuditPrivateReportReceiptReason::PlaintextMetadataForbidden,
        persisted_record: false,
        record_count: 0,
        receipt_sequence: 1,
        report_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_mark_delivered: false,
        can_stop_retrying: false,
        can_show_delivery_status: false,
        requires_private_report_outbox: false,
        requires_receipt: false,
        requires_gateway_transparency: false,
        requires_delivery_replay_guard: false,
        requires_monitor_proof: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
    }
}

fn assert_rejected(
    decision: SealedAuditPrivateReportReconciliationDecision,
    reason: SealedAuditPrivateReportReconciliationReason,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.persisted_record);
    assert!(!decision.can_reconcile_delivery);
    assert!(!decision.can_schedule_retry);
    assert!(!decision.can_show_retry_status);
    assert!(decision.keeps_digest_only);
}
