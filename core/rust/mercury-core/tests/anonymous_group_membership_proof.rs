use mercury_core::{
    AnonymousCredentialIssuerTrustInput, AnonymousCredentialIssuerTrustReason,
    AnonymousGroupMembershipProofInput, AnonymousGroupMembershipProofReason,
    AnonymousGroupMembershipProofScheme, AnonymousIssuerWitnessAuditInput, GroupChatCryptoSuite,
    GroupChatInput, GroupChatProtocol, KeyTransparencyProofInput, KeyTransparencyProofStatus,
    KeyTransparencyWitnessStatus, MlsProviderSecurityInput, RoomMode,
    evaluate_anonymous_credential_issuer_trust, evaluate_anonymous_group_membership_proof,
    evaluate_anonymous_issuer_witness_audit, evaluate_group_chat, evaluate_key_transparency,
    evaluate_mls_provider_security,
};

#[test]
fn anonymous_group_membership_proof_accepts_bound_unlinkable_presentation() {
    let decision = evaluate_anonymous_group_membership_proof(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_authenticate_member);
    assert!(decision.can_redeem_once);
    assert!(decision.can_rate_limit_anonymously);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_rekey);
    assert!(!decision.requires_user_action);
    assert!(decision.forbids_plaintext_member_identity);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(
        decision.reason,
        AnonymousGroupMembershipProofReason::Accepted
    );
}

#[test]
fn anonymous_group_membership_proof_requires_group_and_high_security_pq_posture() {
    let mut rejected_group = valid_input();
    rejected_group.group_chat = {
        let mut group = valid_group_chat_input();
        group.room_state_available = false;
        group.evaluate()
    };
    let rejected_group_decision = rejected_group.evaluate();
    assert_rejected(
        rejected_group_decision,
        AnonymousGroupMembershipProofReason::GroupRejected,
    );
    assert!(rejected_group_decision.requires_sync);

    let mut high_security = valid_input();
    high_security.high_security_room = true;
    high_security.scheme = AnonymousGroupMembershipProofScheme::BbsUnlinkablePresentation;
    high_security.scheme_post_quantum_safe = false;
    let high_security_decision = high_security.evaluate();
    assert_rejected(
        high_security_decision,
        AnonymousGroupMembershipProofReason::HighSecurityRequiresPqProof,
    );
    assert!(high_security_decision.requires_rekey);
    assert!(high_security_decision.requires_user_action);
}

#[test]
fn anonymous_group_membership_proof_requires_trusted_issuer() {
    let mut stale_issuer = valid_input();
    stale_issuer.issuer_trust = {
        let mut issuer = valid_issuer_trust_input();
        issuer.directory_age_s = 600;
        issuer.evaluate()
    };
    let stale_issuer_decision = stale_issuer.evaluate();

    assert_rejected(
        stale_issuer_decision,
        AnonymousGroupMembershipProofReason::IssuerTrustRejected,
    );
    assert!(stale_issuer_decision.requires_sync);

    let mut revoked_issuer = valid_input();
    revoked_issuer.issuer_trust = {
        let mut issuer = valid_issuer_trust_input();
        issuer.issuer_key_revoked = true;
        issuer.evaluate()
    };
    let revoked_issuer_decision = revoked_issuer.evaluate();

    assert_rejected(
        revoked_issuer_decision,
        AnonymousGroupMembershipProofReason::IssuerTrustRejected,
    );
    assert!(revoked_issuer_decision.requires_rekey);
    assert!(revoked_issuer_decision.requires_user_action);
    assert_eq!(
        revoked_issuer.issuer_trust.reason,
        AnonymousCredentialIssuerTrustReason::KeyRevoked
    );
}

