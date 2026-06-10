//! Attachment (media) encryption for Mercury (Stage 9).
//!
//! A large attachment is encrypted under a fresh random 32-byte CONTENT KEY with
//! chunked XChaCha20-Poly1305. The recipient-independent sealed blob is uploaded
//! to a (possibly untrusted) blob store, while the content key travels inside the
//! end-to-end-encrypted message (mercury-message / mercury-sealedbox) — so the
//! store never sees the key, and recipients recover it from the message and
//! decrypt the blob. This mirrors the standard messenger attachment pattern
//! (symmetric blob encryption + key-in-message), keeping the large ciphertext off
//! the E2E path.
//!
//! Each chunk is sealed with a fresh random 192-bit nonce and binds its position
//! (`chunk_index`) plus the whole-attachment `plaintext_len` as associated data,
//! so truncation, reordering, extension, and tampering all fail closed; the
//! decoder also verifies the exact expected frame length up front. All
//! primitives come from audited RustCrypto crates; nothing here implements crypto
//! by hand. The content key zeroizes on drop and `MediaError` carries no secret
//! material.

#![forbid(unsafe_code)]

use core::fmt;

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest as _, Sha256};
use zeroize::ZeroizeOnDrop;

/// Domain separation for the per-chunk associated data.
const DOMAIN: &[u8] = b"mercury-attachment-v1";
/// Frame magic + version (anti-confusion + anti-downgrade).
const MAGIC: [u8; 4] = *b"MATT";
const VERSION: u8 = 1;
/// Per-attachment random id, bound into every chunk's AAD so chunks cannot be
/// spliced across attachments — even if the SAME content key is (mis)used for
/// more than one attachment of the same length.
const SALT_LEN: usize = 16;
/// magic(4) + version(1) + salt(16) + plaintext_len(u64 LE, 8). The salt makes the
/// frame self-identifying (anti-splicing); storing the exact plaintext length lets
/// the decoder compute the precise expected frame length up front, so truncation
/// and trailing bytes are rejected structurally. (Neither the salt nor the length
/// is more than the sealed blob's own byte length already reveals.)
const HEADER_LEN: usize = 4 + 1 + SALT_LEN + 8;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
/// Plaintext bytes per chunk (64 KiB). Every chunk except the last holds exactly
/// this many plaintext bytes; the last holds the remainder (possibly 0).
const CHUNK_SIZE: usize = 64 * 1024;

/// A 32-byte symmetric content key for one attachment. Generated fresh per
/// attachment; it travels inside the E2E message, never to the blob store.
/// Zeroized on drop; `Debug` is redacted.
#[derive(ZeroizeOnDrop)]
pub struct AttachmentKey([u8; 32]);

impl AttachmentKey {
    /// Generate a fresh random content key from the system CSPRNG.
    pub fn generate() -> Self {
        let key = XChaCha20Poly1305::generate_key(&mut OsRng);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&key);
        Self(bytes)
    }

    /// Reconstruct a content key from raw bytes (e.g. recovered from a message).
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The raw key bytes, to seal into the end-to-end message. Handle with care:
    /// the returned array is a plaintext copy of the secret.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
}

impl fmt::Debug for AttachmentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AttachmentKey(<redacted>)")
    }
}

/// Errors from sealing/opening an attachment. Carries no secret material; the
/// `Debug`/`Display` forms are safe to log.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaError {
    /// The sealed attachment is shorter than the framing requires, declares a
    /// plaintext length inconsistent with its actual byte length (truncation or
    /// extension), or carries trailing bytes.
    Malformed,
    /// The header carried an unrecognized magic or an unsupported version.
    UnsupportedVersion,
    /// AEAD sealing failed (e.g. the key was rejected by the cipher).
    SealFailed,
    /// AEAD opening failed: wrong key, tampering, truncation, reorder, corruption.
    OpenFailed,
    /// A downloaded sealed blob did not match the digest authenticated by the
    /// message (blob substitution, corruption, or truncation by the store).
    DigestMismatch,
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaError::Malformed => f.write_str("malformed sealed attachment"),
            MediaError::UnsupportedVersion => f.write_str("unsupported attachment version"),
            MediaError::SealFailed => f.write_str("attachment sealing failed"),
            MediaError::OpenFailed => f.write_str("attachment opening failed"),
            MediaError::DigestMismatch => f.write_str("sealed blob digest mismatch"),
        }
    }
}

