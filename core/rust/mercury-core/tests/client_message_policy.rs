use mercury_core::{
    AccessKind, ActorKind, AiGrantFacts, AiLifecycleFacts, AiPolicyFacts, ClientMessageEnvelope,
    ClientMessagePolicyInput, ConversationPolicyState, DeviceKind, DeviceState, MessageKind,
    ProtocolSuite, RoomMode, RoomStateSnapshot, SenderDeviceState,
};

#[test]
fn client_message_policy_accepts_valid_human_message() {
    let decision = valid_human_input().evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason_label().label, "ACCEPT");
    assert_eq!(decision.view().primary_source, "policy_pipeline");
    assert_eq!(decision.view().primary_component, None);
}

#[test]
fn client_message_policy_rejects_stale_room_epoch() {
    let mut input = valid_human_input();
    input.envelope.epoch = 6;
    input.conversation.expected_epoch = 6;
    input.room.current_epoch = 7;

    let decision = input.evaluate();
    let view = decision.view();

    assert!(!decision.accepted);
    assert_eq!(view.reason.label, "ROOM_EPOCH_REJECT");
    assert_eq!(view.primary_source, "room_epoch");
    assert_eq!(
        view.primary_component
            .expect("room epoch reject should have a component")
            .label,
        "STALE_EPOCH"
    );
}

#[test]
fn client_message_policy_rejects_ai_device_in_ai_blocked_room() {
    let mut input = valid_ai_input();
    input.room.mode = RoomMode::AiBlocked;

    let decision = input.evaluate();
    let view = decision.view();

    assert!(!decision.accepted);
    assert_eq!(view.reason.label, "ROOM_EPOCH_REJECT");
    assert_eq!(view.primary_source, "room_epoch");
    assert_eq!(
        view.primary_component
            .expect("room epoch reject should have a component")
            .label,
        "AI_DEVICE_BLOCKED_ROOM"
    );
}

fn valid_human_input() -> ClientMessagePolicyInput {
    ClientMessagePolicyInput {
        actor_kind: ActorKind::Human,
        envelope: ClientMessageEnvelope {
            version: 1,
            suite: ProtocolSuite::HybridPqDev,
            conversation_id_len: 32,
            sender_account_id_len: 32,
            sender_device_id_len: 32,
            epoch: 7,
            sequence: 42,
            kind: MessageKind::Application,
            payload_len: 128,
            critical_flags: 0,
            noncritical_flags: 0,
        },
        conversation: ConversationPolicyState {
            expected_epoch: 7,
            expected_sequence: 42,
            minimum_suite: ProtocolSuite::ClassicalDev,
            max_payload_len: 1024,
        },
        room: RoomStateSnapshot {
            version: 1,
            mode: RoomMode::Standard,
            current_epoch: 7,
            min_accepted_epoch: 1,
        },
        sender_device: SenderDeviceState {
            kind: DeviceKind::Human,
            state: DeviceState::Active,
            revoked_at_epoch: 0,
        },
        access_kind: AccessKind::Write,
        ai: None,
    }
}

fn valid_ai_input() -> ClientMessagePolicyInput {
    let mut input = valid_human_input();
    input.actor_kind = ActorKind::LocalAi;
    input.sender_device.kind = DeviceKind::Ai;
    input.access_kind = AccessKind::Read;
    input.ai = Some(AiPolicyFacts {
        grant: AiGrantFacts {
            version: 1,
            principal_kind: 2,
            room_mode: 1,
            ai_mode: 1,
            ttl_s: 300,
            approver_count: 1,
            read_scope: 1,
            write_scope: 0,
            tool_scope: 0,
            retention_mode: 0,
            training_allowed: 0,
            prompt_store_allowed: 0,
        },
        lifecycle: AiLifecycleFacts {
            version: 1,
            grant_state: 1,
            revoke_reason: 0,
            now_s: 100,
            expires_at_s: 400,
            room_mode: 1,
            access_kind: 1,
            epoch_rotated: 0,
        },
    });
    input
}
