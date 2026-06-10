use mercury_core::{
    ComponentReasons, GroupChatCryptoSuite, GroupChatInput, GroupChatProtocol,
    GroupMessageTranscriptInput, GroupMessageTranscriptReason, LocalStoreKeyBinding,
    LocalStoreKeyDescriptor, LocalStoreKeyScope, LocalStoreRecordKind, LocalStoreRecordLocator,
    LocalStoreSealRequest, LocalStoreSealingSuite, OutboundSendDecision, OutboundSendReason,
    PolicyDecision, RoomMode, evaluate_group_chat, evaluate_group_message_transcript,
    evaluate_mls_provider_security,
};

#[test]
fn group_message_transcript_accepts_bound_mls_application_message() {
    let decision = evaluate_group_message_transcript(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_persist_ciphertext);
    assert!(decision.can_submit_to_relay);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_rekey);
    assert!(!decision.requires_user_action);
    assert!(decision.forbids_plaintext);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, GroupMessageTranscriptReason::Accepted);
}

#[test]
fn group_message_transcript_requires_accepted_group_and_send_gates() {
    let mut rejected_group = valid_input();
    rejected_group.group_chat = {
        let mut group = valid_group_chat_input();
        group.membership_transition_pending = true;
        group.evaluate()
    };
    let rejected_group_decision = rejected_group.evaluate();
    assert_rejected(
        rejected_group_decision,
        GroupMessageTranscriptReason::GroupChatRejected,
    );
    assert!(rejected_group_decision.requires_sync);
    assert!(rejected_group_decision.requires_user_action);

    let mut rejected_send = valid_input();
    rejected_send.outbound_send = OutboundSendDecision {
        accepted: false,
        can_send: false,
        can_persist_ciphertext: false,
        requires_user_action: true,
        reason: OutboundSendReason::MessagePolicyRejected,
    };
    let rejected_send_decision = rejected_send.evaluate();
    assert_rejected(
        rejected_send_decision,
        GroupMessageTranscriptReason::OutboundSendRejected,
    );
    assert!(rejected_send_decision.requires_user_action);
}

#[test]
fn group_message_transcript_requires_group_epoch_sender_and_transcript_context() {
    let mut bad_group_id = valid_input();
    bad_group_id.group_id_len = 0;
    assert_rejected(
        bad_group_id.evaluate(),
        GroupMessageTranscriptReason::BadGroupIdentifier,
    );

    let mut bad_epoch = valid_input();
    bad_epoch.message_epoch = 6;
    assert_rejected(
        bad_epoch.evaluate(),
        GroupMessageTranscriptReason::EpochMismatch,
    );

    let mut bad_sender = valid_input();
    bad_sender.sender_leaf_index = -1;
    assert_rejected(
        bad_sender.evaluate(),
        GroupMessageTranscriptReason::BadSenderLeafIndex,
    );

    let mut bad_generation = valid_input();
    bad_generation.sender_generation = -1;
    assert_rejected(
        bad_generation.evaluate(),
        GroupMessageTranscriptReason::BadSenderGeneration,
    );

    let mut missing_context = valid_input();
    missing_context.confirmed_transcript_hash_len = 0;
    assert_rejected(
        missing_context.evaluate(),
        GroupMessageTranscriptReason::TranscriptContextMissing,
    );
}

#[test]
fn group_message_transcript_requires_sealed_mls_parts_and_generation_deletion() {
    let mut sender_data = valid_input();
    sender_data.sender_data_sealed = false;
    let sender_data_decision = sender_data.evaluate();
    assert_rejected(
        sender_data_decision,
        GroupMessageTranscriptReason::SenderDataNotSealed,
    );
    assert!(sender_data_decision.requires_rekey);

    let mut payload = valid_input();
    payload.application_payload_sealed = false;
    assert_rejected(
        payload.evaluate(),
        GroupMessageTranscriptReason::ApplicationPayloadNotSealed,
    );

    let mut reuse_guard = valid_input();
    reuse_guard.reuse_guard_len = 0;
    assert_rejected(
        reuse_guard.evaluate(),
        GroupMessageTranscriptReason::ReuseGuardMissing,
    );

    let mut used_generation = valid_input();
    used_generation.used_generation_deleted = false;
    assert_rejected(
        used_generation.evaluate(),
        GroupMessageTranscriptReason::UsedGenerationNotDeleted,
    );
}

#[test]
fn group_message_transcript_requires_room_epoch_local_store_binding() {
    let mut bad_seal = valid_input();
    bad_seal.local_store_seal = seal_request(7, LocalStoreRecordKind::MessagePlaintext, 32);
    assert_rejected(
        bad_seal.evaluate(),
        GroupMessageTranscriptReason::LocalStoreSealingRejected,
    );

    let mut wrong_epoch = valid_input();
    wrong_epoch.local_store_seal = seal_request(6, LocalStoreRecordKind::MessageCiphertext, 32);
    assert_rejected(
        wrong_epoch.evaluate(),
        GroupMessageTranscriptReason::LocalStoreEpochBindingMismatch,
    );

    let mut wrong_group = valid_input();
    wrong_group.local_store_seal = seal_request(7, LocalStoreRecordKind::MessageCiphertext, 16);
    assert_rejected(
        wrong_group.evaluate(),
        GroupMessageTranscriptReason::LocalStoreEpochBindingMismatch,
    );
}

