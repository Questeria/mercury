use mercury_core::{
    PrototypeSealedAuditRecoveryExportStore, SealedAuditIncidentEvidenceDecision,
    SealedAuditIncidentEvidenceReason, SealedAuditRecoveryExportDecision,
    SealedAuditRecoveryExportReason, SealedAuditRecoveryExportWrite,
    put_sealed_audit_recovery_export_record,
};

const EXPORT_MANIFEST_DIGEST: [u8; 32] = [0x71; 32];
const NEXT_EXPORT_MANIFEST_DIGEST: [u8; 32] = [0x72; 32];
const DEVICE_SET_DIGEST: [u8; 32] = [0x73; 32];
const RECOVERY_POLICY_DIGEST: [u8; 32] = [0x74; 32];
const VERIFIER_POLICY_DIGEST: [u8; 32] = [0x75; 32];
const PROOF_CACHE_DIGEST: [u8; 32] = [0x76; 32];
const INCIDENT_ID: [u8; 32] = [0x77; 32];
const INCIDENT_EVIDENCE_DIGEST: [u8; 32] = [0x78; 32];
const EXPORT_CIPHERTEXT_DIGEST: [u8; 32] = [0x79; 32];
const RESTORE_AUTHORIZATION_DIGEST: [u8; 32] = [0x7A; 32];
const SYNC_STATE_DIGEST: [u8; 32] = [0x7B; 32];
const AUDIT_LOG_CHECKPOINT_DIGEST: [u8; 32] = [0x7C; 32];

