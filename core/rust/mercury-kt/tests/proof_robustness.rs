//! Track-0c crash resistance for the KT WIRE path. A malicious relay can send a
//! client arbitrary JSON in place of a real proof; the client's deserialize +
//! verify path must never panic on hostile or corrupted input (it may only
//! return a parse error or a non-`Verified` status). Soundness against
//! structurally-valid-but-wrong proofs is covered by the targeted tests
//! (`inclusion.rs`, `consistency_history.rs`); this guards CRASH resistance over
//! ~80k adversarial inputs.

use mercury_kt::{
    AppendOnlyProof, HistoryProof, KeyTransparencyProofStatus as Status, KtDirectory, LookupProof,
    verify_inclusion,
};

const LABEL: &str = "device:alice:a1f3";
const KEY: &[u8] = b"alice-device-key";

/// Deterministic xorshift64 PRNG — reproducible hostile inputs, no dependency.
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

#[tokio::test]
async fn corrupted_inclusion_proofs_never_panic() {
    // Take a REAL serialized inclusion proof, flip random bytes, and feed it back
    // through deserialize -> verify. Parsing usually fails; when it parses, the
    // verify must run to completion WITHOUT PANICKING (its status -- almost always
    // Invalid -- is irrelevant here; soundness against wrong proofs is covered by
    // the targeted tests). Reaching the end of the loop is the assertion.
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY).await.unwrap();
    let (proof, checkpoint) = dir.prove_inclusion(LABEL).await.unwrap();
    let pk = dir.public_key().await.unwrap();
    let canonical = serde_json::to_vec(&proof).unwrap();

    let mut rng = 0x9E37_79B9_7F4A_7C15u64;
    let mut parsed = 0u64;
    for _ in 0..20_000 {
        let mut bytes = canonical.clone();
        let flips = 1 + (xorshift64(&mut rng) % 8);
        for _ in 0..flips {
            let i = (xorshift64(&mut rng) as usize) % bytes.len();
            // Non-zero mask so a flip is never a silent no-op.
            let mask = 1 + (xorshift64(&mut rng) % 255) as u8;
            bytes[i] ^= mask;
        }
        if let Ok(corrupt) = serde_json::from_slice::<LookupProof>(&bytes) {
            parsed += 1;
            let _: Status = verify_inclusion(&pk, &checkpoint, LABEL, KEY, corrupt);
        }
    }
    let _ = parsed; // reaching here == no panic across 20k corruptions
}

#[tokio::test]
async fn random_bytes_never_panic_on_proof_deserialization() {
    // Pure garbage of varying length, deserialized as each served proof type.
    // The only acceptable outcomes are Ok or Err -- never a panic.
    let mut rng = 0x1234_5678_9ABC_DEF0u64;
    for _ in 0..20_000 {
        let len = (xorshift64(&mut rng) % 512) as usize;
        let mut bytes = vec![0u8; len];
        for b in &mut bytes {
            *b = (xorshift64(&mut rng) & 0xff) as u8;
        }
        let _ = serde_json::from_slice::<LookupProof>(&bytes);
        let _ = serde_json::from_slice::<AppendOnlyProof>(&bytes);
        let _ = serde_json::from_slice::<HistoryProof>(&bytes);
    }
}
