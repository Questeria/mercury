//! Local anonymous-sender sealed message objects for Mercury (Phase 2).
//!
//! An age-style X25519 sealed box: authenticated encryption to a recipient
//! device public key with a fresh ephemeral sender key, so the recipient learns
//! nothing about the sender's identity. Local-only: no network, and NOT a
//! session/ratchet protocol (that is deferred to a future libsignal seam).
//!
//! All primitives come from audited RustCrypto crates; nothing here implements
//! crypto by hand. The DH shared secret and the derived AEAD key are zeroized
//! after use, and `SealError` never carries secret material.

#![forbid(unsafe_code)]

use core::fmt;

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use mercury_keys::{DeviceKeyPair, DevicePublicKey, KeyError, MlKemKeyPair, MlKemPublicKey};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

const DOMAIN: &[u8] = b"mercury-sealed-message-v1";
const EPHEMERAL_PUBKEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
/// Length of the KEY COMMITMENT carried in every sealed object. XChaCha20-Poly1305
/// (like all AES-GCM / ChaCha-Poly1305 AEADs) is NOT key-committing: a single
/// ciphertext+tag can be made to decrypt successfully under more than one key
/// (the "invisible salamander" / partitioning-oracle property). Mercury binds a
/// 32-byte commitment derived from the SAME key-agreement secret as the AEAD key
/// (one HKDF expand outputs `aead_key || commitment`), frames it, and constant-time
/// checks it on open — so a sealed object commits to exactly one key and the
/// multi-key-decrypt attack is closed.
const COMMITMENT_LEN: usize = 32;

/// Errors surfaced when sealing to or opening from a sealed message object.
///
/// Carries no secret or plaintext material, and the `Debug`/`Display` forms are
/// safe to log.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SealError {
    /// The sealed object is shorter than the minimum framing.
    Malformed,
    /// The recipient (seal) or ephemeral (open) key produced a non-contributory
    /// (low-order) shared secret.
    NonContributoryPeer,
    /// The embedded ephemeral public key was not a valid X25519 key.
    InvalidEphemeralKey,
    /// AEAD sealing failed (e.g. plaintext too long for the cipher).
    SealFailed,
    /// AEAD opening failed: wrong recipient, tampering, or corruption.
    OpenFailed,
}

impl fmt::Display for SealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SealError::Malformed => f.write_str("malformed sealed message"),
            SealError::NonContributoryPeer => f.write_str("non-contributory peer key"),
            SealError::InvalidEphemeralKey => f.write_str("invalid ephemeral public key"),
            SealError::SealFailed => f.write_str("sealing failed"),
            SealError::OpenFailed => f.write_str("opening failed"),
        }
    }
}

impl std::error::Error for SealError {}

impl From<KeyError> for SealError {
    fn from(err: KeyError) -> Self {
        match err {
            KeyError::NonContributoryPeer => SealError::NonContributoryPeer,
            // Key agreement only surfaces InvalidPublicKey for a bad ephemeral
            // key on the open path; InvalidSignature cannot arise from DH.
            KeyError::InvalidPublicKey | KeyError::InvalidSignature => {
                SealError::InvalidEphemeralKey
            }
        }
    }
}

/// HKDF-SHA256 the AEAD key AND the key commitment from the raw DH secret in ONE
/// expand: the 64-byte output is `aead_key (32) || commitment (32)`. `salt` is the
/// 64-byte `ephemeral_pub || recipient_pub`; the domain string is the `info`. Both
/// halves come from the same keying material, so a successful commitment check
/// proves the AEAD key is the unique one this sealed object was made for. The
/// caller owns zeroizing `shared`; the returned secret bytes are zeroized by the
/// caller after splitting.
fn derive_key_and_commitment(shared: &[u8; 32], salt: &[u8; 64]) -> [u8; 64] {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), shared);
    let mut okm = [0u8; 64];
    hkdf.expand(DOMAIN, &mut okm)
        .expect("64 bytes is a valid HKDF-SHA256 output length");
    okm
}

/// `domain || ephemeral_pub || recipient_pub`, bound as AEAD associated data so
/// the framing cannot be transplanted onto a different recipient.
fn associated_data(ephemeral_pub: &[u8; 32], recipient_pub: &[u8; 32]) -> Vec<u8> {
    let mut aad = Vec::with_capacity(DOMAIN.len() + EPHEMERAL_PUBKEY_LEN + EPHEMERAL_PUBKEY_LEN);
    aad.extend_from_slice(DOMAIN);
    aad.extend_from_slice(ephemeral_pub);
    aad.extend_from_slice(recipient_pub);
    aad
}

