use mercury_core::{
    ActorKind, DeviceKeyChangeState, DeviceKind, DeviceState, DeviceTrustInput,
    DeviceTrustPolicyMode, DeviceTrustReason, KeyTransparencyDecision, KeyTransparencyProofInput,
    KeyTransparencyProofStatus, KeyTransparencyReason, KeyTransparencyState,
    KeyTransparencyWitnessStatus, ManualVerificationState, evaluate_device_trust,
    evaluate_key_transparency,
};

#[test]
fn verified_fresh_proofs_are_consistent() {
    let decision = evaluate_key_transparency(valid_proof());

    assert_eq!(decision.state, KeyTransparencyState::Consistent);
    assert_eq!(decision.reason, KeyTransparencyReason::Consistent);
    assert!(!decision.requires_user_action);
}

#[test]
fn stale_proof_maps_to_stale_state() {
    let decision = evaluate_key_transparency(KeyTransparencyProofInput {
        proof_age_s: 301,
        max_proof_age_s: 300,
        ..valid_proof()
    });

    assert_eq!(decision.state, KeyTransparencyState::StaleProof);
    assert_eq!(decision.reason, KeyTransparencyReason::StaleProof);
    assert!(decision.requires_user_action);
}

#[test]
fn missing_inclusion_proof_maps_to_missing_proof() {
    let decision = evaluate_key_transparency(KeyTransparencyProofInput {
        inclusion: KeyTransparencyProofStatus::Missing,
        ..valid_proof()
    });

    assert_eq!(decision.state, KeyTransparencyState::MissingProof);
    assert_eq!(
        decision.reason,
        KeyTransparencyReason::InclusionProofMissing
    );
    assert!(decision.requires_user_action);
}

#[test]
fn invalid_consistency_proof_maps_to_inconsistent() {
    let decision = evaluate_key_transparency(KeyTransparencyProofInput {
        consistency: KeyTransparencyProofStatus::Invalid,
        ..valid_proof()
    });

    assert_eq!(decision.state, KeyTransparencyState::Inconsistent);
    assert_eq!(
        decision.reason,
        KeyTransparencyReason::ConsistencyProofInvalid
    );
    assert!(decision.requires_user_action);
}

#[test]
fn log_rollback_maps_to_inconsistent() {
    let decision = evaluate_key_transparency(KeyTransparencyProofInput {
        previous_tree_size: 11,
        current_tree_size: 10,
        ..valid_proof()
    });

    assert_eq!(decision.state, KeyTransparencyState::Inconsistent);
    assert_eq!(decision.reason, KeyTransparencyReason::LogRollback);
}

#[test]
fn required_witness_quorum_must_be_present() {
    let decision = evaluate_key_transparency(KeyTransparencyProofInput {
        witness: KeyTransparencyWitnessStatus::NotRequired,
        require_witness: true,
        ..valid_proof()
    });

    assert_eq!(decision.state, KeyTransparencyState::MissingProof);
    assert_eq!(decision.reason, KeyTransparencyReason::WitnessQuorumMissing);
}

#[test]
fn key_transparency_decision_feeds_device_trust() {
    let kt = evaluate_key_transparency(valid_proof());
    let trust = evaluate_device_trust(DeviceTrustInput {
        policy_mode: DeviceTrustPolicyMode::Strict,
        actor_kind: ActorKind::Human,
        device_kind: DeviceKind::Human,
        device_state: DeviceState::Active,
        manual_verification: ManualVerificationState::Verified,
        key_transparency: kt.state,
        key_change: DeviceKeyChangeState::Unchanged,
    });

    assert!(trust.trusted);
    assert!(trust.can_send);
    assert_eq!(trust.reason, DeviceTrustReason::Trusted);
}

#[test]
fn stale_key_transparency_blocks_strict_device_trust() {
    let kt = KeyTransparencyDecision {
        state: KeyTransparencyState::StaleProof,
        reason: KeyTransparencyReason::StaleProof,
        requires_user_action: true,
    };
    let trust = evaluate_device_trust(DeviceTrustInput {
        policy_mode: DeviceTrustPolicyMode::Strict,
        actor_kind: ActorKind::Human,
        device_kind: DeviceKind::Human,
        device_state: DeviceState::Active,
        manual_verification: ManualVerificationState::Verified,
        key_transparency: kt.state,
        key_change: DeviceKeyChangeState::Unchanged,
    });

    assert!(!trust.trusted);
    assert!(!trust.can_send);
    assert_eq!(trust.reason, DeviceTrustReason::KeyTransparencyRequired);
}

fn valid_proof() -> KeyTransparencyProofInput {
    KeyTransparencyProofInput {
        inclusion: KeyTransparencyProofStatus::Verified,
        consistency: KeyTransparencyProofStatus::Verified,
        key_history: KeyTransparencyProofStatus::Verified,
        witness: KeyTransparencyWitnessStatus::QuorumSatisfied,
        require_witness: true,
        previous_tree_size: 10,
        current_tree_size: 12,
        proof_age_s: 60,
        max_proof_age_s: 300,
    }
}
