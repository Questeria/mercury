use mercury_core::{
    GroupChatCryptoSuite, GroupChatInput, GroupChatProtocol, MlsCommitAdmissionDecision,
    MlsCommitAdmissionInput, MlsCommitAdmissionReason, MlsProviderSecurityInput, RoomMode,
    evaluate_group_chat, evaluate_mls_commit_admission, evaluate_mls_provider_security,
};

#[test]
fn commit_admission_accepts_verified_current_epoch_commit() {
    let decision = evaluate_mls_commit_admission(valid_input());

    assert!(decision.accepted);
    assert_eq!(decision.reason, MlsCommitAdmissionReason::Accepted);
    assert!(decision.can_apply_commit);
    assert!(decision.can_initialize_epoch);
    assert!(decision.can_continue_group);
    assert!(!decision.local_member_removed);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_mls_setup);
    assert!(!decision.requires_tree_repair);
    assert!(!decision.requires_rekey);
    assert!(!decision.requires_user_action);
    assert!(decision.prevents_commit_replay);
    assert!(decision.forbids_plaintext_commit_metadata);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn commit_admission_rejects_group_epoch_sender_and_auth_failures() {
    let mut group = valid_group_chat_input();
    group.membership_transition_pending = true;
    let mut group_rejected = valid_input();
    group_rejected.group_chat = group.evaluate();
    let group_decision = group_rejected.evaluate();
    assert_rejected(group_decision, MlsCommitAdmissionReason::GroupChatRejected);
    assert!(group_decision.requires_sync);

    let mut bad_epoch = valid_input();
    bad_epoch.commit_epoch += 1;
    let epoch_decision = bad_epoch.evaluate();
    assert_rejected(epoch_decision, MlsCommitAdmissionReason::BadEpoch);
    assert!(epoch_decision.requires_sync);

    let mut sender = valid_input();
    sender.sender_is_member = false;
    assert_rejected(
        sender.evaluate(),
        MlsCommitAdmissionReason::CommitSenderNotMember,
    );

    let mut external = valid_input();
    external.external_commit = true;
    external.sender_type_new_member_commit = false;
    external.external_init_present = true;
    assert_rejected(
        external.evaluate(),
        MlsCommitAdmissionReason::ExternalCommitSenderInvalid,
    );

    let mut auth = valid_input();
    auth.commit_signature_valid = false;
    let auth_decision = auth.evaluate();
    assert_rejected(
        auth_decision,
        MlsCommitAdmissionReason::CommitAuthenticationInvalid,
    );
    assert!(auth_decision.requires_mls_setup);
}

#[test]
fn commit_admission_rejects_proposal_policy_failures() {
    let mut proposal_list = valid_input();
    proposal_list.proposal_list_valid = false;
    assert_rejected(
        proposal_list.evaluate(),
        MlsCommitAdmissionReason::ProposalListInvalid,
    );

    let mut missing_reference = valid_input();
    missing_reference.referenced_proposals_available = false;
    assert_rejected(
        missing_reference.evaluate(),
        MlsCommitAdmissionReason::ReferencedProposalMissing,
    );

    let mut policy = valid_input();
    policy.application_policy_accepts_proposals = false;
    assert_rejected(
        policy.evaluate(),
        MlsCommitAdmissionReason::ApplicationPolicyRejected,
    );

    let mut duplicate_target = valid_input();
    duplicate_target.duplicate_update_or_remove_targets = true;
    assert_rejected(
        duplicate_target.evaluate(),
        MlsCommitAdmissionReason::DuplicateProposalTargets,
    );

    let mut committer_update = valid_input();
    committer_update.committer_update_present = true;
    assert_rejected(
        committer_update.evaluate(),
        MlsCommitAdmissionReason::CommitterUpdateForbidden,
    );

    let mut committer_remove = valid_input();
    committer_remove.committer_remove_present = true;
    assert_rejected(
        committer_remove.evaluate(),
        MlsCommitAdmissionReason::CommitterRemoveForbidden,
    );
}

