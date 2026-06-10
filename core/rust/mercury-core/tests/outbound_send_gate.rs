use mercury_core::{
    ComponentReasons, DeviceTrustDecision, DeviceTrustReason, LocalStoreRecordKind,
    LocalStoreSealingDecision, LocalStoreSealingReason, OutboundSendInput, OutboundSendReason,
    PolicyDecision, RoomMembershipSendState, RoomMembershipTransitionDecision,
    RoomMembershipTransitionReason, evaluate_outbound_send,
};

#[test]
fn accepted_gate_allows_send_and_ciphertext_persistence() {
    let decision = evaluate_outbound_send(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_send);
    assert!(decision.can_persist_ciphertext);
    assert!(!decision.requires_user_action);
    assert_eq!(decision.reason, OutboundSendReason::Accepted);
}

#[test]
fn sendable_tofu_device_keeps_visible_user_action() {
    let mut input = valid_input();
    input.sender_device_trust = tofu_trust();

    let decision = input.evaluate();

    assert!(decision.accepted);
    assert!(decision.can_send);
    assert!(decision.can_persist_ciphertext);
    assert!(decision.requires_user_action);
    assert_eq!(decision.reason, OutboundSendReason::Accepted);
}

#[test]
fn rejected_device_trust_blocks_send_and_persistence() {
    let mut input = valid_input();
    input.sender_device_trust = rejected_trust();

    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert!(!decision.can_send);
    assert!(!decision.can_persist_ciphertext);
    assert!(decision.requires_user_action);
    assert_eq!(decision.reason, OutboundSendReason::DeviceTrustRejected);
}

#[test]
fn rejected_room_membership_transition_blocks_send() {
    let mut input = valid_input();
    input.room_membership = RoomMembershipSendState::Transition(room_transition(
        false,
        false,
        true,
        RoomMembershipTransitionReason::AiGrantRejected,
    ));

    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        OutboundSendReason::RoomMembershipTransitionRejected
    );
}

#[test]
fn accepted_room_transition_must_rotate_epoch() {
    let mut input = valid_input();
    input.room_membership = RoomMembershipSendState::Transition(room_transition(
        true,
        false,
        false,
        RoomMembershipTransitionReason::Accepted,
    ));

    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        OutboundSendReason::RoomMembershipEpochNotRotated
    );
}

#[test]
fn message_policy_rejection_blocks_send_before_store_commit() {
    let mut input = valid_input();
    input.message_policy = policy_decision(false);

    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert!(!decision.can_persist_ciphertext);
    assert_eq!(decision.reason, OutboundSendReason::MessagePolicyRejected);
}

#[test]
fn local_store_sealing_rejection_blocks_send() {
    let mut input = valid_input();
    input.ciphertext_sealing = sealing_decision(false);

    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert!(!decision.can_send);
    assert!(!decision.can_persist_ciphertext);
    assert_eq!(
        decision.reason,
        OutboundSendReason::LocalStoreSealingRejected
    );
}

fn valid_input() -> OutboundSendInput {
    OutboundSendInput::new(
        full_trust(),
        RoomMembershipSendState::Stable,
        policy_decision(true),
        sealing_decision(true),
    )
}

fn full_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: true,
        can_send: true,
        requires_user_action: false,
        reason: DeviceTrustReason::Trusted,
    }
}

fn tofu_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: false,
        can_send: true,
        requires_user_action: true,
        reason: DeviceTrustReason::TrustOnFirstUse,
    }
}

fn rejected_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: false,
        can_send: false,
        requires_user_action: true,
        reason: DeviceTrustReason::KeyTransparencyRequired,
    }
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

fn sealing_decision(accepted: bool) -> LocalStoreSealingDecision {
    LocalStoreSealingDecision {
        accepted,
        reason: if accepted {
            LocalStoreSealingReason::Accepted
        } else {
            LocalStoreSealingReason::PolicyDecisionRejected
        },
        record_policy: LocalStoreRecordKind::MessageCiphertext.policy(),
    }
}

fn room_transition(
    accepted: bool,
    epoch_rotated: bool,
    requires_user_action: bool,
    reason: RoomMembershipTransitionReason,
) -> RoomMembershipTransitionDecision {
    RoomMembershipTransitionDecision {
        accepted,
        epoch_rotated,
        requires_user_action,
        reason,
    }
}
