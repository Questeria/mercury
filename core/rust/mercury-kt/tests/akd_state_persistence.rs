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
