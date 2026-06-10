//! Track 0c robustness: the MLS engine's wire-parsing seams must NEVER panic on
//! arbitrary, malformed, truncated, or mutated input -- they must fail closed.
//!
//! A relay is UNTRUSTED and can hand a client any bytes. The malicious-relay ->
//! client attack surface in mercury-mls is two raw-bytes entry points:
//! `key_package_admission_input` (parses a PRESENTED KeyPackage via
//! `KeyPackageIn::tls_deserialize` then `validate`, before the admission gate)
//! and `receive_message_bytes` (parses a delivered MLS message via
//! `MlsMessageIn::tls_deserialize` -> protocol message -> `process_message`).
//! Both must reject hostile bytes with `Err` / an all-invalid decision and never
//! crash the client.
//!
//! This is the locally-verifiable, runs-everywhere layer (plain `cargo test`, so
//! it gates crash-resistance in CI on every platform). Coverage-guided
//! cargo-fuzz/libFuzzer is the deeper companion layer over the same two seams.

use mercury_mls::{
    GroupChatCryptoSuite, GroupChatDecision, MlsMember, add_member,
    evaluate_mls_key_package_admission, join_group, join_group_bytes, key_package_admission_input,
    mls_group_session, openmls_provider_security, receive_message_bytes, send_message,
    serialize_message,
};
use openmls_rust_crypto::OpenMlsRustCrypto;

const SUITE: GroupChatCryptoSuite = GroupChatCryptoSuite::ClassicalMls128;
const NOW: i64 = 1_000;
const MAX_LIFETIME: i64 = 7_776_000; // 90 days

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

/// Silence the panic hook for this test process. The engine CATCHES the
/// debug-only dependency assertions (tls_codec `quic_vec`, OpenMLS
/// `private_message_in`) that malformed/mutated input trips in a debug build,
/// converting them to fail-closed errors -- but the default hook would still
/// print each caught panic (tens of thousands of times here). Quieting keeps the
/// output readable; the harness still reports any genuine ESCAPING panic as a
/// test failure (so these tests remain real regression guards for the guard).
fn quiet_panics() {
    std::panic::set_hook(Box::new(|_| {}));
}

/// An accepted 3-member classical group session token (attested against a real
/// OpenMLS provider) so the admission parser exercises its full validate path.
fn accepted_group(provider: &OpenMlsRustCrypto) -> GroupChatDecision {
    let ps = openmls_provider_security(provider, SUITE, GroupChatCryptoSuite::ClassicalMls128);
    mls_group_session(ps, SUITE, 3, 1)
}

/// Parsing a presented KeyPackage must not panic; the gate may reject.
fn check_key_package(group: GroupChatDecision, provider: &OpenMlsRustCrypto, data: &[u8]) {
    let input =
        key_package_admission_input(group, 1, SUITE, data, provider, NOW, MAX_LIFETIME, false);
    let _ = evaluate_mls_key_package_admission(input);
}

#[test]
fn key_package_admission_survives_edge_and_random() {
    quiet_panics();
    let provider = OpenMlsRustCrypto::default();
    let group = accepted_group(&provider);

    // Edge byte patterns across a wide length sweep.
    check_key_package(group, &provider, &[]);
    for len in 0..512usize {
        check_key_package(group, &provider, &vec![0x00u8; len]);
        check_key_package(group, &provider, &vec![0xffu8; len]);
        check_key_package(group, &provider, &vec![0xa5u8; len]);
    }

    // Arbitrary inputs.
    let mut rng = Rng(0x0807_0605_0403_0201);
    for _ in 0..50_000 {
        let len = (rng.next_u64() % 1024) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        check_key_package(group, &provider, &data);
    }

    // Small-length cluster where length-prefix framing is most fragile.
    let mut rng = Rng(0xfeed_face_dead_beef);
    for len in 0..64usize {
        for _ in 0..1_000 {
            let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
            check_key_package(group, &provider, &data);
        }
    }
}

#[test]
fn key_package_admission_survives_mutated_real_package() {
    quiet_panics();
    // Start from a REAL serialized KeyPackage and corrupt it: this drives the
    // deserializer PAST the outer framing into the structured body, where random
    // bytes rarely reach.
    let provider = OpenMlsRustCrypto::default();
    let group = accepted_group(&provider);
    let alice = MlsMember::generate(b"alice@mercury").unwrap();
    let base = alice.key_package_bytes();
    assert!(!base.is_empty());

    let mut rng = Rng(0x1357_9bdf_0246_8ace);
    for _ in 0..20_000 {
        let mut data = base.clone();
        if rng.next_u64() & 1 == 0 {
            // Flip 1..=4 random bytes.
            let flips = 1 + (rng.next_u64() % 4) as usize;
            for _ in 0..flips {
                let pos = (rng.next_u64() as usize) % data.len();
                data[pos] ^= rng.byte();
            }
        } else {
            // Truncate to a random shorter length.
            let keep = (rng.next_u64() as usize) % (data.len() + 1);
            data.truncate(keep);
        }
        check_key_package(group, &provider, &data);
    }
}

