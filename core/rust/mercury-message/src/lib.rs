//! Local encrypted message objects for Mercury (Phase 2).
//!
//! Connects the audited sealed-box crypto ([`mercury_sealedbox`]) to the client
//! message envelope ([`mercury_core::ClientMessageEnvelope`]), producing a local
//! encrypted message object whose envelope is validated by the EXISTING policy
//! gate ([`mercury_policy::validate_envelope`]) — never bypassed. Local-only: no
//! network, no session/ratchet protocol.
//!
//! Fails closed: every fallible path returns [`MessageError`]; the wire parser
//! never panics on arbitrary input. The ciphertext carries no plaintext, so
//! [`SealedMessage`]'s `Debug` is safe; no secret or plaintext is logged.

#![forbid(unsafe_code)]

use core::fmt;

use mercury_core::{
    ClientMessageEnvelope, ConversationPolicyState, EnvelopeFacts, MessageKind, ProtocolSuite,
};
use mercury_keys::{DeviceKeyPair, DevicePublicKey, MlKemKeyPair, MlKemPublicKey};
use mercury_policy::{self as policy, ACCEPT};
use mercury_sealedbox::SealError;

/// `ephemeral_pub (32) || commitment (32) || nonce (24) || tag (16)` framing
/// overhead added by the classical sealed box on top of the plaintext length.
/// Includes the 32-byte key commitment that makes the sealed box key-committing.
pub const SEALED_OVERHEAD: usize = 104;

/// `ephemeral_x (32) || ml_kem_ct (1088) || commitment (32) || nonce (24) || tag (16)`
/// framing overhead added by the HYBRID (X25519 + ML-KEM-768) sealed box. Includes
/// the 32-byte key commitment over both shares.
pub const HYBRID_SEALED_OVERHEAD: usize = 1192;

const HEADER_FIELDS: usize = 10;
const HEADER_LEN: usize = HEADER_FIELDS * 4;
const LEN_PREFIX_LEN: usize = 4;

/// Length-padding bucket size. Before sealing, plaintext is wrapped as
/// `true_len (u32 LE) || plaintext || zero-pad` and the whole thing is grown to the
/// next multiple of `PAD_BUCKET` bytes. So the SEALED length only reveals which
/// 256-byte bucket the message falls in, not its exact size — two different
/// messages in the same bucket seal to identical lengths, blunting length-based
/// traffic analysis. The 4-byte true-length prefix is recovered on open and the
/// padding stripped, so the scheme is exactly reversible.
///
/// Padding inflates the sealed `payload_len` by up to `PAD_BUCKET - 1` bytes plus
/// the 4-byte prefix, and the policy gate checks the PADDED length against
/// `max_payload_len`. So a plaintext within ~`PAD_BUCKET` of the cap can round over
/// it and be `PolicyRejected` — the seal fails closed (never silently sends an
/// undersized object). Size the conversation's `max_payload_len` with this bucket
/// headroom in mind.
pub const PAD_BUCKET: usize = 256;

/// The padded length for a plaintext of `plaintext_len` bytes: the sealed-box
/// plaintext is `true_len(4) || plaintext`, rounded up to the next [`PAD_BUCKET`]
/// boundary. Callers can use this to predict `payload_len = padded_len + OVERHEAD`.
/// Returns `None` if the length cannot be represented.
pub fn padded_len(plaintext_len: usize) -> Option<usize> {
    let unpadded = PAD_PREFIX_LEN.checked_add(plaintext_len)?;
    let buckets = unpadded.div_ceil(PAD_BUCKET).max(1);
    buckets.checked_mul(PAD_BUCKET)
}
/// The padded form carries a 4-byte true-length prefix ahead of the plaintext.
const PAD_PREFIX_LEN: usize = 4;

