use mercury_core::{
    GroupChatCryptoSuite, GroupChatInput, GroupChatProtocol, MlsKeyPackageAdmissionDecision,
    MlsKeyPackageAdmissionInput, MlsKeyPackageAdmissionReason, MlsProviderSecurityInput, RoomMode,
    evaluate_group_chat, evaluate_mls_key_package_admission, evaluate_mls_provider_security,
};

#[test]
fn key_package_admission_accepts_valid_mls_key_package() {
    let decision = evaluate_mls_key_package_admission(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_add_member);
    assert!(decision.can_send_welcome);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_mls_setup);
    assert!(!decision.requires_pq_upgrade);
    assert!(!decision.requires_user_action);
    assert!(decision.prevents_key_reuse);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, MlsKeyPackageAdmissionReason::Accepted);
}

#[test]
fn key_package_admission_rejects_group_protocol_suite_and_lifetime_failures() {
    let mut rejected_group = valid_input();
    let mut group = valid_group_chat_input();
    group.membership_transition_pending = true;
    rejected_group.group_chat = evaluate_group_chat(group);
    let rejected_group_decision = rejected_group.evaluate();
    assert_rejected(
        rejected_group_decision,
        MlsKeyPackageAdmissionReason::GroupChatRejected,
    );
    assert!(rejected_group_decision.requires_sync);
    assert!(rejected_group_decision.requires_user_action);

    let mut protocol = valid_input();
    protocol.key_package_protocol_version = 2;
    let protocol_decision = protocol.evaluate();
    assert_rejected(
        protocol_decision,
        MlsKeyPackageAdmissionReason::ProtocolVersionMismatch,
    );
    assert!(protocol_decision.requires_mls_setup);

    let mut suite = valid_input();
    suite.key_package_suite = GroupChatCryptoSuite::HybridPqMls1024;
    let suite_decision = suite.evaluate();
    assert_rejected(
        suite_decision,
        MlsKeyPackageAdmissionReason::CipherSuiteMismatch,
    );
    assert!(suite_decision.requires_pq_upgrade);

    let mut bad_window = valid_input();
    bad_window.lifetime_not_after_s = bad_window.lifetime_not_before_s;
    assert_rejected(
        bad_window.evaluate(),
        MlsKeyPackageAdmissionReason::BadLifetimeWindow,
    );

    let mut too_long = valid_input();
    too_long.lifetime_not_after_s = too_long.lifetime_not_before_s + too_long.max_lifetime_s + 1;
    assert_rejected(
        too_long.evaluate(),
        MlsKeyPackageAdmissionReason::LifetimeTooLong,
    );

    let mut not_current = valid_input();
    not_current.now_s = not_current.lifetime_not_after_s;
    assert_rejected(
        not_current.evaluate(),
        MlsKeyPackageAdmissionReason::LifetimeNotCurrent,
    );
}

#[test]
fn key_package_admission_rejects_leaf_signature_credential_and_capability_failures() {
    let mut leaf = valid_input();
    leaf.leaf_node_valid = false;
    assert_rejected(
        leaf.evaluate(),
        MlsKeyPackageAdmissionReason::LeafNodeInvalid,
    );

    let mut leaf_signature = valid_input();
    leaf_signature.leaf_signature_valid = false;
    assert_rejected(
        leaf_signature.evaluate(),
        MlsKeyPackageAdmissionReason::LeafSignatureInvalid,
    );

    let mut key_package_signature = valid_input();
    key_package_signature.key_package_signature_valid = false;
    assert_rejected(
        key_package_signature.evaluate(),
        MlsKeyPackageAdmissionReason::KeyPackageSignatureInvalid,
    );

    let mut credential = valid_input();
    credential.credential_valid = false;
    assert_rejected(
        credential.evaluate(),
        MlsKeyPackageAdmissionReason::CredentialInvalid,
    );

    let mut capabilities = valid_input();
    capabilities.required_capabilities_present = false;
    assert_rejected(
        capabilities.evaluate(),
        MlsKeyPackageAdmissionReason::RequiredCapabilitiesMissing,
    );

    let mut unsupported = valid_input();
    unsupported.credential_supported_by_group = false;
    assert_rejected(
        unsupported.evaluate(),
        MlsKeyPackageAdmissionReason::CredentialUnsupported,
    );
}

