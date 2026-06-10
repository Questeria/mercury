//! Integration tests for `mercury-keys`: keygen uniqueness, public-key
//! round-tripping, sign/verify (deterministic RNG), the account-id
//! known-answer vector, invalid-key rejection, and Debug redaction.

use mercury_keys::{
    DeviceKeyPair, DevicePublicKey, IdentityKeyPair, IdentityPublicKey, KeyError, MlKemKeyPair,
    account_id_from_identity_key,
};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

#[test]
fn distinct_identity_keys_round_trip() {
    let a = IdentityKeyPair::generate();
    let b = IdentityKeyPair::generate();
    assert_ne!(a.public().as_bytes(), b.public().as_bytes());

    let imported = IdentityPublicKey::from_bytes(*a.public().as_bytes()).expect("valid key");
    assert_eq!(imported.as_bytes(), a.public().as_bytes());
}

#[test]
fn distinct_device_keys_and_agreement() {
    let a = DeviceKeyPair::generate();
    let b = DeviceKeyPair::generate();
    assert_ne!(a.public().as_bytes(), b.public().as_bytes());

    // X25519 is symmetric: both sides derive the same shared secret.
    assert_eq!(
        a.diffie_hellman(&b.public()).expect("contributory"),
        b.diffie_hellman(&a.public()).expect("contributory")
    );
}

#[test]
fn diffie_hellman_rejects_non_contributory_peer() {
    let a = DeviceKeyPair::generate();
    // The all-zero u-coordinate is a low-order point; the DH result is the
    // all-zero (non-contributory) secret, which must be rejected.
    let low_order = DevicePublicKey::from_bytes([0u8; 32]);
    assert_eq!(
        a.diffie_hellman(&low_order),
        Err(KeyError::NonContributoryPeer)
    );
}

#[test]
fn sign_verify_round_trip_and_tamper_rejected() {
    let mut rng = ChaCha20Rng::seed_from_u64(0xC0FFEE);
    let keypair = IdentityKeyPair::generate_with(&mut rng);
    let public = keypair.public();

    let message = b"mercury identity attestation";
    let sig = keypair.sign(message);
    public.verify(message, &sig).expect("signature must verify");

    let mut bad = sig;
    bad[0] ^= 0x01;
    assert_eq!(
        public.verify(message, &bad),
        Err(KeyError::InvalidSignature)
    );
}

#[test]
fn account_id_known_answer() {
    // Fixed identity public key (bytes 0..31).
    let pubkey: [u8; 32] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    ];
    // SHA-256(b"mercury-account-id-v1" || pubkey), computed independently.
    let expected: [u8; 32] = [
        0x63, 0xfc, 0x25, 0x86, 0x6d, 0x9a, 0xcb, 0x73, 0x11, 0xfb, 0xad, 0xc4, 0x6e, 0x81, 0x11,
        0x69, 0x37, 0x96, 0xde, 0xb2, 0xc9, 0x3b, 0xec, 0x27, 0xa5, 0x26, 0x70, 0x5e, 0x7d, 0xd9,
        0xe8, 0xbf,
    ];

    let public = IdentityPublicKey::from_bytes(pubkey).expect("vector key is valid");
    assert_eq!(account_id_from_identity_key(&public), expected);

    // Cross-check against the recorded vector file.
    let vector = include_str!("../../../../vectors/keys/account_id_v1.json");
    assert!(vector.contains(&hex(&pubkey)));
    assert!(vector.contains(&hex(&expected)));
}

#[test]
fn from_bytes_rejects_invalid_identity_key() {
    // y = 2 has no x on the Edwards curve, so decompression fails. (dalek v2
    // accepts all-0xFF, since that encodes a decompressable low-order point;
    // it only rejects y-values that cannot be a curve point.)
    let mut bad = [0u8; 32];
    bad[0] = 2;
    assert_eq!(
        IdentityPublicKey::from_bytes(bad),
        Err(KeyError::InvalidPublicKey)
    );
}

#[test]
fn debug_redacts_secret_material() {
    let mut rng = ChaCha20Rng::seed_from_u64(7);
    let identity = IdentityKeyPair::generate_with(&mut rng);
    let device = DeviceKeyPair::generate_with(&mut rng);

    for rendered in [format!("{identity:?}"), format!("{device:?}")] {
        let lower = rendered.to_lowercase();
        assert!(
            !lower.contains("secret"),
            "Debug leaked the word secret: {rendered}"
        );
    }

    // The private scalar must not appear in Debug output. Compare against the
    // signing key's own serialized secret bytes.
    let secret_hex = {
        // Re-derive the same key to get its secret bytes for the negative check.
        let mut rng2 = ChaCha20Rng::seed_from_u64(7);
        let sk = ed25519_dalek::SigningKey::generate(&mut rng2);
        hex(&sk.to_bytes())
    };
    assert!(
        !format!("{identity:?}").to_lowercase().contains(&secret_hex),
        "Debug leaked private key bytes"
    );
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        s.push(char::from_digit((b & 0x0f) as u32, 16).unwrap());
    }
    s
}

