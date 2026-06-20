//! The witness/auditor co-sign DECISION (`evaluate_head`) — the safety core of a deployed witness.
//! A witness exists to refuse any head that is not an append-only extension of what it already
//! vouched for. These tests drive a real AKD directory to produce genuine signed heads + consistency
//! proofs and assert every branch: the three safe-to-cosign paths and the five fail-closed refusals,
//! including the two equivocation signals (a same-epoch fork and an inconsistent extension).

use mercury_kt::{KtDirectory, SignedTreeHead, WitnessDecision, WitnessRefusal, evaluate_head};

const SEED: [u8; 32] = [9u8; 32];
const NOW: i64 = 1_900_000_000;

/// Three consecutive epochs of a real directory, with the signed head + log signature captured at
/// each, plus the pinned log key. Heads are what a real witness actually holds (it never sees the
/// internal `EpochHash`es — only published heads), so `last_seen` is always built from a head.
struct Fixture {
    log_key: [u8; 32],
    dir: KtDirectory,
    h0: SignedTreeHead,
    s0: [u8; 64],
    h1: SignedTreeHead,
    s1: [u8; 64],
    h2: SignedTreeHead,
    s2: [u8; 64],
}

async fn fixture() -> Fixture {
    let mut dir = KtDirectory::with_vrf_seed(SEED).await.unwrap();
    let log_key = dir.log_public_key();
    let (h0, s0) = dir.signed_tree_head(NOW).await.unwrap();
    dir.register("username:alice", b"id-alice").await.unwrap(); // -> epoch 1
    let (h1, s1) = dir.signed_tree_head(NOW).await.unwrap();
    dir.register("username:bob", b"id-bob").await.unwrap(); // -> epoch 2
    let (h2, s2) = dir.signed_tree_head(NOW).await.unwrap();
    assert_eq!(
        (h0.tree_size, h1.tree_size, h2.tree_size),
        (0, 1, 2),
        "the fixture advances exactly one epoch per register"
    );
    Fixture { log_key, dir, h0, s0, h1, s1, h2, s2 }
}

#[tokio::test]
async fn the_three_safe_to_cosign_paths() {
    let fx = fixture().await;

    // 1. First head this witness ever sees -> trust-on-first-use (like a client's initial pin).
    assert_eq!(
        evaluate_head(&fx.log_key, None, &fx.h1, &fx.s1, None).await,
        WitnessDecision::Cosign,
        "a validly-signed first head is pinned on first use"
    );

    // 2. The SAME head, re-published with a fresh timestamp (the hourly refresh) -> re-cosign.
    assert_eq!(
        evaluate_head(
            &fx.log_key,
            Some((1, fx.h1.root_hash)),
            &fx.h1,
            &fx.s1,
            None,
        )
        .await,
        WitnessDecision::Cosign,
        "re-cosigning the same head we already vouched for is safe"
    );

    // 3. A verified single-epoch append-only advance (1 -> 2) -> cosign.
    let proof = fx.dir.prove_consistency(1, 2).await.unwrap();
    assert_eq!(
        evaluate_head(
            &fx.log_key,
            Some((1, fx.h1.root_hash)),
            &fx.h2,
            &fx.s2,
            Some(proof),
        )
        .await,
        WitnessDecision::Cosign,
        "a proven append-only single-epoch extension is co-signed"
    );
}

#[tokio::test]
async fn the_five_fail_closed_refusals() {
    let fx = fixture().await;

    // A. A forged/corrupt log signature is not a head at all.
    let mut bad_sig = fx.s1;
    bad_sig[0] ^= 0x01;
    assert_eq!(
        evaluate_head(&fx.log_key, None, &fx.h1, &bad_sig, None).await,
        WitnessDecision::Refuse(WitnessRefusal::BadLogSignature),
    );

    // B. The head's epoch is BELOW the last we co-signed: a rollback.
    assert_eq!(
        evaluate_head(
            &fx.log_key,
            Some((2, fx.h2.root_hash)),
            &fx.h1,
            &fx.s1,
            None,
        )
        .await,
        WitnessDecision::Refuse(WitnessRefusal::EpochRegressed),
    );

    // C. Same epoch, DIFFERENT root than we vouched for: the relay forked this epoch (equivocation).
    assert_eq!(
        evaluate_head(
            &fx.log_key,
            Some((1, [0xBB; 32])),
            &fx.h1,
            &fx.s1,
            None,
        )
        .await,
        WitnessDecision::Refuse(WitnessRefusal::SameEpochFork),
        "a same-epoch root that disagrees with our commitment is an equivocation signal"
    );

    // D. The head jumped more than one epoch (0 -> 2): the single proof we can obtain cannot bridge
    //    the gap (we lack the intermediate checkpoint). Fail-closed; re-bootstrap.
    assert_eq!(
        evaluate_head(
            &fx.log_key,
            Some((0, fx.h0.root_hash)),
            &fx.h2,
            &fx.s2,
            None,
        )
        .await,
        WitnessDecision::Refuse(WitnessRefusal::UnverifiableGap),
    );

    // E. A single-epoch advance whose append-only proof does NOT bind our OWN last root to the new
    //    head (here our remembered epoch-1 root disagrees with the relay's): the new head is not a
    //    consistent successor of what we vouched for -> the split-view signal. Refuse hard.
    let proof = fx.dir.prove_consistency(1, 2).await.unwrap();
    assert_eq!(
        evaluate_head(
            &fx.log_key,
            Some((1, [0xAA; 32])),
            &fx.h2,
            &fx.s2,
            Some(proof),
        )
        .await,
        WitnessDecision::Refuse(WitnessRefusal::InconsistentExtension),
        "a head that does not append-only-extend our remembered root is refused"
    );
}