#[test]
fn commit_admission_rejects_path_transcript_replay_and_plaintext_failures() {
    let mut missing_path = valid_input();
    missing_path.update_path_present = false;
    let missing_path_decision = missing_path.evaluate();
    assert_rejected(
        missing_path_decision,
        MlsCommitAdmissionReason::PathRequiredMissing,
    );
    assert!(missing_path_decision.requires_tree_repair);

    let mut leaf = valid_input();
    leaf.update_path_leaf_valid = false;
    assert_rejected(
        leaf.evaluate(),
        MlsCommitAdmissionReason::UpdatePathLeafInvalid,
    );

    let mut leaf_source = valid_input();
    leaf_source.update_path_leaf_source_commit = false;
    assert_rejected(
        leaf_source.evaluate(),
        MlsCommitAdmissionReason::UpdatePathLeafSourceInvalid,
    );

    let mut parent_hash = valid_input();
    parent_hash.update_path_parent_hash_valid = false;
    assert_rejected(
        parent_hash.evaluate(),
        MlsCommitAdmissionReason::UpdatePathParentHashInvalid,
    );

    let mut secret = valid_input();
    secret.update_path_secret_decryptable = false;
    assert_rejected(
        secret.evaluate(),
        MlsCommitAdmissionReason::UpdatePathSecretDecryptFailed,
    );

    let mut tree = valid_input();
    tree.ratchet_tree_hash_matches = false;
    assert_rejected(
        tree.evaluate(),
        MlsCommitAdmissionReason::RatchetTreeHashMismatch,
    );

    let mut context = valid_input();
    context.provisional_group_context_bound = false;
    assert_rejected(
        context.evaluate(),
        MlsCommitAdmissionReason::ProvisionalGroupContextMismatch,
    );

    let mut epoch_secret = valid_input();
    epoch_secret.epoch_secret_derived = false;
    assert_rejected(
        epoch_secret.evaluate(),
        MlsCommitAdmissionReason::EpochSecretDerivationFailed,
    );

    let mut transcript = valid_input();
    transcript.confirmed_transcript_hash_len = 31;
    assert_rejected(
        transcript.evaluate(),
        MlsCommitAdmissionReason::BadConfirmedTranscriptHash,
    );

    let mut confirmation = valid_input();
    confirmation.confirmation_tag_valid = false;
    assert_rejected(
        confirmation.evaluate(),
        MlsCommitAdmissionReason::ConfirmationTagInvalid,
    );

    let mut tie_break = valid_input();
    tie_break.commit_won_tie_break = false;
    assert_rejected(
        tie_break.evaluate(),
        MlsCommitAdmissionReason::CommitTieBreakRejected,
    );

    let mut bad_hash = valid_input();
    bad_hash.commit_hash_len = 31;
    assert_rejected(bad_hash.evaluate(), MlsCommitAdmissionReason::BadCommitHash);

    let mut replay = valid_input();
    replay.commit_hash_already_processed = true;
    assert_rejected(
        replay.evaluate(),
        MlsCommitAdmissionReason::CommitAlreadyProcessed,
    );

    let mut plaintext = valid_input();
    plaintext.plaintext_commit_metadata_fields = 1;
    let plaintext_decision = plaintext.evaluate();
    assert_rejected(
        plaintext_decision,
        MlsCommitAdmissionReason::PlaintextCommitMetadataForbidden,
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn commit_admission_allows_terminal_local_member_removal() {
    let mut removed = valid_input();
    removed.removes_local_member = true;
    let decision = removed.evaluate();

    assert!(decision.accepted);
    assert_eq!(
        decision.reason,
        MlsCommitAdmissionReason::LocalMemberRemoved
    );
    assert!(decision.can_apply_commit);
    assert!(!decision.can_initialize_epoch);
    assert!(!decision.can_continue_group);
    assert!(decision.local_member_removed);
    assert!(decision.requires_rekey);
    assert!(decision.requires_user_action);
    assert!(decision.prevents_commit_replay);
    assert!(decision.forbids_plaintext_commit_metadata);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn commit_admission_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsCommitAdmissionReason::Accepted, 0, "ACCEPTED"),
        (
            MlsCommitAdmissionReason::LocalMemberRemoved,
            1,
            "LOCAL_MEMBER_REMOVED",
        ),
        (
            MlsCommitAdmissionReason::GroupChatRejected,
            2,
            "GROUP_CHAT_REJECTED",
        ),
        (MlsCommitAdmissionReason::BadEpoch, 3, "BAD_EPOCH"),
        (
            MlsCommitAdmissionReason::CommitSenderNotMember,
            4,
            "COMMIT_SENDER_NOT_MEMBER",
        ),
        (
            MlsCommitAdmissionReason::ExternalCommitSenderInvalid,
            5,
            "EXTERNAL_COMMIT_SENDER_INVALID",
        ),
        (
            MlsCommitAdmissionReason::CommitAuthenticationInvalid,
            6,
            "COMMIT_AUTHENTICATION_INVALID",
        ),
        (
            MlsCommitAdmissionReason::ProposalListInvalid,
            7,
            "PROPOSAL_LIST_INVALID",
        ),
        (
            MlsCommitAdmissionReason::ReferencedProposalMissing,
            8,
            "REFERENCED_PROPOSAL_MISSING",
        ),
        (
            MlsCommitAdmissionReason::ApplicationPolicyRejected,
            9,
            "APPLICATION_POLICY_REJECTED",
        ),
        (
            MlsCommitAdmissionReason::DuplicateProposalTargets,
            10,
            "DUPLICATE_PROPOSAL_TARGETS",
        ),
        (
            MlsCommitAdmissionReason::CommitterUpdateForbidden,
            11,
            "COMMITTER_UPDATE_FORBIDDEN",
        ),
        (
            MlsCommitAdmissionReason::CommitterRemoveForbidden,
            12,
            "COMMITTER_REMOVE_FORBIDDEN",
        ),
        (
            MlsCommitAdmissionReason::PathRequiredMissing,
            13,
            "PATH_REQUIRED_MISSING",
        ),
        (
            MlsCommitAdmissionReason::UpdatePathLeafInvalid,
            14,
            "UPDATE_PATH_LEAF_INVALID",
        ),
        (
            MlsCommitAdmissionReason::UpdatePathLeafSourceInvalid,
            15,
            "UPDATE_PATH_LEAF_SOURCE_INVALID",
        ),
        (
            MlsCommitAdmissionReason::UpdatePathParentHashInvalid,
            16,
            "UPDATE_PATH_PARENT_HASH_INVALID",
        ),
        (
            MlsCommitAdmissionReason::UpdatePathSecretDecryptFailed,
            17,
            "UPDATE_PATH_SECRET_DECRYPT_FAILED",
        ),
        (
            MlsCommitAdmissionReason::RatchetTreeHashMismatch,
            18,
            "RATCHET_TREE_HASH_MISMATCH",
        ),
        (
            MlsCommitAdmissionReason::ProvisionalGroupContextMismatch,
            19,
            "PROVISIONAL_GROUP_CONTEXT_MISMATCH",
        ),
        (
            MlsCommitAdmissionReason::EpochSecretDerivationFailed,
            20,
            "EPOCH_SECRET_DERIVATION_FAILED",
        ),
        (
            MlsCommitAdmissionReason::BadConfirmedTranscriptHash,
            21,
            "BAD_CONFIRMED_TRANSCRIPT_HASH",
        ),
        (
            MlsCommitAdmissionReason::ConfirmationTagInvalid,
            22,
            "CONFIRMATION_TAG_INVALID",
        ),
        (
            MlsCommitAdmissionReason::CommitTieBreakRejected,
            23,
            "COMMIT_TIE_BREAK_REJECTED",
        ),
        (
            MlsCommitAdmissionReason::BadCommitHash,
            24,
            "BAD_COMMIT_HASH",
        ),
        (
            MlsCommitAdmissionReason::CommitAlreadyProcessed,
            25,
            "COMMIT_ALREADY_PROCESSED",
        ),
        (
            MlsCommitAdmissionReason::PlaintextCommitMetadataForbidden,
            26,
            "PLAINTEXT_COMMIT_METADATA_FORBIDDEN",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> MlsCommitAdmissionInput {
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

fn assert_rejected(decision: MlsCommitAdmissionDecision, reason: MlsCommitAdmissionReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_apply_commit);
    assert!(!decision.can_initialize_epoch);
    assert!(!decision.can_continue_group);
    assert!(!decision.local_member_removed);
    assert!(decision.prevents_commit_replay);
    assert!(decision.forbids_plaintext_commit_metadata);
    assert_eq!(decision.reason, reason);
}
