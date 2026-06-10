use mercury_core::{
    ActorKind, ComponentReasons, DeviceKind, DeviceState, DeviceTrustDecision, DeviceTrustReason,
    PolicyDecision, RoomMembershipTransitionInput, RoomMembershipTransitionKind,
    RoomMembershipTransitionReason, RoomMode, evaluate_room_membership_transition,
};

#[test]
fn standard_room_accepts_human_device_with_sendable_trust() {
    let decision = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::AddDevice,
        RoomMode::Standard,
        ActorKind::Human,
        DeviceKind::Human,
        DeviceState::Active,
        tofu_trust(),
        None,
        7,
        8,
    ));

    assert!(decision.accepted);
    assert!(decision.epoch_rotated);
    assert!(!decision.requires_user_action);
    assert_eq!(decision.reason, RoomMembershipTransitionReason::Accepted);
}

#[test]
fn high_security_room_requires_fully_trusted_human_device() {
    let decision = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::AddDevice,
        RoomMode::HighSecurity,
        ActorKind::Human,
        DeviceKind::Human,
        DeviceState::Active,
        tofu_trust(),
        None,
        7,
        8,
    ));

    assert!(!decision.accepted);
    assert!(!decision.epoch_rotated);
    assert_eq!(
        decision.reason,
        RoomMembershipTransitionReason::DeviceFullTrustRequired
    );
}

#[test]
fn ai_blocked_room_rejects_ai_device_addition() {
    let decision = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::AddDevice,
        RoomMode::AiBlocked,
        ActorKind::RemoteAi,
        DeviceKind::Ai,
        DeviceState::Active,
        full_trust(),
        Some(policy_decision(true)),
        7,
        8,
    ));

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        RoomMembershipTransitionReason::AiBlockedRoom
    );
}

#[test]
fn ai_device_addition_requires_accepted_grant_and_full_trust() {
    let missing_grant = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::AddDevice,
        RoomMode::Standard,
        ActorKind::RemoteAi,
        DeviceKind::Ai,
        DeviceState::Active,
        full_trust(),
        None,
        7,
        8,
    ));
    assert!(!missing_grant.accepted);
    assert_eq!(
        missing_grant.reason,
        RoomMembershipTransitionReason::AiGrantRequired
    );

    let rejected_grant = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::AddDevice,
        RoomMode::Standard,
        ActorKind::RemoteAi,
        DeviceKind::Ai,
        DeviceState::Active,
        full_trust(),
        Some(policy_decision(false)),
        7,
        8,
    ));
    assert!(!rejected_grant.accepted);
    assert_eq!(
        rejected_grant.reason,
        RoomMembershipTransitionReason::AiGrantRejected
    );

    let accepted = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::AddDevice,
        RoomMode::Standard,
        ActorKind::RemoteAi,
        DeviceKind::Ai,
        DeviceState::Active,
        full_trust(),
        Some(policy_decision(true)),
        7,
        8,
    ));
    assert!(accepted.accepted);
    assert_eq!(accepted.reason, RoomMembershipTransitionReason::Accepted);
}

#[test]
fn every_transition_must_advance_epoch() {
    let decision = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::RemoveDevice,
        RoomMode::Standard,
        ActorKind::Human,
        DeviceKind::Human,
        DeviceState::Removed,
        full_trust(),
        None,
        7,
        7,
    ));

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        RoomMembershipTransitionReason::EpochMustAdvance
    );
}

#[test]
fn remove_device_requires_removed_target_state() {
    let wrong_state = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::RemoveDevice,
        RoomMode::Standard,
        ActorKind::Human,
        DeviceKind::Human,
        DeviceState::Active,
        full_trust(),
        None,
        7,
        8,
    ));
    assert!(!wrong_state.accepted);
    assert_eq!(
        wrong_state.reason,
        RoomMembershipTransitionReason::TargetDeviceMustBeRemoved
    );

    let accepted = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::RemoveDevice,
        RoomMode::HighSecurity,
        ActorKind::Human,
        DeviceKind::Human,
        DeviceState::Removed,
        full_trust(),
        None,
        7,
        8,
    ));
    assert!(accepted.accepted);
    assert!(accepted.epoch_rotated);
}

#[test]
fn compromised_device_transition_requires_compromised_target_state() {
    let wrong_state = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::MarkDeviceCompromised,
        RoomMode::Standard,
        ActorKind::Human,
        DeviceKind::Human,
        DeviceState::Removed,
        full_trust(),
        None,
        7,
        8,
    ));
    assert!(!wrong_state.accepted);
    assert_eq!(
        wrong_state.reason,
        RoomMembershipTransitionReason::TargetDeviceMustBeCompromised
    );

    let accepted = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::MarkDeviceCompromised,
        RoomMode::HighSecurity,
        ActorKind::Human,
        DeviceKind::Human,
        DeviceState::Compromised,
        full_trust(),
        None,
        7,
        8,
    ));
    assert!(accepted.accepted);
    assert!(accepted.epoch_rotated);
}

#[test]
fn actor_device_mismatch_rejects_transition() {
    let decision = evaluate_room_membership_transition(input(
        RoomMembershipTransitionKind::AddDevice,
        RoomMode::Standard,
        ActorKind::Human,
        DeviceKind::Ai,
        DeviceState::Active,
        full_trust(),
        None,
        7,
        8,
    ));

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        RoomMembershipTransitionReason::ActorDeviceMismatch
    );
}

fn input(
    kind: RoomMembershipTransitionKind,
    room_mode: RoomMode,
    target_actor_kind: ActorKind,
    target_device_kind: DeviceKind,
    target_device_state: DeviceState,
    target_device_trust: DeviceTrustDecision,
    ai_policy_decision: Option<PolicyDecision>,
    current_epoch: i32,
    proposed_epoch: i32,
) -> RoomMembershipTransitionInput {
    RoomMembershipTransitionInput {
        kind,
        room_mode,
        current_epoch,
        proposed_epoch,
        target_actor_kind,
        target_device_kind,
        target_device_state,
        target_device_trust,
        ai_policy_decision,
    }
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
