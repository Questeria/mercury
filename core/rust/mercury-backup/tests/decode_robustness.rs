//! Robustness: `SealedBackup::from_bytes` (the encrypted-backup wire parser) must
//! NEVER panic on arbitrary or malformed bytes -- only Ok/Err. An untrusted or
//! corrupted backup blob (e.g. fetched from cloud storage) must fail closed.

use mercury_backup::SealedBackup;

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
    let _ = SealedBackup::from_bytes(data);
}

#[test]
fn from_bytes_survives_edge_cases() {
    check(&[]);
    for len in 0..512usize {
        check(&vec![0x00u8; len]);
        check(&vec![0xffu8; len]);
        check(&vec![0xa5u8; len]);
        // Frames carrying the real magic ("MBAK") plus both supported versions drive
        // execution past the magic guard into version-specific header parsing,
        // which is where a bounds bug would surface.
        for version in [1u8, 2u8] {
            let mut framed = vec![0x5cu8; len];
            if framed.len() >= 5 {
                framed[0] = b'M';
                framed[1] = b'B';
                framed[2] = b'A';
                framed[3] = b'K';
                framed[4] = version;
            }
            check(&framed);
        }
    }
}

#[test]
fn from_bytes_survives_random_inputs() {
    let mut rng = Rng(0xba0b_ab00_d00d_f00d);
    for _ in 0..50_000 {
        let len = (rng.next_u64() % 600) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        check(&data);
    }
}
