use mercury_core::{
    GroupChatCryptoSuite, GroupChatInput, GroupChatProtocol, MlsKeyPackageAdmissionInput,
    MlsProviderSecurityInput, MlsWelcomeAdmissionDecision, MlsWelcomeAdmissionReason,
    MlsWelcomeReplayStoreReason, MlsWelcomeReplayStoreWrite, PrototypeMlsWelcomeReplayStore,
    RoomMode, evaluate_group_chat, evaluate_mls_key_package_admission,
    evaluate_mls_provider_security, evaluate_mls_welcome_replay_store_write,
    put_mls_welcome_replay_record,
};

const GROUP_ID: [u8; 32] = [0x61; 32];
const WELCOME_HASH: [u8; 32] = [0x62; 32];
const OTHER_WELCOME_HASH: [u8; 32] = [0x63; 32];
const CONSUMED_KEY_PACKAGE_REF: [u8; 32] = [0x64; 32];
const OTHER_CONSUMED_KEY_PACKAGE_REF: [u8; 32] = [0x65; 32];
const TREE_HASH: [u8; 32] = [0x66; 32];
const CONFIRMED_TRANSCRIPT_HASH: [u8; 32] = [0x67; 32];
const GROUP_STATE_COMMIT_DIGEST: [u8; 32] = [0x68; 32];
const SHORT_DIGEST: [u8; 16] = [0x69; 16];