impl std::error::Error for MediaError {}

/// Per-chunk associated data:
/// `DOMAIN || VERSION || salt || chunk_index || plaintext_len` (counts
/// little-endian). The `salt` (per-attachment random id) pins each chunk to THIS
/// attachment instance so chunks cannot be spliced across attachments even under
/// content-key reuse; `chunk_index` pins position (reordering fails); the
/// whole-attachment `plaintext_len` pins structure (truncation/extension fail).
fn chunk_aad(salt: &[u8; SALT_LEN], chunk_index: u64, plaintext_len: u64) -> Vec<u8> {
    let mut aad = Vec::with_capacity(DOMAIN.len() + 1 + SALT_LEN + 16);
    aad.extend_from_slice(DOMAIN);
    aad.push(VERSION);
    aad.extend_from_slice(salt);
    aad.extend_from_slice(&chunk_index.to_le_bytes());
    aad.extend_from_slice(&plaintext_len.to_le_bytes());
    aad
}

/// Seal `plaintext` under a freshly generated content key. Returns the key (to
/// seal into the end-to-end message) and the sealed blob (to store/upload).
pub fn seal_attachment(plaintext: &[u8]) -> Result<(AttachmentKey, Vec<u8>), MediaError> {
    let key = AttachmentKey::generate();
    let sealed = seal_attachment_with_key(&key, plaintext)?;
    Ok((key, sealed))
}

/// Seal `plaintext` under a caller-supplied content `key`. Produces
/// `magic || version || salt || plaintext_len || (nonce || chunk_ciphertext)*`.
///
/// A fresh random per-attachment `salt` is bound into every chunk, so the result
/// is robust against cross-attachment chunk splicing even if the caller reuses
/// `key` across attachments. (Still prefer [`seal_attachment`], which mints a
/// unique key per attachment.)
pub fn seal_attachment_with_key(
    key: &AttachmentKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, MediaError> {
    let plaintext_len = plaintext.len() as u64;
    // ceil(len / CHUNK_SIZE), but always at least one chunk (an empty attachment
    // is a single empty chunk, so it is still authenticated).
    let total_chunks = plaintext.len().div_ceil(CHUNK_SIZE).max(1);

    // Fresh per-attachment random id (first 16 bytes of a CSPRNG draw).
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&XChaCha20Poly1305::generate_nonce(&mut OsRng)[..SALT_LEN]);

    let cipher = XChaCha20Poly1305::new_from_slice(&key.0).map_err(|_| MediaError::SealFailed)?;

    let mut out = Vec::with_capacity(HEADER_LEN + plaintext.len() + total_chunks * 16);
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&plaintext_len.to_le_bytes());

    for index in 0..total_chunks {
        let start = index * CHUNK_SIZE;
        let end = start.saturating_add(CHUNK_SIZE).min(plaintext.len());
        let chunk = &plaintext[start..end];

        let aad = chunk_aad(&salt, index as u64, plaintext_len);
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: chunk,
                    aad: &aad,
                },
            )
            .map_err(|_| MediaError::SealFailed)?;

        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
    }
    Ok(out)
}