#[test]
fn anonymous_group_membership_proof_requires_bound_challenge_nonce_and_proof() {
    let mut bad_issuer = valid_input();
    bad_issuer.issuer_key_id_len = 0;
    assert_rejected(
        bad_issuer.evaluate(),
        AnonymousGroupMembershipProofReason::BadIssuerKey,
    );

    let mut bad_challenge = valid_input();
    bad_challenge.challenge_digest_len = 16;
    assert_rejected(
        bad_challenge.evaluate(),
        AnonymousGroupMembershipProofReason::BadChallengeDigest,
    );

    let mut bad_nonce = valid_input();
    bad_nonce.presentation_nonce_len = 0;
    assert_rejected(
        bad_nonce.evaluate(),
        AnonymousGroupMembershipProofReason::BadPresentationNonce,
    );

    let mut missing_proof = valid_input();
    missing_proof.proof_len = 0;
    assert_rejected(
        missing_proof.evaluate(),
        AnonymousGroupMembershipProofReason::ProofMissing,
    );

    let mut unbound_header = valid_input();
    unbound_header.presentation_header_bound = false;
    assert_rejected(
        unbound_header.evaluate(),
        AnonymousGroupMembershipProofReason::PresentationHeaderNotBound,
    );
}

#[test]
fn anonymous_group_membership_proof_requires_epoch_route_replay_and_freshness() {
    let mut unbound_epoch = valid_input();
    unbound_epoch.group_epoch_bound = false;
    assert_rejected(
        unbound_epoch.evaluate(),
        AnonymousGroupMembershipProofReason::GroupEpochNotBound,
    );

    let mut unbound_route = valid_input();
    unbound_route.route_bound = false;
    assert_rejected(
        unbound_route.evaluate(),
        AnonymousGroupMembershipProofReason::RouteNotBound,
    );

    let mut missing_nullifier = valid_input();
    missing_nullifier.replay_nullifier_len = 0;
    assert_rejected(
        missing_nullifier.evaluate(),
        AnonymousGroupMembershipProofReason::ReplayNullifierMissing,
    );

    let mut replayed = valid_input();
    replayed.replay_nullifier_seen = true;
    assert_rejected(
        replayed.evaluate(),
        AnonymousGroupMembershipProofReason::ReplayNullifierAlreadySeen,
    );

    let mut expired = valid_input();
    expired.now_s = expired.expires_at_s;
    assert_rejected(
        expired.evaluate(),
        AnonymousGroupMembershipProofReason::ProofExpired,
    );
}

#[test]
fn anonymous_group_membership_proof_rejects_plaintext_member_identity() {
    let mut input = valid_input();
    input.plaintext_member_identifier_fields = 1;

    assert_rejected(
        input.evaluate(),
        AnonymousGroupMembershipProofReason::PlaintextMemberIdentity,
    );
}

