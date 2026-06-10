//! Stage 6 spine: REAL AKD key-transparency proofs drive the REAL mercury-core
//! device-trust gate. A consistent, bound transparency bundle lets device trust
//! proceed; a tampered one forces `KeyTransparencyFailed` (no send). This proves
//! the KT verifier is wired THROUGH the gate into the actual trust decision --
//! not a standalone checker whose verdict is ignored. (Companion to the S1<->S4
//! relay spine test.)

use mercury_core::{
    ActorKind, DeviceKeyChangeState, DeviceKind, DeviceState, DeviceTrustDecision,
    DeviceTrustInput, DeviceTrustPolicyMode, DeviceTrustReason, KeyTransparencyState,
    ManualVerificationState, evaluate_device_trust, evaluate_key_transparency,
};
use mercury_kt::{
    KeyTransparencyWitnessStatus as WitnessStatus, KtDirectory, SignedTreeHead, TransparencyBundle,
    Witness, WitnessCosignature, verify_transparency_bundle, verify_witnessed_tree_head,
};

const LABEL: &str = "device:alice:a1f3";
const KEY_V1: &[u8] = b"alice-device-key-v1";
const KEY_V2: &[u8] = b"alice-device-key-v2";

/// Run the full real KT path on a fresh directory and return the gate's state.
/// Registers v1, rotates to v2, then verifies the bound bundle
/// (inclusion + consistency + key-history) against the pinned `[ep1, ep2]` chain.
/// `tamper` swaps in a consistency proof for the wrong epoch window, which the
/// engine rejects -> the gate returns `Inconsistent`.
async fn real_kt_state(tamper: bool) -> KeyTransparencyState {
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    let ep2 = dir.register(LABEL, KEY_V2).await.unwrap();
    let pk = dir.public_key().await.unwrap();

    // Capture inclusion + key-history at the epoch-2 head BEFORE any further
    // publish, so they stay anchored to the pinned head.
    let (inclusion, _) = dir.prove_inclusion(LABEL).await.unwrap();
    let (key_history, _) = dir.prove_key_history(LABEL).await.unwrap();

    let consistency = if tamper {
        // A genuine but WRONG-window proof: advance to epoch 3, prove 2->3, then
        // present it against the pinned 1->2 chain. The epoch binding rejects it.
        dir.register("device:bob:9c20", b"bob-device-key")
            .await
            .unwrap();
        dir.prove_consistency(2, 3).await.unwrap()
    } else {
        dir.prove_consistency(1, 2).await.unwrap()
    };

    let input = verify_transparency_bundle(
        &pk,
        &[ep1, ep2],
        LABEL,
        KEY_V2,
        TransparencyBundle {
            inclusion,
            consistency,
            key_history,
        },
        WitnessStatus::NotRequired,
        false, // device-trust path here does not require witnessing
        30,
        300,
    )
    .await;

    evaluate_key_transparency(input).state
}

fn device_trust(
    kt: KeyTransparencyState,
    mode: DeviceTrustPolicyMode,
    manual: ManualVerificationState,
) -> DeviceTrustDecision {
    evaluate_device_trust(DeviceTrustInput {
        policy_mode: mode,
        actor_kind: ActorKind::Human,
        device_kind: DeviceKind::Human,
        device_state: DeviceState::Active,
        manual_verification: manual,
        key_transparency: kt,
        key_change: DeviceKeyChangeState::Unchanged,
    })
}

#[tokio::test]
async fn consistent_transparency_permits_device_trust() {
    let kt = real_kt_state(false).await;
    assert_eq!(kt, KeyTransparencyState::Consistent);

    // Strict mode + manual verification + a consistent KT log -> fully trusted.
    let strict = device_trust(
        kt,
        DeviceTrustPolicyMode::Strict,
        ManualVerificationState::Verified,
    );
    assert!(strict.trusted && strict.can_send);
    assert_eq!(strict.reason, DeviceTrustReason::Trusted);

    // Opportunistic mode + consistent KT (manual not yet done) -> TOFU send.
    let tofu = device_trust(
        kt,
        DeviceTrustPolicyMode::Opportunistic,
        ManualVerificationState::Unverified,
    );
    assert!(tofu.can_send);
    assert_eq!(tofu.reason, DeviceTrustReason::TrustOnFirstUse);
}

