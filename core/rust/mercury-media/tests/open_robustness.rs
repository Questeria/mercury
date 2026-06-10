//! Track 0c robustness: `open_attachment` must NEVER panic on arbitrary,
//! malformed, truncated, or mutated input -- it must fail closed with `Err`.
//!
//! A blob store is UNTRUSTED: it can hand a client any bytes for an attachment.
//! The decoder parses a header (magic/version/plaintext_len), computes the exact
//! expected frame length, and reads each chunk to an exact boundary; this asserts
//! that no attacker-controlled length, count, or byte content can drive an
//! index/slice panic. This is the locally-verifiable, runs-everywhere layer.

use mercury_media::{AttachmentKey, open_attachment, seal_attachment_with_key};

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

fn key() -> AttachmentKey {
    AttachmentKey::from_bytes([0x42u8; 32])
}

/// Opening must not panic; a panic aborts the test (= failure).
fn check(k: &AttachmentKey, data: &[u8]) {
    let _ = open_attachment(k, data);
}

#[test]
fn open_attachment_survives_edge_cases() {
    let k = key();
    check(&k, &[]);
    for len in 0..512usize {
        check(&k, &vec![0x00u8; len]);
        check(&k, &vec![0xffu8; len]);
        check(&k, &vec![0xa5u8; len]);
    }
}

#[test]
fn open_attachment_survives_random_inputs() {
    let k = key();
    let mut rng = Rng(0x0bad_f00d_dead_c0de);
    for _ in 0..50_000 {
        let len = (rng.next_u64() % 2048) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        check(&k, &data);
    }
}

#[test]
fn open_attachment_survives_boundary_lengths() {
    // Cluster at small lengths around the 13-byte header where length-prefix
    // framing is most likely to mishandle truncation.
    let k = key();
    let mut rng = Rng(0x1122_3344_5566_7788);
    for len in 0..64usize {
        for _ in 0..1_000 {
            let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
            check(&k, &data);
        }
    }
}

#[test]
fn open_attachment_survives_mutated_real_frames() {
    // Start from a REAL multi-chunk sealed attachment and corrupt it: this drives
    // the decoder past the header into per-chunk splitting/decryption, where
    // random bytes rarely reach. Also stresses attacker-controlled header lengths.
    let k = key();
    let base = seal_attachment_with_key(&k, &[0x5au8; 64 * 1024 * 2 + 123]).unwrap();
    assert!(!base.is_empty());

    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for _ in 0..20_000 {
        let mut data = base.clone();
        match rng.next_u64() % 3 {
            0 => {
                // Flip 1..=6 random bytes (incl. possibly the header length field).
                let flips = 1 + (rng.next_u64() % 6) as usize;
                for _ in 0..flips {
                    let pos = (rng.next_u64() as usize) % data.len();
                    data[pos] ^= rng.byte();
                }
            }
            1 => {
                // Overwrite the 8-byte plaintext_len header field (bytes
                // [21..29] = after magic(4)+version(1)+salt(16)) with an arbitrary
                // value, exercising the overflow / expected-length guards.
                let bytes = rng.next_u64().to_le_bytes();
                data[21..29].copy_from_slice(&bytes);
            }
            _ => {
                // Truncate to a random shorter length.
                let keep = (rng.next_u64() as usize) % (data.len() + 1);
                data.truncate(keep);
            }
        }
        check(&k, &data);
    }
}