// ---------------------------------------------------------------------------
// Secret-key serialization (for encrypted-at-rest persistence)
// ---------------------------------------------------------------------------

#[test]
fn identity_secret_bytes_round_trip() {
    let kp = IdentityKeyPair::generate();
    let pub_before = *kp.public().as_bytes();
    let restored = IdentityKeyPair::from_secret_bytes(kp.to_secret_bytes());
    // The public key is derived from the secret, so a matching public proves the secret
    // round-tripped; and a signature from the restored key carries the same identity.
    assert_eq!(*restored.public().as_bytes(), pub_before);
    assert_eq!(restored.sign(b"persisted"), kp.sign(b"persisted"));
}

#[test]
fn device_secret_bytes_round_trip() {
    let kp = DeviceKeyPair::generate();
    let pub_before = *kp.public().as_bytes();
    let restored = DeviceKeyPair::from_secret_bytes(kp.to_secret_bytes());
    assert_eq!(*restored.public().as_bytes(), pub_before);
    // The restored key computes the same DH shared secret with a peer.
    let peer = DeviceKeyPair::generate();
    assert_eq!(
        restored.diffie_hellman(&peer.public()).unwrap(),
        kp.diffie_hellman(&peer.public()).unwrap()
    );
}

#[test]
fn ml_kem_secret_bytes_round_trip() {
    let kp = MlKemKeyPair::generate();
    let pub_before = kp.public().to_bytes();
    let bytes = kp.to_secret_bytes();
    assert_eq!(
        bytes.len(),
        64,
        "ML-KEM keypair serializes to its 64-byte seed"
    );
    let restored = MlKemKeyPair::from_secret_bytes(&bytes).expect("seed round-trips");
    assert_eq!(restored.public().to_bytes(), pub_before);
    // The restored keypair decapsulates what was encapsulated to the ORIGINAL public key.
    let (ct, secret) = kp.public().encapsulate();
    assert_eq!(restored.decapsulate(&ct).unwrap(), secret);
}

#[test]
fn ml_kem_from_secret_bytes_is_fail_closed() {
    assert!(matches!(
        MlKemKeyPair::from_secret_bytes(&[]),
        Err(KeyError::InvalidPublicKey)
    ));
    assert!(MlKemKeyPair::from_secret_bytes(&[0u8; 63]).is_err());
    assert!(MlKemKeyPair::from_secret_bytes(&[0u8; 65]).is_err());
    // Any 64-byte value is a valid FIPS-203 seed and reconstructs a keypair.
    assert!(MlKemKeyPair::from_secret_bytes(&[7u8; 64]).is_ok());
    // No panic on any length.
    for len in 0..256usize {
        let _ = MlKemKeyPair::from_secret_bytes(&vec![0xa5u8; len]);
    }
}

// === directory-publish proof of possession ===

#[test]
fn directory_pop_round_trips_and_derives_the_slot() {
    let identity = IdentityKeyPair::generate();
    let card = b"an opaque contact card";
    let sig = mercury_keys::sign_directory_publish(&identity, card);
    // A valid proof yields exactly the signer's own account id (slot).
    assert_eq!(
        mercury_keys::verify_directory_publish(identity.public().as_bytes(), card, &sig),
        Some(account_id_from_identity_key(&identity.public()))
    );
}

#[test]
fn directory_pop_rejects_a_wrong_identity_key() {
    let signer = IdentityKeyPair::generate();
    let other = IdentityKeyPair::generate();
    let card = b"card";
    let sig = mercury_keys::sign_directory_publish(&signer, card);
    // The signature does not verify under a different identity -> cannot claim another slot.
    assert_eq!(
        mercury_keys::verify_directory_publish(other.public().as_bytes(), card, &sig),
        None
    );
}

#[test]
fn directory_pop_rejects_a_tampered_card() {
    let identity = IdentityKeyPair::generate();
    let sig = mercury_keys::sign_directory_publish(&identity, b"original card");
    assert_eq!(
        mercury_keys::verify_directory_publish(
            identity.public().as_bytes(),
            b"different card",
            &sig
        ),
        None
    );
}

#[test]
fn directory_pop_rejects_a_tampered_signature() {
    let identity = IdentityKeyPair::generate();
    let card = b"card";
    let mut sig = mercury_keys::sign_directory_publish(&identity, card);
    sig[0] ^= 0x01;
    assert_eq!(
        mercury_keys::verify_directory_publish(identity.public().as_bytes(), card, &sig),
        None
    );
}

#[test]
fn directory_pop_never_panics_on_arbitrary_identity_bytes() {
    let identity = IdentityKeyPair::generate();
    let card = b"card";
    let sig = mercury_keys::sign_directory_publish(&identity, card);
    // Most 32-byte values are not valid Ed25519 points; verification must fail closed, never
    // panic, for any of them.
    for b in 0u8..=255 {
        let _ = mercury_keys::verify_directory_publish(&[b; 32], card, &sig);
    }
}