/// Pad `plaintext` to a bucket boundary: `true_len(u32 LE) || plaintext || zeros`,
/// total length rounded up to the next multiple of [`PAD_BUCKET`] (always at least
/// one bucket). Reversible by [`unpad`]. Returns `None` only if the length cannot be
/// represented (plaintext longer than `u32::MAX`), which the caller maps to
/// `PlaintextTooLarge`.
fn pad_to_bucket(plaintext: &[u8]) -> Option<Vec<u8>> {
    let true_len = u32::try_from(plaintext.len()).ok()?;
    let unpadded = PAD_PREFIX_LEN.checked_add(plaintext.len())?;
    // Round up to the next whole bucket (a 0-length-in-last-bucket message still
    // occupies a full bucket, so the padded length is always a positive multiple).
    let buckets = unpadded.div_ceil(PAD_BUCKET).max(1);
    let padded_len = buckets.checked_mul(PAD_BUCKET)?;
    let mut padded = Vec::with_capacity(padded_len);
    padded.extend_from_slice(&true_len.to_le_bytes());
    padded.extend_from_slice(plaintext);
    padded.resize(padded_len, 0u8);
    Some(padded)
}

/// Reverse [`pad_to_bucket`]: read the 4-byte true length and return exactly that
/// many plaintext bytes. Fails closed (`Malformed`) if the buffer is too short to
/// hold the prefix, if the declared length overflows the buffer, or if the framing
/// is not a whole number of buckets — so a corrupted length header can never cause
/// an out-of-bounds read or surface padding bytes as plaintext.
fn unpad(padded: &[u8]) -> Result<Vec<u8>, MessageError> {
    if padded.len() < PAD_PREFIX_LEN
        || padded.is_empty()
        || !padded.len().is_multiple_of(PAD_BUCKET)
    {
        return Err(MessageError::Malformed);
    }
    let mut len_bytes = [0u8; PAD_PREFIX_LEN];
    len_bytes.copy_from_slice(&padded[..PAD_PREFIX_LEN]);
    let true_len = u32::from_le_bytes(len_bytes) as usize;
    let end = PAD_PREFIX_LEN
        .checked_add(true_len)
        .ok_or(MessageError::Malformed)?;
    if end > padded.len() {
        return Err(MessageError::Malformed);
    }
    Ok(padded[PAD_PREFIX_LEN..end].to_vec())
}

/// A locally encrypted message: a policy-validated envelope plus the sealed-box
/// ciphertext. The ciphertext is encrypted, so `Debug` is safe to log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedMessage {
    pub envelope: ClientMessageEnvelope,
    pub ciphertext: Vec<u8>,
}

/// Errors surfaced when sealing, opening, or (de)serializing a message object.
///
/// Carries no secret or plaintext material; `Debug`/`Display` are safe to log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageError {
    /// The envelope policy gate rejected the envelope (carries the reason code).
    PolicyRejected(i32),
    /// Sealing failed.
    Seal(SealError),
    /// Opening failed: wrong recipient, tampering, or corruption.
    Open(SealError),
    /// `envelope.payload_len` did not match the ciphertext length.
    PayloadLenMismatch,
    /// Wire bytes were truncated, had trailing bytes, or were otherwise invalid.
    Malformed,
    /// The serialized suite code did not map to a known suite.
    UnsupportedSuite,
    /// The serialized kind code did not map to a known message kind.
    UnsupportedKind,
    /// The plaintext (plus overhead) does not fit a positive `i32` payload_len.
    PlaintextTooLarge,
    /// The envelope's protocol suite does not match the seal/open function used
    /// (e.g. a classical seal on a `HybridPqDev` envelope, or vice-versa), which
    /// would mislabel the crypto. The suite must match the cryptography applied.
    SuiteMismatch,
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::PolicyRejected(code) => write!(f, "envelope policy rejected: {code}"),
            MessageError::Seal(err) => write!(f, "seal failed: {err}"),
            MessageError::Open(err) => write!(f, "open failed: {err}"),
            MessageError::PayloadLenMismatch => f.write_str("payload length mismatch"),
            MessageError::Malformed => f.write_str("malformed message bytes"),
            MessageError::UnsupportedSuite => f.write_str("unsupported suite code"),
            MessageError::UnsupportedKind => f.write_str("unsupported message kind code"),
            MessageError::PlaintextTooLarge => f.write_str("plaintext too large for payload_len"),
            MessageError::SuiteMismatch => {
                f.write_str("protocol suite does not match the seal/open crypto")
            }
        }
    }
}

impl std::error::Error for MessageError {}

