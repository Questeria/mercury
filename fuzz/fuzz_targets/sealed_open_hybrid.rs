#![no_main]
//! Coverage-guided fuzzing of the HYBRID AEAD seam `open_hybrid` (X25519 +
//! ML-KEM-768). Arbitrary/malformed sealed bytes must fail closed (Err), never
//! panic -- across the framing parse, the X25519 DH, the ML-KEM decapsulation
//! (implicit rejection), and the AEAD. One fixed-per-process recipient so the
//! fuzzer varies only the input. (Same seam as the deterministic hybrid cases in
//! mercury-sealedbox/tests/open_robustness.rs.)

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use mercury_keys::{DeviceKeyPair, MlKemKeyPair};

static RECIPIENT: OnceLock<(DeviceKeyPair, MlKemKeyPair)> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let (x, pq) = RECIPIENT.get_or_init(|| (DeviceKeyPair::generate(), MlKemKeyPair::generate()));
    let _ = mercury_sealedbox::open_hybrid(x, pq, data);
});