fn hkdf_salt(ephemeral_pub: &[u8; 32], recipient_pub: &[u8; 32]) -> [u8; 64] {
    let mut salt = [0u8; 64];
    salt[..32].copy_from_slice(ephemeral_pub);
    salt[32..].copy_from_slice(recipient_pub);
    salt
}

/// Split a 64-byte HKDF output into the 32-byte AEAD key and 32-byte commitment.
fn split_key_and_commitment(okm: &[u8; 64]) -> ([u8; 32], [u8; 32]) {
    let mut key = [0u8; 32];
    let mut commitment = [0u8; 32];
    key.copy_from_slice(&okm[..32]);
    commitment.copy_from_slice(&okm[32..]);
    (key, commitment)
}

/// Seal `plaintext` for `recipient`, returning
/// `ephemeral_pub (32) || commitment (32) || nonce (24) || ciphertext (plaintext.len() + 16)`.
///
/// A fresh ephemeral key is generated per call, so two seals of the same
/// plaintext to the same recipient differ and the sender stays anonymous. The
/// framed `commitment` makes the object KEY-COMMITTING (see [`COMMITMENT_LEN`]).
pub fn seal(recipient: &DevicePublicKey, plaintext: &[u8]) -> Result<Vec<u8>, SealError> {
    let ephemeral = DeviceKeyPair::generate();
    let e = ephemeral.public();

    let mut shared = ephemeral.diffie_hellman(recipient)?;
    let salt = hkdf_salt(e.as_bytes(), recipient.as_bytes());
    let mut okm = derive_key_and_commitment(&shared, &salt);
    shared.zeroize();
    let (mut key, commitment) = split_key_and_commitment(&okm);
    okm.zeroize();

    let cipher = XChaCha20Poly1305::new_from_slice(&key)
        .expect("32-byte key is valid for XChaCha20Poly1305");
    key.zeroize();

    let aad = associated_data(e.as_bytes(), recipient.as_bytes());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| SealError::SealFailed)?;

    let mut out =
        Vec::with_capacity(EPHEMERAL_PUBKEY_LEN + COMMITMENT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(e.as_bytes());
    out.extend_from_slice(&commitment);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Open a sealed object produced by [`seal`] for `recipient`, returning the
/// recovered plaintext. Verifies the KEY COMMITMENT (constant time) before the
/// AEAD, and fails closed on any tampering, wrong recipient, commitment mismatch,
/// or malformed framing.
pub fn open(recipient: &DeviceKeyPair, sealed: &[u8]) -> Result<Vec<u8>, SealError> {
    if sealed.len() < EPHEMERAL_PUBKEY_LEN + COMMITMENT_LEN + NONCE_LEN + TAG_LEN {
        return Err(SealError::Malformed);
    }

    let (e_bytes, rest) = sealed.split_at(EPHEMERAL_PUBKEY_LEN);
    let (commitment_bytes, rest) = rest.split_at(COMMITMENT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);
    let e_array: [u8; 32] = e_bytes.try_into().expect("split at 32 bytes");
    let e = DevicePublicKey::from_bytes(e_array);

    let r = recipient.public();
    let mut shared = recipient.diffie_hellman(&e)?;
    let salt = hkdf_salt(&e_array, r.as_bytes());
    let mut okm = derive_key_and_commitment(&shared, &salt);
    shared.zeroize();
    let (mut key, expected_commitment) = split_key_and_commitment(&okm);
    okm.zeroize();

    // Constant-time key-commitment check BEFORE the AEAD: a sealed object commits
    // to exactly one key, closing the multi-key-decrypt (partitioning oracle) gap.
    if commitment_bytes.ct_eq(&expected_commitment).unwrap_u8() != 1 {
        key.zeroize();
        return Err(SealError::OpenFailed);
    }

    let cipher = XChaCha20Poly1305::new_from_slice(&key)
        .expect("32-byte key is valid for XChaCha20Poly1305");
    key.zeroize();

    let aad = associated_data(&e_array, r.as_bytes());
    let nonce = XNonce::from_slice(nonce_bytes);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| SealError::OpenFailed)
}

// ===========================================================================
// Hybrid post-quantum sealed box (X25519 + ML-KEM-768) -- ProtocolSuite::HybridPqDev
// ===========================================================================
//
// The AEAD key is derived from BOTH a classical X25519 DH secret AND an ML-KEM
// shared secret, so a message stays confidential as long as EITHER primitive
// holds (a future quantum break of X25519, or a flaw in the newer ML-KEM). This
// is a straightforward concatenation-KDF hybrid: HKDF over `ss_x25519 || ss_mlkem`
// with the full transcript (ephemeral X25519 pub || ML-KEM ciphertext ||
// recipient X25519 pub) bound as both the HKDF salt and the AEAD associated
// data, so neither KEM share can be mauled independently. (A "Dev" suite using
// the RustCrypto `ml-kem` crate; not a formally-standardized combiner such as
// X-Wing.)