#[test]
fn anonymous_group_membership_proof_reasons_and_schemes_have_stable_codes_and_labels() {
    let schemes = [
        (
            AnonymousGroupMembershipProofScheme::PrivacyPassVoprf,
            1,
            "privacy_pass_voprf",
        ),
        (
            AnonymousGroupMembershipProofScheme::BbsUnlinkablePresentation,
            2,
            "bbs_unlinkable_presentation",
        ),
        (
            AnonymousGroupMembershipProofScheme::KvacPrivateGroup,
            3,
            "kvac_private_group",
        ),
        (
            AnonymousGroupMembershipProofScheme::PqGroupWrapper,
            4,
            "pq_group_wrapper",
        ),
    ];

    for (scheme, code, label) in schemes {
        assert_eq!(scheme.code(), code);
        assert_eq!(scheme.label(), label);
    }

    let reasons = [
        (AnonymousGroupMembershipProofReason::Accepted, 0, "ACCEPTED"),
        (
            AnonymousGroupMembershipProofReason::GroupRejected,
            1,
            "GROUP_REJECTED",
        ),
        (
            AnonymousGroupMembershipProofReason::BadIssuerKey,
            2,
            "BAD_ISSUER_KEY",
        ),
        (
            AnonymousGroupMembershipProofReason::BadChallengeDigest,
            3,
            "BAD_CHALLENGE_DIGEST",
        ),
        (
            AnonymousGroupMembershipProofReason::BadPresentationNonce,
            4,
            "BAD_PRESENTATION_NONCE",
        ),
        (
            AnonymousGroupMembershipProofReason::ProofMissing,
            5,
            "PROOF_MISSING",
        ),
        (
            AnonymousGroupMembershipProofReason::PresentationHeaderNotBound,
            6,
            "PRESENTATION_HEADER_NOT_BOUND",
        ),
        (
            AnonymousGroupMembershipProofReason::GroupEpochNotBound,
            7,
            "GROUP_EPOCH_NOT_BOUND",
        ),
        (
            AnonymousGroupMembershipProofReason::RouteNotBound,
            8,
            "ROUTE_NOT_BOUND",
        ),
        (
            AnonymousGroupMembershipProofReason::ReplayNullifierMissing,
            9,
            "REPLAY_NULLIFIER_MISSING",
        ),
        (
            AnonymousGroupMembershipProofReason::ReplayNullifierAlreadySeen,
            10,
            "REPLAY_NULLIFIER_ALREADY_SEEN",
        ),
        (
            AnonymousGroupMembershipProofReason::ProofExpired,
            11,
            "PROOF_EXPIRED",
        ),
        (
            AnonymousGroupMembershipProofReason::PlaintextMemberIdentity,
            12,
            "PLAINTEXT_MEMBER_IDENTITY",
        ),
        (
            AnonymousGroupMembershipProofReason::HighSecurityRequiresPqProof,
            13,
            "HIGH_SECURITY_REQUIRES_PQ_PROOF",
        ),
        (
            AnonymousGroupMembershipProofReason::IssuerTrustRejected,
            14,
            "ISSUER_TRUST_REJECTED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> AnonymousGroupMembershipProofInput {
    AnonymousGroupMembershipProofInput {
        group_chat: evaluate_group_chat(valid_group_chat_input()),
        scheme: AnonymousGroupMembershipProofScheme::PqGroupWrapper,
        issuer_trust: evaluate_anonymous_credential_issuer_trust(valid_issuer_trust_input()),
        high_security_room: true,
        scheme_post_quantum_safe: true,
        issuer_key_id_len: 32,
        challenge_digest_len: 32,
        presentation_nonce_len: 32,
        proof_len: 128,
        presentation_header_bound: true,
        group_epoch_bound: true,
        route_bound: true,
        replay_nullifier_len: 32,
        replay_nullifier_seen: false,
        issued_at_s: 1000,
        expires_at_s: 1300,
        now_s: 1100,
        plaintext_member_identifier_fields: 0,
    }
}

fn valid_group_chat_input() -> GroupChatInput {
    GroupChatInput {
        protocol: GroupChatProtocol::Mls,
        crypto_suite: GroupChatCryptoSuite::HybridPqMls1024,
        room_mode: RoomMode::HighSecurity,
        member_count: 5,
        active_member_devices: 5,
        local_device_is_member: true,
        room_state_available: true,
        group_secret_sealed: true,
        membership_transition_pending: false,
        current_epoch: 7,
        local_epoch: 7,
        key_transparency_ready: true,
        mls_provider_configured: true,
        mls_provider_security: evaluate_mls_provider_security(MlsProviderSecurityInput {
            provider_configured: true,
            selected_suite: GroupChatCryptoSuite::HybridPqMls1024,
            minimum_suite: GroupChatCryptoSuite::HybridPqMls1024,
            provider_supports_selected_suite: true,
            ml_kem_parameter_set: 1024,
            classical_kem_component_present: true,
            requires_pq_signatures: true,
            pq_signature_ready: true,
            suite_id_bound_to_group_context: true,
            downgrade_evidence_verified: true,
            known_answer_tests_passed: true,
            secret_zeroization_available: true,
            unsafe_crypto_backend: false,
            plaintext_key_export_fields: 0,
        }),
        plaintext_member_metadata_fields: 0,
    }
}

fn valid_issuer_trust_input() -> AnonymousCredentialIssuerTrustInput {
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
    decision: mercury_core::AnonymousGroupMembershipProofDecision,
    reason: AnonymousGroupMembershipProofReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_authenticate_member);
    assert!(!decision.can_redeem_once);
    assert!(!decision.can_rate_limit_anonymously);
    assert!(decision.forbids_plaintext_member_identity);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
