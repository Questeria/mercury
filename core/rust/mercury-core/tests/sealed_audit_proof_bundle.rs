use mercury_core::{
    SealedAuditProofBundleDecision, SealedAuditProofBundleInput, SealedAuditProofBundleReason,
    SealedAuditWitnessClientDecision, SealedAuditWitnessClientReason,
    evaluate_sealed_audit_proof_bundle,
};

#[test]
fn proof_bundle_accepts_offline_verifiable_digest_only_bundle() {
    let decision = evaluate_sealed_audit_proof_bundle(valid_input());

    assert!(decision.accepted);
    assert_eq!(decision.reason, SealedAuditProofBundleReason::Accepted);
    assert_eq!(decision.event_sequence, 42);
    assert_eq!(decision.log_index, 42);
    assert_eq!(decision.checkpoint_size, 43);
    assert_eq!(decision.verifier_policy_epoch, 7);
    assert!(decision.can_verify_offline);
    assert!(decision.can_persist_proof_bundle);
    assert!(decision.can_show_ui_status);
    assert!(!decision.can_recover_proof_cache);
    assert!(!decision.requires_policy_refresh);
    assert!(!decision.requires_witness_refresh);
    assert!(!decision.requires_proof_cache_recovery);
    assert!(!decision.requires_redaction);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn proof_bundle_rejects_failed_or_privacy_leaking_witness_client_decision() {
    let client_rejected = SealedAuditProofBundleInput {
        witness_client_decision: rejected_witness_client_decision(false),
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_proof_bundle(client_rejected),
        SealedAuditProofBundleReason::WitnessClientRejected,
    );

    let client_plaintext = SealedAuditProofBundleInput {
        witness_client_decision: rejected_witness_client_decision(true),
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_proof_bundle(client_plaintext);
    assert_rejected_with(
        decision,
        SealedAuditProofBundleReason::WitnessClientRejected,
        false,
        false,
        false,
        false,
        true,
    );
}

#[test]
fn proof_bundle_rejects_verifier_policy_mismatches_and_quorum_gaps() {
    let stale_policy = SealedAuditProofBundleInput {
        verifier_policy_epoch: 6,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_proof_bundle(stale_policy),
        SealedAuditProofBundleReason::PolicyRejected,
        true,
        false,
        false,
        false,
        false,
    );

    let unpinned_witnesses = SealedAuditProofBundleInput {
        verifier_witness_key_pin_count: 1,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_proof_bundle(unpinned_witnesses),
        SealedAuditProofBundleReason::PolicyRejected,
        true,
        false,
        false,
        false,
        false,
    );

    let weak_cosignatures = SealedAuditProofBundleInput {
        verified_witness_cosignature_count: 1,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_proof_bundle(weak_cosignatures),
        SealedAuditProofBundleReason::PolicyRejected,
        true,
        false,
        false,
        false,
        false,
    );
}

#[test]
fn proof_bundle_rejects_bad_proof_shape_and_missing_consistency_evidence() {
    let bad_version = SealedAuditProofBundleInput {
        bundle_format_version: 2,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_proof_bundle(bad_version),
        SealedAuditProofBundleReason::ProofShapeRejected,
    );

    let proof_too_large = SealedAuditProofBundleInput {
        inclusion_proof_hash_count: 64,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_proof_bundle(proof_too_large),
        SealedAuditProofBundleReason::ProofShapeRejected,
    );

    let missing_consistency = SealedAuditProofBundleInput {
        consistency_proof_verified: false,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_proof_bundle(missing_consistency),
        SealedAuditProofBundleReason::ProofShapeRejected,
    );
}

#[test]
fn proof_bundle_rejects_inclusion_proof_failures() {
    let inclusion_failed = SealedAuditProofBundleInput {
        inclusion_proof_verified: false,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_proof_bundle(inclusion_failed),
        SealedAuditProofBundleReason::InclusionProofRejected,
    );

    let root_mismatch = SealedAuditProofBundleInput {
        inclusion_root_matches_checkpoint: false,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_proof_bundle(root_mismatch),
        SealedAuditProofBundleReason::InclusionProofRejected,
    );
}

#[test]
fn proof_bundle_rejects_stale_witnesses_and_missing_monitor_freshness() {
    let stale_witness = SealedAuditProofBundleInput {
        witness_timestamp_s: 1_769_990_400,
        verification_time_s: 1_769_990_400 + 901,
        max_witness_age_s: 900,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_proof_bundle(stale_witness),
        SealedAuditProofBundleReason::WitnessFreshnessRejected,
        false,
        true,
        false,
        false,
        false,
    );

    let no_freshness_check = SealedAuditProofBundleInput {
        monitor_freshness_checked: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_proof_bundle(no_freshness_check),
        SealedAuditProofBundleReason::WitnessFreshnessRejected,
        false,
        true,
        false,
        false,
        false,
    );
}

#[test]
fn proof_bundle_rejects_privacy_leaks_and_accepts_authenticated_cache_recovery() {
    let plaintext_selector = SealedAuditProofBundleInput {
        plaintext_selector_count: 1,
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_proof_bundle(plaintext_selector);
    assert_rejected_with(
        decision,
        SealedAuditProofBundleReason::PrivacyRejected,
        false,
        false,
        false,
        true,
        true,
    );

    let unauthenticated_recovery = SealedAuditProofBundleInput {
        local_proof_cache_available: false,
        proof_cache_recovery_authenticated: true,
        proof_cache_recovery_user_verified: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_proof_bundle(unauthenticated_recovery),
        SealedAuditProofBundleReason::CacheRecoveryRejected,
        false,
        false,
        true,
        false,
        false,
    );

    let authenticated_recovery = SealedAuditProofBundleInput {
        local_proof_cache_available: false,
        proof_cache_recovery_authenticated: true,
        proof_cache_recovery_user_verified: true,
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_proof_bundle(authenticated_recovery);
    assert!(decision.accepted);
    assert!(decision.can_recover_proof_cache);
    assert!(decision.requires_proof_cache_recovery);
}

#[test]
fn proof_bundle_reasons_have_stable_codes_and_labels() {
    let cases = [
        (SealedAuditProofBundleReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditProofBundleReason::WitnessClientRejected,
            1,
            "WITNESS_CLIENT_REJECTED",
        ),
        (
            SealedAuditProofBundleReason::PolicyRejected,
            2,
            "POLICY_REJECTED",
        ),
        (
            SealedAuditProofBundleReason::ProofShapeRejected,
            3,
            "PROOF_SHAPE_REJECTED",
        ),
        (
            SealedAuditProofBundleReason::InclusionProofRejected,
            4,
            "INCLUSION_PROOF_REJECTED",
        ),
        (
            SealedAuditProofBundleReason::WitnessFreshnessRejected,
            5,
            "WITNESS_FRESHNESS_REJECTED",
        ),
        (
            SealedAuditProofBundleReason::PrivacyRejected,
            6,
            "PRIVACY_REJECTED",
        ),
        (
            SealedAuditProofBundleReason::CacheRecoveryRejected,
            7,
            "CACHE_RECOVERY_REJECTED",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(decision: SealedAuditProofBundleDecision, reason: SealedAuditProofBundleReason) {
    assert_rejected_with(decision, reason, false, false, false, false, false);
}

fn assert_rejected_with(
    decision: SealedAuditProofBundleDecision,
    reason: SealedAuditProofBundleReason,
    requires_policy_refresh: bool,
    requires_witness_refresh: bool,
    requires_proof_cache_recovery: bool,
    requires_redaction: bool,
    plaintext_bytes_exposed: bool,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.can_verify_offline);
    assert!(!decision.can_persist_proof_bundle);
    assert!(!decision.can_show_ui_status);
    assert_eq!(decision.requires_policy_refresh, requires_policy_refresh);
    assert_eq!(decision.requires_witness_refresh, requires_witness_refresh);
    assert_eq!(
        decision.requires_proof_cache_recovery,
        requires_proof_cache_recovery
    );
    assert_eq!(decision.requires_redaction, requires_redaction);
    assert_eq!(decision.plaintext_bytes_exposed, plaintext_bytes_exposed);
}

fn valid_input() -> SealedAuditProofBundleInput {
    SealedAuditProofBundleInput {
        witness_client_decision: accepted_witness_client_decision(),
        bundle_format_version: 1,
        proof_bundle_persisted: true,
        proof_cache_digest_len: 32,
        proof_cache_encrypted: true,
        proof_cache_append_only: true,
        local_proof_cache_available: true,
        proof_cache_recovery_authenticated: false,
        proof_cache_recovery_user_verified: false,
        verifier_policy_snapshot_digest_len: 32,
        verifier_policy_epoch: 7,
        verifier_policy_matches_witness_policy: true,
        verifier_log_key_pin_count: 1,
        verifier_witness_key_pin_count: 3,
        verifier_witness_threshold: 2,
        verified_witness_cosignature_count: 3,
        event_sequence: 42,
        event_hash_len: 32,
        leaf_hash_len: 32,
        log_index: 42,
        checkpoint_size: 43,
        inclusion_proof_hash_count: 6,
        inclusion_proof_verified: true,
        inclusion_root_matches_checkpoint: true,
        consistency_proof_hash_count: 6,
        consistency_proof_verified: true,
        witness_timestamp_s: 1_769_990_400,
        verification_time_s: 1_769_990_430,
        max_witness_age_s: 900,
        monitor_freshness_checked: true,
        extra_data_authenticated_or_opaque: true,
        audit_subject_digest_len: 32,
        plaintext_selector_count: 0,
        ui_status_digest_only: true,
    }
}

const fn accepted_witness_client_decision() -> SealedAuditWitnessClientDecision {
    SealedAuditWitnessClientDecision {
        accepted: true,
        reason: SealedAuditWitnessClientReason::Accepted,
        checkpoint_size: 43,
        policy_epoch: 7,
        witness_quorum_threshold: 2,
        response_status_code: 200,
        can_submit_add_checkpoint: true,
        can_publish_witnessed_checkpoint: true,
        can_monitor_privately: true,
        can_retry_witness_conflict: false,
        can_alert_split_view: true,
        requires_policy_rotation: false,
        requires_witness_repair: false,
        requires_operator_alert: false,
        requires_local_recovery: false,
        plaintext_bytes_exposed: false,
    }
}

const fn rejected_witness_client_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditWitnessClientDecision {
    SealedAuditWitnessClientDecision {
        accepted: false,
        reason: SealedAuditWitnessClientReason::WitnessUnavailable,
        checkpoint_size: 43,
        policy_epoch: 7,
        witness_quorum_threshold: 2,
        response_status_code: 503,
        can_submit_add_checkpoint: false,
        can_publish_witnessed_checkpoint: false,
        can_monitor_privately: false,
        can_retry_witness_conflict: false,
        can_alert_split_view: true,
        requires_policy_rotation: false,
        requires_witness_repair: true,
        requires_operator_alert: false,
        requires_local_recovery: false,
        plaintext_bytes_exposed,
    }
}