#[test]
fn receive_message_bytes_survives_hostile_wire() {
    quiet_panics();
    // A real two-member group; feed `alice` hostile bytes as if delivered by a
    // malicious relay. process_message must reject them all without panicking.
    let creator = MlsMember::generate(b"creator@mercury").unwrap();
    let alice = MlsMember::generate(b"alice@mercury").unwrap();
    let mut creator_group = creator.create_group([0x33u8; 32]).unwrap();
    let (welcome, _consumed) =
        add_member(&mut creator_group, &creator, alice.key_package()).unwrap();
    let mut alice_group = join_group(&alice, &welcome).unwrap();

    // Edge patterns.
    let _ = receive_message_bytes(&mut alice_group, &alice, &[]);
    for len in 0..256usize {
        let _ = receive_message_bytes(&mut alice_group, &alice, &vec![0x00u8; len]);
        let _ = receive_message_bytes(&mut alice_group, &alice, &vec![0xffu8; len]);
    }

    // Arbitrary inputs.
    let mut rng = Rng(0x2468_ace0_1357_9bdf);
    for _ in 0..20_000 {
        let len = (rng.next_u64() % 1024) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        let _ = receive_message_bytes(&mut alice_group, &alice, &data);
    }

    // Small-length boundary cluster.
    let mut rng = Rng(0x0f1e_2d3c_4b5a_6978);
    for len in 0..64usize {
        for _ in 0..500 {
            let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
            let _ = receive_message_bytes(&mut alice_group, &alice, &data);
        }
    }
}

#[test]
fn receive_message_bytes_survives_mutated_real_message() {
    quiet_panics();
    // Corrupt a REAL encrypted application message: drives the MLS message
    // deserializer + protocol-message conversion past the outer framing.
    let creator = MlsMember::generate(b"creator@mercury").unwrap();
    let alice = MlsMember::generate(b"alice@mercury").unwrap();
    let mut creator_group = creator.create_group([0x77u8; 32]).unwrap();
    let (welcome, _consumed) =
        add_member(&mut creator_group, &creator, alice.key_package()).unwrap();
    let mut alice_group = join_group(&alice, &welcome).unwrap();

    let base = serialize_message(&send_message(&mut creator_group, &creator, b"baseline").unwrap())
        .unwrap();
    assert!(!base.is_empty());

    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for _ in 0..20_000 {
        let mut data = base.clone();
        if rng.next_u64() & 1 == 0 {
            let flips = 1 + (rng.next_u64() % 4) as usize;
            for _ in 0..flips {
                let pos = (rng.next_u64() as usize) % data.len();
                data[pos] ^= rng.byte();
            }
        } else {
            let keep = (rng.next_u64() as usize) % (data.len() + 1);
            data.truncate(keep);
        }
        let _ = receive_message_bytes(&mut alice_group, &alice, &data);
    }
}

#[test]
fn join_group_bytes_survives_hostile_welcome() {
    quiet_panics();
    // The Welcome also transits the UNTRUSTED relay as bytes; feed `alice` hostile
    // / mutated Welcome bytes. join_group_bytes must fail closed without panic. A
    // corrupt Welcome never successfully joins, so alice's KeyPackage secrets are
    // not consumed and she can be reused across iterations.
    let creator = MlsMember::generate(b"creator@mercury").unwrap();
    let alice = MlsMember::generate(b"alice@mercury").unwrap();
    let mut creator_group = creator.create_group([0x5cu8; 32]).unwrap();
    let (welcome, _consumed) =
        add_member(&mut creator_group, &creator, alice.key_package()).unwrap();
    let base = serialize_message(&welcome).unwrap();
    assert!(!base.is_empty());

    // Edge + arbitrary inputs.
    let _ = join_group_bytes(&alice, &[]);
    let mut rng = Rng(0x5a5a_a5a5_3c3c_c3c3);
    for _ in 0..10_000 {
        let len = (rng.next_u64() % 1024) as usize;
        let data: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        let _ = join_group_bytes(&alice, &data);
    }

    // Structure-aware mutation of the REAL Welcome (drives StagedWelcome past the
    // outer framing).
    let mut rng = Rng(0xc0ff_eec0_de15_dead);
    for _ in 0..10_000 {
        let mut data = base.clone();
        if rng.next_u64() & 1 == 0 {
            let flips = 1 + (rng.next_u64() % 4) as usize;
            for _ in 0..flips {
                let pos = (rng.next_u64() as usize) % data.len();
                data[pos] ^= rng.byte();
            }
        } else {
            let keep = (rng.next_u64() as usize) % (data.len() + 1);
            data.truncate(keep);
        }
        let _ = join_group_bytes(&alice, &data);
    }
}
