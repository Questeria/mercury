//! Track 0c robustness: the AEAD seams `open` (classical) and `open_hybrid`
//! (X25519 + ML-KEM-768) must NEVER panic on arbitrary or malformed sealed bytes
//! -- they must fail closed with `Err`, including at the exact framing boundaries
//! (classical 104 = 32+32+24+16; hybrid 1192 = 32+1088+32+24+16, where the extra
//! 32 is the key commitment). The hybrid path also exercises the X25519 DH +
//! ML-KEM decapsulation on hostile input.
//!
//! Locally-verifiable, runs-everywhere layer (plain `cargo test`). The deeper
//! coverage-guided cargo-fuzz/libFuzzer layer (Linux fuzz toolchain) targets
//! the same seams.

use mercury_keys::{DeviceKeyPair, MlKemKeyPair};
use mercury_sealedbox::{open, open_hybrid};

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

#[test]
fn open_survives_boundary_lengths() {
    let recipient = DeviceKeyPair::generate();
    // 72 is the minimum well-formed length; sweep across and past it.
    for len in 0..200usize {
        let _ = open(&recipient, &vec![0x00u8; len]);
        let _ = open(&recipient, &vec![0xffu8; len]);
        let _ = open(&recipient, &vec![0xa5u8; len]);
    }
}

#[test]
fn open_survives_arbitrary_ciphertext() {
    let recipient = DeviceKeyPair::generate();
    let mut rng = Rng(0xcafe_f00d_d00d_feed);
    for _ in 0..20_000 {
        let len = (rng.next_u64() % 512) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        // Any tampering, wrong recipient, or bad framing must fail closed.
        let _ = open(&recipient, &data);
    }
}

// 32 (ephemeral X25519) + 32 (commitment) + 24 (nonce) + 16 (tag).
const CLASSICAL_MIN: usize = 104;

#[test]
fn open_survives_well_framed_garbage() {
    // Inputs that are exactly/over the minimum length but otherwise random,
    // exercising the commitment + DH + AEAD-decrypt path (not just the early
    // length reject).
    let recipient = DeviceKeyPair::generate();
    let mut rng = Rng(0x0bad_c0de_dead_beef);
    for _ in 0..10_000 {
        let len = CLASSICAL_MIN + (rng.next_u64() % 256) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        let _ = open(&recipient, &data);
    }
}

// 32 (ephemeral X25519) + 1088 (ML-KEM-768 ct) + 32 (commitment) + 24 (nonce) + 16 (tag).
const HYBRID_MIN: usize = 1192;

#[test]
fn open_hybrid_survives_boundary_lengths() {
    let x = DeviceKeyPair::generate();
    let pq = MlKemKeyPair::generate();
    // Sweep across and past the 1192-byte minimum hybrid framing
    // (32 ephemeral + 1088 ML-KEM ct + 32 commitment + 24 nonce + 16 tag).
    for len in 0..1200usize {
        let _ = open_hybrid(&x, &pq, &vec![0x00u8; len]);
        let _ = open_hybrid(&x, &pq, &vec![0xffu8; len]);
        let _ = open_hybrid(&x, &pq, &vec![0xa5u8; len]);
    }
}

#[test]
fn open_hybrid_survives_arbitrary_input() {
    let x = DeviceKeyPair::generate();
    let pq = MlKemKeyPair::generate();
    let mut rng = Rng(0xfeed_face_cafe_d00d);
    for _ in 0..10_000 {
        let len = (rng.next_u64() % 1400) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        // Any tampering, wrong recipient, or bad framing must fail closed.
        let _ = open_hybrid(&x, &pq, &data);
    }
}

#[test]
fn open_hybrid_survives_well_framed_garbage() {
    // Inputs at/over the 1192 minimum but otherwise random, driving the full
    // X25519 DH + ML-KEM decapsulation + commitment + AEAD path (ML-KEM implicit
    // rejection yields an unrelated secret for a random ciphertext, which then
    // fails the commitment/AEAD -- never a panic).
    let x = DeviceKeyPair::generate();
    let pq = MlKemKeyPair::generate();
    let mut rng = Rng(0x5151_2323_8787_b0b0);
    for _ in 0..2_500 {
        let len = HYBRID_MIN + (rng.next_u64() % 256) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        let _ = open_hybrid(&x, &pq, &data);
    }
}
