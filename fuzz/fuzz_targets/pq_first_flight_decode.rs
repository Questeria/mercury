#![no_main]
//! Coverage-guided fuzzing of the post-quantum hybrid first-flight wire parser.
//! Arbitrary or corrupted bytes must fail closed (Err), never panic. (Same seam as
//! the deterministic mercury-session/tests/decode_robustness.rs.)

use libfuzzer_sys::fuzz_target;
use mercury_session::HybridFirstFlight;

fuzz_target!(|data: &[u8]| {
    let _ = HybridFirstFlight::from_bytes(data);
});
