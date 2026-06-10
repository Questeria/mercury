use mercury_core::{
    ActorKind, DeviceKeyChangeState, DeviceKind, DeviceState, DeviceTrustInput,
    DeviceTrustPolicyMode, DeviceTrustReason, KeyTransparencyState, ManualVerificationState,
    evaluate_device_trust,
};

#[test]
fn strict_mode_accepts_verified_transparent_human_device() {
    let decision = evaluate_device_trust(input(
        DeviceTrustPolicyMode::Strict,
        ActorKind::Human,
        DeviceKind::Human,
        ManualVerificationState::Verified,
        KeyTransparencyState::Consistent,
        DeviceKeyChangeState::Unchanged,
    ));

    assert!(decision.trusted);
    assert!(decision.can_send);
    assert!(!decision.requires_user_action);
    assert_eq!(decision.reason, DeviceTrustReason::Trusted);
}

#[test]
fn strict_mode_rejects_unverified_device_even_with_transparency() {
    let decision = evaluate_device_trust(input(
        DeviceTrustPolicyMode::Strict,
        ActorKind::Human,
        DeviceKind::Human,
        ManualVerificationState::Unverified,
        KeyTransparencyState::Consistent,
        DeviceKeyChangeState::Unchanged,
    ));

    assert!(!decision.trusted);
    assert!(!decision.can_send);
    assert_eq!(
        decision.reason,
        DeviceTrustReason::ManualVerificationRequired
    );
}

#[test]
fn opportunistic_mode_can_send_with_consistent_transparency_but_marks_tofu() {
    let decision = evaluate_device_trust(input(
        DeviceTrustPolicyMode::Opportunistic,
        ActorKind::Human,
        DeviceKind::Human,
        ManualVerificationState::Unverified,
        KeyTransparencyState::Consistent,
        DeviceKeyChangeState::Unchanged,
    ));

    assert!(!decision.trusted);
    assert!(decision.can_send);
    assert!(decision.requires_user_action);
    assert_eq!(decision.reason, DeviceTrustReason::TrustOnFirstUse);
}

#[test]
fn key_transparency_inconsistency_blocks_all_modes() {
    for mode in [
        DeviceTrustPolicyMode::Opportunistic,
        DeviceTrustPolicyMode::Strict,
        DeviceTrustPolicyMode::HighSecurity,
    ] {
        let decision = evaluate_device_trust(input(
            mode,
            ActorKind::Human,
            DeviceKind::Human,
            ManualVerificationState::Verified,
            KeyTransparencyState::Inconsistent,
            DeviceKeyChangeState::Unchanged,
        ));

        assert!(!decision.trusted);
        assert!(!decision.can_send);
        assert_eq!(decision.reason, DeviceTrustReason::KeyTransparencyFailed);
    }
}

#[test]
fn key_change_requires_manual_verification_before_send() {
    let decision = evaluate_device_trust(input(
        DeviceTrustPolicyMode::Opportunistic,
        ActorKind::Human,
        DeviceKind::Human,
        ManualVerificationState::Unverified,
        KeyTransparencyState::Consistent,
        DeviceKeyChangeState::DeviceIdentityChanged,
    ));

    assert!(!decision.trusted);
    assert!(!decision.can_send);
    assert_eq!(
        decision.reason,
        DeviceTrustReason::KeyChangeRequiresVerification
    );
}

#[test]
fn actor_device_mismatch_is_never_trusted() {
    let decision = evaluate_device_trust(input(
        DeviceTrustPolicyMode::Strict,
        ActorKind::Human,
        DeviceKind::Ai,
        ManualVerificationState::Verified,
        KeyTransparencyState::Consistent,
        DeviceKeyChangeState::Unchanged,
    ));

    assert!(!decision.trusted);
    assert!(!decision.can_send);
    assert_eq!(decision.reason, DeviceTrustReason::ActorDeviceMismatch);
}

#[test]
fn compromised_device_is_never_trusted() {
    let mut trust_input = input(
        DeviceTrustPolicyMode::HighSecurity,
        ActorKind::RemoteAi,
        DeviceKind::Ai,
        ManualVerificationState::Verified,
        KeyTransparencyState::Consistent,
        DeviceKeyChangeState::Unchanged,
    );
    trust_input.device_state = DeviceState::Compromised;

    let decision = evaluate_device_trust(trust_input);

    assert!(!decision.trusted);
    assert!(!decision.can_send);
    assert_eq!(decision.reason, DeviceTrustReason::DeviceCompromised);
}

#[test]
fn high_security_accepts_verified_ai_device_with_transparency() {
    let decision = evaluate_device_trust(input(
        DeviceTrustPolicyMode::HighSecurity,
        ActorKind::RemoteAi,
        DeviceKind::Ai,
        ManualVerificationState::Verified,
        KeyTransparencyState::Consistent,
        DeviceKeyChangeState::NewDevice,
    ));

    assert!(decision.trusted);
    assert!(decision.can_send);
    assert_eq!(decision.reason, DeviceTrustReason::Trusted);
}

fn input(
    policy_mode: DeviceTrustPolicyMode,
    actor_kind: ActorKind,
    device_kind: DeviceKind,
    manual_verification: ManualVerificationState,
    key_transparency: KeyTransparencyState,
    key_change: DeviceKeyChangeState,
) -> DeviceTrustInput {
    DeviceTrustInput {
        policy_mode,
        actor_kind,
        device_kind,
        device_state: DeviceState::Active,
        manual_verification,
        key_transparency,
        key_change,
    }
}
