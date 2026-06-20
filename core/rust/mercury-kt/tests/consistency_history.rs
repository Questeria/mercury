//! Stage 6: real AKD CONSISTENCY (append-only) and KEY-HISTORY proofs verify and
//! feed the mercury-core key-transparency gate; a tampered checkpoint and a
//! wrong pinned VRF key are rejected. Closes the loop opened by `inclusion.rs`:
//! all three proof dimensions are now produced + verified against real AKD.

use mercury_core::{KeyTransparencyReason, KeyTransparencyState, evaluate_key_transparency};
use mercury_kt::{
    KeyTransparencyProofInput, KeyTransparencyProofStatus as Status,
    KeyTransparencyWitnessStatus as Witness, KtDirectory, TransparencyBundle, proof_input,
    verify_against_signed_head, verify_consistency, verify_inclusion, verify_key_history,
    verify_transparency_bundle,
};

const LABEL: &str = "device:alice:a1f3";
const LABEL_B: &str = "device:bob:9c20";
const KEY_V1: &[u8] = b"alice-device-key-v1";
const KEY_V2: &[u8] = b"alice-device-key-v2";
const KEY_B: &[u8] = b"bob-device-key-v1";

/// Bundle three real, independently-verified statuses into the gate input with a
/// fresh, non-rolled-back log and no witness requirement (witness quorum is the
/// remaining S6 increment).
fn bundle(
    inclusion: Status,
    consistency: Status,
    key_history: Status,
) -> KeyTransparencyProofInput {
    proof_input(
        inclusion,
        consistency,
        key_history,
        Witness::NotRequired,
        false, // require_witness
        1,     // previous_tree_size
        2,     // current_tree_size (>= previous: no rollback)
        30,    // proof_age_s
        300,   // max_proof_age_s
    )
}

#[tokio::test]
async fn append_only_proof_verifies_across_epochs() {
    // Two publishes -> two epochs. The append-only proof for the 1->2 transition
    // must verify against the real per-epoch root checkpoints.
    let mut dir = KtDirectory::new().await.expect("directory");
    let ep1 = dir.register(LABEL, KEY_V1).await.expect("publish epoch 1");
    let ep2 = dir.register(LABEL_B, KEY_B).await.expect("publish epoch 2");
    assert_eq!(
        (ep1.epoch(), ep2.epoch()),
        (1, 2),
        "publishes advance epochs"
    );

    let proof = dir.prove_consistency(1, 2).await.expect("audit proof");
    let status = verify_consistency(&[ep1, ep2], proof).await;
    assert_eq!(status, Status::Verified);
}

#[tokio::test]
async fn consistency_with_tampered_checkpoint_is_rejected() {
    // A relay that rewrote the log would present a proof whose recomputed root
    // does not match the client's pinned checkpoint. Simulate that by corrupting
    // the pinned end-of-epoch root hash: the append-only hash check fails.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    let ep2 = dir.register(LABEL_B, KEY_B).await.unwrap();
    let proof = dir.prove_consistency(1, 2).await.unwrap();

    let mut bad_end = ep2.clone();
    bad_end.1 = [0u8; 32]; // corrupt the pinned end-epoch root hash

    let status = verify_consistency(&[ep1, bad_end], proof).await;
    assert_eq!(status, Status::Invalid);

    // And the gate turns that into a rejection requiring user action.
    let decision = evaluate_key_transparency(bundle(Status::Verified, status, Status::Verified));
    assert_eq!(decision.state, KeyTransparencyState::Inconsistent);
    assert_eq!(
        decision.reason,
        KeyTransparencyReason::ConsistencyProofInvalid
    );
    assert!(decision.requires_user_action);
}

#[tokio::test]
async fn too_few_checkpoints_cannot_pass_consistency() {
    // A single checkpoint bounds no transition; the engine must not vacuously
    // accept it (fail closed rather than "no transition == consistent").
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    dir.register(LABEL_B, KEY_B).await.unwrap();
    let proof = dir.prove_consistency(1, 2).await.unwrap();

    assert_eq!(verify_consistency(&[ep1], proof).await, Status::Invalid);
}

