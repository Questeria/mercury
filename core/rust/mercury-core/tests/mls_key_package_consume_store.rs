use mercury_core::{
    GroupChatCryptoSuite, GroupChatInput, GroupChatProtocol, MlsKeyPackageAdmissionDecision,
    MlsKeyPackageAdmissionInput, MlsKeyPackageAdmissionReason, MlsKeyPackageConsumeStoreReason,
    MlsKeyPackageConsumeStoreWrite, MlsProviderSecurityInput, PrototypeMlsKeyPackageConsumeStore,
    RoomMode, evaluate_group_chat, evaluate_mls_key_package_admission,
    evaluate_mls_key_package_consume_store_write, evaluate_mls_provider_security,
    put_mls_key_package_consumption_record,
};

const GROUP_ID: [u8; 32] = [0x71; 32];
const OTHER_GROUP_ID: [u8; 32] = [0x72; 32];
const KEY_PACKAGE_HASH: [u8; 32] = [0x73; 32];
const OTHER_KEY_PACKAGE_HASH: [u8; 32] = [0x74; 32];
const ADDED_MEMBER_REF: [u8; 32] = [0x75; 32];
const WELCOME_SEND_TRANSACTION_DIGEST: [u8; 32] = [0x76; 32];
const SHORT_DIGEST: [u8; 16] = [0x77; 16];

