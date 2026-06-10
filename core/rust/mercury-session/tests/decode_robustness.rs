//! Track 0c robustness: every 1:1 session wire parser must NEVER panic on
//! arbitrary or malformed bytes -- only Ok/Err. Deserializing untrusted/corrupted
//! bytes must fail closed. Covers `SessionCiphertext`, `HybridFirstFlight`,
//! `HybridMessage`, and `RatchetMessage` (the canonical versioned codecs).

use mercury_session::{HybridFirstFlight, HybridMessage, RatchetMessage, SessionCiphertext};

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

fn check(data: &[u8]) {
    let _ = SessionCiphertext::from_bytes(data);
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
    let mut rng = Rng(0x51ed_c0de_f00d_1234);
    for _ in 0..50_000 {
        let len = (rng.next_u64() % 1024) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        check(&data);
    }
}

/// Feed the same bytes to every PQ wire parser; none may panic on any input.
fn check_pq(data: &[u8]) {
    let _ = HybridFirstFlight::from_bytes(data);
    let _ = HybridMessage::from_bytes(data);
    let _ = RatchetMessage::from_bytes(data);
}

#[test]
fn pq_codecs_survive_edge_cases() {
    check_pq(&[]);
    // Sweep across every PQ frame boundary: the index frame (5), the first-flight frame
    // (1+1088+40), and well past the ratchet frame (1+1184+1088+8+40 = 2321).
    for len in 0..2400usize {
        check_pq(&vec![0x00u8; len]);
        check_pq(&vec![0xffu8; len]);
        check_pq(&vec![0xa5u8; len]);
        // Same length but with a valid leading version byte, to drive past the
        // version+length guards into the fixed-offset field slicing.
        let mut framed = vec![0x5cu8; len];
        if !framed.is_empty() {
            framed[0] = 1;
        }
        check_pq(&framed);
    }
}

#[test]
fn pq_codecs_survive_random_inputs() {
    let mut rng = Rng(0xc0ff_eeba_df00_d00d);
    for _ in 0..50_000 {
        let len = (rng.next_u64() % 2600) as usize;
        let mut data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        // Half the time, force a valid version byte so the parser proceeds to slicing.
        if !data.is_empty() && rng.byte() & 1 == 0 {
            data[0] = 1;
        }
        check_pq(&data);
    }
}
