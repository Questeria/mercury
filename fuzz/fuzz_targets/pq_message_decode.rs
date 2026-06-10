#![no_main]
//! Coverage-guided fuzzing of the continuous post-quantum message wire parser.
//! Arbitrary or corrupted bytes must fail closed (Err), never panic. (Same seam as
//! the deterministic mercury-session/tests/decode_robustness.rs.)

use libfuzzer_sys::fuzz_target;
use mercury_session::HybridMessage;

fuzz_target!(|data: &[u8]| {
    let _ = HybridMessage::from_bytes(data);
});