#[tokio::test]
async fn consistency_rejects_nonconsecutive_checkpoint_epochs() {
    // Two checkpoints for the SAME epoch (or any non-consecutive pair) bound no
    // real transition; the epoch-chain guard must reject even though the hash
    // count parity (2 == 1 + 1) is satisfied.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    dir.register(LABEL_B, KEY_B).await.unwrap();
    let proof = dir.prove_consistency(1, 2).await.unwrap();

    // [epoch1, epoch1] -> not strictly consecutive -> Invalid.
    let status = verify_consistency(&[ep1.clone(), ep1], proof).await;
    assert_eq!(status, Status::Invalid);
}

#[tokio::test]
async fn consistency_rejects_proof_for_a_different_epoch_window() {
    // The core composition attack: a relay presents a genuine append-only proof
    // for the 2->3 window but the client pins checkpoints for the 1->2 window.
    // Count parity holds, but the proof's start epoch (2) does not match the
    // pinned chain's (1) -> the epoch binding rejects it.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap(); // epoch 1
    let ep2 = dir.register(LABEL_B, KEY_B).await.unwrap(); // epoch 2
    dir.register(LABEL, KEY_V2).await.unwrap(); // epoch 3

    let wrong_window = dir.prove_consistency(2, 3).await.unwrap(); // proof.epochs == [2]
    let status = verify_consistency(&[ep1, ep2], wrong_window).await; // pinned 1->2
    assert_eq!(status, Status::Invalid);
}

#[tokio::test]
async fn key_history_proves_full_rotation_chain() {
    // Rotate the same label twice; the key-history proof must carry and verify
    // the complete ordered chain of versions against the pinned VRF key.
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY_V1).await.unwrap(); // version 1
    dir.register(LABEL, KEY_V2).await.unwrap(); // version 2 (rotation)

    let (proof, checkpoint) = dir.prove_key_history(LABEL).await.expect("history proof");
    let pk = dir.public_key().await.unwrap();

    let status = verify_key_history(&pk, &checkpoint, LABEL, proof);
    assert_eq!(status, Status::Verified);
}

#[tokio::test]
async fn key_history_rejects_corrupted_pinned_key() {
    // The VRF public key is pinned out-of-band; a corrupted pinned key must fail
    // verification (the history is bound to the pinned key). This corrupts the
    // pinned bytes; a genuine *cross-directory* substitution (a valid foreign
    // directory's key) is covered separately now that each directory has a
    // distinct VRF key -- see `proof_from_another_directory_is_rejected`.
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY_V1).await.unwrap();
    dir.register(LABEL, KEY_V2).await.unwrap();
    let (proof, checkpoint) = dir.prove_key_history(LABEL).await.unwrap();

    let mut corrupted_pk = dir.public_key().await.unwrap();
    for b in &mut corrupted_pk {
        *b ^= 0xff;
    }

    let status = verify_key_history(&corrupted_pk, &checkpoint, LABEL, proof);
    assert_eq!(status, Status::Invalid);
}

#[tokio::test]
async fn key_history_rejects_tampered_checkpoint_root() {
    // A relay that anchors a real history proof to a tree root the client did not
    // pin must be caught: corrupt the checkpoint hash -> verification fails.
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY_V1).await.unwrap();
    dir.register(LABEL, KEY_V2).await.unwrap();
    let (proof, checkpoint) = dir.prove_key_history(LABEL).await.unwrap();
    let pk = dir.public_key().await.unwrap();

    let mut bad = checkpoint.clone();
    bad.1 = [0u8; 32]; // corrupt the pinned root hash the history anchors to

    assert_eq!(verify_key_history(&pk, &bad, LABEL, proof), Status::Invalid);
}

