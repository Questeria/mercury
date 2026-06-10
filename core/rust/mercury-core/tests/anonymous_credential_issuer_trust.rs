use mercury_core::{
    AnonymousCredentialIssuerTrustInput, AnonymousCredentialIssuerTrustReason,
    AnonymousIssuerWitnessAuditDecision, AnonymousIssuerWitnessAuditInput,
    AnonymousIssuerWitnessAuditReason, KeyTransparencyDecision, KeyTransparencyProofInput,
    KeyTransparencyProofStatus, KeyTransparencyReason, KeyTransparencyState,
    KeyTransparencyWitnessStatus, evaluate_anonymous_credential_issuer_trust,
    evaluate_anonymous_issuer_witness_audit, evaluate_key_transparency,
};

#[test]
fn issuer_trust_accepts_consistent_fresh_global_key_set() {
    let decision = evaluate_anonymous_credential_issuer_trust(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_issue_or_verify_tokens);
    assert!(decision.can_use_for_anonymous_membership_proof);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_rekey);
    assert!(!decision.requires_user_action);
    assert!(decision.protects_anonymity_set);
    assert_eq!(
        decision.reason,
        AnonymousCredentialIssuerTrustReason::Accepted
    );
}

#[test]
fn issuer_trust_requires_consistent_transparency_and_directory_inclusion() {
    let mut missing_transparency = valid_input();
    missing_transparency.key_transparency = KeyTransparencyDecision {
        state: KeyTransparencyState::MissingProof,
        reason: KeyTransparencyReason::InclusionProofMissing,
        requires_user_action: true,
    };
    let missing_transparency_decision = missing_transparency.evaluate();
    assert_rejected(
        missing_transparency_decision,
        AnonymousCredentialIssuerTrustReason::KeyTransparencyRequired,
    );
    assert!(missing_transparency_decision.requires_sync);
    assert!(missing_transparency_decision.requires_user_action);

    let mut missing_directory = valid_input();
    missing_directory.issuer_directory_inclusion_verified = false;
    let missing_directory_decision = missing_directory.evaluate();
    assert_rejected(
        missing_directory_decision,
        AnonymousCredentialIssuerTrustReason::IssuerDirectoryMissing,
    );
    assert!(missing_directory_decision.requires_sync);
}

#[test]
fn issuer_trust_requires_accepted_witness_audit() {
    let mut input = valid_input();
    input.issuer_witness_audit = AnonymousIssuerWitnessAuditDecision {
        accepted: false,
        reason: AnonymousIssuerWitnessAuditReason::SplitViewReported,
        can_use_issuer_key: false,
        has_witness_quorum: false,
        detects_split_view: false,
        requires_sync: false,
        requires_rekey: true,
        requires_user_action: true,
        protects_anonymity_set: false,
        plaintext_bytes_exposed: false,
    };
    let decision = input.evaluate();

    assert_rejected(
        decision,
        AnonymousCredentialIssuerTrustReason::IssuerWitnessAuditRejected,
    );
    assert!(decision.requires_rekey);
    assert!(decision.requires_user_action);
}

#[test]
fn issuer_trust_rejects_bad_key_ids_and_stale_directories() {
    let mut bad_key = valid_input();
    bad_key.issuer_key_id_len = 16;
    let bad_key_decision = bad_key.evaluate();
    assert_rejected(
        bad_key_decision,
        AnonymousCredentialIssuerTrustReason::BadIssuerKeyId,
    );
    assert!(bad_key_decision.requires_user_action);

    let mut stale = valid_input();
    stale.directory_age_s = 301;
    stale.max_directory_age_s = 300;
    let stale_decision = stale.evaluate();
    assert_rejected(
        stale_decision,
        AnonymousCredentialIssuerTrustReason::DirectoryStale,
    );
    assert!(stale_decision.requires_sync);
}

#[test]
fn issuer_trust_rejects_partitioning_key_sets_and_metadata() {
    let mut too_many_active_keys = valid_input();
    too_many_active_keys.active_issuer_key_count = 9;
    too_many_active_keys.max_active_issuer_key_count = 8;
    let too_many_decision = too_many_active_keys.evaluate();
    assert_rejected(
        too_many_decision,
        AnonymousCredentialIssuerTrustReason::ActiveKeySetPartitioningRisk,
    );
    assert!(too_many_decision.requires_rekey);
    assert!(too_many_decision.requires_user_action);

    let mut partitioning_metadata = valid_input();
    partitioning_metadata.opaque_partitioning_metadata_bits = 1;
    let metadata_decision = partitioning_metadata.evaluate();
    assert_rejected(
        metadata_decision,
        AnonymousCredentialIssuerTrustReason::PartitioningMetadataPresent,
    );
    assert!(metadata_decision.requires_rekey);
    assert!(metadata_decision.requires_user_action);
}