// === relay route-ownership proof of possession ===

#[test]
fn relay_route_pop_round_trips_for_the_owned_route() {
    let identity = IdentityKeyPair::generate();
    // For a 1:1 route, the route IS the signer's own account id.
    let route = account_id_from_identity_key(&identity.public());
    let ts = 1_700_000_000;
    let op = mercury_keys::RelayRouteOperation::Poll;
    let sig = mercury_keys::sign_relay_route_pop(&identity, &route, op, ts);
    assert!(mercury_keys::verify_relay_route_pop(
        identity.public().as_bytes(),
        &route,
        op,
        ts,
        &sig
    ));
}

#[test]
fn relay_route_pop_rejects_a_non_owned_route() {
    // Signing a proof for a route that is NOT the signer's account id must not verify — you can
    // only prove ownership of your own route.
    let identity = IdentityKeyPair::generate();
    let not_my_route = [0x42u8; 32];
    assert_ne!(
        not_my_route,
        account_id_from_identity_key(&identity.public())
    );
    let ts = 1_700_000_000;
    let op = mercury_keys::RelayRouteOperation::Poll;
    let sig = mercury_keys::sign_relay_route_pop(&identity, &not_my_route, op, ts);
    assert!(!mercury_keys::verify_relay_route_pop(
        identity.public().as_bytes(),
        &not_my_route,
        op,
        ts,
        &sig
    ));
}

#[test]
fn relay_route_pop_rejects_a_wrong_identity() {
    let signer = IdentityKeyPair::generate();
    let other = IdentityKeyPair::generate();
    let route = account_id_from_identity_key(&signer.public());
    let ts = 1_700_000_000;
    let op = mercury_keys::RelayRouteOperation::Poll;
    let sig = mercury_keys::sign_relay_route_pop(&signer, &route, op, ts);
    // Another identity cannot present this proof for the signer's route.
    assert!(!mercury_keys::verify_relay_route_pop(
        other.public().as_bytes(),
        &route,
        op,
        ts,
        &sig
    ));
}

#[test]
fn relay_route_pop_is_bound_to_the_timestamp() {
    let identity = IdentityKeyPair::generate();
    let route = account_id_from_identity_key(&identity.public());
    let op = mercury_keys::RelayRouteOperation::Poll;
    let sig = mercury_keys::sign_relay_route_pop(&identity, &route, op, 1_700_000_000);
    // A proof signed at one timestamp does not verify against another (so the relay's freshness
    // window check is load-bearing, not bypassable by changing the claimed timestamp).
    assert!(!mercury_keys::verify_relay_route_pop(
        identity.public().as_bytes(),
        &route,
        op,
        1_700_000_001,
        &sig
    ));
}

#[test]
fn relay_route_pop_rejects_a_tampered_signature() {
    let identity = IdentityKeyPair::generate();
    let route = account_id_from_identity_key(&identity.public());
    let ts = 1_700_000_000;
    let op = mercury_keys::RelayRouteOperation::Poll;
    let mut sig = mercury_keys::sign_relay_route_pop(&identity, &route, op, ts);
    sig[0] ^= 0x01;
    assert!(!mercury_keys::verify_relay_route_pop(
        identity.public().as_bytes(),
        &route,
        op,
        ts,
        &sig
    ));
}

#[test]
fn relay_route_pop_never_panics_on_arbitrary_identity_bytes() {
    let identity = IdentityKeyPair::generate();
    let route = account_id_from_identity_key(&identity.public());
    let op = mercury_keys::RelayRouteOperation::Poll;
    let sig = mercury_keys::sign_relay_route_pop(&identity, &route, op, 1_700_000_000);
    for b in 0u8..=255 {
        let _ = mercury_keys::verify_relay_route_pop(&[b; 32], &route, op, 1_700_000_000, &sig);
    }
}

#[test]
fn relay_route_pop_is_bound_to_the_operation() {
    // A proof signed for one operation must NOT verify for another, so a captured poll proof
    // cannot be replayed as a destructive ack/delete within the freshness window.
    let identity = IdentityKeyPair::generate();
    let route = account_id_from_identity_key(&identity.public());
    let ts = 1_700_000_000;
    let poll_sig = mercury_keys::sign_relay_route_pop(
        &identity,
        &route,
        mercury_keys::RelayRouteOperation::Poll,
        ts,
    );
    // Same key, route, and timestamp — only the operation differs.
    assert!(mercury_keys::verify_relay_route_pop(
        identity.public().as_bytes(),
        &route,
        mercury_keys::RelayRouteOperation::Poll,
        ts,
        &poll_sig
    ));
    for wrong in [
        mercury_keys::RelayRouteOperation::Ack,
        mercury_keys::RelayRouteOperation::Delete,
    ] {
        assert!(
            !mercury_keys::verify_relay_route_pop(
                identity.public().as_bytes(),
                &route,
                wrong,
                ts,
                &poll_sig
            ),
            "a poll proof must not verify for {wrong:?}"
        );
    }
}
