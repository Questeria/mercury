//! Track 0c robustness: the canonical message parser `from_bytes` must NEVER
//! panic on arbitrary or malformed input -- it must fail closed with `Err`.
//!
//! This is the locally-verifiable, runs-everywhere layer (plain `cargo test`,
//! so it gates crash-resistance in CI on every platform). Coverage-guided
//! cargo-fuzz/libFuzzer (which needs a Linux fuzz toolchain) is the deeper
//! layer that explores execution paths; both target the same `from_bytes` seam.

use mercury_message::from_bytes;

/// Deterministic xorshift64 PRNG -- reproducible, no external deps.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn byte(&mut self) -> u8 {
        (self.next_u64() & 0xff) as u8
    }
}

/// Calling `from_bytes` must not panic; a panic aborts the test (= failure).
fn check(data: &[u8]) {
    let _ = from_bytes(data);
}

#[test]
fn from_bytes_survives_edge_cases() {
    check(&[]);
    for len in 0..512usize {
        check(&vec![0x00u8; len]);
        check(&vec![0xffu8; len]);
        check(&vec![0xa5u8; len]);
    }
}

#[test]
fn from_bytes_survives_random_inputs() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for _ in 0..50_000 {
        let len = (rng.next_u64() % 1024) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        check(&data);
    }
}

#[test]
fn from_bytes_survives_boundary_lengths() {
    // Cluster random inputs at small lengths where header/length-prefix framing
    // is most likely to mis-handle truncation.
    let mut rng = Rng(0x1234_5678_9abc_def0);
    for len in 0..64usize {
        for _ in 0..1_000 {
            let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
            check(&data);
        }
    }
}