/// Decode a [`ProtocolSuite`] from its wire code (mercury-core exposes only the
/// forward `.code()`, so the reverse map lives here).
fn suite_from_code(code: i32) -> Option<ProtocolSuite> {
    match code {
        257 => Some(ProtocolSuite::ClassicalDev),
        513 => Some(ProtocolSuite::HybridPqDev),
        _ => None,
    }
}

/// Decode a [`MessageKind`] from its wire code (reverse of `.code()`).
fn kind_from_code(code: i32) -> Option<MessageKind> {
    match code {
        1 => Some(MessageKind::Application),
        2 => Some(MessageKind::Control),
        3 => Some(MessageKind::AiContext),
        _ => None,
    }
}

/// Build the policy-gate input from an envelope and its conversation context,
/// mirroring `ClientMessagePolicyInput::envelope_facts`, then run the EXISTING
/// `mercury_policy::validate_envelope`. Returns the reason code (ACCEPT == 0).
fn validate(envelope: &ClientMessageEnvelope, context: &ConversationPolicyState) -> i32 {
    let facts = EnvelopeFacts {
        version: envelope.version,
        suite_id: envelope.suite.code(),
        conversation_id_len: envelope.conversation_id_len,
        sender_account_id_len: envelope.sender_account_id_len,
        sender_device_id_len: envelope.sender_device_id_len,
        epoch: envelope.epoch,
        sequence: envelope.sequence,
        message_kind: envelope.kind.code(),
        payload_len: envelope.payload_len,
        critical_flags: envelope.critical_flags,
        noncritical_flags: envelope.noncritical_flags,
        expected_epoch: context.expected_epoch,
        expected_sequence: context.expected_sequence,
        min_suite_id: context.minimum_suite.code(),
        max_payload_len: context.max_payload_len,
    };
    policy::validate_envelope(facts.into())
}

/// Canonical fixed-order bytes of the envelope header, bound into the sealed-box AEAD associated data
/// so that tampering with the (otherwise plaintext, stateless-validated) envelope — flipping `kind`
/// Application↔Control, the suite, a length field, or a flag — flips the AEAD tag on open
/// (`OpenFailed`). `epoch`/`sequence` are also exact-matched by the policy gate; binding the WHOLE
/// header here closes the residual field-confusion gap. Must be identical on seal + open.
fn envelope_aad(e: &ClientMessageEnvelope) -> Vec<u8> {
    let mut aad = Vec::with_capacity(11 * 4);
    aad.extend_from_slice(&e.version.to_le_bytes());
    aad.extend_from_slice(&e.suite.code().to_le_bytes());
    aad.extend_from_slice(&e.conversation_id_len.to_le_bytes());
    aad.extend_from_slice(&e.sender_account_id_len.to_le_bytes());
    aad.extend_from_slice(&e.sender_device_id_len.to_le_bytes());
    aad.extend_from_slice(&e.epoch.to_le_bytes());
    aad.extend_from_slice(&e.sequence.to_le_bytes());
    aad.extend_from_slice(&e.kind.code().to_le_bytes());
    aad.extend_from_slice(&e.payload_len.to_le_bytes());
    aad.extend_from_slice(&e.critical_flags.to_le_bytes());
    aad.extend_from_slice(&e.noncritical_flags.to_le_bytes());
    aad
}

/// Seal `plaintext` for `recipient`, gated by the envelope policy.
///
/// The plaintext is first LENGTH-PADDED to a [`PAD_BUCKET`] boundary, so the
/// sealed length only reveals which bucket the message falls in (not its exact
/// size). `envelope.payload_len` is set to `padded_len + SEALED_OVERHEAD`; the
/// policy gate runs BEFORE any sealing, so a rejected envelope produces no
/// ciphertext.
///
/// The envelope header is also bound into the sealed-box AEAD (see [`envelope_aad`]), so it cannot be
/// rewritten in transit.
pub fn seal_message(
    recipient: &DevicePublicKey,
    plaintext: &[u8],
    mut envelope: ClientMessageEnvelope,
    context: &ConversationPolicyState,
) -> Result<SealedMessage, MessageError> {
    let padded = pad_to_bucket(plaintext).ok_or(MessageError::PlaintextTooLarge)?;
    let ciphertext_len = padded
        .len()
        .checked_add(SEALED_OVERHEAD)
        .ok_or(MessageError::PlaintextTooLarge)?;
    let payload_len = i32::try_from(ciphertext_len).map_err(|_| MessageError::PlaintextTooLarge)?;
    envelope.payload_len = payload_len;

    let reason = validate(&envelope, context);
    if reason != ACCEPT {
        return Err(MessageError::PolicyRejected(reason));
    }

    // A classical seal must not fulfill a post-quantum suite, or the message
    // would be labelled `HybridPqDev` while only classically protected.
    if matches!(envelope.suite, ProtocolSuite::HybridPqDev) {
        return Err(MessageError::SuiteMismatch);
    }

    let ciphertext = mercury_sealedbox::seal_with_context(recipient, &padded, &envelope_aad(&envelope))
        .map_err(MessageError::Seal)?;
    Ok(SealedMessage {
        envelope,
        ciphertext,
    })
}