#[test]
fn recovery_export_persists_only_accepted_digest_records() {
    let mut store = PrototypeSealedAuditRecoveryExportStore::default();

    let decision = put_sealed_audit_recovery_export_record(&mut store, valid_write())
        .expect("prototype recovery export store is infallible");

    assert_eq!(decision.reason, SealedAuditRecoveryExportReason::Accepted);
    assert!(decision.accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert_eq!(decision.export_sequence, 1);
    assert_eq!(decision.policy_epoch, 7);
    assert_eq!(decision.proof_cache_log_index, 42);
    assert_eq!(decision.latest_checked_log_index, 45);
    assert!(decision.can_export_state);
    assert!(decision.can_restore_state);
    assert!(decision.can_sync_cross_device);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store
        .get_by_digest(&EXPORT_MANIFEST_DIGEST)
        .expect("accepted export should be stored");
    assert_eq!(record.export_manifest_digest, EXPORT_MANIFEST_DIGEST);
    assert!(record.previous_export_manifest_digest.is_empty());
    assert_eq!(record.device_set_digest, DEVICE_SET_DIGEST);
    assert_eq!(record.recovery_policy_digest, RECOVERY_POLICY_DIGEST);
    assert_eq!(record.verifier_policy_digest, VERIFIER_POLICY_DIGEST);
    assert_eq!(record.proof_cache_digest, PROOF_CACHE_DIGEST);
    assert_eq!(record.incident_id, INCIDENT_ID);
    assert_eq!(record.incident_evidence_digest, INCIDENT_EVIDENCE_DIGEST);
    assert_eq!(record.export_ciphertext_digest, EXPORT_CIPHERTEXT_DIGEST);
    assert_eq!(
        record.restore_authorization_digest,
        RESTORE_AUTHORIZATION_DIGEST
    );
    assert_eq!(record.sync_state_digest, SYNC_STATE_DIGEST);
    assert_eq!(
        record.audit_log_checkpoint_digest,
        AUDIT_LOG_CHECKPOINT_DIGEST
    );
    assert_eq!(record.device_count, 3);
    assert_eq!(record.device_quorum_threshold, 2);
    assert_eq!(record.approving_device_count, 2);
    assert_eq!(record.recovery_share_count, 3);
    assert_eq!(record.recovery_share_threshold, 2);
    assert!(record.can_export_state);
    assert!(record.can_restore_state);
    assert!(record.can_sync_cross_device);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn recovery_export_rejects_incident_failures_and_bad_shapes_without_mutation() {
    let mut store = PrototypeSealedAuditRecoveryExportStore::default();

    let incident_rejected = SealedAuditRecoveryExportWrite {
        incident_evidence_decision: SealedAuditIncidentEvidenceDecision {
            accepted: false,
            reason: SealedAuditIncidentEvidenceReason::VerifierPolicyRejected,
            plaintext_bytes_exposed: true,
            ..valid_incident_decision()
        },
        ..valid_write()
    };
    let incident = put_sealed_audit_recovery_export_record(&mut store, incident_rejected)
        .expect("prototype recovery export store is infallible");
    assert_eq!(
        incident.reason,
        SealedAuditRecoveryExportReason::IncidentEvidenceRejected
    );
    assert!(incident.plaintext_bytes_exposed);

    let bad_digest = SealedAuditRecoveryExportWrite {
        export_manifest_digest: &[0x71; 31],
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_recovery_export_record(&mut store, bad_digest)
            .expect("prototype recovery export store is infallible"),
        SealedAuditRecoveryExportReason::BadRecordShape,
    );

    let bad_checkpoint = SealedAuditRecoveryExportWrite {
        audit_log_checkpoint_verified: false,
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_recovery_export_record(&mut store, bad_checkpoint)
            .expect("prototype recovery export store is infallible"),
        SealedAuditRecoveryExportReason::BadRecordShape,
    );

    assert!(store.is_empty());
}

#[test]
fn recovery_export_rejects_quorum_stale_rollback_device_and_plaintext_state() {
    let quorum = SealedAuditRecoveryExportWrite {
        approving_device_count: 1,
        restore_quorum_met: false,
        ..valid_write()
    };
    let quorum_decision = evaluate(quorum);
    assert_eq!(
        quorum_decision.reason,
        SealedAuditRecoveryExportReason::RestoreQuorumRequired
    );
    assert!(quorum_decision.requires_restore_quorum);

    let stale = SealedAuditRecoveryExportWrite {
        restored_at_s: 1_769_994_000,
        ..valid_write()
    };
    let stale_decision = evaluate(stale);
    assert_eq!(
        stale_decision.reason,
        SealedAuditRecoveryExportReason::StalePolicySnapshot
    );
    assert!(stale_decision.requires_policy_refresh);

    let rollback = SealedAuditRecoveryExportWrite {
        export_sequence: 1,
        previous_export_sequence: 1,
        previous_export_manifest_digest: &EXPORT_MANIFEST_DIGEST,
        previous_export_bound: true,
        ..valid_write()
    };
    let rollback_decision = evaluate(rollback);
    assert_eq!(
        rollback_decision.reason,
        SealedAuditRecoveryExportReason::RollbackExportRejected
    );
    assert!(rollback_decision.rejects_rollback);

    let device = SealedAuditRecoveryExportWrite {
        device_binding_verified: false,
        ..valid_write()
    };
    let device_decision = evaluate(device);
    assert_eq!(
        device_decision.reason,
        SealedAuditRecoveryExportReason::DeviceBindingRequired
    );
    assert!(device_decision.requires_device_binding);

    let plaintext = SealedAuditRecoveryExportWrite {
        plaintext_selector_count: 1,
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let plaintext_decision = evaluate(plaintext);
    assert_eq!(
        plaintext_decision.reason,
        SealedAuditRecoveryExportReason::PlaintextMetadataForbidden
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn recovery_export_store_rejects_duplicate_and_rollback_sequences() {
    let mut store = PrototypeSealedAuditRecoveryExportStore::default();

    let first = put_sealed_audit_recovery_export_record(&mut store, valid_write())
        .expect("prototype recovery export store is infallible");
    assert!(first.accepted);

    let duplicate = put_sealed_audit_recovery_export_record(&mut store, valid_write())
        .expect("prototype recovery export store is infallible");
    assert_eq!(
        duplicate.reason,
        SealedAuditRecoveryExportReason::BadRecordShape
    );
    assert_eq!(duplicate.record_count, 1);

    let rollback = SealedAuditRecoveryExportWrite {
        export_manifest_digest: &NEXT_EXPORT_MANIFEST_DIGEST,
        export_sequence: 1,
        previous_export_sequence: 0,
        ..valid_write()
    };
    let rollback_decision = put_sealed_audit_recovery_export_record(&mut store, rollback)
        .expect("prototype recovery export store is infallible");
    assert_eq!(
        rollback_decision.reason,
        SealedAuditRecoveryExportReason::RollbackExportRejected
    );
    assert!(rollback_decision.rejects_rollback);
    assert_eq!(rollback_decision.record_count, 1);

    let next = SealedAuditRecoveryExportWrite {
        export_manifest_digest: &NEXT_EXPORT_MANIFEST_DIGEST,
        previous_export_manifest_digest: &EXPORT_MANIFEST_DIGEST,
        export_sequence: 2,
        previous_export_sequence: 1,
        previous_export_bound: true,
        created_at_s: 1_769_991_100,
        restored_at_s: 1_769_991_200,
        ..valid_write()
    };
    let next_decision = put_sealed_audit_recovery_export_record(&mut store, next)
        .expect("prototype recovery export store is infallible");
    assert!(next_decision.accepted);
    assert_eq!(next_decision.record_count, 2);
    assert_eq!(store.latest().expect("latest export").export_sequence, 2);
}

#[test]
fn recovery_export_reasons_have_stable_codes_and_labels() {
    let cases = [
        (SealedAuditRecoveryExportReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditRecoveryExportReason::IncidentEvidenceRejected,
            1,
            "INCIDENT_EVIDENCE_REJECTED",
        ),
        (
            SealedAuditRecoveryExportReason::RestoreQuorumRequired,
            2,
            "RESTORE_QUORUM_REQUIRED",
        ),
        (
            SealedAuditRecoveryExportReason::StalePolicySnapshot,
            3,
            "STALE_POLICY_SNAPSHOT",
        ),
        (
            SealedAuditRecoveryExportReason::RollbackExportRejected,
            4,
            "ROLLBACK_EXPORT_REJECTED",
        ),
        (
            SealedAuditRecoveryExportReason::DeviceBindingRequired,
            5,
            "DEVICE_BINDING_REQUIRED",
        ),
        (
            SealedAuditRecoveryExportReason::PlaintextMetadataForbidden,
            6,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditRecoveryExportReason::BadRecordShape,
            7,
            "BAD_RECORD_SHAPE",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn evaluate(write: SealedAuditRecoveryExportWrite<'_>) -> SealedAuditRecoveryExportDecision {
    mercury_core::evaluate_sealed_audit_recovery_export(write)
}

fn assert_rejected(
    decision: SealedAuditRecoveryExportDecision,
    reason: SealedAuditRecoveryExportReason,
) {
    assert_eq!(decision.reason, reason);
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert_eq!(decision.record_count, 0);
}

fn valid_write() -> SealedAuditRecoveryExportWrite<'static> {
    SealedAuditRecoveryExportWrite {
        incident_evidence_decision: valid_incident_decision(),
        export_format_version: 1,
        export_manifest_digest: &EXPORT_MANIFEST_DIGEST,
        previous_export_manifest_digest: &[],
        device_set_digest: &DEVICE_SET_DIGEST,
        recovery_policy_digest: &RECOVERY_POLICY_DIGEST,
        verifier_policy_digest: &VERIFIER_POLICY_DIGEST,
        proof_cache_digest: &PROOF_CACHE_DIGEST,
        incident_id: &INCIDENT_ID,
        incident_evidence_digest: &INCIDENT_EVIDENCE_DIGEST,
        export_ciphertext_digest: &EXPORT_CIPHERTEXT_DIGEST,
        restore_authorization_digest: &RESTORE_AUTHORIZATION_DIGEST,
        sync_state_digest: &SYNC_STATE_DIGEST,
        audit_log_checkpoint_digest: &AUDIT_LOG_CHECKPOINT_DIGEST,
        export_sequence: 1,
        previous_export_sequence: 0,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        created_at_s: 1_769_991_000,
        expires_at_s: 1_769_994_000,
        restored_at_s: 1_769_991_050,
        device_count: 3,
        device_quorum_threshold: 2,
        approving_device_count: 2,
        recovery_share_count: 3,
        recovery_share_threshold: 2,
        manifest_signature_verified: true,
        device_binding_verified: true,
        recovery_policy_verified: true,
        export_ciphertext_encrypted: true,
        export_ciphertext_authenticated: true,
        restore_authorization_verified: true,
        restore_quorum_met: true,
        rollback_guard_verified: true,
        previous_export_bound: false,
        cross_device_sync_private: true,
        incident_selectors_redacted: true,
        audit_log_checkpoint_verified: true,
        store_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn valid_incident_decision() -> SealedAuditIncidentEvidenceDecision {
    SealedAuditIncidentEvidenceDecision {
        accepted: true,
        reason: SealedAuditIncidentEvidenceReason::Accepted,
        persisted_record: true,
        record_count: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_escalate_incident: true,
        can_report_privately: true,
        can_show_ui_status: true,
        requires_missing_proof_report: true,
        requires_split_view_escalation: true,
        requires_operator_accountability: true,
        requires_retry_backoff: true,
        suppressed_by_authenticated_policy: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}
