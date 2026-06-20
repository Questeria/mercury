//! AKD-state persistence — the fix for the boot-time epoch reset. A snapshot taken before a
//! simulated relay restart RESTORES the full epoch history, so a consistency (append-only) proof
//! SPANS the restart boundary — exactly what the in-memory rebuild-from-bindings path could not do
//! (it reset the epoch counter). Also covers the FAIL-CLOSED contract: a corrupt or
//! structurally-invalid snapshot must NEVER silently rebuild (that would reset history + mask
//! tamper); only a genuinely absent file is the first-run / migration path.

use mercury_kt::{KeyTransparencyProofStatus as Status, KtDirectory, verify_consistency};
use std::path::PathBuf;

const SEED: [u8; 32] = [7u8; 32];

/// A per-test scratch snapshot path (unique by test name so parallel runs do not collide),
/// cleared first so each test starts from a clean slate.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mercury_kt_persist_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("akd_state.json")
}

#[tokio::test]
async fn restart_preserves_epoch_history_and_consistency_spans_the_boundary() {
    let snap = scratch("restart_spans");

    // --- before the restart: two epochs, snapshot after each ---
    let (mut dir1, restored) = KtDirectory::with_vrf_seed_persistent(SEED, &snap).await.unwrap();
    assert!(!restored, "first run: there is no snapshot to restore");
    let ep1 = dir1.register("username:alice", b"id-alice-v1").await.unwrap();
    dir1.snapshot().await.unwrap();
    let ep2 = dir1.register("username:alice", b"id-alice-v2").await.unwrap(); // rotation -> epoch 2
    dir1.snapshot().await.unwrap();
    let vrf_before = dir1.public_key().await.unwrap();
    assert_eq!((ep1.epoch(), ep2.epoch()), (1, 2));
    drop(dir1); // simulate the process exiting (the in-memory tree is gone)

    // --- after the restart: restore from the snapshot ---
    let (mut dir2, restored2) = KtDirectory::with_vrf_seed_persistent(SEED, &snap).await.unwrap();
    assert!(restored2, "the snapshot must be restored on boot");
    // The VRF key is deterministic from the seed, so it is identical across the restart — every
    // proof a client pinned before the restart still verifies.
    assert_eq!(dir2.public_key().await.unwrap(), vrf_before);

    // A THIRD epoch CONTINUES the history (an in-memory reset would mint epoch 1 again).
    let ep3 = dir2.register("username:alice", b"id-alice-v3").await.unwrap();
    dir2.snapshot().await.unwrap();
    assert_eq!(ep3.epoch(), 3, "epoch continued across the restart (a reset would give 1)");

    // The append-only proof from epoch 1 to 3 SPANS the restart and verifies client-side against the
    // per-epoch checkpoints pinned BEFORE (ep1, ep2) and AFTER (ep3) the boundary.
    let proof = dir2.prove_consistency(1, 3).await.unwrap();
    let status = verify_consistency(&[ep1, ep2, ep3], proof).await;
    assert_eq!(status, Status::Verified, "consistency proof must span the restart");
}

#[tokio::test]
async fn a_corrupt_snapshot_fails_closed_not_rebuild() {
    let snap = scratch("corrupt");
    std::fs::write(&snap, b"this is not valid json").unwrap();
    let r = KtDirectory::with_vrf_seed_persistent(SEED, &snap).await;
    assert!(r.is_err(), "a corrupt snapshot must FAIL CLOSED, never silently rebuild");
}

#[tokio::test]
async fn a_snapshot_with_no_azks_record_fails_closed() {
    let snap = scratch("no_azks");
    std::fs::write(&snap, b"[]").unwrap(); // valid JSON but an empty record set (no Azks)
    assert!(
        KtDirectory::with_vrf_seed_persistent(SEED, &snap).await.is_err(),
        "a structurally-invalid snapshot (no Azks) must fail closed"
    );
}

#[tokio::test]
async fn an_absent_snapshot_is_the_migration_path_not_an_error() {
    let snap = scratch("absent"); // scratch() makes the dir but not the file
    let (_dir, restored) = KtDirectory::with_vrf_seed_persistent(SEED, &snap).await.unwrap();
    assert!(!restored, "no snapshot file -> a fresh directory (migration), not an error");
}

#[tokio::test]
async fn reconcile_republishes_only_the_bindings_absent_from_the_restored_snapshot() {
    let snap = scratch("reconcile");

    // Pre-restart: alice is registered AND snapshotted (her epoch reaches disk).
    let (mut dir1, _) = KtDirectory::with_vrf_seed_persistent(SEED, &snap).await.unwrap();
    dir1.register("username:alice", b"id-alice").await.unwrap();
    dir1.snapshot().await.unwrap();
    drop(dir1);

    // Restart. The restored snapshot has alice but NOT bob — modelling the crash residual where the
    // durable username store recorded bob (his `claim` landed) but the relay died before bob's KT
    // epoch was snapshotted. Boot then reconciles against the authoritative store = [alice, bob].
    let (mut dir2, restored) = KtDirectory::with_vrf_seed_persistent(SEED, &snap).await.unwrap();
    assert!(restored);
    assert!(dir2.prove_inclusion("username:alice").await.is_ok(), "alice survived the restart");
    assert!(dir2.prove_inclusion("username:bob").await.is_err(), "bob was not in the snapshot");

    let bindings = vec![
        ("username:alice".to_string(), b"id-alice".to_vec()),
        ("username:bob".to_string(), b"id-bob".to_vec()),
    ];
    let republished = dir2.reconcile(&bindings).await.unwrap();
    assert_eq!(republished, 1, "only bob (absent) is re-published; alice is already present");
    assert!(dir2.prove_inclusion("username:bob").await.is_ok(), "bob is provable after reconcile");

    // Reconcile is idempotent + persists: a second pass finds nothing, and the catch-up was
    // snapshotted (a fresh restore sees bob without needing to reconcile again).
    assert_eq!(dir2.reconcile(&bindings).await.unwrap(), 0, "second reconcile is a no-op");
    drop(dir2);
    let (dir3, _) = KtDirectory::with_vrf_seed_persistent(SEED, &snap).await.unwrap();
    assert!(dir3.prove_inclusion("username:bob").await.is_ok(), "the catch-up was persisted");
}

#[tokio::test]
async fn reconcile_leaves_orphan_akd_labels_alone() {
    let snap = scratch("orphan");

    // The AKD has carol, but the authoritative store does NOT list her (an orphan label — e.g. a
    // claim whose KT register + snapshot landed but whose durable username put failed). Reconcile
    // against a store WITHOUT carol must NOT touch her and must re-publish nothing.
    let (mut dir, _) = KtDirectory::with_vrf_seed_persistent(SEED, &snap).await.unwrap();
    dir.register("username:carol", b"id-carol").await.unwrap();
    dir.snapshot().await.unwrap();

    let store_without_carol: Vec<(String, Vec<u8>)> = vec![];
    assert_eq!(
        dir.reconcile(&store_without_carol).await.unwrap(),
        0,
        "an orphan AKD label is tolerated, never re-published or pruned"
    );
    assert!(dir.prove_inclusion("username:carol").await.is_ok(), "the orphan label is untouched");
}