const HYBRID_DOMAIN: &[u8] = b"mercury-sealed-message-hybrid-v1";
/// ML-KEM-768 ciphertext length (FIPS 203): fixed, so the framing needs no
/// length prefix. Asserted in tests against a real encapsulation.
const MLKEM_CT_LEN: usize = 1088;

/// Build the hybrid transcript `ephemeral_x || mlkem_ct || recipient_x`, bound
/// into both the KDF salt and the AEAD associated data so the derived key + the
/// framing are tied to every public KEM value.
fn hybrid_transcript(ephemeral_x: &[u8; 32], mlkem_ct: &[u8], recipient_x: &[u8; 32]) -> Vec<u8> {
    let mut t = Vec::with_capacity(32 + mlkem_ct.len() + 32);
    t.extend_from_slice(ephemeral_x);
    t.extend_from_slice(mlkem_ct);
    t.extend_from_slice(recipient_x);
    t
}

/// HKDF-SHA256 the AEAD key AND the key commitment from `ss_x25519 || ss_mlkem`
/// (IKM) in ONE expand, salted by the transcript: the 64-byte output is
/// `aead_key (32) || commitment (32)`. Both halves come from the SAME hybrid keying
/// material, so the framed commitment makes the hybrid sealed object key-committing
/// over BOTH shares. Both shared secrets are zeroized by the caller; the returned
/// secret bytes are zeroized after splitting.
fn derive_hybrid_key_and_commitment(
    ss_x25519: &[u8; 32],
    ss_mlkem: &[u8; 32],
    transcript: &[u8],
) -> [u8; 64] {
    let mut ikm = [0u8; 64];
    ikm[..32].copy_from_slice(ss_x25519);
    ikm[32..].copy_from_slice(ss_mlkem);
    let hkdf = Hkdf::<Sha256>::new(Some(transcript), &ikm);
    let mut okm = [0u8; 64];
    hkdf.expand(HYBRID_DOMAIN, &mut okm)
        .expect("64 bytes is a valid HKDF-SHA256 output length");
    ikm.zeroize();
    okm
}

/// Seal `plaintext` to a recipient's HYBRID identity (`recipient_x` X25519 +
/// `recipient_pq` ML-KEM-768), returning
/// `ephemeral_x (32) || mlkem_ct (1088) || commitment (32) || nonce (24) || ciphertext`.
///
/// A fresh X25519 ephemeral + a fresh ML-KEM encapsulation are used per call, so
/// the sender stays anonymous and two seals of the same plaintext differ. The
/// framed `commitment` makes the hybrid object KEY-COMMITTING over both shares.
pub fn seal_hybrid(
    recipient_x: &DevicePublicKey,
    recipient_pq: &MlKemPublicKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, SealError> {
    let ephemeral = DeviceKeyPair::generate();
    let e = ephemeral.public();

    let mut ss_x = ephemeral.diffie_hellman(recipient_x)?;
    let (mlkem_ct, mut ss_pq) = recipient_pq.encapsulate();

    let transcript = hybrid_transcript(e.as_bytes(), &mlkem_ct, recipient_x.as_bytes());
    let mut okm = derive_hybrid_key_and_commitment(&ss_x, &ss_pq, &transcript);
    ss_x.zeroize();
    ss_pq.zeroize();
    let (mut key, commitment) = split_key_and_commitment(&okm);
    okm.zeroize();

    let cipher = XChaCha20Poly1305::new_from_slice(&key)
        .expect("32-byte key is valid for XChaCha20Poly1305");
    key.zeroize();

    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: &transcript,
            },
        )
        .map_err(|_| SealError::SealFailed)?;

    let mut out = Vec::with_capacity(
        EPHEMERAL_PUBKEY_LEN + MLKEM_CT_LEN + COMMITMENT_LEN + NONCE_LEN + ciphertext.len(),
    );
    out.extend_from_slice(e.as_bytes());
    out.extend_from_slice(&mlkem_ct);
    out.extend_from_slice(&commitment);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Open a hybrid sealed object produced by [`seal_hybrid`] for the recipient's
