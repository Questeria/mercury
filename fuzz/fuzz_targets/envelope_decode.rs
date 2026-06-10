#![no_main]
//! Coverage-guided fuzzing of the canonical message parser. It must never
//! panic on any input -- only return Ok/Err. (Same seam as the deterministic
//! mercury-message/tests/decode_robustness.rs.)

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = mercury_message::from_bytes(data);
});