#[test]
fn group_message_transcript_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (GroupMessageTranscriptReason::Accepted, 0, "ACCEPTED"),
        (
            GroupMessageTranscriptReason::GroupChatRejected,
            1,
            "GROUP_CHAT_REJECTED",
        ),
        (
            GroupMessageTranscriptReason::OutboundSendRejected,
            2,
            "OUTBOUND_SEND_REJECTED",
        ),
        (
            GroupMessageTranscriptReason::BadGroupIdentifier,
            3,
            "BAD_GROUP_IDENTIFIER",
        ),
        (
            GroupMessageTranscriptReason::EpochMismatch,
            4,
            "EPOCH_MISMATCH",
        ),
        (
            GroupMessageTranscriptReason::BadSenderLeafIndex,
            5,
            "BAD_SENDER_LEAF_INDEX",
        ),
        (
            GroupMessageTranscriptReason::BadSenderGeneration,
            6,
            "BAD_SENDER_GENERATION",
        ),
        (
            GroupMessageTranscriptReason::TranscriptContextMissing,
            7,
            "TRANSCRIPT_CONTEXT_MISSING",
        ),
        (
            GroupMessageTranscriptReason::SenderDataNotSealed,
            8,
            "SENDER_DATA_NOT_SEALED",
        ),
        (
            GroupMessageTranscriptReason::ApplicationPayloadNotSealed,
            9,
            "APPLICATION_PAYLOAD_NOT_SEALED",
        ),
        (
            GroupMessageTranscriptReason::ReuseGuardMissing,
            10,
            "REUSE_GUARD_MISSING",
        ),
        (
            GroupMessageTranscriptReason::LocalStoreSealingRejected,
            11,
            "LOCAL_STORE_SEALING_REJECTED",
        ),
        (
            GroupMessageTranscriptReason::LocalStoreEpochBindingMismatch,
            12,
            "LOCAL_STORE_EPOCH_BINDING_MISMATCH",
        ),
        (
            GroupMessageTranscriptReason::UsedGenerationNotDeleted,
            13,
            "USED_GENERATION_NOT_DELETED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> GroupMessageTranscriptInput<'static> {
    GroupMessageTranscriptInput {
        group_chat: evaluate_group_chat(valid_group_chat_input()),
        outbound_send: accepted_send(),
        local_store_seal: seal_request(7, LocalStoreRecordKind::MessageCiphertext, 32),
        group_id_len: 32,
        message_epoch: 7,
        local_epoch: 7,
        sender_leaf_index: 2,
        sender_generation: 4,
        group_context_digest_len: 32,
        confirmed_transcript_hash_len: 32,
        sender_data_sealed: true,
        application_payload_sealed: true,
        reuse_guard_len: 4,
        used_generation_deleted: true,
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
        mls_provider_security: evaluate_mls_provider_security(
            mercury_core::MlsProviderSecurityInput {
                provider_configured: true,
                selected_suite: GroupChatCryptoSuite::HybridPqMls768,
                minimum_suite: GroupChatCryptoSuite::HybridPqMls768,
                provider_supports_selected_suite: true,
                ml_kem_parameter_set: 768,
                classical_kem_component_present: true,
                requires_pq_signatures: false,
                pq_signature_ready: false,
                suite_id_bound_to_group_context: true,
                downgrade_evidence_verified: true,
                known_answer_tests_passed: true,
                secret_zeroization_available: true,
                unsafe_crypto_backend: false,
                plaintext_key_export_fields: 0,
            },
        ),
        plaintext_member_metadata_fields: 0,
    }
}

fn accepted_send() -> OutboundSendDecision {
    OutboundSendDecision {
        accepted: true,
        can_send: true,
        can_persist_ciphertext: true,
        requires_user_action: false,
        reason: OutboundSendReason::Accepted,
    }
}

fn seal_request(
    room_epoch: i32,
    record_kind: LocalStoreRecordKind,
    group_id_len: i32,
) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        LocalStoreRecordLocator::new("group-7", "message-42"),
        record_kind,
        LocalStoreKeyDescriptor::new(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreSealingSuite::MercuryLocalStoreV1,
            1,
            LocalStoreKeyBinding::room_epoch(32, group_id_len, room_epoch),
        ),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        64,
        Some(policy_decision(true)),
    )
}

fn policy_decision(accepted: bool) -> PolicyDecision {
    PolicyDecision {
        accepted,
        reason_code: if accepted { 0 } else { 1 },
        audit_class: 0,
        components: ComponentReasons {
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    }
}

fn assert_rejected(
    decision: mercury_core::GroupMessageTranscriptDecision,
    reason: GroupMessageTranscriptReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_persist_ciphertext);
    assert!(!decision.can_submit_to_relay);
    assert!(decision.forbids_plaintext);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
