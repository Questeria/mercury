//! Stage 6: a real AKD inclusion proof verifies and feeds the mercury-core
//! key-transparency gate; key substitution and a mismatched label are rejected.

use mercury_core::{KeyTransparencyReason, KeyTransparencyState, evaluate_key_transparency};
use mercury_kt::{
    KeyTransparencyProofStatus as Status, KeyTransparencyWitnessStatus as Witness, KtDirectory,
    proof_input, verify_inclusion,
};

const LABEL: &str = "device:alice:a1f3";
const KEY: &[u8] = b"alice-device-key-v1";

/// Assemble a full proof bundle around the (real) inclusion status. The other
/// dimensions stand in as "verified separately" for this increment.
fn bundle(inclusion: Status) -> mercury_kt::KeyTransparencyProofInput {
    proof_input(
        inclusion,
        Status::Verified, // consistency (append-only) -- next increment
        Status::Verified, // key history -- next increment
        Witness::NotRequired,
        false, // require_witness -- witness quorum is the next increment
        0,
        1,
        30,
        300,
    )
}

#[tokio::test]
async fn real_inclusion_proof_verifies_and_gate_accepts() {
    let mut dir = KtDirectory::new().await.expect("directory");
    dir.register(LABEL, KEY).await.expect("publish");
    let (proof, checkpoint) = dir.prove_inclusion(LABEL).await.expect("lookup proof");
    let pk = dir.public_key().await.expect("public key");

    // Client verifies the real proof against the pinned key + checkpoint.
    let inclusion = verify_inclusion(&pk, &checkpoint, LABEL, KEY, proof);
    assert_eq!(inclusion, Status::Verified);

    let decision = evaluate_key_transparency(bundle(inclusion));
    assert_eq!(decision.state, KeyTransparencyState::Consistent);
    assert!(!decision.requires_user_action);
}

#[tokio::test]
async fn key_substitution_is_rejected() {
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY).await.unwrap();
    let (proof, checkpoint) = dir.prove_inclusion(LABEL).await.unwrap();
    let pk = dir.public_key().await.unwrap();

    // The proof is cryptographically valid, but the value is NOT the key the
    // client expected -- a substitution. The engine surfaces Invalid.
    let inclusion = verify_inclusion(&pk, &checkpoint, LABEL, b"attacker-key", proof);
    assert_eq!(inclusion, Status::Invalid);

    let decision = evaluate_key_transparency(bundle(inclusion));
    assert_eq!(decision.state, KeyTransparencyState::Inconsistent);
    assert_eq!(
        decision.reason,
        KeyTransparencyReason::InclusionProofInvalid
    );
    assert!(decision.requires_user_action);
}

#[tokio::test]
async fn proof_for_wrong_label_is_rejected() {
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY).await.unwrap();
    let (proof, checkpoint) = dir.prove_inclusion(LABEL).await.unwrap();
    let pk = dir.public_key().await.unwrap();

    // Verifying alice's proof as if it were bob's must fail cryptographically
    // (the VRF binds the label) -> Invalid.
    let inclusion = verify_inclusion(&pk, &checkpoint, "device:bob:9c20", KEY, proof);
    assert_eq!(inclusion, Status::Invalid);

    let decision = evaluate_key_transparency(bundle(inclusion));
    assert_eq!(
        decision.reason,
        KeyTransparencyReason::InclusionProofInvalid
    );
}

#[tokio::test]
async fn proof_from_another_directory_is_rejected() {
    // Each directory now has a DISTINCT per-directory VRF key (no longer the
    // shared demo key), so an inclusion proof from directory A must fail
    // verification under directory B's pinned key -- the real VRF-key-
    // substitution defense that the hard-coded demo key could not provide.
    let mut dir_a = KtDirectory::new().await.unwrap();
    dir_a.register(LABEL, KEY).await.unwrap();
    let (proof, checkpoint) = dir_a.prove_inclusion(LABEL).await.unwrap();
    let pk_a = dir_a.public_key().await.unwrap();

    let dir_b = KtDirectory::new().await.unwrap();
    let pk_b = dir_b.public_key().await.unwrap();

    assert_ne!(
        pk_a, pk_b,
        "distinct directories have distinct VRF public keys"
    );
    // A genuine proof from A, verified against B's key, is rejected.
    let inclusion = verify_inclusion(&pk_b, &checkpoint, LABEL, KEY, proof);
    assert_eq!(inclusion, Status::Invalid);
    assert_eq!(
        evaluate_key_transparency(bundle(inclusion)).reason,
        KeyTransparencyReason::InclusionProofInvalid
    );
}

#[tokio::test]
async fn vrf_seed_determines_a_stable_public_key() {
    // The production persistence path: a directory built from a fixed VRF seed
    // has a STABLE public key (clients keep pinning it across restarts), and a
    // different seed yields a different key.
    let dir1 = KtDirectory::with_vrf_seed([7u8; 32]).await.unwrap();
    let dir2 = KtDirectory::with_vrf_seed([7u8; 32]).await.unwrap();
    let dir3 = KtDirectory::with_vrf_seed([8u8; 32]).await.unwrap();

    let pk1 = dir1.public_key().await.unwrap();
    let pk2 = dir2.public_key().await.unwrap();
    let pk3 = dir3.public_key().await.unwrap();

    assert_eq!(pk1, pk2, "same VRF seed -> same pinnable public key");
    assert_ne!(pk1, pk3, "different VRF seed -> different public key");
}
