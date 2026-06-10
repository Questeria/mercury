use mercury_core::{
    PrototypeSealedAuditIncidentEvidenceStore, SealedAuditIncidentEvidenceDecision,
    SealedAuditIncidentEvidenceReason, SealedAuditIncidentEvidenceWrite,
    SealedAuditVerifierPolicyDecision, SealedAuditVerifierPolicyReason,
    put_sealed_audit_incident_evidence_record,
};

const INCIDENT_ID: [u8; 32] = [0x61; 32];
const NEXT_INCIDENT_ID: [u8; 32] = [0x62; 32];
const VERIFIER_POLICY_DIGEST: [u8; 32] = [0x31; 32];
const PROOF_CACHE_DIGEST: [u8; 32] = [0x51; 32];
const CHECKPOINT_DIGEST: [u8; 32] = [0x52; 32];
const WITNESS_OPERATOR_DIGEST: [u8; 32] = [0x53; 32];
const CONTRADICTION_DIGEST: [u8; 32] = [0x54; 32];
const MISSING_PROOF_REPORT_DIGEST: [u8; 32] = [0x55; 32];
const MONITOR_REPORT_DIGEST: [u8; 32] = [0x56; 32];
const ACCOUNTABILITY_ROUTE_DIGEST: [u8; 32] = [0x57; 32];