/// Open a [`SealedMessage`] for `recipient`, gated by the envelope policy.
///
/// Re-validates the envelope, checks that `payload_len` matches the carried
/// ciphertext, decrypts, then strips the length padding. Fails closed on
/// tampering, wrong recipient, or corrupt padding.
///
/// REPLAY is the CALLER's responsibility — this is a STATELESS validator. It checks `sequence`/`epoch`
/// against the supplied `context` but never advances it, caches no nonce, and keeps no seen-set, so
/// the same `SealedMessage` opens twice if handed the same `context` twice. The embedding session must
/// advance `expected_sequence` on every accepted message; in a real deployment the relay's per-submit
/// replay token + the session ratchet own replay (this crate is local-only, no session protocol).
pub fn open_message(
    recipient: &DeviceKeyPair,
    msg: &SealedMessage,
    context: &ConversationPolicyState,
) -> Result<Vec<u8>, MessageError> {
    let reason = validate(&msg.envelope, context);
    if reason != ACCEPT {
        return Err(MessageError::PolicyRejected(reason));
    }

    if matches!(msg.envelope.suite, ProtocolSuite::HybridPqDev) {
        return Err(MessageError::SuiteMismatch);
    }

    if usize::try_from(msg.envelope.payload_len).ok() != Some(msg.ciphertext.len()) {
        return Err(MessageError::PayloadLenMismatch);
    }

    let padded =
        mercury_sealedbox::open_with_context(recipient, &msg.ciphertext, &envelope_aad(&msg.envelope))
            .map_err(MessageError::Open)?;
    unpad(&padded)
}

/// Seal `plaintext` to a recipient's HYBRID identity (`recipient_x` X25519 +
/// `recipient_pq` ML-KEM-768) under a `HybridPqDev` envelope, gated by the
/// envelope policy. The envelope's suite MUST be `HybridPqDev` (else
/// [`MessageError::SuiteMismatch`]), so the label always matches the crypto.
pub fn seal_message_hybrid(
    recipient_x: &DevicePublicKey,
    recipient_pq: &MlKemPublicKey,
    plaintext: &[u8],
    mut envelope: ClientMessageEnvelope,
    context: &ConversationPolicyState,
) -> Result<SealedMessage, MessageError> {
    let padded = pad_to_bucket(plaintext).ok_or(MessageError::PlaintextTooLarge)?;
    let ciphertext_len = padded
        .len()
        .checked_add(HYBRID_SEALED_OVERHEAD)
        .ok_or(MessageError::PlaintextTooLarge)?;
    let payload_len = i32::try_from(ciphertext_len).map_err(|_| MessageError::PlaintextTooLarge)?;
    envelope.payload_len = payload_len;

    let reason = validate(&envelope, context);
    if reason != ACCEPT {
        return Err(MessageError::PolicyRejected(reason));
    }

    // The hybrid crypto must be labelled as the post-quantum suite.
    if !matches!(envelope.suite, ProtocolSuite::HybridPqDev) {
        return Err(MessageError::SuiteMismatch);
    }

    let ciphertext = mercury_sealedbox::seal_hybrid_with_context(
        recipient_x,
        recipient_pq,
        &padded,
        &envelope_aad(&envelope),
    )
    .map_err(MessageError::Seal)?;
    Ok(SealedMessage {
        envelope,
        ciphertext,
    })
}

