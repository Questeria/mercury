use mercury_core::{
    AccessKind, ActorKind, AiGrantFacts, AiLifecycleFacts, AiPolicyFacts, ClientMessageEnvelope,
    ClientMessagePolicyInput, ClientRoomState, ClientSenderState, ClientStateError,
    ConversationPolicyState, DeviceKind, DeviceState, MessageKind, OutboundMessageDraft,
    ProtocolSuite, RoomMode, RoomStateSnapshot, SenderDeviceState,
};

#[test]
fn checked_builder_accepts_human_message() {
    let input = ClientMessagePolicyInput::try_from_local_state(
        room_state(RoomMode::Standard),
        ClientSenderState::human(DeviceState::Active, 0),
        draft(AccessKind::Write),
    )
    .expect("valid human sender state should build");

    let decision = input.evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason_label().label, "ACCEPT");
}

#[test]
fn checked_builder_rejects_human_with_ai_device() {
    let result = ClientMessagePolicyInput::try_from_local_state(
        room_state(RoomMode::Standard),
        ClientSenderState {
            actor_kind: ActorKind::Human,
            device: SenderDeviceState {
                kind: DeviceKind::Ai,
                state: DeviceState::Active,
                revoked_at_epoch: 0,
            },
            ai: None,
        },
        draft(AccessKind::Write),
    );

    assert_eq!(result, Err(ClientStateError::HumanActorRequiresHumanDevice));
}

#[test]
fn checked_builder_rejects_ai_without_policy() {
    let result = ClientMessagePolicyInput::try_from_local_state(
        room_state(RoomMode::Standard),
        ClientSenderState {
            actor_kind: ActorKind::LocalAi,
            device: SenderDeviceState {
                kind: DeviceKind::Ai,
                state: DeviceState::Active,
                revoked_at_epoch: 0,
            },
            ai: None,
        },
        draft(AccessKind::Read),
    );

    assert_eq!(result, Err(ClientStateError::AiActorRequiresAiPolicy));
}

#[test]
fn checked_builder_rejects_active_device_with_revoked_epoch() {
    let result = ClientMessagePolicyInput::try_from_local_state(
        room_state(RoomMode::Standard),
        ClientSenderState::human(DeviceState::Active, 7),
        draft(AccessKind::Write),
    );

    assert_eq!(result, Err(ClientStateError::DeviceRevocationMismatch));
}

#[test]
fn checked_builder_rejects_ai_room_mode_mismatch() {
    let result = ClientMessagePolicyInput::try_from_local_state(
        room_state(RoomMode::Sensitive),
        ClientSenderState::local_ai(
            DeviceState::Active,
            0,
            ai_policy(RoomMode::Standard, AccessKind::Read, 1),
        ),
        draft(AccessKind::Read),
    );

    assert_eq!(result, Err(ClientStateError::AiGrantRoomModeMismatch));
}

#[test]
fn checked_builder_rejects_ai_access_mismatch() {
    let result = ClientMessagePolicyInput::try_from_local_state(
        room_state(RoomMode::Standard),
        ClientSenderState::local_ai(
            DeviceState::Active,
            0,
            ai_policy(RoomMode::Standard, AccessKind::Read, 1),
        ),
        draft(AccessKind::Write),
    );

    assert_eq!(result, Err(ClientStateError::AiLifecycleAccessKindMismatch));
}

#[test]
fn checked_builder_rejects_remote_ai_with_local_ai_mode() {
    let result = ClientMessagePolicyInput::try_from_local_state(
        room_state(RoomMode::Standard),
        ClientSenderState::remote_ai(
            DeviceState::Active,
            0,
            ai_policy(RoomMode::Standard, AccessKind::Read, 1),
        ),
        draft(AccessKind::Read),
    );

    assert_eq!(result, Err(ClientStateError::AiModeMismatch));
}

fn room_state(mode: RoomMode) -> ClientRoomState {
    ClientRoomState::new(
        ConversationPolicyState {
            expected_epoch: 7,
            expected_sequence: 42,
            minimum_suite: ProtocolSuite::ClassicalDev,
            max_payload_len: 1024,
        },
        RoomStateSnapshot {
            version: 1,
            mode,
            current_epoch: 7,
            min_accepted_epoch: 1,
        },
    )
}

fn draft(access_kind: AccessKind) -> OutboundMessageDraft {
    OutboundMessageDraft::new(
        ClientMessageEnvelope {
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
        access_kind,
    )
}

fn ai_policy(room_mode: RoomMode, access_kind: AccessKind, ai_mode: i32) -> AiPolicyFacts {
    AiPolicyFacts {
        grant: AiGrantFacts {
            version: 1,
            principal_kind: 2,
            room_mode: room_mode.code(),
            ai_mode,
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
            room_mode: room_mode.code(),
            access_kind: access_kind.code(),
            epoch_rotated: 0,
        },
    }
}
