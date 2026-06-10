#![no_main]
//! Coverage-guided fuzzing of the AEAD seam `open`. Arbitrary/malformed sealed
//! bytes must fail closed (Err), never panic. Uses one fixed-per-process
//! recipient so the fuzzer varies only the input. (Same seam as the
//! deterministic mercury-sealedbox/tests/open_robustness.rs.)

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use mercury_keys::DeviceKeyPair;

static RECIPIENT: OnceLock<DeviceKeyPair> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let recipient = RECIPIENT.get_or_init(DeviceKeyPair::generate);
    let _ = mercury_sealedbox::open(recipient, data);
});