/// Open a hybrid [`SealedMessage`] (produced by [`seal_message_hybrid`]) for the
/// recipient's `recipient_x` (X25519) + `recipient_pq` (ML-KEM-768) keypairs.
/// Re-validates the envelope + requires the `HybridPqDev` suite, then decrypts.
/// Fails closed on tampering, wrong recipient, or a suite/length mismatch.
pub fn open_message_hybrid(
    recipient_x: &DeviceKeyPair,
    recipient_pq: &MlKemKeyPair,
    msg: &SealedMessage,
    context: &ConversationPolicyState,
) -> Result<Vec<u8>, MessageError> {
    let reason = validate(&msg.envelope, context);
    if reason != ACCEPT {
        return Err(MessageError::PolicyRejected(reason));
    }

    if !matches!(msg.envelope.suite, ProtocolSuite::HybridPqDev) {
        return Err(MessageError::SuiteMismatch);
    }

    if usize::try_from(msg.envelope.payload_len).ok() != Some(msg.ciphertext.len()) {
        return Err(MessageError::PayloadLenMismatch);
    }

    let padded = mercury_sealedbox::open_hybrid_with_context(
        recipient_x,
        recipient_pq,
        &msg.ciphertext,
        &envelope_aad(&msg.envelope),
    )
    .map_err(MessageError::Open)?;
    unpad(&padded)
}

/// Seal `plaintext` PREFERRING the post-quantum hybrid path: when `recipient_pq`
/// (an ML-KEM-768 public key) is `Some`, the message is sealed with the hybrid
/// X25519 + ML-KEM-768 box and the envelope is labelled `HybridPqDev`; when it is
/// `None`, it falls back to the classical X25519 box labelled `ClassicalDev`. So
/// post-quantum protection is the DEFAULT whenever the recipient has a PQ key, with
/// classical only as an explicit fallback.
///
/// The envelope's suite is SET to match the cryptography actually applied (so the
/// label can never overstate the protection); the existing per-path `SuiteMismatch`
/// guard still enforces the match. Both branches go through the same length-padding
/// + key-committing sealed box.
pub fn seal_message_auto(
    recipient_x: &DevicePublicKey,
    recipient_pq: Option<&MlKemPublicKey>,
    plaintext: &[u8],
    mut envelope: ClientMessageEnvelope,
    context: &ConversationPolicyState,
) -> Result<SealedMessage, MessageError> {
    match recipient_pq {
        Some(pq) => {
            // Prefer post-quantum: label honestly as the hybrid suite.
            envelope.suite = ProtocolSuite::HybridPqDev;
            seal_message_hybrid(recipient_x, pq, plaintext, envelope, context)
        }
        None => {
            // No PQ key available: classical fallback, labelled honestly.
            envelope.suite = ProtocolSuite::ClassicalDev;
            seal_message(recipient_x, plaintext, envelope, context)
        }
    }
}

/// Open a [`SealedMessage`] produced by [`seal_message_auto`] (or any seal),
/// dispatching on the envelope's declared suite: `HybridPqDev` uses the hybrid
/// open path (which REQUIRES `recipient_pq`), `ClassicalDev` uses the classical
/// path. Fails closed with [`MessageError::SuiteMismatch`] if a hybrid message is
/// presented without an ML-KEM keypair, so the dispatch can never silently
/// downgrade.
pub fn open_message_auto(
    recipient_x: &DeviceKeyPair,
    recipient_pq: Option<&MlKemKeyPair>,
    msg: &SealedMessage,
    context: &ConversationPolicyState,
) -> Result<Vec<u8>, MessageError> {
    match msg.envelope.suite {
        ProtocolSuite::HybridPqDev => {
            let pq = recipient_pq.ok_or(MessageError::SuiteMismatch)?;
            open_message_hybrid(recipient_x, pq, msg, context)
        }
        ProtocolSuite::ClassicalDev => open_message(recipient_x, msg, context),
    }
}