/// Open a sealed attachment produced by [`seal_attachment`] /
/// [`seal_attachment_with_key`] under the matching `key`, returning the recovered
/// plaintext. Fails closed on a wrong key, tampering, truncation, reordering,
/// extension, or any malformed framing.
pub fn open_attachment(key: &AttachmentKey, sealed: &[u8]) -> Result<Vec<u8>, MediaError> {
    if sealed.len() < HEADER_LEN {
        return Err(MediaError::Malformed);
    }
    let (header, mut body) = sealed.split_at(HEADER_LEN);
    if header[..4] != MAGIC {
        return Err(MediaError::UnsupportedVersion);
    }
    if header[4] != VERSION {
        return Err(MediaError::UnsupportedVersion);
    }
    // header = magic(0..4) || version(4) || salt(5..21) || plaintext_len(21..29).
    let salt: [u8; SALT_LEN] = header[5..5 + SALT_LEN].try_into().expect("16 salt bytes");
    let plaintext_len_u64 = u64::from_le_bytes(
        header[5 + SALT_LEN..HEADER_LEN]
            .try_into()
            .expect("8 bytes"),
    );
    // Reject a length we cannot even index on this platform (defensive; on 64-bit
    // usize == u64 so this never triggers for an in-memory blob).
    let plaintext_len = usize::try_from(plaintext_len_u64).map_err(|_| MediaError::Malformed)?;
    let total_chunks = plaintext_len.div_ceil(CHUNK_SIZE).max(1);

    // The frame body length is fully determined by the plaintext length:
    // total_chunks * (nonce + tag) + plaintext. A body of any other length is
    // malformed (this rejects truncation AND trailing/extension bytes up front).
    // `checked_*` guards the attacker-controlled length against overflow.
    let expected_body = total_chunks
        .checked_mul(NONCE_LEN + TAG_LEN)
        .and_then(|overhead| overhead.checked_add(plaintext_len))
        .ok_or(MediaError::Malformed)?;
    if body.len() != expected_body {
        return Err(MediaError::Malformed);
    }

    let cipher = XChaCha20Poly1305::new_from_slice(&key.0).map_err(|_| MediaError::OpenFailed)?;

    let mut plaintext = Vec::with_capacity(plaintext_len);
    for index in 0..total_chunks {
        // Every chunk but the last carries exactly CHUNK_SIZE plaintext; the last
        // carries the remainder (0..=CHUNK_SIZE).
        let chunk_plaintext_len = if index < total_chunks - 1 {
            CHUNK_SIZE
        } else {
            plaintext_len - (total_chunks - 1) * CHUNK_SIZE
        };
        // In-bounds by construction: the up-front exact-length check guarantees
        // the body holds precisely these slices for every chunk.
        let (nonce_bytes, rest) = body.split_at(NONCE_LEN);
        let (ciphertext, next) = rest.split_at(chunk_plaintext_len + TAG_LEN);

        let nonce = XNonce::from_slice(nonce_bytes);
        let aad = chunk_aad(&salt, index as u64, plaintext_len_u64);
        let chunk = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| MediaError::OpenFailed)?;
        plaintext.extend_from_slice(&chunk);
        body = next;
    }
    Ok(plaintext)
}

/// SHA-256 digest of a sealed attachment blob, used in the [`AttachmentReference`].
fn sealed_blob_digest(sealed: &[u8]) -> [u8; 32] {
    Sha256::digest(sealed).into()
}

/// What a sender places inside the END-TO-END message so a recipient can fetch +
/// open an attachment: the content `key`, a `sealed_digest` (SHA-256 of the
/// sealed blob), and the `plaintext_len`. The whole reference is carried as the
/// E2E message payload (sealed by mercury-message), so the key stays confidential
/// and the digest is AUTHENTICATED end-to-end — letting the recipient verify a
/// blob downloaded from the untrusted store BEFORE decrypting, which closes
/// whole-blob substitution (the store cannot swap in a different blob without
/// failing the digest check). The blob's storage locator is left to the caller
/// (it is non-secret routing the app already manages).
///
/// `to_bytes` is the canonical payload encoding; it exposes the content key
/// (necessarily — it is the secret the recipient needs), so the bytes MUST be
/// sealed into the E2E message and never stored or transmitted in the clear.
pub struct AttachmentReference {
    content_key: AttachmentKey,
    sealed_digest: [u8; 32],
    plaintext_len: u64,
}

/// Reference framing: `magic || version || key(32) || sealed_digest(32) || plaintext_len(8)`.
const REF_MAGIC: [u8; 4] = *b"MREF";
const REF_LEN: usize = 4 + 1 + 32 + 32 + 8;

impl AttachmentReference {
    /// Build the reference for a `sealed` blob produced by [`seal_attachment`] /
    /// [`seal_attachment_with_key`] under `content_key`, recording the blob's
    /// SHA-256 digest and the original `plaintext_len`.
    pub fn new(content_key: AttachmentKey, sealed: &[u8], plaintext_len: u64) -> Self {
        Self {
            content_key,
            sealed_digest: sealed_blob_digest(sealed),
            plaintext_len,
        }
    }

    /// The authenticated SHA-256 digest of the sealed blob.
    pub fn sealed_digest(&self) -> &[u8; 32] {
        &self.sealed_digest
    }

    /// The original attachment plaintext length (UI hint + post-decrypt sanity).
    pub fn plaintext_len(&self) -> u64 {
        self.plaintext_len
    }

