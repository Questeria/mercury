//! Integration tests for `mercury-sealedbox`: round-trip across sizes,
//! wrong-recipient rejection, tamper detection in each framing region,
//! malformed-length rejection, the non-contributory-recipient guard, and the
//! fixed output shape.

use mercury_keys::{DeviceKeyPair, DevicePublicKey};
use mercury_sealedbox::{SealError, open, seal};

// ephemeral_pub (32) || commitment (32) || nonce (24) || tag (16).
const OVERHEAD: usize = 32 + 32 + 24 + 16;
/// Byte offset of the 24-byte nonce within the framing (after pubkey + commitment).
const NONCE_OFFSET: usize = 32 + 32;

#[test]
fn round_trip_across_sizes() {
    let recipient = DeviceKeyPair::generate();
    for len in [0usize, 1, 1000] {
        let plaintext: Vec<u8> = (0..len).map(|i| i as u8).collect();
        let sealed = seal(&recipient.public(), &plaintext).expect("seal");
        let opened = open(&recipient, &sealed).expect("open");
        assert_eq!(opened, plaintext, "round-trip failed at len {len}");
    }
}

#[test]
fn output_shape_is_overhead_plus_plaintext() {
    let recipient = DeviceKeyPair::generate();
    for len in [0usize, 1, 1000] {
        let plaintext = vec![0xABu8; len];
        let sealed = seal(&recipient.public(), &plaintext).expect("seal");
        assert_eq!(sealed.len(), OVERHEAD + len);
    }
}

#[test]
fn fresh_ephemeral_per_seal() {
    // Same plaintext + recipient must yield distinct sealed objects.
    let recipient = DeviceKeyPair::generate();
    let a = seal(&recipient.public(), b"same").expect("seal");
    let b = seal(&recipient.public(), b"same").expect("seal");
    assert_ne!(a, b);
}

#[test]
fn wrong_recipient_cannot_open() {
    let recipient = DeviceKeyPair::generate();
    let other = DeviceKeyPair::generate();
    let sealed = seal(&recipient.public(), b"for the right device only").expect("seal");
    assert_eq!(open(&other, &sealed), Err(SealError::OpenFailed));
}

#[test]
fn tampered_ciphertext_rejected() {
    let recipient = DeviceKeyPair::generate();
    let mut sealed = seal(&recipient.public(), b"authenticated payload").expect("seal");
    let last = sealed.len() - 1; // inside the ciphertext/tag region
    sealed[last] ^= 0x01;
    assert_eq!(open(&recipient, &sealed), Err(SealError::OpenFailed));
}

#[test]
fn tampered_nonce_rejected() {
    let recipient = DeviceKeyPair::generate();
    let mut sealed = seal(&recipient.public(), b"authenticated payload").expect("seal");
    sealed[NONCE_OFFSET] ^= 0x01; // first byte of the 24-byte nonce
    assert_eq!(open(&recipient, &sealed), Err(SealError::OpenFailed));
}

#[test]
fn tampered_commitment_rejected() {
    // Flipping a byte of the framed key commitment must fail closed at the
    // constant-time commitment check, BEFORE the AEAD runs. This is the property
    // that makes the sealed box key-committing.
    let recipient = DeviceKeyPair::generate();
    let mut sealed = seal(&recipient.public(), b"authenticated payload").expect("seal");
    sealed[32] ^= 0x01; // first byte of the 32-byte commitment (right after the pubkey)
    assert_eq!(open(&recipient, &sealed), Err(SealError::OpenFailed));
}

#[test]
fn tampered_ephemeral_pubkey_rejected() {
    let recipient = DeviceKeyPair::generate();
    let mut sealed = seal(&recipient.public(), b"authenticated payload").expect("seal");
    sealed[0] ^= 0x01; // first byte of the ephemeral public key
    // Changing the ephemeral key changes both the DH input and the AAD, so this
    // fails closed. A bit flip yields OpenFailed; a flip onto a low-order point
    // would surface NonContributoryPeer/InvalidEphemeralKey instead.
    let err = open(&recipient, &sealed).expect_err("tampered ephemeral key must fail");
    assert!(
        matches!(
            err,
            SealError::OpenFailed | SealError::NonContributoryPeer | SealError::InvalidEphemeralKey
        ),
        "unexpected error variant: {err:?}"
    );
}

#[test]
fn too_short_is_malformed() {
    let recipient = DeviceKeyPair::generate();
    assert_eq!(open(&recipient, &[]), Err(SealError::Malformed));
    let almost = vec![0u8; OVERHEAD - 1];
    assert_eq!(open(&recipient, &almost), Err(SealError::Malformed));
    // Exactly the overhead is the minimum valid length: a real empty-plaintext
    // seal is `OVERHEAD` bytes and parses past the length check, so the boundary
    // failure is authentication, not framing. (All-zero bytes would instead trip
    // the low-order ephemeral-key guard before the AEAD runs.)
    let other = DeviceKeyPair::generate();
    let exact = seal(&other.public(), b"").expect("seal empty");
    assert_eq!(exact.len(), OVERHEAD);
    assert_eq!(open(&recipient, &exact), Err(SealError::OpenFailed));
}

#[test]
fn non_contributory_recipient_rejected() {
    // The all-zero u-coordinate is a low-order point; sealing to it would force
    // an all-zero shared secret, which mercury-keys rejects.
    let low_order = DevicePublicKey::from_bytes([0u8; 32]);
    assert_eq!(
        seal(&low_order, b"to nobody"),
        Err(SealError::NonContributoryPeer)
    );
}