/// Serialize a message to canonical little-endian wire bytes: the ten envelope
/// scalars as `i32` LE in fixed order, then the ciphertext length as `u32` LE,
/// then the ciphertext. `payload_len` is recomputed from the ciphertext on
/// parse, so it is not serialized.
pub fn to_bytes(msg: &SealedMessage) -> Vec<u8> {
    let e = &msg.envelope;
    let fields = [
        e.version,
        e.suite.code(),
        e.conversation_id_len,
        e.sender_account_id_len,
        e.sender_device_id_len,
        e.epoch,
        e.sequence,
        e.kind.code(),
        e.critical_flags,
        e.noncritical_flags,
    ];
    let mut out = Vec::with_capacity(HEADER_LEN + LEN_PREFIX_LEN + msg.ciphertext.len());
    for field in fields {
        out.extend_from_slice(&field.to_le_bytes());
    }
    out.extend_from_slice(&(msg.ciphertext.len() as u32).to_le_bytes());
    out.extend_from_slice(&msg.ciphertext);
    out
}

/// Strict, panic-free cursor over input bytes. Every read is bounds-checked; no
/// `unwrap`/`expect`/indexing on input-derived data.
struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let slice = self.bytes.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    fn read_i32(&mut self) -> Option<i32> {
        let bytes: [u8; 4] = self.take(4)?.try_into().ok()?;
        Some(i32::from_le_bytes(bytes))
    }

    fn read_u32(&mut self) -> Option<u32> {
        let bytes: [u8; 4] = self.take(4)?.try_into().ok()?;
        Some(u32::from_le_bytes(bytes))
    }

    fn finished(&self) -> bool {
        self.pos == self.bytes.len()
    }
}

/// Parse canonical wire bytes produced by [`to_bytes`]. STRICT: requires exactly
/// the header + length prefix + that many ciphertext bytes with NO trailing
/// bytes. Never panics on any input.
pub fn from_bytes(bytes: &[u8]) -> Result<SealedMessage, MessageError> {
    let mut cur = Cursor::new(bytes);

    let version = cur.read_i32().ok_or(MessageError::Malformed)?;
    let suite_code = cur.read_i32().ok_or(MessageError::Malformed)?;
    let conversation_id_len = cur.read_i32().ok_or(MessageError::Malformed)?;
    let sender_account_id_len = cur.read_i32().ok_or(MessageError::Malformed)?;
    let sender_device_id_len = cur.read_i32().ok_or(MessageError::Malformed)?;
    let epoch = cur.read_i32().ok_or(MessageError::Malformed)?;
    let sequence = cur.read_i32().ok_or(MessageError::Malformed)?;
    let kind_code = cur.read_i32().ok_or(MessageError::Malformed)?;
    let critical_flags = cur.read_i32().ok_or(MessageError::Malformed)?;
    let noncritical_flags = cur.read_i32().ok_or(MessageError::Malformed)?;

    let ciphertext_len = cur.read_u32().ok_or(MessageError::Malformed)? as usize;
    let ciphertext = cur
        .take(ciphertext_len)
        .ok_or(MessageError::Malformed)?
        .to_vec();

    if !cur.finished() {
        return Err(MessageError::Malformed);
    }

    let suite = suite_from_code(suite_code).ok_or(MessageError::UnsupportedSuite)?;
    let kind = kind_from_code(kind_code).ok_or(MessageError::UnsupportedKind)?;
    let payload_len = i32::try_from(ciphertext_len).map_err(|_| MessageError::Malformed)?;

    Ok(SealedMessage {
        envelope: ClientMessageEnvelope {
            version,
            suite,
            conversation_id_len,
            sender_account_id_len,
            sender_device_id_len,
            epoch,
            sequence,
            kind,
            payload_len,
            critical_flags,
            noncritical_flags,
        },
        ciphertext,
    })
}

#[cfg(test)]
mod hybrid_message_tests {
    use super::*;

    fn envelope(suite: ProtocolSuite) -> ClientMessageEnvelope {
        ClientMessageEnvelope {
            version: 1,
            suite,
            conversation_id_len: 32,
            sender_account_id_len: 32,
            sender_device_id_len: 32,
            epoch: 7,
            sequence: 42,
            kind: MessageKind::Application,
            payload_len: 0,
            critical_flags: 0,
            noncritical_flags: 0,
        }
    }

    fn context() -> ConversationPolicyState {
        ConversationPolicyState {
            expected_epoch: 7,
            expected_sequence: 42,
            minimum_suite: ProtocolSuite::ClassicalDev,
            max_payload_len: 8192,
        }
    }