#[tokio::test]
async fn empty_pinned_key_rejects_key_history() {
    // Mirror the inclusion guard: never hand an empty key to the verifier.
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY_V1).await.unwrap();
    dir.register(LABEL, KEY_V2).await.unwrap();
    let (proof, checkpoint) = dir.prove_key_history(LABEL).await.unwrap();

    assert_eq!(
        verify_key_history(&[], &checkpoint, LABEL, proof),
        Status::Invalid
    );
}

#[tokio::test]
async fn all_three_real_dimensions_compose_to_gate_accept() {
    // The strongest assertion: inclusion + consistency + key-history are ALL
    // produced and verified against real AKD on one directory, then fed to the
    // actual mercury-core gate, which returns Consistent with no user action.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    let ep2 = dir.register(LABEL, KEY_V2).await.unwrap(); // rotation -> epoch 2
    let pk = dir.public_key().await.unwrap();

    // Inclusion: the label currently maps to its latest key (v2).
    let (incl_proof, incl_ckpt) = dir.prove_inclusion(LABEL).await.unwrap();
    let inclusion = verify_inclusion(&pk, &incl_ckpt, LABEL, KEY_V2, incl_proof);

    // Consistency: the 1->2 transition only appended.
    let cons_proof = dir.prove_consistency(1, 2).await.unwrap();
    let consistency = verify_consistency(&[ep1, ep2], cons_proof).await;

    // Key history: the full rotation chain verifies.
    let (hist_proof, hist_ckpt) = dir.prove_key_history(LABEL).await.unwrap();
    let key_history = verify_key_history(&pk, &hist_ckpt, LABEL, hist_proof);

    assert_eq!(
        (inclusion, consistency, key_history),
        (Status::Verified, Status::Verified, Status::Verified),
        "every real proof dimension must verify"
    );

    let decision = evaluate_key_transparency(bundle(inclusion, consistency, key_history));
    assert_eq!(decision.state, KeyTransparencyState::Consistent);
    assert_eq!(decision.reason, KeyTransparencyReason::Consistent);
    assert!(!decision.requires_user_action);
}

#[tokio::test]
async fn bound_bundle_verifies_and_gate_accepts() {
    // The canonical client path: verify_transparency_bundle binds inclusion +
    // key-history + consistency to ONE head (the tail of the pinned chain) and
    // derives the gate's tree sizes from the chain's epochs.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    let ep2 = dir.register(LABEL, KEY_V2).await.unwrap(); // rotation -> head at epoch 2
    let pk = dir.public_key().await.unwrap();

    let (inclusion, _) = dir.prove_inclusion(LABEL).await.unwrap();
    let consistency = dir.prove_consistency(1, 2).await.unwrap();
    let (key_history, _) = dir.prove_key_history(LABEL).await.unwrap();

    let input = verify_transparency_bundle(
        &pk,
        &[ep1, ep2], // pinned chain, oldest first; head = epoch 2
        LABEL,
        KEY_V2, // the current key the client expects
        TransparencyBundle {
            inclusion,
            consistency,
            key_history,
        },
        Witness::NotRequired,
        false,
        30,
        300,
    )
    .await;

    // Tree sizes were derived from the chain epochs (1 and 2), not trusted.
    assert_eq!((input.previous_tree_size, input.current_tree_size), (1, 2));

    let decision = evaluate_key_transparency(input);
    assert_eq!(decision.state, KeyTransparencyState::Consistent);
    assert!(!decision.requires_user_action);
}