#[tokio::test]
async fn inconsistent_transparency_blocks_device_trust() {
    let kt = real_kt_state(true).await;
    assert_eq!(kt, KeyTransparencyState::Inconsistent);

    // An inconsistent KT log fails closed even with manual verification + strict
    // policy: the gate cannot be talked past a proven log inconsistency.
    let decision = device_trust(
        kt,
        DeviceTrustPolicyMode::Strict,
        ManualVerificationState::Verified,
    );
    assert!(!decision.trusted && !decision.can_send);
    assert_eq!(decision.reason, DeviceTrustReason::KeyTransparencyFailed);
    assert!(decision.requires_user_action);
}

#[tokio::test]
async fn fully_witnessed_transparency_permits_device_trust() {
    use ed25519_dalek::{Signer, SigningKey};

    // The complete S6 spine: all FOUR proof dimensions real (inclusion +
    // consistency + key-history from AKD, witness from real Ed25519 cosigning),
    // require_witness=true, composing to a Consistent KT decision that the real
    // device-trust gate turns into full Trusted.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    let ep2 = dir.register(LABEL, KEY_V2).await.unwrap();
    let pk = dir.public_key().await.unwrap();
    let head_root = ep2.hash();
    let head_size = ep2.epoch();

    let (inclusion, _) = dir.prove_inclusion(LABEL).await.unwrap();
    let (key_history, _) = dir.prove_key_history(LABEL).await.unwrap();
    let consistency = dir.prove_consistency(1, 2).await.unwrap();

    // A real signed tree head over the SAME head the proofs use: log + 2
    // witnesses across 2 operators + auditor, all signing the canonical bytes.
    let sth = SignedTreeHead {
        tree_size: head_size,
        root_hash: head_root,
        timestamp_s: 1_000,
    };
    let bytes = sth.signing_bytes();
    let log = SigningKey::from_bytes(&[1u8; 32]);
    let auditor = SigningKey::from_bytes(&[2u8; 32]);
    let w1 = SigningKey::from_bytes(&[10u8; 32]);
    let w2 = SigningKey::from_bytes(&[11u8; 32]);
    let witnesses = [
        Witness {
            key: w1.verifying_key(),
            operator_id: 100,
        },
        Witness {
            key: w2.verifying_key(),
            operator_id: 200,
        },
    ];
    let cosigs = [
        WitnessCosignature {
            witness_index: 0,
            signature: w1.sign(&bytes).to_bytes(),
        },
        WitnessCosignature {
            witness_index: 1,
            signature: w2.sign(&bytes).to_bytes(),
        },
    ];
    let witnessed = verify_witnessed_tree_head(
        &sth,
        &log.verifying_key(),
        &log.sign(&bytes).to_bytes(),
        &auditor.verifying_key(),
        &auditor.sign(&bytes).to_bytes(),
        &witnesses,
        &cosigs,
        head_root,
        head_size,
        1_030,
    );
    let witness_status = witnessed.kt_witness_status(2);
    assert_eq!(witness_status, WitnessStatus::QuorumSatisfied);

    // The KT bundle REQUIRES a witnessed head and gets a real quorum.
    let input = verify_transparency_bundle(
        &pk,
        &[ep1, ep2],
        LABEL,
        KEY_V2,
        TransparencyBundle {
            inclusion,
            consistency,
            key_history,
        },
        witness_status,
        true, // require_witness
        30,
        300,
    )
    .await;
    let kt = evaluate_key_transparency(input);
    assert_eq!(kt.state, KeyTransparencyState::Consistent);

    let decision = device_trust(
        kt.state,
        DeviceTrustPolicyMode::Strict,
        ManualVerificationState::Verified,
    );
    assert!(decision.trusted && decision.can_send);
    assert_eq!(decision.reason, DeviceTrustReason::Trusted);
}