#[test]
fn welcome_replay_store_persists_only_accepted_digest_records() {
    let mut store = PrototypeMlsWelcomeReplayStore::default();
    let decision =
        put_mls_welcome_replay_record(&mut store, valid_write()).expect("store cannot fail");

    assert!(decision.accepted);
    assert_eq!(decision.reason, MlsWelcomeReplayStoreReason::Accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert!(decision.can_initialize_group_once);
    assert!(decision.can_open_group);
    assert!(decision.consumes_key_package);
    assert!(decision.deletes_init_key);
    assert!(decision.binds_tree_hash);
    assert!(decision.binds_confirmed_transcript_hash);
    assert!(decision.commits_group_state_transactionally);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(store.len(), 1);

    let record = store
        .get(&GROUP_ID, &WELCOME_HASH)
        .expect("welcome replay record should persist");
    assert_eq!(record.group_id, GROUP_ID);
    assert_eq!(record.welcome_hash, WELCOME_HASH);
    assert_eq!(record.consumed_key_package_ref, CONSUMED_KEY_PACKAGE_REF);
    assert_eq!(record.tree_hash, TREE_HASH);
    assert_eq!(record.confirmed_transcript_hash, CONFIRMED_TRANSCRIPT_HASH);
    assert_eq!(record.group_state_commit_digest, GROUP_STATE_COMMIT_DIGEST);
    assert_eq!(record.epoch, 8);
    assert_eq!(record.joined_at_s, 1_100);
    assert!(record.init_key_deleted);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn welcome_replay_store_rejects_admission_gate_and_bad_shapes() {
    let rejected_gate = MlsWelcomeReplayStoreWrite {
        welcome_admission: rejected_welcome_admission(),
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(rejected_gate),
        MlsWelcomeReplayStoreReason::WelcomeAdmissionRejected,
    );

    let bad_group = MlsWelcomeReplayStoreWrite {
        group_id: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(bad_group),
        MlsWelcomeReplayStoreReason::BadGroupId,
    );

    let bad_hash = MlsWelcomeReplayStoreWrite {
        welcome_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(bad_hash),
        MlsWelcomeReplayStoreReason::BadWelcomeHash,
    );

    let bad_key_package_ref = MlsWelcomeReplayStoreWrite {
        consumed_key_package_ref: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(bad_key_package_ref),
        MlsWelcomeReplayStoreReason::BadConsumedKeyPackageRef,
    );

    let bad_tree_hash = MlsWelcomeReplayStoreWrite {
        tree_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(bad_tree_hash),
        MlsWelcomeReplayStoreReason::BadTreeHash,
    );

    let bad_transcript_hash = MlsWelcomeReplayStoreWrite {
        confirmed_transcript_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(bad_transcript_hash),
        MlsWelcomeReplayStoreReason::BadConfirmedTranscriptHash,
    );

    let bad_state_digest = MlsWelcomeReplayStoreWrite {
        group_state_commit_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(bad_state_digest),
        MlsWelcomeReplayStoreReason::BadGroupStateCommitDigest,
    );

    let bad_epoch = MlsWelcomeReplayStoreWrite {
        epoch: 0,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(bad_epoch),
        MlsWelcomeReplayStoreReason::BadEpoch,
    );

    let bad_joined_at = MlsWelcomeReplayStoreWrite {
        joined_at_s: -1,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(bad_joined_at),
        MlsWelcomeReplayStoreReason::BadJoinedAt,
    );

    let init_key_not_deleted = MlsWelcomeReplayStoreWrite {
        init_key_deleted: false,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(init_key_not_deleted),
        MlsWelcomeReplayStoreReason::InitKeyNotDeleted,
    );

    let state_not_committed = MlsWelcomeReplayStoreWrite {
        group_state_committed: false,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_welcome_replay_store_write(state_not_committed),
        MlsWelcomeReplayStoreReason::GroupStateNotCommitted,
    );

    let plaintext = MlsWelcomeReplayStoreWrite {
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let decision = evaluate_mls_welcome_replay_store_write(plaintext);
    assert_rejected(
        decision,
        MlsWelcomeReplayStoreReason::PlaintextMetadataForbidden,
    );
    assert!(decision.plaintext_bytes_exposed);
}

#[test]
fn welcome_replay_store_rejects_duplicate_welcome_hashes_per_group() {
    let mut store = PrototypeMlsWelcomeReplayStore::default();
    let first = store.put(valid_write());
    assert!(first.accepted);

    let replay = store.put(valid_write());
    assert_rejected(replay, MlsWelcomeReplayStoreReason::WelcomeAlreadyRecorded);
    assert_eq!(replay.record_count, 1);
    assert_eq!(store.len(), 1);

    let other_welcome = MlsWelcomeReplayStoreWrite {
        welcome_hash: &OTHER_WELCOME_HASH,
        consumed_key_package_ref: &OTHER_CONSUMED_KEY_PACKAGE_REF,
        ..valid_write()
    };
    let other = store.put(other_welcome);
    assert!(other.accepted);
    assert_eq!(other.record_count, 2);
    assert_eq!(store.len(), 2);
}

#[test]
fn welcome_replay_store_rejects_consumed_key_package_refs() {
    let mut store = PrototypeMlsWelcomeReplayStore::default();
    let first = store.put(valid_write());
    assert!(first.accepted);

    let reused_key_package = MlsWelcomeReplayStoreWrite {
        welcome_hash: &OTHER_WELCOME_HASH,
        ..valid_write()
    };
    let replay = store.put(reused_key_package);
    assert_rejected(
        replay,
        MlsWelcomeReplayStoreReason::KeyPackageAlreadyConsumed,
    );
    assert_eq!(replay.record_count, 1);
    assert_eq!(store.len(), 1);
}

#[test]
fn welcome_replay_store_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsWelcomeReplayStoreReason::Accepted, 0, "ACCEPTED"),
        (
            MlsWelcomeReplayStoreReason::WelcomeAdmissionRejected,
            1,
            "WELCOME_ADMISSION_REJECTED",
        ),
        (MlsWelcomeReplayStoreReason::BadGroupId, 2, "BAD_GROUP_ID"),
        (
            MlsWelcomeReplayStoreReason::BadWelcomeHash,
            3,
            "BAD_WELCOME_HASH",
        ),
        (
            MlsWelcomeReplayStoreReason::BadConsumedKeyPackageRef,
            4,
            "BAD_CONSUMED_KEY_PACKAGE_REF",
        ),
        (MlsWelcomeReplayStoreReason::BadTreeHash, 5, "BAD_TREE_HASH"),
        (
            MlsWelcomeReplayStoreReason::BadConfirmedTranscriptHash,
            6,
            "BAD_CONFIRMED_TRANSCRIPT_HASH",
        ),
        (
            MlsWelcomeReplayStoreReason::BadGroupStateCommitDigest,
            7,
            "BAD_GROUP_STATE_COMMIT_DIGEST",
        ),
        (MlsWelcomeReplayStoreReason::BadEpoch, 8, "BAD_EPOCH"),
        (MlsWelcomeReplayStoreReason::BadJoinedAt, 9, "BAD_JOINED_AT"),
        (
            MlsWelcomeReplayStoreReason::InitKeyNotDeleted,
            10,
            "INIT_KEY_NOT_DELETED",
        ),
        (
            MlsWelcomeReplayStoreReason::GroupStateNotCommitted,
            11,
            "GROUP_STATE_NOT_COMMITTED",
        ),
        (
            MlsWelcomeReplayStoreReason::PlaintextMetadataForbidden,
            12,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            MlsWelcomeReplayStoreReason::WelcomeAlreadyRecorded,
            13,
            "WELCOME_ALREADY_RECORDED",
        ),
        (
            MlsWelcomeReplayStoreReason::KeyPackageAlreadyConsumed,
            14,
            "KEY_PACKAGE_ALREADY_CONSUMED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_write() -> MlsWelcomeReplayStoreWrite<'static> {
    MlsWelcomeReplayStoreWrite {
        welcome_admission: accepted_welcome_admission(),
        group_id: &GROUP_ID,
        welcome_hash: &WELCOME_HASH,
        consumed_key_package_ref: &CONSUMED_KEY_PACKAGE_REF,
        tree_hash: &TREE_HASH,
        confirmed_transcript_hash: &CONFIRMED_TRANSCRIPT_HASH,
        group_state_commit_digest: &GROUP_STATE_COMMIT_DIGEST,
        epoch: 8,
        joined_at_s: 1_100,
        init_key_deleted: true,
        group_state_committed: true,
        plaintext_metadata_fields: 0,
    }
}

fn accepted_welcome_admission() -> MlsWelcomeAdmissionDecision {
    valid_welcome_admission_input().evaluate()
}

fn rejected_welcome_admission() -> MlsWelcomeAdmissionDecision {
    MlsWelcomeAdmissionDecision {
        accepted: false,
        reason: MlsWelcomeAdmissionReason::BadWelcomeHash,
        can_join_group: false,
        can_initialize_epoch: false,
        can_open_group: false,
        requires_sync: true,
        requires_mls_setup: false,
        requires_pq_upgrade: false,
        requires_user_action: false,
        requires_tree_fetch: false,
        prevents_welcome_replay: true,
        forbids_plaintext_group_metadata: true,
        plaintext_bytes_exposed: false,
    }
}

fn valid_welcome_admission_input() -> mercury_core::MlsWelcomeAdmissionInput {
    mercury_core::MlsWelcomeAdmissionInput {
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

fn assert_rejected(
    decision: mercury_core::MlsWelcomeReplayStoreDecision,
    reason: MlsWelcomeReplayStoreReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert!(!decision.can_initialize_group_once);
    assert!(!decision.can_open_group);
    assert!(!decision.consumes_key_package);
    assert!(!decision.deletes_init_key);
    assert!(!decision.binds_tree_hash);
    assert!(!decision.binds_confirmed_transcript_hash);
    assert!(!decision.commits_group_state_transactionally);
    assert!(decision.keeps_digest_only);
    assert_eq!(decision.reason, reason);
}