#[tokio::test]
async fn bound_bundle_with_short_chain_fails_closed() {
    // A pinned chain with no previous->current span cannot prove append-only;
    // the bundle must fail every dimension closed rather than partially trust.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    let (inclusion, _) = dir.prove_inclusion(LABEL).await.unwrap();
    // A degenerate single-epoch "audit" cannot be produced (start < end), so reuse
    // a real proof object; the short chain must reject before it even matters.
    dir.register(LABEL, KEY_V2).await.unwrap();
    let consistency = dir.prove_consistency(1, 2).await.unwrap();
    let (key_history, _) = dir.prove_key_history(LABEL).await.unwrap();
    let pk = dir.public_key().await.unwrap();

    let input = verify_transparency_bundle(
        &pk,
        &[ep1], // only one checkpoint -> no span
        LABEL,
        KEY_V1,
        TransparencyBundle {
            inclusion,
            consistency,
            key_history,
        },
        Witness::NotRequired,
        false,
        30,
        300,
    )
    .await;

    assert_eq!(input.inclusion, Status::Invalid);
    assert_eq!(input.consistency, Status::Invalid);
    assert_eq!(input.key_history, Status::Invalid);
    let decision = evaluate_key_transparency(input);
    assert_eq!(decision.state, KeyTransparencyState::Inconsistent);
    assert!(decision.requires_user_action);
}

#[tokio::test]
async fn bound_bundle_rejects_mismatched_consistency_window() {
    // End-to-end single-head binding: the relay anchors a real inclusion + a real
    // key-history proof at the correct head (epoch 3) but supplies a consistency
    // proof for the WRONG window (1->2 instead of 2->3). The bundle must reject:
    // inclusion + key-history verify, but consistency is Invalid -> gate rejects.
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY_V1).await.unwrap(); // epoch 1
    let ep2 = dir.register(LABEL, KEY_V2).await.unwrap(); // epoch 2 (LABEL -> v2)
    let ep3 = dir.register(LABEL_B, KEY_B).await.unwrap(); // epoch 3 (head)
    let pk = dir.public_key().await.unwrap();

    let (inclusion, _) = dir.prove_inclusion(LABEL).await.unwrap(); // at head, LABEL -> v2
    let (key_history, _) = dir.prove_key_history(LABEL).await.unwrap();
    let mismatched = dir.prove_consistency(1, 2).await.unwrap(); // wrong window for [ep2,ep3]

    let input = verify_transparency_bundle(
        &pk,
        &[ep2, ep3], // pinned chain head = epoch 3, needs a 2->3 proof
        LABEL,
        KEY_V2,
        TransparencyBundle {
            inclusion,
            consistency: mismatched,
            key_history,
        },
        Witness::NotRequired,
        false,
        30,
        300,
    )
    .await;

    assert_eq!(
        input.inclusion,
        Status::Verified,
        "inclusion is at the real head"
    );
    assert_eq!(
        input.key_history,
        Status::Verified,
        "history is at the real head"
    );
    assert_eq!(
        input.consistency,
        Status::Invalid,
        "wrong consistency window rejected"
    );
    let decision = evaluate_key_transparency(input);
    assert_eq!(decision.state, KeyTransparencyState::Inconsistent);
    assert_eq!(
        decision.reason,
        KeyTransparencyReason::ConsistencyProofInvalid
    );
    assert!(decision.requires_user_action);
}

#[tokio::test]
async fn end_to_end_against_signed_head_accepts() {
    // The capstone: the client AUTHENTICATES the head via the log signature, then
    // binds inclusion + key-history + consistency to that authenticated head.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap(); // client's last-pinned head
    dir.register(LABEL, KEY_V2).await.unwrap(); // rotation -> head at epoch 2
    let vrf_pk = dir.public_key().await.unwrap();
    let log_pk = dir.log_public_key();
    let (sth, log_sig) = dir.signed_tree_head(1_000).await.unwrap(); // signed head @ epoch 2

    let (inclusion, _) = dir.prove_inclusion(LABEL).await.unwrap();
    let (key_history, _) = dir.prove_key_history(LABEL).await.unwrap();
    let consistency = dir.prove_consistency(1, 2).await.unwrap();

    let input = verify_against_signed_head(
        &vrf_pk,
        &log_pk,
        &sth,
        &log_sig,
        &ep1, // previously-pinned checkpoint
        LABEL,
        KEY_V2,
        TransparencyBundle {
            inclusion,
            consistency,
            key_history,
        },
        Witness::NotRequired,
        false,
        1_030, // now_s: head timestamp (1_000) + 30s -> the proof is 30s old (fresh)
        300,
    )
    .await;

    // Tree sizes derived from the authenticated head (epoch 1 -> 2).
    assert_eq!((input.previous_tree_size, input.current_tree_size), (1, 2));
    let decision = evaluate_key_transparency(input);
    assert_eq!(decision.state, KeyTransparencyState::Consistent);
    assert!(!decision.requires_user_action);
}