#[test]
fn key_package_admission_rejects_extension_key_reuse_replay_and_plaintext_identity() {
    let mut source = valid_input();
    source.leaf_source_key_package = false;
    assert_rejected(
        source.evaluate(),
        MlsKeyPackageAdmissionReason::LeafSourceNotKeyPackage,
    );

    let mut extensions = valid_input();
    extensions.extensions_supported = false;
    assert_rejected(
        extensions.evaluate(),
        MlsKeyPackageAdmissionReason::UnsupportedExtension,
    );

    let mut reused_key = valid_input();
    reused_key.encryption_key_reuses_init_key = true;
    assert_rejected(
        reused_key.evaluate(),
        MlsKeyPackageAdmissionReason::EncryptionKeyReusesInitKey,
    );

    let mut bad_init_key = valid_input();
    bad_init_key.init_key_len = 31;
    assert_rejected(
        bad_init_key.evaluate(),
        MlsKeyPackageAdmissionReason::BadInitKey,
    );

    let mut bad_hash = valid_input();
    bad_hash.key_package_hash_len = 31;
    assert_rejected(
        bad_hash.evaluate(),
        MlsKeyPackageAdmissionReason::BadKeyPackageHash,
    );

    let mut replay = valid_input();
    replay.key_package_hash_already_used = true;
    let replay_decision = replay.evaluate();
    assert_rejected(
        replay_decision,
        MlsKeyPackageAdmissionReason::KeyPackageAlreadyUsed,
    );
    assert!(replay_decision.requires_sync);

    let mut plaintext = valid_input();
    plaintext.plaintext_identity_fields = 1;
    assert_rejected(
        plaintext.evaluate(),
        MlsKeyPackageAdmissionReason::PlaintextIdentityForbidden,
    );
}

