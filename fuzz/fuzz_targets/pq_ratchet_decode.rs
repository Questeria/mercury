#![no_main]
//! Coverage-guided fuzzing of the post-quantum ratchet message wire parser.
//! Arbitrary or corrupted bytes must fail closed (Err), never panic. (Same seam as
//! the deterministic mercury-session/tests/decode_robustness.rs.)

use libfuzzer_sys::fuzz_target;
use mercury_session::RatchetMessage;

fuzz_target!(|data: &[u8]| {
    let _ = RatchetMessage::from_bytes(data);
});