/// `recipient_x` (X25519) + `recipient_pq` (ML-KEM-768) keypairs. Verifies the KEY
/// COMMITMENT (constant time) before the AEAD, and fails closed on tampering
/// (including to the ML-KEM ciphertext, via implicit rejection), a wrong recipient,
/// a commitment mismatch, or malformed framing.
pub fn open_hybrid(
    recipient_x: &DeviceKeyPair,
    recipient_pq: &MlKemKeyPair,
    sealed: &[u8],
) -> Result<Vec<u8>, SealError> {
    if sealed.len() < EPHEMERAL_PUBKEY_LEN + MLKEM_CT_LEN + COMMITMENT_LEN + NONCE_LEN + TAG_LEN {
        return Err(SealError::Malformed);
    }

    let (e_bytes, rest) = sealed.split_at(EPHEMERAL_PUBKEY_LEN);
    let (mlkem_ct, rest) = rest.split_at(MLKEM_CT_LEN);
    let (commitment_bytes, rest) = rest.split_at(COMMITMENT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);
    let e_array: [u8; 32] = e_bytes.try_into().expect("split at 32 bytes");
    let e = DevicePublicKey::from_bytes(e_array);

    let r = recipient_x.public();
    let mut ss_x = recipient_x.diffie_hellman(&e)?;
    let mut ss_pq = recipient_pq.decapsulate(mlkem_ct)?;

    let transcript = hybrid_transcript(&e_array, mlkem_ct, r.as_bytes());
    let mut okm = derive_hybrid_key_and_commitment(&ss_x, &ss_pq, &transcript);
    ss_x.zeroize();
    ss_pq.zeroize();
    let (mut key, expected_commitment) = split_key_and_commitment(&okm);
    okm.zeroize();

    // Constant-time key-commitment check BEFORE the AEAD: the hybrid object commits
    // to exactly one key over both shares, closing the multi-key-decrypt gap.
    if commitment_bytes.ct_eq(&expected_commitment).unwrap_u8() != 1 {
        key.zeroize();
        return Err(SealError::OpenFailed);
    }

    let cipher = XChaCha20Poly1305::new_from_slice(&key)
        .expect("32-byte key is valid for XChaCha20Poly1305");
    key.zeroize();

    let nonce = XNonce::from_slice(nonce_bytes);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad: &transcript,
            },
        )
        .map_err(|_| SealError::OpenFailed)
}

#[cfg(test)]
mod hybrid_tests {
    use super::*;

    fn recipient() -> (DeviceKeyPair, MlKemKeyPair) {
        (DeviceKeyPair::generate(), MlKemKeyPair::generate())
    }

    #[test]
    fn hybrid_round_trips() {
        let (x, pq) = recipient();
        let msg = b"post-quantum hybrid sealed message";
        let sealed = seal_hybrid(&x.public(), &pq.public(), msg).unwrap();
        assert_eq!(open_hybrid(&x, &pq, &sealed).unwrap(), msg);
    }

    #[test]
    fn mlkem_ciphertext_length_is_as_framed() {
        // Guard the hard-coded MLKEM_CT_LEN against a real encapsulation.
        let (_, pq) = recipient();
        let (ct, _) = pq.public().encapsulate();
        assert_eq!(ct.len(), MLKEM_CT_LEN);
    }

    #[test]
    fn wrong_classical_key_fails() {
        let (x, pq) = recipient();
        let (other_x, _) = recipient();
        let sealed = seal_hybrid(&x.public(), &pq.public(), b"secret").unwrap();
        // Right ML-KEM key, wrong X25519 key -> hybrid KDF differs -> OpenFailed.
        assert_eq!(
            open_hybrid(&other_x, &pq, &sealed),
            Err(SealError::OpenFailed)
        );
    }

    #[test]
    fn wrong_pq_key_fails() {
        let (x, pq) = recipient();
        let (_, other_pq) = recipient();
        let sealed = seal_hybrid(&x.public(), &pq.public(), b"secret").unwrap();
        // Right X25519 key, wrong ML-KEM key -> implicit rejection -> OpenFailed.
        assert_eq!(
            open_hybrid(&x, &other_pq, &sealed),
            Err(SealError::OpenFailed)
        );
    }

    #[test]
    fn tampered_mlkem_ciphertext_fails() {
        let (x, pq) = recipient();
        let mut sealed = seal_hybrid(&x.public(), &pq.public(), b"secret").unwrap();
        sealed[EPHEMERAL_PUBKEY_LEN + 10] ^= 0xff; // flip a byte in the ML-KEM ct
        assert_eq!(open_hybrid(&x, &pq, &sealed), Err(SealError::OpenFailed));
    }

    #[test]
    fn truncated_hybrid_is_malformed() {
        let (x, pq) = recipient();
        let sealed = seal_hybrid(&x.public(), &pq.public(), b"hi").unwrap();
        assert_eq!(
            open_hybrid(&x, &pq, &sealed[..100]),
            Err(SealError::Malformed)
        );
    }

    #[test]
    fn two_seals_differ() {
        // Fresh ephemeral + encapsulation per call -> distinct ciphertexts.
        let (x, pq) = recipient();
        let a = seal_hybrid(&x.public(), &pq.public(), b"same").unwrap();
        let b = seal_hybrid(&x.public(), &pq.public(), b"same").unwrap();
        assert_ne!(a, b);
    }
}