#[test]
fn key_package_admission_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsKeyPackageAdmissionReason::Accepted, 0, "ACCEPTED"),
        (
            MlsKeyPackageAdmissionReason::GroupChatRejected,
            1,
            "GROUP_CHAT_REJECTED",
        ),
        (
            MlsKeyPackageAdmissionReason::ProtocolVersionMismatch,
            2,
            "PROTOCOL_VERSION_MISMATCH",
        ),
        (
            MlsKeyPackageAdmissionReason::CipherSuiteMismatch,
            3,
            "CIPHER_SUITE_MISMATCH",
        ),
        (
            MlsKeyPackageAdmissionReason::LeafNodeInvalid,
            4,
            "LEAF_NODE_INVALID",
        ),
        (
            MlsKeyPackageAdmissionReason::LeafSignatureInvalid,
            5,
            "LEAF_SIGNATURE_INVALID",
        ),
        (
            MlsKeyPackageAdmissionReason::KeyPackageSignatureInvalid,
            6,
            "KEY_PACKAGE_SIGNATURE_INVALID",
        ),
        (
            MlsKeyPackageAdmissionReason::CredentialInvalid,
            7,
            "CREDENTIAL_INVALID",
        ),
        (
            MlsKeyPackageAdmissionReason::RequiredCapabilitiesMissing,
            8,
            "REQUIRED_CAPABILITIES_MISSING",
        ),
        (
            MlsKeyPackageAdmissionReason::CredentialUnsupported,
            9,
            "CREDENTIAL_UNSUPPORTED",
        ),
        (
            MlsKeyPackageAdmissionReason::BadLifetimeWindow,
            10,
            "BAD_LIFETIME_WINDOW",
        ),
        (
            MlsKeyPackageAdmissionReason::LifetimeTooLong,
            11,
            "LIFETIME_TOO_LONG",
        ),
        (
            MlsKeyPackageAdmissionReason::LifetimeNotCurrent,
            12,
            "LIFETIME_NOT_CURRENT",
        ),
        (
            MlsKeyPackageAdmissionReason::LeafSourceNotKeyPackage,
            13,
            "LEAF_SOURCE_NOT_KEY_PACKAGE",
        ),
        (
            MlsKeyPackageAdmissionReason::UnsupportedExtension,
            14,
            "UNSUPPORTED_EXTENSION",
        ),
        (
            MlsKeyPackageAdmissionReason::EncryptionKeyReusesInitKey,
            15,
            "ENCRYPTION_KEY_REUSES_INIT_KEY",
        ),
        (MlsKeyPackageAdmissionReason::BadInitKey, 16, "BAD_INIT_KEY"),
        (
            MlsKeyPackageAdmissionReason::BadKeyPackageHash,
            17,
            "BAD_KEY_PACKAGE_HASH",
        ),
        (
            MlsKeyPackageAdmissionReason::KeyPackageAlreadyUsed,
            18,
            "KEY_PACKAGE_ALREADY_USED",
        ),
        (
            MlsKeyPackageAdmissionReason::PlaintextIdentityForbidden,
            19,
            "PLAINTEXT_IDENTITY_FORBIDDEN",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> MlsKeyPackageAdmissionInput {
    MlsKeyPackageAdmissionInput {
        group_chat: evaluate_group_chat(valid_group_chat_input()),
        group_protocol_version: 1,
        key_package_protocol_version: 1,
        group_suite: GroupChatCryptoSuite::HybridPqMls768,
        key_package_suite: GroupChatCryptoSuite::HybridPqMls768,
        leaf_node_valid: true,
        leaf_signature_valid: true,
        key_package_signature_valid: true,
        credential_valid: true,
        required_capabilities_present: true,
        credential_supported_by_group: true,
        lifetime_not_before_s: 1_000,
        lifetime_not_after_s: 1_300,
        now_s: 1_100,
        max_lifetime_s: 600,
        leaf_source_key_package: true,
        extensions_supported: true,
        encryption_key_reuses_init_key: false,
        init_key_len: 32,
        key_package_hash_len: 32,
        key_package_hash_already_used: false,
        plaintext_identity_fields: 0,
    }
}

fn valid_group_chat_input() -> GroupChatInput {
    GroupChatInput {
        protocol: GroupChatProtocol::Mls,
        crypto_suite: GroupChatCryptoSuite::HybridPqMls768,
        room_mode: RoomMode::Standard,
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
        mls_provider_security: evaluate_mls_provider_security(valid_mls_provider_security_input(
            GroupChatCryptoSuite::HybridPqMls768,
        )),
        plaintext_member_metadata_fields: 0,
    }
}

fn valid_mls_provider_security_input(suite: GroupChatCryptoSuite) -> MlsProviderSecurityInput {
    MlsProviderSecurityInput {
        provider_configured: true,
        selected_suite: suite,
        minimum_suite: suite,
        provider_supports_selected_suite: true,
        ml_kem_parameter_set: suite.required_ml_kem_parameter_set(),
        classical_kem_component_present: suite.requires_pq_traditional_hybrid(),
        requires_pq_signatures: matches!(suite, GroupChatCryptoSuite::HybridPqMls1024),
        pq_signature_ready: matches!(suite, GroupChatCryptoSuite::HybridPqMls1024),
        suite_id_bound_to_group_context: true,
        downgrade_evidence_verified: true,
        known_answer_tests_passed: true,
        secret_zeroization_available: true,
        unsafe_crypto_backend: false,
        plaintext_key_export_fields: 0,
    }
}

fn assert_rejected(decision: MlsKeyPackageAdmissionDecision, reason: MlsKeyPackageAdmissionReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_add_member);
    assert!(!decision.can_send_welcome);
    assert!(decision.prevents_key_reuse);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