    #[test]
    fn hybrid_seal_open_round_trips() {
        let x = DeviceKeyPair::generate();
        let pq = MlKemKeyPair::generate();
        let pt = b"post-quantum message object";
        let sealed = seal_message_hybrid(
            &x.public(),
            &pq.public(),
            pt,
            envelope(ProtocolSuite::HybridPqDev),
            &context(),
        )
        .unwrap();
        // payload_len reflects the PADDED plaintext (bucketed) plus the hybrid overhead.
        assert_eq!(
            sealed.envelope.payload_len as usize,
            padded_len(pt.len()).unwrap() + HYBRID_SEALED_OVERHEAD
        );
        assert_eq!(
            open_message_hybrid(&x, &pq, &sealed, &context()).unwrap(),
            pt
        );
    }

    #[test]
    fn hybrid_message_survives_the_wire() {
        let x = DeviceKeyPair::generate();
        let pq = MlKemKeyPair::generate();
        let pt = b"hybrid over the wire";
        let sealed = seal_message_hybrid(
            &x.public(),
            &pq.public(),
            pt,
            envelope(ProtocolSuite::HybridPqDev),
            &context(),
        )
        .unwrap();
        let parsed = from_bytes(&to_bytes(&sealed)).unwrap();
        assert_eq!(parsed, sealed);
        assert_eq!(
            open_message_hybrid(&x, &pq, &parsed, &context()).unwrap(),
            pt
        );
    }

    #[test]
    fn a_tampered_envelope_field_fails_to_open_classical() {
        // An attacker holding a valid SealedMessage flips a policy-VALID envelope field — kind
        // Application -> Control — and re-serializes. The envelope header is bound into the AEAD AAD,
        // so open fails closed instead of delivering the message tagged as a control frame.
        let x = DeviceKeyPair::generate();
        let pt = b"application payload";
        let sealed =
            seal_message(&x.public(), pt, envelope(ProtocolSuite::ClassicalDev), &context()).unwrap();
        assert_eq!(sealed.envelope.kind, MessageKind::Application);
        // The honest round-trip still opens.
        assert_eq!(open_message(&x, &sealed, &context()).unwrap(), pt);

        // Tamper: a different, still-policy-valid kind (the gate does not context-constrain kind), so
        // ONLY the AEAD binding catches the swap.
        let mut tampered = sealed.clone();
        tampered.envelope.kind = MessageKind::Control;
        assert!(
            matches!(open_message(&x, &tampered, &context()), Err(MessageError::Open(_))),
            "a flipped envelope field is rejected by the bound AEAD"
        );
    }

    #[test]
    fn a_tampered_envelope_field_fails_to_open_hybrid() {
        let x = DeviceKeyPair::generate();
        let pq = MlKemKeyPair::generate();
        let pt = b"pq application payload";
        let sealed = seal_message_hybrid(
            &x.public(),
            &pq.public(),
            pt,
            envelope(ProtocolSuite::HybridPqDev),
            &context(),
        )
        .unwrap();
        assert_eq!(open_message_hybrid(&x, &pq, &sealed, &context()).unwrap(), pt);

        let mut tampered = sealed.clone();
        tampered.envelope.kind = MessageKind::Control;
        assert!(
            matches!(
                open_message_hybrid(&x, &pq, &tampered, &context()),
                Err(MessageError::Open(_))
            ),
            "a flipped envelope field is rejected by the bound hybrid AEAD"
        );
    }

    #[test]
    fn a_tampered_gate_blind_flag_fails_to_open() {
        // `noncritical_flags` is entirely UNCONSTRAINED by the policy gate (it accepts any value), so
        // before the AEAD binding it was an even cleaner field-confusion vector than `kind`. Flipping
        // it must still fail to open — proving the binding covers the gate's BLIND SPOTS, not only the
        // fields the gate range-checks.
        let x = DeviceKeyPair::generate();
        let pt = b"gate-blind flag tamper";
        let sealed =
            seal_message(&x.public(), pt, envelope(ProtocolSuite::ClassicalDev), &context()).unwrap();
        let mut tampered = sealed.clone();
        tampered.envelope.noncritical_flags = 0x1234;
        assert!(
            matches!(open_message(&x, &tampered, &context()), Err(MessageError::Open(_))),
            "a flipped gate-blind flag is still rejected by the bound AEAD"
        );
    }

