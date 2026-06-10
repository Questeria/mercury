use mercury_core::{
    GroupChatCryptoSuite, GroupChatInput, GroupChatProtocol, MlsKeyPackageAdmissionInput,
    MlsProviderSecurityInput, MlsWelcomeAdmissionDecision, MlsWelcomeAdmissionInput,
    MlsWelcomeAdmissionReason, RoomMode, evaluate_group_chat, evaluate_mls_key_package_admission,
    evaluate_mls_provider_security, evaluate_mls_welcome_admission,
};

#[test]
fn welcome_admission_accepts_verified_group_info_and_tree() {
    let decision = evaluate_mls_welcome_admission(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_join_group);
    assert!(decision.can_initialize_epoch);
    assert!(decision.can_open_group);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_mls_setup);
    assert!(!decision.requires_pq_upgrade);
    assert!(!decision.requires_user_action);
    assert!(!decision.requires_tree_fetch);
    assert!(decision.prevents_welcome_replay);
    assert!(decision.forbids_plaintext_group_metadata);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, MlsWelcomeAdmissionReason::Accepted);
}

#[test]
fn welcome_admission_rejects_key_package_secrets_suite_and_psk_failures() {
    let mut key_package = valid_input();
    let mut key_package_input = valid_key_package_admission_input();
    key_package_input.key_package_hash_already_used = true;
    key_package.key_package_admission = evaluate_mls_key_package_admission(key_package_input);
    let key_package_decision = key_package.evaluate();
    assert_rejected(
        key_package_decision,
        MlsWelcomeAdmissionReason::KeyPackageAdmissionRejected,
    );
    assert!(key_package_decision.requires_sync);

    let mut missing_secrets = valid_input();
    missing_secrets.matching_encrypted_group_secrets = false;
    assert_rejected(
        missing_secrets.evaluate(),
        MlsWelcomeAdmissionReason::NoMatchingEncryptedGroupSecrets,
    );

    let mut suite = valid_input();
    suite.welcome_cipher_suite = GroupChatCryptoSuite::HybridPqMls1024;
    let suite_decision = suite.evaluate();
    assert_rejected(
        suite_decision,
        MlsWelcomeAdmissionReason::CipherSuiteMismatch,
    );
    assert!(suite_decision.requires_pq_upgrade);

    let mut decrypt = valid_input();
    decrypt.group_secrets_decrypted = false;
    assert_rejected(
        decrypt.evaluate(),
        MlsWelcomeAdmissionReason::GroupSecretsDecryptFailed,
    );

    let mut psk_missing = valid_input();
    psk_missing.psks_available = false;
    assert_rejected(
        psk_missing.evaluate(),
        MlsWelcomeAdmissionReason::PskUnavailable,
    );

    let mut psk_count = valid_input();
    psk_count.resumption_psk_count = 2;
    assert_rejected(
        psk_count.evaluate(),
        MlsWelcomeAdmissionReason::TooManyResumptionPsks,
    );
}

#[test]
fn welcome_admission_rejects_group_info_and_ratchet_tree_failures() {
    let mut group_info = valid_input();
    group_info.encrypted_group_info_decrypted = false;
    assert_rejected(
        group_info.evaluate(),
        MlsWelcomeAdmissionReason::GroupInfoDecryptFailed,
    );

    let mut signature = valid_input();
    signature.group_info_signature_valid = false;
    assert_rejected(
        signature.evaluate(),
        MlsWelcomeAdmissionReason::GroupInfoSignatureInvalid,
    );

    let mut group_id = valid_input();
    group_id.group_id_unique_locally = false;
    let group_id_decision = group_id.evaluate();
    assert_rejected(
        group_id_decision,
        MlsWelcomeAdmissionReason::GroupIdAlreadyInUse,
    );
    assert!(group_id_decision.requires_sync);

    let mut missing_tree = valid_input();
    missing_tree.ratchet_tree_available_confidentially = false;
    let missing_tree_decision = missing_tree.evaluate();
    assert_rejected(
        missing_tree_decision,
        MlsWelcomeAdmissionReason::RatchetTreeMissing,
    );
    assert!(missing_tree_decision.requires_tree_fetch);

    let mut tree_hash = valid_input();
    tree_hash.ratchet_tree_hash_matches = false;
    assert_rejected(
        tree_hash.evaluate(),
        MlsWelcomeAdmissionReason::RatchetTreeHashMismatch,
    );

    let mut parent_hash = valid_input();
    parent_hash.ratchet_tree_parent_hash_valid = false;
    assert_rejected(
        parent_hash.evaluate(),
        MlsWelcomeAdmissionReason::RatchetTreeParentHashInvalid,
    );

    let mut leaves = valid_input();
    leaves.ratchet_tree_leaves_valid = false;
    assert_rejected(
        leaves.evaluate(),
        MlsWelcomeAdmissionReason::RatchetTreeLeafInvalid,
    );

    let mut unmerged = valid_input();
    unmerged.ratchet_tree_unmerged_leaves_valid = false;
    assert_rejected(
        unmerged.evaluate(),
        MlsWelcomeAdmissionReason::RatchetTreeUnmergedLeavesInvalid,
    );

    let mut key_reuse = valid_input();
    key_reuse.ratchet_tree_unique_encryption_keys = false;
    assert_rejected(
        key_reuse.evaluate(),
        MlsWelcomeAdmissionReason::RatchetTreeEncryptionKeyReuse,
    );
}