#[tokio::test]
async fn end_to_end_rejects_a_forged_head_signature() {
    // If the head's log signature does not verify, the head is unauthenticated and
    // EVERY dimension is forced closed -- no proof hanging off a forged head counts.
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    dir.register(LABEL, KEY_V2).await.unwrap();
    let vrf_pk = dir.public_key().await.unwrap();
    let log_pk = dir.log_public_key();
    let (sth, log_sig) = dir.signed_tree_head(1_000).await.unwrap();

    let (inclusion, _) = dir.prove_inclusion(LABEL).await.unwrap();
    let (key_history, _) = dir.prove_key_history(LABEL).await.unwrap();
    let consistency = dir.prove_consistency(1, 2).await.unwrap();

    let mut forged = log_sig;
    forged[0] ^= 0xff; // corrupt the log signature on the head

    let input = verify_against_signed_head(
        &vrf_pk,
        &log_pk,
        &sth,
        &forged,
        &ep1,
        LABEL,
        KEY_V2,
        TransparencyBundle {
            inclusion,
            consistency,
            key_history,
        },
        Witness::NotRequired,
        false,
        1_030, // now_s: head timestamp (1_000) + 30s (irrelevant here — the forged head fails closed)
        300,
    )
    .await;

    assert_eq!(input.inclusion, Status::Invalid);
    assert_eq!(input.consistency, Status::Invalid);
    assert_eq!(input.key_history, Status::Invalid);
    let decision = evaluate_key_transparency(input);
    assert_eq!(decision.state, KeyTransparencyState::Inconsistent);
    assert!(decision.requires_user_action);
}

#[tokio::test]
async fn end_to_end_rejects_a_future_dated_head() {
    // Freshness is now DERIVED from the head's OWN signed timestamp. A head dated in the FUTURE
    // relative to `now_s` yields a negative age the gate rejects (BadFreshnessWindow). Previously
    // the age was caller-supplied, so a future-dated head could be passed off as fresh -- this
    // locks the fix (a skewed or malicious "now" can no longer mask a future-dated head).
    let mut dir = KtDirectory::new().await.unwrap();
    let ep1 = dir.register(LABEL, KEY_V1).await.unwrap();
    dir.register(LABEL, KEY_V2).await.unwrap();
    let vrf_pk = dir.public_key().await.unwrap();
    let log_pk = dir.log_public_key();
    let (sth, log_sig) = dir.signed_tree_head(1_000).await.unwrap(); // head dated t=1_000

    let (inclusion, _) = dir.prove_inclusion(LABEL).await.unwrap();
    let (key_history, _) = dir.prove_key_history(LABEL).await.unwrap();
    let consistency = dir.prove_consistency(1, 2).await.unwrap();

    let input = verify_against_signed_head(
        &vrf_pk,
        &log_pk,
        &sth,
        &log_sig,
        &ep1,
        LABEL,
        KEY_V2,
        TransparencyBundle {
            inclusion,
            consistency,
            key_history,
        },
        Witness::NotRequired,
        false,
        900, // now_s is BEFORE the head's signed timestamp (1_000) -> negative derived age
        300,
    )
    .await;

    assert!(
        input.proof_age_s < 0,
        "a future-dated head must yield a negative derived age, got {}",
        input.proof_age_s
    );
    let decision = evaluate_key_transparency(input);
    assert_ne!(
        decision.state,
        KeyTransparencyState::Consistent,
        "a future-dated head must NOT be accepted as consistent"
    );
    assert!(decision.requires_user_action);
}