    #[test]
    fn classical_seal_rejects_a_pq_suite() {
        let x = DeviceKeyPair::generate();
        assert_eq!(
            seal_message(
                &x.public(),
                b"x",
                envelope(ProtocolSuite::HybridPqDev),
                &context()
            ),
            Err(MessageError::SuiteMismatch)
        );
    }

    #[test]
    fn hybrid_seal_rejects_a_classical_suite() {
        let x = DeviceKeyPair::generate();
        let pq = MlKemKeyPair::generate();
        assert_eq!(
            seal_message_hybrid(
                &x.public(),
                &pq.public(),
                b"x",
                envelope(ProtocolSuite::ClassicalDev),
                &context()
            ),
            Err(MessageError::SuiteMismatch)
        );
    }

    #[test]
    fn wrong_pq_key_fails_hybrid_open() {
        let x = DeviceKeyPair::generate();
        let pq = MlKemKeyPair::generate();
        let other_pq = MlKemKeyPair::generate();
        let sealed = seal_message_hybrid(
            &x.public(),
            &pq.public(),
            b"secret",
            envelope(ProtocolSuite::HybridPqDev),
            &context(),
        )
        .unwrap();
        assert_eq!(
            open_message_hybrid(&x, &other_pq, &sealed, &context()),
            Err(MessageError::Open(SealError::OpenFailed))
        );
    }

    #[test]
    fn auto_prefers_post_quantum_when_a_pq_key_exists() {
        // With an ML-KEM key present, seal_message_auto labels HybridPqDev and the
        // hybrid (post-quantum) crypto is what actually protects the message.
        let x = DeviceKeyPair::generate();
        let pq = MlKemKeyPair::generate();
        let pt = b"prefer post-quantum by default";
        let sealed = seal_message_auto(
            &x.public(),
            Some(&pq.public()),
            pt,
            envelope(ProtocolSuite::ClassicalDev), // caller-supplied label is overridden
            &context(),
        )
        .unwrap();
        // The label was set to match the crypto actually applied.
        assert!(matches!(sealed.envelope.suite, ProtocolSuite::HybridPqDev));
        // A purely-classical open MUST NOT succeed on a hybrid message.
        assert_eq!(
            open_message(&x, &sealed, &context()),
            Err(MessageError::SuiteMismatch)
        );
        // The auto + explicit-hybrid opens both recover the plaintext.
        assert_eq!(
            open_message_auto(&x, Some(&pq), &sealed, &context()).unwrap(),
            pt
        );
        assert_eq!(
            open_message_hybrid(&x, &pq, &sealed, &context()).unwrap(),
            pt
        );
    }

    #[test]
    fn auto_falls_back_to_classical_without_a_pq_key() {
        let x = DeviceKeyPair::generate();
        let pt = b"classical fallback";
        let sealed = seal_message_auto(
            &x.public(),
            None,
            pt,
            envelope(ProtocolSuite::HybridPqDev), // overridden down to classical, honestly
            &context(),
        )
        .unwrap();
        assert!(matches!(sealed.envelope.suite, ProtocolSuite::ClassicalDev));
        // open_message_auto with no pq key dispatches to the classical path.
        assert_eq!(
            open_message_auto(&x, None, &sealed, &context()).unwrap(),
            pt
        );
    }

    #[test]
    fn auto_open_of_a_hybrid_message_without_a_pq_key_fails_closed() {
        // A hybrid-labelled message presented to open_message_auto WITHOUT an ML-KEM
        // keypair must fail closed (never silently downgrade to a classical attempt).
        let x = DeviceKeyPair::generate();
        let pq = MlKemKeyPair::generate();
        let sealed = seal_message_auto(
            &x.public(),
            Some(&pq.public()),
            b"needs the pq key",
            envelope(ProtocolSuite::HybridPqDev),
            &context(),
        )
        .unwrap();
        assert_eq!(
            open_message_auto(&x, None, &sealed, &context()),
            Err(MessageError::SuiteMismatch)
        );
    }
}