#[test]
fn incident_evidence_persists_only_accepted_digest_records() {
    let mut store = PrototypeSealedAuditIncidentEvidenceStore::default();

    let decision = put_sealed_audit_incident_evidence_record(&mut store, valid_write())
        .expect("prototype incident store is infallible");

    assert_eq!(decision.reason, SealedAuditIncidentEvidenceReason::Accepted);
    assert!(decision.accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert_eq!(decision.policy_epoch, 7);
    assert_eq!(decision.proof_cache_log_index, 42);
    assert_eq!(decision.latest_checked_log_index, 45);
    assert!(decision.can_escalate_incident);
    assert!(decision.can_report_privately);
    assert!(decision.can_show_ui_status);
    assert!(decision.requires_missing_proof_report);
    assert!(decision.requires_split_view_escalation);
    assert!(decision.requires_operator_accountability);
    assert!(decision.requires_retry_backoff);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store
        .get_by_incident_id(&INCIDENT_ID)
        .expect("accepted incident evidence should be stored");
    assert_eq!(record.incident_id, INCIDENT_ID);
    assert_eq!(record.verifier_policy_digest, VERIFIER_POLICY_DIGEST);
    assert_eq!(record.proof_cache_digest, PROOF_CACHE_DIGEST);
    assert_eq!(record.checkpoint_digest, CHECKPOINT_DIGEST);
    assert_eq!(record.witness_operator_digest, WITNESS_OPERATOR_DIGEST);
    assert_eq!(record.contradiction_digest, CONTRADICTION_DIGEST);
    assert_eq!(
        record.missing_proof_report_digest,
        MISSING_PROOF_REPORT_DIGEST
    );
    assert_eq!(record.monitor_report_digest, MONITOR_REPORT_DIGEST);
    assert_eq!(
        record.accountability_route_digest,
        ACCOUNTABILITY_ROUTE_DIGEST
    );
    assert_eq!(record.split_view_evidence_count, 1);
    assert_eq!(record.missing_proof_count, 1);
    assert_eq!(record.monitor_failure_count, 1);
    assert_eq!(record.operator_signature_count, 2);
    assert_eq!(record.witness_quorum_threshold, 2);
    assert!(record.can_escalate_incident);
    assert!(record.can_report_privately);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn incident_evidence_rejects_policy_failures_bad_shapes_and_empty_incidents_without_mutation() {
    let mut store = PrototypeSealedAuditIncidentEvidenceStore::default();

    let policy_rejected = SealedAuditIncidentEvidenceWrite {
        verifier_policy_decision: SealedAuditVerifierPolicyDecision {
            accepted: false,
            reason: SealedAuditVerifierPolicyReason::ProofCacheRejected,
            plaintext_bytes_exposed: true,
            ..valid_policy_decision()
        },
        ..valid_write()
    };
    let policy = put_sealed_audit_incident_evidence_record(&mut store, policy_rejected)
        .expect("prototype incident store is infallible");
    assert_eq!(
        policy.reason,
        SealedAuditIncidentEvidenceReason::VerifierPolicyRejected
    );
    assert!(policy.plaintext_bytes_exposed);

    let bad_digest = SealedAuditIncidentEvidenceWrite {
        incident_id: &[0x61; 31],
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_incident_evidence_record(&mut store, bad_digest)
            .expect("prototype incident store is infallible"),
        SealedAuditIncidentEvidenceReason::BadRecordShape,
    );

    let empty_incident = SealedAuditIncidentEvidenceWrite {
        split_view_evidence_count: 0,
        missing_proof_count: 0,
        monitor_failure_count: 0,
        contradiction_proof_verified: false,
        ..valid_write()
    };
    assert_rejected(
        put_sealed_audit_incident_evidence_record(&mut store, empty_incident)
            .expect("prototype incident store is infallible"),
        SealedAuditIncidentEvidenceReason::NoIncidentEvidence,
    );

    assert!(store.is_empty());
}

#[test]
fn incident_evidence_rejects_unblinded_split_view_accountability_and_plaintext_state() {
    let missing_proof = SealedAuditIncidentEvidenceWrite {
        missing_proof_report_blinded: false,
        ..valid_write()
    };
    let missing = evaluate(missing_proof);
    assert_eq!(
        missing.reason,
        SealedAuditIncidentEvidenceReason::MissingProofReportRequired
    );
    assert!(missing.requires_missing_proof_report);
    assert!(missing.requires_retry_backoff);

    let split_view = SealedAuditIncidentEvidenceWrite {
        contradiction_proof_verified: false,
        ..valid_write()
    };
    let split = evaluate(split_view);
    assert_eq!(
        split.reason,
        SealedAuditIncidentEvidenceReason::SplitViewEvidenceRequired
    );
    assert!(split.requires_split_view_escalation);

    let accountability = SealedAuditIncidentEvidenceWrite {
        operator_signature_count: 1,
        ..valid_write()
    };
    let operator = evaluate(accountability);
    assert_eq!(
        operator.reason,
        SealedAuditIncidentEvidenceReason::OperatorAccountabilityRequired
    );
    assert!(operator.requires_operator_accountability);

    let plaintext = SealedAuditIncidentEvidenceWrite {
        plaintext_selector_count: 1,
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let plaintext_decision = evaluate(plaintext);
    assert_eq!(
        plaintext_decision.reason,
        SealedAuditIncidentEvidenceReason::PlaintextMetadataForbidden
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn incident_evidence_store_rejects_duplicate_incident_ids() {
    let mut store = PrototypeSealedAuditIncidentEvidenceStore::default();

    let first = put_sealed_audit_incident_evidence_record(&mut store, valid_write())
        .expect("prototype incident store is infallible");
    assert!(first.accepted);

    let duplicate = put_sealed_audit_incident_evidence_record(&mut store, valid_write())
        .expect("prototype incident store is infallible");
    assert_eq!(
        duplicate.reason,
        SealedAuditIncidentEvidenceReason::BadRecordShape
    );
    assert_eq!(duplicate.record_count, 1);

    let next = SealedAuditIncidentEvidenceWrite {
        incident_id: &NEXT_INCIDENT_ID,
        reported_at_s: 1_769_991_600,
        evidence_observed_at_s: 1_769_991_500,
        ..valid_write()
    };
    let next_decision = put_sealed_audit_incident_evidence_record(&mut store, next)
        .expect("prototype incident store is infallible");
    assert!(next_decision.accepted);
    assert_eq!(next_decision.record_count, 2);
}

#[test]
fn incident_evidence_reasons_have_stable_codes_and_labels() {
    let cases = [
        (SealedAuditIncidentEvidenceReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditIncidentEvidenceReason::VerifierPolicyRejected,
            1,
            "VERIFIER_POLICY_REJECTED",
        ),
        (
            SealedAuditIncidentEvidenceReason::NoIncidentEvidence,
            2,
            "NO_INCIDENT_EVIDENCE",
        ),
        (
            SealedAuditIncidentEvidenceReason::MissingProofReportRequired,
            3,
            "MISSING_PROOF_REPORT_REQUIRED",
        ),
        (
            SealedAuditIncidentEvidenceReason::SplitViewEvidenceRequired,
            4,
            "SPLIT_VIEW_EVIDENCE_REQUIRED",
        ),
        (
            SealedAuditIncidentEvidenceReason::OperatorAccountabilityRequired,
            5,
            "OPERATOR_ACCOUNTABILITY_REQUIRED",
        ),
        (
            SealedAuditIncidentEvidenceReason::PlaintextMetadataForbidden,
            6,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditIncidentEvidenceReason::BadRecordShape,
            7,
            "BAD_RECORD_SHAPE",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn evaluate(write: SealedAuditIncidentEvidenceWrite<'_>) -> SealedAuditIncidentEvidenceDecision {
    mercury_core::evaluate_sealed_audit_incident_evidence(write)
}

fn assert_rejected(
    decision: SealedAuditIncidentEvidenceDecision,
    reason: SealedAuditIncidentEvidenceReason,
) {
    assert_eq!(decision.reason, reason);
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert_eq!(decision.record_count, 0);
}

fn valid_write() -> SealedAuditIncidentEvidenceWrite<'static> {
    SealedAuditIncidentEvidenceWrite {
        verifier_policy_decision: valid_policy_decision(),
        incident_format_version: 1,
        incident_id: &INCIDENT_ID,
        verifier_policy_digest: &VERIFIER_POLICY_DIGEST,
        proof_cache_digest: &PROOF_CACHE_DIGEST,
        checkpoint_digest: &CHECKPOINT_DIGEST,
        witness_operator_digest: &WITNESS_OPERATOR_DIGEST,
        contradiction_digest: &CONTRADICTION_DIGEST,
        missing_proof_report_digest: &MISSING_PROOF_REPORT_DIGEST,
        monitor_report_digest: &MONITOR_REPORT_DIGEST,
        accountability_route_digest: &ACCOUNTABILITY_ROUTE_DIGEST,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        reported_at_s: 1_769_991_000,
        evidence_observed_at_s: 1_769_990_900,
        split_view_evidence_count: 1,
        missing_proof_count: 1,
        monitor_failure_count: 1,
        operator_signature_count: 2,
        witness_quorum_threshold: 2,
        incident_signature_verified: true,
        contradiction_proof_verified: true,
        missing_proof_report_blinded: true,
        monitor_report_private: true,
        accountability_route_configured: true,
        escalation_ack_required: true,
        retry_after_s: 300,
        suppression_authenticated: false,
        store_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn valid_policy_decision() -> SealedAuditVerifierPolicyDecision {
    SealedAuditVerifierPolicyDecision {
        accepted: true,
        reason: SealedAuditVerifierPolicyReason::Accepted,
        persisted_snapshot: true,
        snapshot_count: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_verify_offline: true,
        can_schedule_private_monitor: true,
        can_show_ui_status: true,
        requires_policy_refresh: false,
        requires_monitor_refresh: false,
        requires_key_rotation: false,
        escalates_split_view: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}