    /// Canonical bytes to seal as the E2E message payload. Exposes the content
    /// key — seal these into the message; never store/transmit them in the clear.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(REF_LEN);
        out.extend_from_slice(&REF_MAGIC);
        out.push(VERSION);
        out.extend_from_slice(&self.content_key.0);
        out.extend_from_slice(&self.sealed_digest);
        out.extend_from_slice(&self.plaintext_len.to_le_bytes());
        out
    }

    /// Parse a reference from the (decrypted) E2E message payload. Fails closed on
    /// a bad magic/version, wrong length, or any malformed framing.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MediaError> {
        if bytes.len() != REF_LEN {
            return Err(MediaError::Malformed);
        }
        if bytes[..4] != REF_MAGIC || bytes[4] != VERSION {
            return Err(MediaError::UnsupportedVersion);
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes[5..37]);
        let mut sealed_digest = [0u8; 32];
        sealed_digest.copy_from_slice(&bytes[37..69]);
        let plaintext_len = u64::from_le_bytes(bytes[69..REF_LEN].try_into().expect("8 bytes"));
        Ok(Self {
            content_key: AttachmentKey::from_bytes(key),
            sealed_digest,
            plaintext_len,
        })
    }

    /// Verify a `sealed` blob downloaded from the (untrusted) store against the
    /// authenticated digest, then open it. Rejects with [`MediaError::DigestMismatch`]
    /// if the blob does not match (substitution/corruption/truncation) — a check
    /// that also lets a client avoid decrypting a large bad blob. The per-attachment
    /// key would already reject a foreign blob, so this is defense-in-depth + an
    /// early, cheap integrity gate; the recovered plaintext length is also checked
    /// against the authenticated `plaintext_len`.
    pub fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, MediaError> {
        if sealed_blob_digest(sealed) != self.sealed_digest {
            return Err(MediaError::DigestMismatch);
        }
        let plaintext = open_attachment(&self.content_key, sealed)?;
        if plaintext.len() as u64 != self.plaintext_len {
            return Err(MediaError::OpenFailed);
        }
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(plaintext: &[u8]) {
        let (key, sealed) = seal_attachment(plaintext).unwrap();
        assert_eq!(open_attachment(&key, &sealed).unwrap(), plaintext);
    }

    #[test]
    fn round_trips_across_sizes() {
        roundtrip(b"");
        roundtrip(b"a");
        roundtrip(b"a short attachment");
        roundtrip(&[0x5au8; CHUNK_SIZE - 1]);
        roundtrip(&[0x5au8; CHUNK_SIZE]); // exact one-chunk boundary
        roundtrip(&[0x5au8; CHUNK_SIZE + 1]); // spills into a 2nd chunk
        roundtrip(&[0x5au8; CHUNK_SIZE * 2]); // exact two-chunk boundary
        roundtrip(&[0x33u8; CHUNK_SIZE * 2 + 7]); // 3 chunks, partial last
    }

    #[test]
    fn distinct_keys_and_nonces_per_seal() {
        let a = AttachmentKey::generate();
        let b = AttachmentKey::generate();
        assert_ne!(a.to_bytes(), b.to_bytes());

        // Two seals of the same plaintext under the same key differ (fresh nonce).
        let plaintext = b"same bytes";
        let s1 = seal_attachment_with_key(&a, plaintext).unwrap();
        let s2 = seal_attachment_with_key(&a, plaintext).unwrap();
        assert_ne!(s1, s2);
        // ...yet both open to the original.
        assert_eq!(open_attachment(&a, &s1).unwrap(), plaintext);
        assert_eq!(open_attachment(&a, &s2).unwrap(), plaintext);
    }

    #[test]
    fn wrong_key_fails_closed() {
        let (_key, sealed) = seal_attachment(b"secret media").unwrap();
        let other = AttachmentKey::generate();
        assert_eq!(
            open_attachment(&other, &sealed),
            Err(MediaError::OpenFailed)
        );
    }

    #[test]
    fn tampered_ciphertext_fails_closed() {
        let (key, mut sealed) = seal_attachment(b"tamper target bytes").unwrap();
        // Flip a byte inside the first chunk's ciphertext (past the header+nonce).
        let pos = HEADER_LEN + NONCE_LEN + 1;
        sealed[pos] ^= 0x01;
        assert_eq!(open_attachment(&key, &sealed), Err(MediaError::OpenFailed));
    }

    #[test]
    fn truncation_fails_closed() {
        // A 2-chunk attachment; dropping the last chunk's bytes (header still
        // claims 2 chunks) must be rejected, not silently truncated.
        let (key, sealed) = seal_attachment(&[0x11u8; CHUNK_SIZE + 100]).unwrap();
        let chopped = &sealed[..HEADER_LEN + NONCE_LEN + CHUNK_SIZE + TAG_LEN + 8];
        assert!(matches!(
            open_attachment(&key, chopped),
            Err(MediaError::Malformed) | Err(MediaError::OpenFailed)
        ));
    }

    #[test]
    fn tampering_the_header_length_fails_closed() {
        // The header plaintext_len both determines the expected frame length and
        // is bound into every chunk's AAD, so altering it fails closed.
        let (key, mut sealed) = seal_attachment(&[0x22u8; CHUNK_SIZE + 50]).unwrap();
        // plaintext_len occupies header bytes [5 + SALT_LEN .. HEADER_LEN].
        let lo = 5 + SALT_LEN;
        let real_len = u64::from_le_bytes(sealed[lo..HEADER_LEN].try_into().unwrap());
        assert_eq!(real_len, (CHUNK_SIZE + 50) as u64);
        // Declared length no longer matches the actual body length -> Malformed
        // (and even if it did, the AAD would mismatch -> OpenFailed). Rejected.
        sealed[lo..HEADER_LEN].copy_from_slice(&(real_len + 1).to_le_bytes());
        assert!(open_attachment(&key, &sealed).is_err());
    }

    #[test]
    fn reordered_chunks_fail_closed() {
        // Two full equal-size chunks -> their frames are identical length, so we
        // can swap them. Each was sealed with AAD binding its index, so the swap
        // is detected.
        let (key, sealed) = seal_attachment(&[0x44u8; CHUNK_SIZE * 2]).unwrap();
        let frame = NONCE_LEN + CHUNK_SIZE + TAG_LEN;
        let (header, body) = sealed.split_at(HEADER_LEN);
        let (c0, c1) = body.split_at(frame);
        let mut swapped = Vec::new();
        swapped.extend_from_slice(header);
        swapped.extend_from_slice(c1);
        swapped.extend_from_slice(c0);
        assert_eq!(swapped.len(), sealed.len());
        assert_eq!(open_attachment(&key, &swapped), Err(MediaError::OpenFailed));
    }

    #[test]
    fn chunk_splicing_across_attachments_fails_under_key_reuse() {
        // Defense-in-depth: even if a caller WRONGLY reuses one content key for two
        // attachments of the same length, the per-attachment random salt (bound
        // into every chunk's AAD) makes cross-attachment chunk splicing fail
        // closed. Without the salt this splice would open to a Frankenstein blob.
        let key = AttachmentKey::from_bytes([9u8; 32]);
        let a = seal_attachment_with_key(&key, &[0xAAu8; CHUNK_SIZE * 2]).unwrap();
        let b = seal_attachment_with_key(&key, &[0xBBu8; CHUNK_SIZE * 2]).unwrap();
        assert_eq!(a.len(), b.len()); // identical length -> identical framing layout
        // Two distinct attachments get distinct salts.
        assert_ne!(a[5..5 + SALT_LEN], b[5..5 + SALT_LEN]);

        // Splice A's first chunk frame into B (keeping B's header, hence B's salt).
        let frame = NONCE_LEN + CHUNK_SIZE + TAG_LEN;
        let mut spliced = Vec::new();
        spliced.extend_from_slice(&b[..HEADER_LEN]);
        spliced.extend_from_slice(&a[HEADER_LEN..HEADER_LEN + frame]); // A's chunk 0
        spliced.extend_from_slice(&b[HEADER_LEN + frame..]); // B's chunk 1
        assert_eq!(spliced.len(), b.len());
        // A's chunk 0 was sealed with A's salt; under B's header (B's salt) the
        // recomputed AAD differs -> AEAD rejects.
        assert_eq!(open_attachment(&key, &spliced), Err(MediaError::OpenFailed));
    }

    #[test]
    fn trailing_bytes_fail_closed() {
        let (key, mut sealed) = seal_attachment(b"exactly framed").unwrap();
        sealed.push(0x00); // append one byte
        assert_eq!(open_attachment(&key, &sealed), Err(MediaError::Malformed));
    }

    #[test]
    fn bad_header_is_rejected() {
        let (key, sealed) = seal_attachment(b"hi").unwrap();
        // Too short for even the header.
        assert_eq!(
            open_attachment(&key, &sealed[..HEADER_LEN - 1]),
            Err(MediaError::Malformed)
        );
        // Wrong magic.
        let mut bad_magic = sealed.clone();
        bad_magic[0] ^= 0xff;
        assert_eq!(
            open_attachment(&key, &bad_magic),
            Err(MediaError::UnsupportedVersion)
        );
        // Unsupported version.
        let mut bad_version = sealed.clone();
        bad_version[4] = 0xff;
        assert_eq!(
            open_attachment(&key, &bad_version),
            Err(MediaError::UnsupportedVersion)
        );
    }

    #[test]
    fn debug_redacts_key() {
        let key = AttachmentKey::from_bytes([7u8; 32]);
        assert_eq!(format!("{key:?}"), "AttachmentKey(<redacted>)");
    }

    #[test]
    fn reference_round_trips_and_opens() {
        let plaintext = b"an attachment to reference in a message";
        let (key, sealed) = seal_attachment(plaintext).unwrap();
        let reference = AttachmentReference::new(key, &sealed, plaintext.len() as u64);

        // The reference serializes to the E2E payload and parses back identically.
        let wire = reference.to_bytes();
        assert_eq!(wire.len(), REF_LEN);
        let parsed = AttachmentReference::from_bytes(&wire).unwrap();
        assert_eq!(parsed.sealed_digest(), reference.sealed_digest());
        assert_eq!(parsed.plaintext_len(), plaintext.len() as u64);

        // The parsed reference verifies + opens the blob to the original.
        assert_eq!(parsed.open(&sealed).unwrap(), plaintext);
    }

    #[test]
    fn reference_detects_blob_substitution() {
        // A reference for attachment A must reject a (validly-sealed) DIFFERENT
        // attachment B substituted by the store -- the digest is authenticated.
        let (key_a, sealed_a) = seal_attachment(b"attachment A bytes").unwrap();
        let (_key_b, sealed_b) = seal_attachment(b"attachment B bytes").unwrap();
        let reference = AttachmentReference::new(key_a, &sealed_a, 18);
        assert_eq!(reference.open(&sealed_b), Err(MediaError::DigestMismatch));
        // ...and still opens the genuine blob.
        assert_eq!(reference.open(&sealed_a).unwrap(), b"attachment A bytes");
    }

    #[test]
    fn reference_detects_tampered_blob() {
        let (key, mut sealed) = seal_attachment(b"tamper me at rest").unwrap();
        let reference =
            AttachmentReference::new(AttachmentKey::from_bytes(key.to_bytes()), &sealed, 17);
        // Flip a byte of the stored blob -> the digest no longer matches.
        let pos = sealed.len() / 2;
        sealed[pos] ^= 0x01;
        assert_eq!(reference.open(&sealed), Err(MediaError::DigestMismatch));
    }

    #[test]
    fn reference_plaintext_len_is_checked() {
        // A reference that lies about plaintext_len (digest still matches the
        // blob) is rejected once the decrypted length disagrees.
        let plaintext = b"len-bound payload";
        let (key, sealed) = seal_attachment(plaintext).unwrap();
        let wrong = AttachmentReference::new(key, &sealed, plaintext.len() as u64 + 1);
        assert_eq!(wrong.open(&sealed), Err(MediaError::OpenFailed));
    }

    #[test]
    fn reference_from_bytes_fails_closed() {
        let (key, sealed) = seal_attachment(b"x").unwrap();
        let wire = AttachmentReference::new(key, &sealed, 1).to_bytes();
        // Wrong length (AttachmentReference holds a secret key, so it is not
        // Debug/PartialEq -> assert on the error via matches!).
        assert!(matches!(
            AttachmentReference::from_bytes(&wire[..REF_LEN - 1]),
            Err(MediaError::Malformed)
        ));
        assert!(matches!(
            AttachmentReference::from_bytes(&[]),
            Err(MediaError::Malformed)
        ));
        // Bad magic / version.
        let mut bad = wire.clone();
        bad[0] ^= 0xff;
        assert!(matches!(
            AttachmentReference::from_bytes(&bad),
            Err(MediaError::UnsupportedVersion)
        ));
        let mut bad_v = wire.clone();
        bad_v[4] = 0xff;
        assert!(matches!(
            AttachmentReference::from_bytes(&bad_v),
            Err(MediaError::UnsupportedVersion)
        ));
    }
}
