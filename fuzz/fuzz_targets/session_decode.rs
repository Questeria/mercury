#![no_main]
//! Coverage-guided fuzzing of the 1:1 session ciphertext parser. Arbitrary or
//! corrupted bytes must fail closed (Err), never panic. (Same seam as the
//! deterministic mercury-session/tests/decode_robustness.rs.)

use libfuzzer_sys::fuzz_target;
use mercury_session::SessionCiphertext;

fuzz_target!(|data: &[u8]| {
    let _ = SessionCiphertext::from_bytes(data);
});