#[test]
fn welcome_admission_rejects_leaf_path_confirmation_replay_and_plaintext_failures() {
    let mut own_leaf = valid_input();
    own_leaf.own_leaf_found = false;
    assert_rejected(
        own_leaf.evaluate(),
        MlsWelcomeAdmissionReason::OwnLeafMissing,
    );

    let mut own_leaf_match = valid_input();
    own_leaf_match.own_leaf_matches_key_package = false;
    assert_rejected(
        own_leaf_match.evaluate(),
        MlsWelcomeAdmissionReason::OwnLeafMismatch,
    );

    let mut path = valid_input();
    path.path_secret_valid = false;
    assert_rejected(
        path.evaluate(),
        MlsWelcomeAdmissionReason::PathSecretInvalid,
    );

    let mut epoch_secret = valid_input();
    epoch_secret.epoch_secret_derived = false;
    assert_rejected(
        epoch_secret.evaluate(),
        MlsWelcomeAdmissionReason::EpochSecretDerivationFailed,
    );

    let mut transcript = valid_input();
    transcript.confirmed_transcript_hash_len = 31;
    assert_rejected(
        transcript.evaluate(),
        MlsWelcomeAdmissionReason::BadConfirmedTranscriptHash,
    );

    let mut confirmation = valid_input();
    confirmation.confirmation_tag_valid = false;
    assert_rejected(
        confirmation.evaluate(),
        MlsWelcomeAdmissionReason::ConfirmationTagInvalid,
    );

    let mut tie_break = valid_input();
    tie_break.commit_won_tie_break = false;
    assert_rejected(
        tie_break.evaluate(),
        MlsWelcomeAdmissionReason::CommitTieBreakRejected,
    );

    let mut bad_epoch = valid_input();
    bad_epoch.group_epoch = 0;
    assert_rejected(bad_epoch.evaluate(), MlsWelcomeAdmissionReason::BadEpoch);

    let mut reinit = valid_input();
    reinit.reinit_psk_used = true;
    reinit.reinit_epoch_is_one = false;
    assert_rejected(
        reinit.evaluate(),
        MlsWelcomeAdmissionReason::ReinitPskEpochMismatch,
    );

    let mut bad_hash = valid_input();
    bad_hash.welcome_hash_len = 31;
    assert_rejected(
        bad_hash.evaluate(),
        MlsWelcomeAdmissionReason::BadWelcomeHash,
    );

    let mut replay = valid_input();
    replay.welcome_hash_already_processed = true;
    assert_rejected(
        replay.evaluate(),
        MlsWelcomeAdmissionReason::WelcomeAlreadyProcessed,
    );

    let mut plaintext = valid_input();
    plaintext.plaintext_group_metadata_fields = 1;
    let plaintext_decision = plaintext.evaluate();
    assert_rejected(
        plaintext_decision,
        MlsWelcomeAdmissionReason::PlaintextGroupMetadataForbidden,
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn welcome_admission_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsWelcomeAdmissionReason::Accepted, 0, "ACCEPTED"),
        (
            MlsWelcomeAdmissionReason::KeyPackageAdmissionRejected,
            1,
            "KEY_PACKAGE_ADMISSION_REJECTED",
        ),
        (
            MlsWelcomeAdmissionReason::NoMatchingEncryptedGroupSecrets,
            2,
            "NO_MATCHING_ENCRYPTED_GROUP_SECRETS",
        ),
        (
            MlsWelcomeAdmissionReason::CipherSuiteMismatch,
            3,
            "CIPHER_SUITE_MISMATCH",
        ),
        (
            MlsWelcomeAdmissionReason::GroupSecretsDecryptFailed,
            4,
            "GROUP_SECRETS_DECRYPT_FAILED",
        ),
        (
            MlsWelcomeAdmissionReason::PskUnavailable,
            5,
            "PSK_UNAVAILABLE",
        ),
        (
            MlsWelcomeAdmissionReason::TooManyResumptionPsks,
            6,
            "TOO_MANY_RESUMPTION_PSKS",
        ),
        (
            MlsWelcomeAdmissionReason::GroupInfoDecryptFailed,
            7,
            "GROUP_INFO_DECRYPT_FAILED",
        ),
        (
            MlsWelcomeAdmissionReason::GroupInfoSignatureInvalid,
            8,
            "GROUP_INFO_SIGNATURE_INVALID",
        ),
        (
            MlsWelcomeAdmissionReason::GroupIdAlreadyInUse,
            9,
            "GROUP_ID_ALREADY_IN_USE",
        ),
        (
            MlsWelcomeAdmissionReason::RatchetTreeMissing,
            10,
            "RATCHET_TREE_MISSING",
        ),
        (
            MlsWelcomeAdmissionReason::RatchetTreeHashMismatch,
            11,
            "RATCHET_TREE_HASH_MISMATCH",
        ),
        (
            MlsWelcomeAdmissionReason::RatchetTreeParentHashInvalid,
            12,
            "RATCHET_TREE_PARENT_HASH_INVALID",
        ),
        (
            MlsWelcomeAdmissionReason::RatchetTreeLeafInvalid,
            13,
            "RATCHET_TREE_LEAF_INVALID",
        ),
        (
            MlsWelcomeAdmissionReason::RatchetTreeUnmergedLeavesInvalid,
            14,
            "RATCHET_TREE_UNMERGED_LEAVES_INVALID",
        ),
        (
            MlsWelcomeAdmissionReason::RatchetTreeEncryptionKeyReuse,
            15,
            "RATCHET_TREE_ENCRYPTION_KEY_REUSE",
        ),
        (
            MlsWelcomeAdmissionReason::OwnLeafMissing,
            16,
            "OWN_LEAF_MISSING",
        ),
        (
            MlsWelcomeAdmissionReason::OwnLeafMismatch,
            17,
            "OWN_LEAF_MISMATCH",
        ),
        (
            MlsWelcomeAdmissionReason::PathSecretInvalid,
            18,
            "PATH_SECRET_INVALID",
        ),
        (
            MlsWelcomeAdmissionReason::EpochSecretDerivationFailed,
            19,
            "EPOCH_SECRET_DERIVATION_FAILED",
        ),
        (
            MlsWelcomeAdmissionReason::BadConfirmedTranscriptHash,
            20,
            "BAD_CONFIRMED_TRANSCRIPT_HASH",
        ),
        (
            MlsWelcomeAdmissionReason::ConfirmationTagInvalid,
            21,
            "CONFIRMATION_TAG_INVALID",
        ),
        (
            MlsWelcomeAdmissionReason::CommitTieBreakRejected,
            22,
            "COMMIT_TIE_BREAK_REJECTED",
        ),
        (MlsWelcomeAdmissionReason::BadEpoch, 23, "BAD_EPOCH"),
        (
            MlsWelcomeAdmissionReason::ReinitPskEpochMismatch,
            24,
            "REINIT_PSK_EPOCH_MISMATCH",
        ),
        (
            MlsWelcomeAdmissionReason::BadWelcomeHash,
            25,
            "BAD_WELCOME_HASH",
        ),
        (
            MlsWelcomeAdmissionReason::WelcomeAlreadyProcessed,
            26,
            "WELCOME_ALREADY_PROCESSED",
        ),
        (
            MlsWelcomeAdmissionReason::PlaintextGroupMetadataForbidden,
            27,
            "PLAINTEXT_GROUP_METADATA_FORBIDDEN",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> MlsWelcomeAdmissionInput {
    MlsWelcomeAdmissionInput {
        key_package_admission: evaluate_mls_key_package_admission(
            valid_key_package_admission_input(),
        ),
        welcome_cipher_suite: GroupChatCryptoSuite::HybridPqMls768,
        key_package_suite: GroupChatCryptoSuite::HybridPqMls768,
        group_info_suite: GroupChatCryptoSuite::HybridPqMls768,
        matching_encrypted_group_secrets: true,
        group_secrets_decrypted: true,
        psks_available: true,
        resumption_psk_count: 0,
        encrypted_group_info_decrypted: true,
        group_info_signature_valid: true,
        group_id_unique_locally: true,
        ratchet_tree_available_confidentially: true,
        ratchet_tree_hash_matches: true,
        ratchet_tree_parent_hash_valid: true,
        ratchet_tree_leaves_valid: true,
        ratchet_tree_unmerged_leaves_valid: true,
        ratchet_tree_unique_encryption_keys: true,
        own_leaf_found: true,
        own_leaf_matches_key_package: true,
        path_secret_valid: true,
        epoch_secret_derived: true,
        confirmed_transcript_hash_len: 32,
        confirmation_tag_valid: true,
        commit_won_tie_break: true,
        group_epoch: 8,
        reinit_psk_used: false,
        reinit_epoch_is_one: false,
        welcome_hash_len: 32,
        welcome_hash_already_processed: false,
        plaintext_group_metadata_fields: 0,
    }
}

fn valid_key_package_admission_input() -> MlsKeyPackageAdmissionInput {
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

fn assert_rejected(decision: MlsWelcomeAdmissionDecision, reason: MlsWelcomeAdmissionReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_join_group);
    assert!(!decision.can_initialize_epoch);
    assert!(!decision.can_open_group);
    assert!(decision.prevents_welcome_replay);
    assert!(decision.forbids_plaintext_group_metadata);
    assert_eq!(decision.reason, reason);
}
