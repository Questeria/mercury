use mercury_core::{
    PrototypeSealedAuditVerifierPolicyStore, SealedAuditProofCacheDecision,
    SealedAuditProofCacheReason, SealedAuditVerifierPolicyDecision,
    SealedAuditVerifierPolicyReason, SealedAuditVerifierPolicySnapshot,
    put_sealed_audit_verifier_policy_snapshot,
};

const POLICY_DIGEST: [u8; 32] = [0x31; 32];
const NEXT_POLICY_DIGEST: [u8; 32] = [0x32; 32];
const LOG_KEY_PINSET_DIGEST: [u8; 32] = [0x41; 32];
const WITNESS_KEY_PINSET_DIGEST: [u8; 32] = [0x42; 32];
const MONITOR_QUERY_PLAN_DIGEST: [u8; 32] = [0x43; 32];
const PROOF_CACHE_DIGEST: [u8; 32] = [0x51; 32];

#[test]
fn verifier_policy_persists_only_accepted_digest_snapshots() {
    let mut store = PrototypeSealedAuditVerifierPolicyStore::default();

    let decision = put_sealed_audit_verifier_policy_snapshot(&mut store, valid_snapshot())
        .expect("prototype policy store is infallible");

    assert_eq!(decision.reason, SealedAuditVerifierPolicyReason::Accepted);
    assert!(decision.accepted);
    assert!(decision.persisted_snapshot);
    assert_eq!(decision.snapshot_count, 1);
    assert_eq!(decision.policy_epoch, 7);
    assert_eq!(decision.proof_cache_log_index, 42);
    assert_eq!(decision.latest_checked_log_index, 45);
    assert!(decision.can_verify_offline);
    assert!(decision.can_schedule_private_monitor);
    assert!(decision.can_show_ui_status);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store
        .get_by_digest(&POLICY_DIGEST)
        .expect("accepted policy snapshot should be stored");
    assert_eq!(record.policy_snapshot_digest, POLICY_DIGEST);
    assert!(record.previous_policy_snapshot_digest.is_empty());
    assert_eq!(record.log_key_pinset_digest, LOG_KEY_PINSET_DIGEST);
    assert_eq!(record.witness_key_pinset_digest, WITNESS_KEY_PINSET_DIGEST);
    assert_eq!(record.monitor_query_plan_digest, MONITOR_QUERY_PLAN_DIGEST);
    assert_eq!(record.proof_cache_digest, PROOF_CACHE_DIGEST);
    assert_eq!(record.policy_epoch, 7);
    assert_eq!(record.proof_cache_log_index, 42);
    assert_eq!(record.latest_checked_log_index, 45);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn verifier_policy_rejects_proof_cache_failures_and_bad_shapes_without_mutation() {
    let mut store = PrototypeSealedAuditVerifierPolicyStore::default();

    let rejected_cache = SealedAuditVerifierPolicySnapshot {
        proof_cache_decision: SealedAuditProofCacheDecision {
            accepted: false,
            reason: SealedAuditProofCacheReason::ProofBundleRejected,
            plaintext_bytes_exposed: true,
            ..valid_proof_cache_decision()
        },
        ..valid_snapshot()
    };
    assert_rejected(
        put_sealed_audit_verifier_policy_snapshot(&mut store, rejected_cache)
            .expect("prototype policy store is infallible"),
        SealedAuditVerifierPolicyReason::ProofCacheRejected,
    );

    let bad_digest = SealedAuditVerifierPolicySnapshot {
        policy_snapshot_digest: &[0x31; 31],
        ..valid_snapshot()
    };
    assert_rejected(
        put_sealed_audit_verifier_policy_snapshot(&mut store, bad_digest)
            .expect("prototype policy store is infallible"),
        SealedAuditVerifierPolicyReason::BadRecordShape,
    );

    let bad_consistency = SealedAuditVerifierPolicySnapshot {
        policy_consistency_proof_verified: false,
        ..valid_snapshot()
    };
    assert_rejected(
        put_sealed_audit_verifier_policy_snapshot(&mut store, bad_consistency)
            .expect("prototype policy store is infallible"),
        SealedAuditVerifierPolicyReason::BadRecordShape,
    );

    assert!(store.is_empty());
}

#[test]
fn verifier_policy_rejects_stale_rotation_monitor_split_view_and_plaintext_state() {
    let expired = SealedAuditVerifierPolicySnapshot {
        verification_time_s: 1_769_994_000,
        expires_at_s: 1_769_994_000,
        ..valid_snapshot()
    };
    assert_policy_refresh(
        evaluate(expired),
        SealedAuditVerifierPolicyReason::PolicySnapshotExpired,
    );

    let unauthenticated_rotation = SealedAuditVerifierPolicySnapshot {
        policy_epoch: 8,
        key_rotation_required: true,
        key_rotation_authenticated: false,
        ..valid_snapshot()
    };
    let rotation = evaluate(unauthenticated_rotation);
    assert_eq!(
        rotation.reason,
        SealedAuditVerifierPolicyReason::KeyRotationRequired
    );
    assert!(rotation.requires_key_rotation);
    assert!(rotation.requires_policy_refresh);

    let stale_monitor = SealedAuditVerifierPolicySnapshot {
        monitor_last_refresh_s: 1_769_980_000,
        ..valid_snapshot()
    };
    let monitor = evaluate(stale_monitor);
    assert_eq!(
        monitor.reason,
        SealedAuditVerifierPolicyReason::MonitorFreshnessStale
    );
    assert!(monitor.requires_monitor_refresh);

    let split_view = SealedAuditVerifierPolicySnapshot {
        split_view_evidence_count: 1,
        ..valid_snapshot()
    };
    let split = evaluate(split_view);
    assert_eq!(
        split.reason,
        SealedAuditVerifierPolicyReason::SplitViewEscalationRequired
    );
    assert!(split.escalates_split_view);

    let plaintext = SealedAuditVerifierPolicySnapshot {
        monitor_query_plaintext_selector_count: 1,
        plaintext_metadata_fields: 1,
        ..valid_snapshot()
    };
    let plaintext_decision = evaluate(plaintext);
    assert_eq!(
        plaintext_decision.reason,
        SealedAuditVerifierPolicyReason::PlaintextMetadataForbidden
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn verifier_policy_store_rejects_duplicate_and_rollback_epochs() {
    let mut store = PrototypeSealedAuditVerifierPolicyStore::default();

    let first = put_sealed_audit_verifier_policy_snapshot(&mut store, valid_snapshot())
        .expect("prototype policy store is infallible");
    assert!(first.accepted);

    let duplicate = put_sealed_audit_verifier_policy_snapshot(&mut store, valid_snapshot())
        .expect("prototype policy store is infallible");
    assert_eq!(
        duplicate.reason,
        SealedAuditVerifierPolicyReason::BadRecordShape
    );
    assert_eq!(duplicate.snapshot_count, 1);

    let rollback_epoch = SealedAuditVerifierPolicySnapshot {
        policy_snapshot_digest: &NEXT_POLICY_DIGEST,
        ..valid_snapshot()
    };
    let rollback = put_sealed_audit_verifier_policy_snapshot(&mut store, rollback_epoch)
        .expect("prototype policy store is infallible");
    assert_eq!(
        rollback.reason,
        SealedAuditVerifierPolicyReason::PolicySnapshotExpired
    );
    assert!(rollback.requires_policy_refresh);
    assert_eq!(rollback.snapshot_count, 1);
}

#[test]
fn verifier_policy_accepts_authenticated_policy_rotation() {
    let mut store = PrototypeSealedAuditVerifierPolicyStore::default();
    assert!(
        put_sealed_audit_verifier_policy_snapshot(&mut store, valid_snapshot())
            .expect("prototype policy store is infallible")
            .accepted
    );

    let rotated = SealedAuditVerifierPolicySnapshot {
        policy_snapshot_digest: &NEXT_POLICY_DIGEST,
        previous_policy_snapshot_digest: &POLICY_DIGEST,
        policy_epoch: 8,
        key_rotation_required: true,
        key_rotation_authenticated: true,
        latest_checked_log_index: 46,
        ..valid_snapshot()
    };
    let decision = put_sealed_audit_verifier_policy_snapshot(&mut store, rotated)
        .expect("prototype policy store is infallible");

    assert!(decision.accepted);
    assert_eq!(decision.snapshot_count, 2);
    assert_eq!(store.latest().expect("latest snapshot").policy_epoch, 8);
}

#[test]
fn verifier_policy_reasons_have_stable_codes_and_labels() {
    let cases = [
        (SealedAuditVerifierPolicyReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditVerifierPolicyReason::ProofCacheRejected,
            1,
            "PROOF_CACHE_REJECTED",
        ),
        (
            SealedAuditVerifierPolicyReason::PolicySnapshotExpired,
            2,
            "POLICY_SNAPSHOT_EXPIRED",
        ),
        (
            SealedAuditVerifierPolicyReason::KeyRotationRequired,
            3,
            "KEY_ROTATION_REQUIRED",
        ),
        (
            SealedAuditVerifierPolicyReason::MonitorFreshnessStale,
            4,
            "MONITOR_FRESHNESS_STALE",
        ),
        (
            SealedAuditVerifierPolicyReason::SplitViewEscalationRequired,
            5,
            "SPLIT_VIEW_ESCALATION_REQUIRED",
        ),
        (
            SealedAuditVerifierPolicyReason::PlaintextMetadataForbidden,
            6,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditVerifierPolicyReason::BadRecordShape,
            7,
            "BAD_RECORD_SHAPE",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn evaluate(snapshot: SealedAuditVerifierPolicySnapshot<'_>) -> SealedAuditVerifierPolicyDecision {
    mercury_core::evaluate_sealed_audit_verifier_policy_snapshot(snapshot)
}

fn assert_rejected(
    decision: SealedAuditVerifierPolicyDecision,
    reason: SealedAuditVerifierPolicyReason,
) {
    assert_eq!(decision.reason, reason);
    assert!(!decision.accepted);
    assert!(!decision.persisted_snapshot);
    assert_eq!(decision.snapshot_count, 0);
}

fn assert_policy_refresh(
    decision: SealedAuditVerifierPolicyDecision,
    reason: SealedAuditVerifierPolicyReason,
) {
    assert_eq!(decision.reason, reason);
    assert!(!decision.accepted);
    assert!(decision.requires_policy_refresh);
}

fn valid_snapshot() -> SealedAuditVerifierPolicySnapshot<'static> {
    SealedAuditVerifierPolicySnapshot {
        proof_cache_decision: valid_proof_cache_decision(),
        policy_format_version: 1,
        policy_snapshot_digest: &POLICY_DIGEST,
        previous_policy_snapshot_digest: &[],
        log_key_pinset_digest: &LOG_KEY_PINSET_DIGEST,
        witness_key_pinset_digest: &WITNESS_KEY_PINSET_DIGEST,
        monitor_query_plan_digest: &MONITOR_QUERY_PLAN_DIGEST,
        proof_cache_digest: &PROOF_CACHE_DIGEST,
        policy_epoch: 7,
        imported_at_s: 1_769_990_000,
        verification_time_s: 1_769_991_000,
        expires_at_s: 1_769_994_000,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        log_key_pin_count: 1,
        witness_key_pin_count: 3,
        witness_quorum_threshold: 2,
        private_monitor_endpoint_count: 2,
        monitor_last_refresh_s: 1_769_990_700,
        monitor_freshness_max_age_s: 3_600,
        monitor_query_plaintext_selector_count: 0,
        policy_signature_verified: true,
        policy_consistency_proof_verified: true,
        offline_reverification_passed: true,
        key_rotation_required: false,
        key_rotation_authenticated: false,
        split_view_evidence_count: 0,
        scheduler_state_encrypted: true,
        scheduler_append_only: true,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn valid_proof_cache_decision() -> SealedAuditProofCacheDecision {
    SealedAuditProofCacheDecision {
        accepted: true,
        reason: SealedAuditProofCacheReason::Accepted,
        persisted_record: true,
        record_count: 1,
        event_sequence: 42,
        log_index: 42,
        checkpoint_size: 43,
        verifier_policy_epoch: 7,
        can_verify_offline: true,
        can_show_ui_status: true,
        can_refresh_monitor: true,
        requires_policy_refresh: false,
        requires_witness_refresh: false,
        requires_cache_recovery: false,
        append_only: true,
        keeps_digest_only: true,
        keeps_plaintext_metadata: false,
        plaintext_bytes_exposed: false,
    }
}