#[test]
fn key_package_consume_store_persists_only_accepted_digest_records() {
    let mut store = PrototypeMlsKeyPackageConsumeStore::default();
    let decision = put_mls_key_package_consumption_record(&mut store, valid_write())
        .expect("store cannot fail");

    assert!(decision.accepted);
    assert_eq!(decision.reason, MlsKeyPackageConsumeStoreReason::Accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert!(decision.can_consume_key_package_once);
    assert!(decision.can_send_welcome_once);
    assert!(decision.prevents_key_package_reuse);
    assert!(decision.binds_added_member_ref);
    assert!(decision.binds_welcome_send_transaction);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(store.len(), 1);

    let record = store
        .get(&KEY_PACKAGE_HASH)
        .expect("key package consumption record should persist");
    assert_eq!(record.group_id, GROUP_ID);
    assert_eq!(record.key_package_hash, KEY_PACKAGE_HASH);
    assert_eq!(record.added_member_ref, ADDED_MEMBER_REF);
    assert_eq!(
        record.welcome_send_transaction_digest,
        WELCOME_SEND_TRANSACTION_DIGEST
    );
    assert_eq!(record.consumed_at_s, 1_100);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn key_package_consume_store_rejects_admission_gate_and_bad_shapes() {
    let rejected_gate = MlsKeyPackageConsumeStoreWrite {
        key_package_admission: rejected_key_package_admission(),
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_key_package_consume_store_write(rejected_gate),
        MlsKeyPackageConsumeStoreReason::KeyPackageAdmissionRejected,
    );

    let bad_group = MlsKeyPackageConsumeStoreWrite {
        group_id: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_key_package_consume_store_write(bad_group),
        MlsKeyPackageConsumeStoreReason::BadGroupId,
    );

    let bad_hash = MlsKeyPackageConsumeStoreWrite {
        key_package_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_key_package_consume_store_write(bad_hash),
        MlsKeyPackageConsumeStoreReason::BadKeyPackageHash,
    );

    let bad_member_ref = MlsKeyPackageConsumeStoreWrite {
        added_member_ref: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_key_package_consume_store_write(bad_member_ref),
        MlsKeyPackageConsumeStoreReason::BadAddedMemberRef,
    );

    let bad_welcome_send_transaction = MlsKeyPackageConsumeStoreWrite {
        welcome_send_transaction_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_key_package_consume_store_write(bad_welcome_send_transaction),
        MlsKeyPackageConsumeStoreReason::BadWelcomeSendTransactionDigest,
    );

    let bad_consumed_at = MlsKeyPackageConsumeStoreWrite {
        consumed_at_s: -1,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_key_package_consume_store_write(bad_consumed_at),
        MlsKeyPackageConsumeStoreReason::BadConsumedAt,
    );

    let plaintext = MlsKeyPackageConsumeStoreWrite {
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let decision = evaluate_mls_key_package_consume_store_write(plaintext);
    assert_rejected(
        decision,
        MlsKeyPackageConsumeStoreReason::PlaintextMetadataForbidden,
    );
    assert!(decision.plaintext_bytes_exposed);
}

#[test]
fn key_package_consume_store_rejects_duplicate_key_package_hashes_globally() {
    let mut store = PrototypeMlsKeyPackageConsumeStore::default();
    let first = store.put(valid_write());
    assert!(first.accepted);

    let replay_same_group = store.put(valid_write());
    assert_rejected(
        replay_same_group,
        MlsKeyPackageConsumeStoreReason::KeyPackageAlreadyConsumed,
    );
    assert_eq!(replay_same_group.record_count, 1);
    assert_eq!(store.len(), 1);

    let replay_other_group = MlsKeyPackageConsumeStoreWrite {
        group_id: &OTHER_GROUP_ID,
        ..valid_write()
    };
    let global_replay = store.put(replay_other_group);
    assert_rejected(
        global_replay,
        MlsKeyPackageConsumeStoreReason::KeyPackageAlreadyConsumed,
    );
    assert_eq!(global_replay.record_count, 1);
    assert_eq!(store.len(), 1);

    let other_key_package = MlsKeyPackageConsumeStoreWrite {
        group_id: &OTHER_GROUP_ID,
        key_package_hash: &OTHER_KEY_PACKAGE_HASH,
        ..valid_write()
    };
    let other = store.put(other_key_package);
    assert!(other.accepted);
    assert_eq!(other.record_count, 2);
    assert_eq!(store.len(), 2);
}

#[test]
fn key_package_consume_store_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsKeyPackageConsumeStoreReason::Accepted, 0, "ACCEPTED"),
        (
            MlsKeyPackageConsumeStoreReason::KeyPackageAdmissionRejected,
            1,
            "KEY_PACKAGE_ADMISSION_REJECTED",
        ),
        (
            MlsKeyPackageConsumeStoreReason::BadGroupId,
            2,
            "BAD_GROUP_ID",
        ),
        (
            MlsKeyPackageConsumeStoreReason::BadKeyPackageHash,
            3,
            "BAD_KEY_PACKAGE_HASH",
        ),
        (
            MlsKeyPackageConsumeStoreReason::BadAddedMemberRef,
            4,
            "BAD_ADDED_MEMBER_REF",
        ),
        (
            MlsKeyPackageConsumeStoreReason::BadWelcomeSendTransactionDigest,
            5,
            "BAD_WELCOME_SEND_TRANSACTION_DIGEST",
        ),
        (
            MlsKeyPackageConsumeStoreReason::BadConsumedAt,
            6,
            "BAD_CONSUMED_AT",
        ),
        (
            MlsKeyPackageConsumeStoreReason::PlaintextMetadataForbidden,
            7,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            MlsKeyPackageConsumeStoreReason::KeyPackageAlreadyConsumed,
            8,
            "KEY_PACKAGE_ALREADY_CONSUMED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_write() -> MlsKeyPackageConsumeStoreWrite<'static> {
    MlsKeyPackageConsumeStoreWrite {
        key_package_admission: accepted_key_package_admission(),
        group_id: &GROUP_ID,
        key_package_hash: &KEY_PACKAGE_HASH,
        added_member_ref: &ADDED_MEMBER_REF,
        welcome_send_transaction_digest: &WELCOME_SEND_TRANSACTION_DIGEST,
        consumed_at_s: 1_100,
        plaintext_metadata_fields: 0,
    }
}

fn accepted_key_package_admission() -> MlsKeyPackageAdmissionDecision {
    evaluate_mls_key_package_admission(valid_key_package_admission_input())
}

fn rejected_key_package_admission() -> MlsKeyPackageAdmissionDecision {
    MlsKeyPackageAdmissionDecision {
        accepted: false,
        reason: MlsKeyPackageAdmissionReason::BadKeyPackageHash,
        can_add_member: false,
        can_send_welcome: false,
        requires_sync: false,
        requires_mls_setup: true,
        requires_pq_upgrade: false,
        requires_user_action: true,
        prevents_key_reuse: true,
        plaintext_bytes_exposed: false,
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
    decision: mercury_core::MlsKeyPackageConsumeStoreDecision,
    reason: MlsKeyPackageConsumeStoreReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert!(!decision.can_consume_key_package_once);
    assert!(!decision.can_send_welcome_once);
    assert!(decision.prevents_key_package_reuse);
    assert!(!decision.binds_added_member_ref);
    assert!(!decision.binds_welcome_send_transaction);
    assert!(decision.keeps_digest_only);
    assert_eq!(decision.reason, reason);
}