#[test]
fn issuer_trust_rejects_invalid_key_windows_and_revocation_state() {
    let mut not_yet_valid = valid_input();
    not_yet_valid.now_s = 900;
    let not_yet_valid_decision = not_yet_valid.evaluate();
    assert_rejected(
        not_yet_valid_decision,
        AnonymousCredentialIssuerTrustReason::KeyNotYetValid,
    );
    assert!(not_yet_valid_decision.requires_sync);

    let mut expired = valid_input();
    expired.now_s = 1300;
    let expired_decision = expired.evaluate();
    assert_rejected(
        expired_decision,
        AnonymousCredentialIssuerTrustReason::KeyExpired,
    );
    assert!(expired_decision.requires_sync);
    assert!(expired_decision.requires_rekey);

    let mut stale_revocation = valid_input();
    stale_revocation.revocation_status_fresh = false;
    let stale_revocation_decision = stale_revocation.evaluate();
    assert_rejected(
        stale_revocation_decision,
        AnonymousCredentialIssuerTrustReason::RevocationStatusStale,
    );
    assert!(stale_revocation_decision.requires_sync);

    let mut revoked = valid_input();
    revoked.issuer_key_revoked = true;
    let revoked_decision = revoked.evaluate();
    assert_rejected(
        revoked_decision,
        AnonymousCredentialIssuerTrustReason::KeyRevoked,
    );
    assert!(revoked_decision.requires_rekey);
    assert!(revoked_decision.requires_user_action);
}

#[test]
fn issuer_trust_requires_challenge_key_binding() {
    let mut unbound = valid_input();
    unbound.issuer_key_bound_to_challenge = false;
    let decision = unbound.evaluate();

    assert_rejected(
        decision,
        AnonymousCredentialIssuerTrustReason::ChallengeKeyBindingMissing,
    );
    assert!(decision.requires_user_action);
}

#[test]
fn issuer_trust_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (
            AnonymousCredentialIssuerTrustReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            AnonymousCredentialIssuerTrustReason::KeyTransparencyRequired,
            1,
            "KEY_TRANSPARENCY_REQUIRED",
        ),
        (
            AnonymousCredentialIssuerTrustReason::BadIssuerKeyId,
            2,
            "BAD_ISSUER_KEY_ID",
        ),
        (
            AnonymousCredentialIssuerTrustReason::IssuerDirectoryMissing,
            3,
            "ISSUER_DIRECTORY_MISSING",
        ),
        (
            AnonymousCredentialIssuerTrustReason::DirectoryStale,
            4,
            "DIRECTORY_STALE",
        ),
        (
            AnonymousCredentialIssuerTrustReason::ActiveKeySetPartitioningRisk,
            5,
            "ACTIVE_KEY_SET_PARTITIONING_RISK",
        ),
        (
            AnonymousCredentialIssuerTrustReason::KeyNotYetValid,
            6,
            "KEY_NOT_YET_VALID",
        ),
        (
            AnonymousCredentialIssuerTrustReason::KeyExpired,
            7,
            "KEY_EXPIRED",
        ),
        (
            AnonymousCredentialIssuerTrustReason::RevocationStatusStale,
            8,
            "REVOCATION_STATUS_STALE",
        ),
        (
            AnonymousCredentialIssuerTrustReason::KeyRevoked,
            9,
            "KEY_REVOKED",
        ),
        (
            AnonymousCredentialIssuerTrustReason::ChallengeKeyBindingMissing,
            10,
            "CHALLENGE_KEY_BINDING_MISSING",
        ),
        (
            AnonymousCredentialIssuerTrustReason::PartitioningMetadataPresent,
            11,
            "PARTITIONING_METADATA_PRESENT",
        ),
        (
            AnonymousCredentialIssuerTrustReason::IssuerWitnessAuditRejected,
            12,
            "ISSUER_WITNESS_AUDIT_REJECTED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> AnonymousCredentialIssuerTrustInput {
    AnonymousCredentialIssuerTrustInput {
        key_transparency: evaluate_key_transparency(valid_transparency_proof()),
        issuer_witness_audit: evaluate_anonymous_issuer_witness_audit(valid_witness_audit()),
        issuer_key_id_len: 32,
        issuer_directory_inclusion_verified: true,
        issuer_key_bound_to_challenge: true,
        active_issuer_key_count: 2,
        max_active_issuer_key_count: 8,
        directory_age_s: 60,
        max_directory_age_s: 300,
        key_not_before_s: 1000,
        key_not_after_s: 1300,
        now_s: 1100,
        revocation_status_fresh: true,
        issuer_key_revoked: false,
        opaque_partitioning_metadata_bits: 0,
    }
}

fn valid_witness_audit() -> AnonymousIssuerWitnessAuditInput {
    AnonymousIssuerWitnessAuditInput {
        key_transparency: evaluate_key_transparency(valid_transparency_proof()),
        signed_tree_head_len: 32,
        inclusion_root_len: 32,
        previous_tree_size: 12,
        current_tree_size: 13,
        required_witness_count: 2,
        verified_witness_count: 3,
        independent_operator_count: 2,
        audit_age_s: 60,
        max_audit_age_s: 300,
        split_view_reports: 0,
        auditor_signature_len: 64,
        plaintext_partitioning_fields: 0,
    }
}

fn valid_transparency_proof() -> KeyTransparencyProofInput {
    KeyTransparencyProofInput {
        inclusion: KeyTransparencyProofStatus::Verified,
        consistency: KeyTransparencyProofStatus::Verified,
        key_history: KeyTransparencyProofStatus::Verified,
        witness: KeyTransparencyWitnessStatus::QuorumSatisfied,
        require_witness: true,
        previous_tree_size: 12,
        current_tree_size: 13,
        proof_age_s: 60,
        max_proof_age_s: 300,
    }
}

fn assert_rejected(
    decision: mercury_core::AnonymousCredentialIssuerTrustDecision,
    reason: AnonymousCredentialIssuerTrustReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_issue_or_verify_tokens);
    assert!(!decision.can_use_for_anonymous_membership_proof);
    assert!(!decision.protects_anonymity_set);
    assert_eq!(decision.reason, reason);
}
