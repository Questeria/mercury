use mercury_core::{
    GroupChatCryptoSuite, GroupChatInput, GroupChatProtocol, MlsCommitAdmissionDecision,
    MlsCommitAdmissionInput, MlsCommitAdmissionReason, MlsCommitReplayStoreReason,
    MlsCommitReplayStoreWrite, MlsProviderSecurityInput, PrototypeMlsCommitReplayStore, RoomMode,
    evaluate_group_chat, evaluate_mls_commit_replay_store_write, evaluate_mls_provider_security,
    put_mls_commit_replay_record,
};

const GROUP_ID: [u8; 32] = [0x51; 32];
const COMMIT_HASH: [u8; 32] = [0x52; 32];
const OTHER_COMMIT_HASH: [u8; 32] = [0x53; 32];
const SHORT_DIGEST: [u8; 16] = [0x54; 16];

#[test]
fn commit_replay_store_persists_only_accepted_digest_records() {
    let mut store = PrototypeMlsCommitReplayStore::default();
    let decision =
        put_mls_commit_replay_record(&mut store, valid_write()).expect("store cannot fail");

    assert!(decision.accepted);
    assert_eq!(decision.reason, MlsCommitReplayStoreReason::Accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert!(decision.can_apply_commit_once);
    assert!(decision.can_continue_group);
    assert!(!decision.local_member_removed);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(store.len(), 1);

    let record = store
        .get(&GROUP_ID, &COMMIT_HASH)
        .expect("commit replay record should persist");
    assert_eq!(record.group_id, GROUP_ID);
    assert_eq!(record.commit_hash, COMMIT_HASH);
    assert_eq!(record.epoch, 7);
    assert_eq!(record.applied_at_s, 1_100);
    assert!(!record.local_member_removed);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn commit_replay_store_rejects_admission_gate_and_bad_shapes() {
    let rejected_gate = MlsCommitReplayStoreWrite {
        commit_admission: rejected_commit_admission(),
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_commit_replay_store_write(rejected_gate),
        MlsCommitReplayStoreReason::CommitAdmissionRejected,
    );

    let bad_group = MlsCommitReplayStoreWrite {
        group_id: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_commit_replay_store_write(bad_group),
        MlsCommitReplayStoreReason::BadGroupId,
    );

    let bad_hash = MlsCommitReplayStoreWrite {
        commit_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_commit_replay_store_write(bad_hash),
        MlsCommitReplayStoreReason::BadCommitHash,
    );

    let bad_epoch = MlsCommitReplayStoreWrite {
        epoch: 0,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_commit_replay_store_write(bad_epoch),
        MlsCommitReplayStoreReason::BadEpoch,
    );

    let bad_applied_at = MlsCommitReplayStoreWrite {
        applied_at_s: -1,
        ..valid_write()
    };
    assert_rejected(
        evaluate_mls_commit_replay_store_write(bad_applied_at),
        MlsCommitReplayStoreReason::BadAppliedAt,
    );

    let plaintext = MlsCommitReplayStoreWrite {
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let decision = evaluate_mls_commit_replay_store_write(plaintext);
    assert_rejected(
        decision,
        MlsCommitReplayStoreReason::PlaintextMetadataForbidden,
    );
    assert!(decision.plaintext_bytes_exposed);
}

#[test]
fn commit_replay_store_rejects_duplicate_commit_hashes_per_group() {
    let mut store = PrototypeMlsCommitReplayStore::default();
    let first = store.put(valid_write());
    assert!(first.accepted);

    let replay = store.put(valid_write());
    assert_rejected(replay, MlsCommitReplayStoreReason::CommitAlreadyRecorded);
    assert_eq!(replay.record_count, 1);
    assert_eq!(store.len(), 1);

    let other_commit = MlsCommitReplayStoreWrite {
        commit_hash: &OTHER_COMMIT_HASH,
        ..valid_write()
    };
    let other = store.put(other_commit);
    assert!(other.accepted);
    assert_eq!(other.record_count, 2);
    assert_eq!(store.len(), 2);
}

#[test]
fn commit_replay_store_persists_terminal_local_member_removal() {
    let mut store = PrototypeMlsCommitReplayStore::default();
    let write = MlsCommitReplayStoreWrite {
        commit_admission: local_member_removed_admission(),
        ..valid_write()
    };
    let decision = store.put(write);

    assert!(decision.accepted);
    assert!(decision.can_apply_commit_once);
    assert!(!decision.can_continue_group);
    assert!(decision.local_member_removed);

    let record = store
        .get(&GROUP_ID, &COMMIT_HASH)
        .expect("terminal replay record should persist");
    assert!(record.local_member_removed);
}

#[test]
fn commit_replay_store_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsCommitReplayStoreReason::Accepted, 0, "ACCEPTED"),
        (
            MlsCommitReplayStoreReason::CommitAdmissionRejected,
            1,
            "COMMIT_ADMISSION_REJECTED",
        ),
        (MlsCommitReplayStoreReason::BadGroupId, 2, "BAD_GROUP_ID"),
        (
            MlsCommitReplayStoreReason::BadCommitHash,
            3,
            "BAD_COMMIT_HASH",
        ),
        (MlsCommitReplayStoreReason::BadEpoch, 4, "BAD_EPOCH"),
        (
            MlsCommitReplayStoreReason::BadAppliedAt,
            5,
            "BAD_APPLIED_AT",
        ),
        (
            MlsCommitReplayStoreReason::PlaintextMetadataForbidden,
            6,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            MlsCommitReplayStoreReason::CommitAlreadyRecorded,
            7,
            "COMMIT_ALREADY_RECORDED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_write() -> MlsCommitReplayStoreWrite<'static> {
    MlsCommitReplayStoreWrite {
        commit_admission: accepted_commit_admission(),
        group_id: &GROUP_ID,
        commit_hash: &COMMIT_HASH,
        epoch: 7,
        applied_at_s: 1_100,
        plaintext_metadata_fields: 0,
    }
}

fn accepted_commit_admission() -> MlsCommitAdmissionDecision {
    valid_commit_admission_input().evaluate()
}

fn local_member_removed_admission() -> MlsCommitAdmissionDecision {
    let mut input = valid_commit_admission_input();
    input.removes_local_member = true;
    input.evaluate()
}

fn rejected_commit_admission() -> MlsCommitAdmissionDecision {
    MlsCommitAdmissionDecision {
        accepted: false,
        reason: MlsCommitAdmissionReason::BadCommitHash,
        can_apply_commit: false,
        can_initialize_epoch: false,
        can_continue_group: false,
        local_member_removed: false,
        requires_sync: true,
        requires_mls_setup: false,
        requires_tree_repair: false,
        requires_rekey: false,
        requires_user_action: false,
        prevents_commit_replay: true,
        forbids_plaintext_commit_metadata: true,
        plaintext_bytes_exposed: false,
    }
}

fn valid_commit_admission_input() -> MlsCommitAdmissionInput {
    MlsCommitAdmissionInput {
        group_chat: evaluate_group_chat(valid_group_chat_input()),
        current_epoch: 7,
        commit_epoch: 7,
        external_commit: false,
        sender_is_member: true,
        sender_type_new_member_commit: false,
        external_init_present: false,
        commit_signature_valid: true,
        commit_membership_tag_valid: true,
        proposal_list_valid: true,
        referenced_proposals_available: true,
        application_policy_accepts_proposals: true,
        duplicate_update_or_remove_targets: false,
        committer_update_present: false,
        committer_remove_present: false,
        path_required: true,
        update_path_present: true,
        update_path_leaf_valid: true,
        update_path_leaf_source_commit: true,
        update_path_parent_hash_valid: true,
        update_path_secret_decryptable: true,
        ratchet_tree_hash_matches: true,
        provisional_group_context_bound: true,
        epoch_secret_derived: true,
        confirmed_transcript_hash_len: 32,
        confirmation_tag_valid: true,
        commit_won_tie_break: true,
        commit_hash_len: 32,
        commit_hash_already_processed: false,
        removes_local_member: false,
        plaintext_commit_metadata_fields: 0,
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
    decision: mercury_core::MlsCommitReplayStoreDecision,
    reason: MlsCommitReplayStoreReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert!(!decision.can_apply_commit_once);
    assert!(!decision.can_continue_group);
    assert!(!decision.local_member_removed);
    assert!(decision.keeps_digest_only);
    assert_eq!(decision.reason, reason);
}
