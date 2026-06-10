//! Track 0c robustness: the relay hex decoder must NEVER panic on arbitrary
//! input strings -- including odd lengths, non-hex bytes, and multi-byte
//! Unicode (which would expose any non-char-boundary slicing bug). It must
//! only return Ok/Err.

use mercury_relay::hex;

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
}

#[test]
fn decode_survives_edge_cases() {
    for s in [
        "", "0", "00", "000", "0g", "zz", "GG", "  ", "0x", "0x00", "deadbeef", "DEADBEEF",
        "DeAdBeEf", "12 34", "\n", "\t", "ÿÿ", "café", "🦀", "a🦀b",
    ] {
        let _ = hex::decode(s);
    }
}

#[test]
fn decode_survives_random_ascii() {
    let mut rng = Rng(0xa11c_e0ff_1ce0_0001);
    for _ in 0..50_000 {
        let len = (rng.next_u64() % 256) as usize;
        let s: String = (0..len)
            .map(|_| ((rng.next_u64() % 128) as u8) as char)
            .collect();
        let _ = hex::decode(&s);
    }
    // Bias toward the hex alphabet so deeper decode paths are exercised.
    let alphabet = b"0123456789abcdefABCDEF";
    for _ in 0..50_000 {
        let len = (rng.next_u64() % 64) as usize;
        let s: String = (0..len)
            .map(|_| alphabet[(rng.next_u64() as usize) % alphabet.len()] as char)
            .collect();
        let _ = hex::decode(&s);
    }
}

#[test]
fn decode_survives_arbitrary_unicode() {
    let mut rng = Rng(0xfeed_face_cafe_0009);
    for _ in 0..30_000 {
        let len = (rng.next_u64() % 32) as usize;
        let s: String = (0..len)
            .filter_map(|_| char::from_u32((rng.next_u64() % 0x11_0000) as u32))
            .collect();
        let _ = hex::decode(&s);
    }
}
