//! 1:1 Double-Ratchet session engine for Mercury (Phase 2).
//!
//! Wraps the audited [vodozemac](https://github.com/matrix-org/vodozemac) Olm
//! implementation (Matrix's Double Ratchet) and binds each session to a Mercury
//! identity via [`mercury_keys`]. Nothing here implements crypto by hand: the
//! ratchet, 3DH, and pickle encryption all come from vodozemac; the long-term
//! identity signature comes from `mercury-keys` (ed25519-dalek).
//!
//! A [`LocalSessionAccount`] holds the device's Olm [`Account`] and publishes a
//! [`SignedPreKeyBundle`] whose contents are signed by the Mercury identity key
//! and tied to the account id derived from that identity. Peers verify the
//! bundle before establishing a [`MercurySession`], so a session can only be
//! built against a key set that the claimed identity actually attested to.
//!
//! Fails closed: every fallible path returns [`SessionError`]. Session and
//! account state can be persisted only as an *encrypted* pickle. No secret key,
//! pickle, or plaintext is ever logged or exposed through `Debug` — the
//! secret-bearing wrappers redact themselves and [`SessionError`] carries no key
//! or plaintext bytes.

#![forbid(unsafe_code)]

use core::fmt;
use std::collections::BTreeMap;

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use mercury_keys::{
    IdentityKeyPair, IdentityPublicKey, KeyError, MlKemKeyPair, MlKemPublicKey,
    account_id_from_identity_key,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vodozemac::olm::{
    Account, EncryptionError, InboundCreationResult, OlmMessage, Session, SessionConfig,
    SessionPickle,
};
use vodozemac::{Curve25519PublicKey, PickleError};
use zeroize::Zeroize;

mod pq_ratchet;
pub use pq_ratchet::{
    HybridRatchetSession, PqRatchet, RATCHET_MAX_SKIP, RatchetHeader, RatchetMessage,
};

/// Domain separator for the signed pre-key bundle (v2 — adds the ML-KEM-768
/// post-quantum key to the signed contents). Bound into every signature so a
/// signature can never be replayed across protocols or versions. Bumped from v1
/// because the to-be-signed layout grew the PQ field.
const BUNDLE_DOMAIN: &[u8] = b"mercury-prekey-bundle-v2";

/// ML-KEM-768 public (encapsulation) key length (FIPS 203), as serialized by
/// `mercury_keys::MlKemPublicKey::to_bytes`. Used to fix the bundle's PQ-key
/// length so a presented bundle with a wrong-length PQ key is rejected before any
/// encapsulation.
const ML_KEM_PK_LEN: usize = 1184;

/// Domain separator for the hybrid first-flight outer AEAD key derivation, kept
/// distinct from the bundle-signing domain so the PQ shared secret derives a key
/// only for this purpose.
const FIRST_FLIGHT_DOMAIN: &[u8] = b"mercury-session-pq-first-flight-v1";
/// XChaCha20-Poly1305 nonce length (24 bytes).
const NONCE_LEN: usize = 24;
/// Poly1305 authentication tag length (16 bytes).
const TAG_LEN: usize = 16;
/// ML-KEM-768 ciphertext length (FIPS 203), framed at a fixed offset in the hybrid
/// first-flight wire form.
const MLKEM_CT_LEN: usize = 1088;
/// Wire-format version byte for the hybrid first-flight and continuous-message codecs.
/// Bumped if either wire layout changes, so an old/foreign frame is rejected (fail-closed)
/// rather than silently misparsed.
const HYBRID_WIRE_V1: u8 = 1;

/// HKDF info for seeding the CONTINUOUS post-quantum chain from the bootstrap
/// ML-KEM secret. The two role labels make the initiator's send chain equal the
/// responder's receive chain (and vice versa) without the two ends colliding.
const PQ_CHAIN_SEED_DOMAIN: &[u8] = b"mercury-session-pq-chain-seed-v1";
/// Role label: the chain the session INITIATOR sends on / the RESPONDER receives on.
const PQ_CHAIN_LABEL_I2R: &[u8] = b"i2r";
/// Role label: the chain the session RESPONDER sends on / the INITIATOR receives on.
const PQ_CHAIN_LABEL_R2I: &[u8] = b"r2i";
/// HKDF info to derive a per-message KEY from a chain key (forward-secret ratchet).
const PQ_CHAIN_MSG_INFO: &[u8] = b"mercury-session-pq-chain-msg-v1";
/// HKDF info to STEP a chain key to its successor (the old key is then zeroized).
const PQ_CHAIN_STEP_INFO: &[u8] = b"mercury-session-pq-chain-step-v1";
/// Associated-data domain for the continuous per-message outer AEAD.
const PQ_CHAIN_AAD_DOMAIN: &[u8] = b"mercury-session-pq-chain-aad-v1";
/// Max number of skipped (out-of-order ahead) message keys retained per receive
/// chain. Bounds memory + work a peer can force; a gap larger than this is rejected
/// (in [`PqRecvChain::prepare`]) BEFORE any key derivation, so a forged far-ahead
/// index cannot force an unbounded forward walk. Public so the protocol parameter is
/// documented and exactly testable.
pub const PQ_MAX_SKIP: u32 = 1024;

/// Errors surfaced when publishing/verifying bundles, establishing sessions,
/// encrypting/decrypting, or (un)pickling.
///
/// Deliberately coarse and free of any key or plaintext material so that
/// `Debug`/`Display` are always safe to log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    /// The bundle's identity signature failed verification.
    BundleSignature,
    /// The bundle's `account_id` does not match the one derived from its
    /// identity key.
    AccountIdMismatch,
    /// A public key in the bundle or message was not a valid Curve25519 or
    /// Ed25519 key.
    InvalidKey,
    /// vodozemac could not establish a session from the supplied keys/message.
    SessionCreation,
    /// An established session is not Olm v2. vodozemac derives the Olm version
    /// from the incoming pre-key message, so a V1 (truncated-8-byte-MAC) message
    /// would otherwise create a V1 session even on a V2-configured account — a
    /// silent V2->V1 downgrade. We reject any non-V2 session before returning it.
    VersionDowngrade,
    /// Encryption failed (a non-contributory Diffie-Hellman shared secret).
    /// Should be unreachable for an established local ratchet, but surfaced
    /// rather than swallowed so the channel fails closed.
    Encryption,
    /// Decryption failed: tampering, replay, wrong session, or a corrupt
    /// ciphertext envelope.
    Decryption,
    /// An encrypted pickle could not be produced or restored (wrong key, or
    /// corrupt/invalid blob).
    Pickle,
    /// `establish_inbound` was given a ciphertext that is not a pre-key message.
    NotPreKeyMessage,
    /// The hybrid post-quantum first-flight failed: ML-KEM decapsulation, the
    /// outer AEAD over the bootstrap message, or its framing did not verify.
    /// Fails closed — the inner Olm handshake is never attempted on a first
    /// flight whose post-quantum layer did not authenticate.
    FirstFlightFailed,
    /// A continuous post-quantum chain message failed: the outer AEAD did not
    /// authenticate, the chain index was a replay or too far ahead of the
    /// receive chain (beyond the skip bound), or the message-index counter
    /// overflowed. Fails closed — the inner Olm message is never decrypted on a
    /// continuous message whose post-quantum layer did not authenticate.
    HybridChainFailed,
    /// A post-quantum ratchet ([`PqRatchet`] / [`HybridRatchetSession`]) message
    /// failed: the outer AEAD did not authenticate, ML-KEM decapsulation of the
    /// ratchet header produced an unrelated secret (tampered/foreign ciphertext or
    /// public key), the message index was a replay or beyond the skip bound, a
    /// counter overflowed, or a send was attempted before the peer's ratchet key is
    /// known. Fails closed — no state is advanced and no inner message is decrypted
    /// on a ratchet message that did not authenticate.
    RatchetFailed,
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionError::BundleSignature => f.write_str("pre-key bundle signature invalid"),
            SessionError::AccountIdMismatch => {
                f.write_str("pre-key bundle account id does not match identity key")
            }
            SessionError::InvalidKey => f.write_str("invalid public key in bundle or message"),
            SessionError::SessionCreation => f.write_str("session establishment failed"),
            SessionError::VersionDowngrade => {
                f.write_str("established session is not Olm v2 (downgrade rejected)")
            }
            SessionError::Encryption => f.write_str("session encryption failed"),
            SessionError::Decryption => f.write_str("session decryption failed"),
            SessionError::Pickle => f.write_str("session/account pickle invalid"),
            SessionError::NotPreKeyMessage => f.write_str("ciphertext is not a pre-key message"),
            SessionError::FirstFlightFailed => {
                f.write_str("post-quantum hybrid first-flight failed")
            }
            SessionError::HybridChainFailed => {
                f.write_str("post-quantum hybrid chain message failed")
            }
            SessionError::RatchetFailed => f.write_str("post-quantum ratchet message failed"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<KeyError> for SessionError {
    fn from(_: KeyError) -> Self {
        // KeyError only arises from untrusted public-key bytes or a failed
        // signature check; both collapse to a closed-door rejection. The exact
        // variant is intentionally dropped so no key material leaks.
        SessionError::InvalidKey
    }
}

impl From<PickleError> for SessionError {
    fn from(_: PickleError) -> Self {
        SessionError::Pickle
    }
}

impl From<EncryptionError> for SessionError {
    fn from(_: EncryptionError) -> Self {
        // The only variant is `NonContributoryKey`; collapse it to a closed-door
        // rejection that carries no key material.
        SessionError::Encryption
    }
}

/// Map a vodozemac Curve25519 key to raw 32 bytes.
fn curve_to_bytes(key: &Curve25519PublicKey) -> [u8; 32] {
    key.to_bytes()
}

/// Parse raw 32 bytes as a vodozemac Curve25519 key (every 32-byte string is a
/// syntactically valid Montgomery-u value, so this is infallible).
fn curve_from_bytes(bytes: [u8; 32]) -> Curve25519PublicKey {
    Curve25519PublicKey::from_bytes(bytes)
}

/// The local device's Olm [`Account`]: long-term Curve25519/Ed25519 identity
/// keys plus one-time keys used to bootstrap sessions.
///
/// Secret-bearing; `Debug` is intentionally not derived to avoid leaking key
/// material through the wrapped [`Account`].
pub struct LocalSessionAccount {
    account: Account,
}

impl LocalSessionAccount {
    /// Create a new account with fresh random Olm identity keys.
    pub fn new() -> Self {
        Self {
            account: Account::new(),
        }
    }

    /// The vodozemac identity (sender) Curve25519 key, raw bytes.
    pub fn curve25519_key(&self) -> [u8; 32] {
        curve_to_bytes(&self.account.curve25519_key())
    }

    /// Generate `one_time_keys` one-time keys, pick one for a bundle, sign the
    /// canonical to-be-signed bytes with the Mercury `identity`, mark the keys
    /// as published, and return the [`SignedPreKeyBundle`].
    ///
    /// The bundle binds three things together: the vodozemac account keys
    /// (curve25519 + ed25519 + the chosen one-time key), the Mercury identity
    /// public key, and the account id derived from that identity. The signature
    /// is over all of them, so a peer that verifies the bundle knows the
    /// identity attested to exactly this key set.
    ///
    /// One-time-key accounting is atomic: this method is the ONLY one that
    /// generates one-time keys, it selects the bundle's key from exactly the set
    /// generated in THIS call (vodozemac returns the freshly created keys), and
    /// it marks keys as published only after the bundle is built. So the set
    /// marked published equals the set just generated — no key is ever marked
    /// published without being placed in a bundle, and no stale unpublished key
    /// is mismarked. `one_time_keys == 0` is normalized to 1 so the bundle is
    /// always usable.
    pub fn publish_bundle(
        &mut self,
        identity: &IdentityKeyPair,
        pq_public: &MlKemPublicKey,
        one_time_keys: usize,
    ) -> SignedPreKeyBundle {
        // Generate exactly the keys to publish in this call. Treat a request of
        // zero as one so the bundle always carries a usable one-time key.
        let count = one_time_keys.max(1);
        let created = self.account.generate_one_time_keys(count).created;

        // Pick a deterministic one-time key from the freshly generated set: the
        // smallest by its raw Curve25519 bytes. Drawing only from `created`
        // guarantees the chosen key belongs to this call's generation, so the
        // key we publish is part of the set we mark as published below. The set
        // is non-empty because `count >= 1`.
        let one_time_key = created
            .into_iter()
            .min_by_key(|key| key.to_bytes())
            .expect("generate_one_time_keys(count >= 1) yields at least one key");

        let identity_pub = identity.public();
        let account_id = account_id_from_identity_key(&identity_pub);
        let curve25519 = curve_to_bytes(&self.account.curve25519_key());
        let ed25519 = *self.account.ed25519_key().as_bytes();
        let one_time_key_bytes = curve_to_bytes(&one_time_key);
        // No fallback key in this bundle; the canonical encoding distinguishes
        // the None case with a leading 0u8.
        let fallback: Option<[u8; 32]> = None;
        let pq_kem = pq_public.to_bytes();

        let tbs = bundle_tbs(
            &account_id,
            &curve25519,
            &ed25519,
            &one_time_key_bytes,
            &fallback,
            &pq_kem,
        );
        let signature = identity.sign(&tbs);

        self.account.mark_keys_as_published();

        SignedPreKeyBundle {
            account_id,
            identity_ed25519: *identity_pub.as_bytes(),
            curve25519,
            ed25519,
            one_time_key: one_time_key_bytes,
            fallback,
            pq_kem,
            signature,
        }
    }

    /// Serialize and encrypt this account into a pickle string under `key`.
    pub fn to_encrypted_pickle(&self, key: &[u8; 32]) -> String {
        self.account.pickle().encrypt(key)
    }

    /// Decrypt and restore an account from a pickle produced by
    /// [`LocalSessionAccount::to_encrypted_pickle`]. Wrong key or corrupt blob
    /// yields [`SessionError::Pickle`].
    pub fn from_encrypted_pickle(blob: &str, key: &[u8; 32]) -> Result<Self, SessionError> {
        let pickle = vodozemac::olm::AccountPickle::from_encrypted(blob, key)?;
        Ok(Self {
            account: Account::from_pickle(pickle),
        })
    }
}

impl Default for LocalSessionAccount {
    fn default() -> Self {
        Self::new()
    }
}

/// A pre-key bundle signed by a Mercury identity key.
///
/// Carries only public material, so `Debug`/`Clone` are safe. The `signature`
/// covers the canonical [`bundle_tbs`] encoding of the public fields, binding
/// the vodozemac keys to the Mercury identity and account id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedPreKeyBundle {
    /// Account id derived from the Mercury identity key.
    pub account_id: [u8; 32],
    /// Mercury identity Ed25519 public key (the signer).
    pub identity_ed25519: [u8; 32],
    /// vodozemac identity (sender) Curve25519 key.
    pub curve25519: [u8; 32],
    /// vodozemac account Ed25519 fingerprint key.
    pub ed25519: [u8; 32],
    /// The one-time Curve25519 key offered for session bootstrap.
    pub one_time_key: [u8; 32],
    /// Optional fallback Curve25519 key.
    pub fallback: Option<[u8; 32]>,
    /// The ML-KEM-768 (FIPS 203) post-quantum encapsulation key, canonical bytes.
    /// Signed under the same identity key as the rest of the bundle, so it is
    /// authenticated: an initiator encapsulates to it to add a post-quantum shared
    /// secret to the session bootstrap (see [`MercurySession::establish_outbound_hybrid`]).
    pub pq_kem: Vec<u8>,
    /// Ed25519 signature over [`bundle_tbs`] under `identity_ed25519`.
    #[serde(with = "sig_serde")]
    pub signature: [u8; 64],
}

/// Serde shim for the 64-byte signature: stock serde array impls stop at
/// length 32, so we (de)serialize the signature as a byte sequence and
/// re-validate its length on the way in. Fails closed on any other length.
mod sig_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(sig: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error> {
        sig.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 64], D::Error> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("signature must be exactly 64 bytes"))
    }
}

impl SignedPreKeyBundle {
    /// Verify the bundle's identity signature AND that its `account_id` matches
    /// the one derived from the identity key. Fails closed on any mismatch.
    pub fn verify(&self) -> Result<(), SessionError> {
        let identity_pub = IdentityPublicKey::from_bytes(self.identity_ed25519)?;

        // Reject a wrong-length PQ key before anything else binds it, so a malformed
        // bundle can never reach an encapsulation path.
        if self.pq_kem.len() != ML_KEM_PK_LEN {
            return Err(SessionError::InvalidKey);
        }

        let tbs = bundle_tbs(
            &self.account_id,
            &self.curve25519,
            &self.ed25519,
            &self.one_time_key,
            &self.fallback,
            &self.pq_kem,
        );
        identity_pub
            .verify(&tbs, &self.signature)
            .map_err(|_| SessionError::BundleSignature)?;

        if self.account_id != account_id_from_identity_key(&identity_pub) {
            return Err(SessionError::AccountIdMismatch);
        }

        Ok(())
    }

    /// The bundle's verified ML-KEM-768 post-quantum public key. Call only after
    /// [`SignedPreKeyBundle::verify`] has succeeded (which authenticates the key and
    /// checks its length).
    pub fn pq_public(&self) -> Result<MlKemPublicKey, SessionError> {
        MlKemPublicKey::from_bytes(&self.pq_kem).map_err(|_| SessionError::InvalidKey)
    }
}

/// Build the canonical to-be-signed bytes for a [`SignedPreKeyBundle`].
///
/// Layout (identical in sign and verify):
/// `BUNDLE_DOMAIN || account_id(32) || curve25519(32) || ed25519(32) ||
/// one_time_key(32) || (1u8 || fallback(32)) if Some else (0u8) ||
/// pq_kem_len(u32 LE) || pq_kem`.
///
/// All fixed-width fields appear in this exact order with no length prefixes
/// except the optional fallback (presence byte) and the trailing ML-KEM key, which
/// carries a u32 length prefix so the variable-length PQ key cannot be confused
/// with whatever follows. Including the PQ key here means the identity signature
/// AUTHENTICATES it — a tampered PQ key breaks the signature.
fn bundle_tbs(
    account_id: &[u8; 32],
    curve25519: &[u8; 32],
    ed25519: &[u8; 32],
    one_time_key: &[u8; 32],
    fallback: &Option<[u8; 32]>,
    pq_kem: &[u8],
) -> Vec<u8> {
    let mut tbs = Vec::with_capacity(BUNDLE_DOMAIN.len() + 32 * 4 + 1 + 32 + 4 + pq_kem.len());
    tbs.extend_from_slice(BUNDLE_DOMAIN);
    tbs.extend_from_slice(account_id);
    tbs.extend_from_slice(curve25519);
    tbs.extend_from_slice(ed25519);
    tbs.extend_from_slice(one_time_key);
    match fallback {
        Some(fb) => {
            tbs.push(1u8);
            tbs.extend_from_slice(fb);
        }
        None => tbs.push(0u8),
    }
    tbs.extend_from_slice(&(pq_kem.len() as u32).to_le_bytes());
    tbs.extend_from_slice(pq_kem);
    tbs
}

/// A post-quantum hybrid first flight: the wire payload an initiator sends to
/// bootstrap a session with [`MercurySession::establish_outbound_hybrid`]. Carries
/// the ML-KEM-768 `mlkem_ciphertext` (the recipient decapsulates it) and the
/// `outer` post-quantum AEAD layer (`nonce(24) || sealed`) wrapping the inner Olm
/// pre-key message. All bytes are ciphertext/public, so `Debug`/`Clone` are safe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridFirstFlight {
    /// The ML-KEM-768 ciphertext (the recipient decapsulates this with its keypair).
    pub mlkem_ciphertext: Vec<u8>,
    /// The outer XChaCha20-Poly1305 layer over the inner Olm message: `nonce(24) ||
    /// ciphertext_and_tag`.
    pub outer: Vec<u8>,
}

impl HybridFirstFlight {
    /// Canonical wire bytes: `version(1) || mlkem_ciphertext(1088) || outer`. The ML-KEM
    /// ciphertext is fixed-length, so the layout is unambiguous and `outer` is the rest.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.mlkem_ciphertext.len() + self.outer.len());
        out.push(HYBRID_WIRE_V1);
        out.extend_from_slice(&self.mlkem_ciphertext);
        out.extend_from_slice(&self.outer);
        out
    }

    /// Parse wire bytes produced by [`HybridFirstFlight::to_bytes`]. Never panics; fails
    /// closed ([`SessionError::FirstFlightFailed`]) on a wrong version byte or a frame too
    /// short to hold the version, the full ML-KEM ciphertext, and a minimal outer layer.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SessionError> {
        let min = 1 + MLKEM_CT_LEN + NONCE_LEN + TAG_LEN;
        if bytes.len() < min || bytes[0] != HYBRID_WIRE_V1 {
            return Err(SessionError::FirstFlightFailed);
        }
        Ok(HybridFirstFlight {
            mlkem_ciphertext: bytes[1..1 + MLKEM_CT_LEN].to_vec(),
            outer: bytes[1 + MLKEM_CT_LEN..].to_vec(),
        })
    }
}

/// Derive the 32-byte outer-AEAD key from the ML-KEM shared secret, salted by the
/// ML-KEM ciphertext (transcript binding) and domain-separated. The caller zeroizes
/// `pq_secret` and the returned key.
fn derive_first_flight_key(pq_secret: &[u8; 32], mlkem_ciphertext: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(Some(mlkem_ciphertext), pq_secret);
    let mut key = [0u8; 32];
    hkdf.expand(FIRST_FLIGHT_DOMAIN, &mut key)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    key
}

/// The outer-AEAD associated data: the domain string and the ML-KEM ciphertext, so
/// the outer layer is bound to exactly this encapsulation and cannot be transplanted.
fn first_flight_aad(mlkem_ciphertext: &[u8]) -> Vec<u8> {
    let mut aad = Vec::with_capacity(FIRST_FLIGHT_DOMAIN.len() + mlkem_ciphertext.len());
    aad.extend_from_slice(FIRST_FLIGHT_DOMAIN);
    aad.extend_from_slice(mlkem_ciphertext);
    aad
}

/// One end of a 1:1 Olm Double-Ratchet channel.
///
/// Secret-bearing; `Debug` is intentionally not derived to avoid leaking
/// ratchet state through the wrapped [`Session`].
pub struct MercurySession {
    session: Session,
}

impl MercurySession {
    /// Establish the *outbound* side of a session toward `peer`.
    ///
    /// The peer bundle is verified FIRST (signature + account-id binding); only
    /// then is the vodozemac outbound session created (Olm v2). This guarantees
    /// we never run 3DH against keys the claimed identity did not attest to.
    pub fn establish_outbound(
        account: &LocalSessionAccount,
        peer: &SignedPreKeyBundle,
    ) -> Result<MercurySession, SessionError> {
        peer.verify()?;

        let session = account
            .account
            .create_outbound_session(
                SessionConfig::version_2(),
                curve_from_bytes(peer.curve25519),
                curve_from_bytes(peer.one_time_key),
            )
            .map_err(|_| SessionError::SessionCreation)?;

        // Defensive: we passed `version_2()`, so this should always hold. Assert
        // it anyway so a non-V2 (truncated-MAC) session can never leave here.
        if session.session_config() != SessionConfig::version_2() {
            return Err(SessionError::VersionDowngrade);
        }

        Ok(MercurySession { session })
    }

    /// Establish the *outbound* side of a session toward `peer` with a POST-QUANTUM
    /// hybrid first flight.
    ///
    /// This is the harvest-now-decrypt-later defense for session bootstrap. On top
    /// of the classical Olm handshake it: (1) verifies the bundle (which
    /// authenticates the peer's ML-KEM-768 key under its identity signature), (2)
    /// ENCAPSULATES to that key, producing an ML-KEM ciphertext + a post-quantum
    /// shared secret, and (3) wraps the FIRST Olm message in an OUTER
    /// XChaCha20-Poly1305 layer keyed by `HKDF(pq_secret)` (transcript-bound to the
    /// ML-KEM ciphertext). So the bootstrap message stays confidential as long as
    /// EITHER the classical Olm handshake OR ML-KEM-768 holds.
    ///
    /// Returns the established session and a [`HybridFirstFlight`] to send to the
    /// peer (it carries the ML-KEM ciphertext + the outer-wrapped Olm message). The
    /// inner Olm Double Ratchet is unchanged and remains classical — this augments
    /// only the FIRST flight, NOT the ongoing ratchet (a full post-quantum ratchet
    /// would require a PQXDH-style vodozemac upgrade).
    pub fn establish_outbound_hybrid(
        account: &LocalSessionAccount,
        peer: &SignedPreKeyBundle,
        first_message: &[u8],
    ) -> Result<(MercurySession, HybridFirstFlight), SessionError> {
        // Verify the bundle BEFORE extracting / encapsulating to its ML-KEM key, so
        // we never run post-quantum crypto against an unauthenticated (attacker-
        // swappable) key. `establish_outbound` below verifies again as its own
        // precondition — that second check is intentional defence-in-depth, not an
        // oversight: do not remove this one to "deduplicate", or encapsulation would
        // move ahead of authentication.
        peer.verify()?;
        let pq_public = peer.pq_public()?;

        // Real ML-KEM-768 encapsulation -> (ciphertext, post-quantum shared secret).
        let (mlkem_ciphertext, mut pq_secret) = pq_public.encapsulate();

        // Establish the classical Olm outbound session and produce the first Olm
        // message (a pre-key message). This re-verifies the bundle (see above).
        let mut session = Self::establish_outbound(account, peer)?;
        let inner = session.encrypt(first_message)?;
        let inner_bytes = inner.to_bytes();

        // Outer post-quantum AEAD over the inner Olm message. The AAD binds the
        // ML-KEM ciphertext (the transcript both sides share), so the outer layer
        // cannot be transplanted onto a different encapsulation.
        let mut key = derive_first_flight_key(&pq_secret, &mlkem_ciphertext);
        pq_secret.zeroize();
        let aad = first_flight_aad(&mlkem_ciphertext);
        let cipher = XChaCha20Poly1305::new_from_slice(&key)
            .expect("32-byte key is valid for XChaCha20Poly1305");
        key.zeroize();
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let wrapped = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: &inner_bytes,
                    aad: &aad,
                },
            )
            .map_err(|_| SessionError::FirstFlightFailed)?;

        let mut outer = Vec::with_capacity(NONCE_LEN + wrapped.len());
        outer.extend_from_slice(&nonce);
        outer.extend_from_slice(&wrapped);

        Ok((
            session,
            HybridFirstFlight {
                mlkem_ciphertext,
                outer,
            },
        ))
    }

    /// Establish the *inbound* side from a POST-QUANTUM hybrid first flight produced
    /// by [`MercurySession::establish_outbound_hybrid`].
    ///
    /// DECAPSULATES the ML-KEM ciphertext with `account`'s ML-KEM keypair,
    /// re-derives the outer AEAD key, UNWRAPS the outer post-quantum layer (failing
    /// closed if it does not authenticate — so the inner Olm handshake is never run
    /// on a first flight whose PQ layer was tampered with or made under the wrong
    /// key), then runs the classical inbound establishment on the recovered inner
    /// Olm pre-key message. Returns the session and the recovered first plaintext.
    pub fn establish_inbound_hybrid(
        account: &mut LocalSessionAccount,
        pq_keypair: &MlKemKeyPair,
        their_curve25519: [u8; 32],
        flight: &HybridFirstFlight,
    ) -> Result<(MercurySession, Vec<u8>), SessionError> {
        // Real ML-KEM-768 decapsulation -> the same post-quantum shared secret. A
        // wrong ML-KEM key or a tampered ciphertext yields a different secret (via
        // implicit rejection), which then fails the outer AEAD below.
        let mut pq_secret = pq_keypair
            .decapsulate(&flight.mlkem_ciphertext)
            .map_err(|_| SessionError::FirstFlightFailed)?;

        if flight.outer.len() < NONCE_LEN + TAG_LEN {
            pq_secret.zeroize();
            return Err(SessionError::FirstFlightFailed);
        }
        let (nonce_bytes, wrapped) = flight.outer.split_at(NONCE_LEN);

        let mut key = derive_first_flight_key(&pq_secret, &flight.mlkem_ciphertext);
        pq_secret.zeroize();
        let aad = first_flight_aad(&flight.mlkem_ciphertext);
        let cipher = XChaCha20Poly1305::new_from_slice(&key)
            .expect("32-byte key is valid for XChaCha20Poly1305");
        key.zeroize();
        let nonce = XNonce::from_slice(nonce_bytes);
        let inner_bytes = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: wrapped,
                    aad: &aad,
                },
            )
            .map_err(|_| SessionError::FirstFlightFailed)?;

        // The outer PQ layer authenticated; now run the classical inbound handshake
        // on the recovered inner Olm pre-key message.
        let inner = SessionCiphertext::from_bytes(&inner_bytes)?;
        Self::establish_inbound(account, their_curve25519, &inner)
    }

    /// Establish the *inbound* side of a session from a pre-key ciphertext.
    ///
    /// Only valid when `ct` is a pre-key message; otherwise returns
    /// [`SessionError::NotPreKeyMessage`]. Returns the new session and the
    /// recovered plaintext of the bootstrap message. Consumes one of our
    /// one-time keys (vodozemac removes it on success).
    pub fn establish_inbound(
        account: &mut LocalSessionAccount,
        their_curve25519: [u8; 32],
        ct: &SessionCiphertext,
    ) -> Result<(MercurySession, Vec<u8>), SessionError> {
        let message = ct.to_olm_message()?;
        let pre_key = match message {
            OlmMessage::PreKey(m) => m,
            OlmMessage::Normal(_) => return Err(SessionError::NotPreKeyMessage),
        };

        let InboundCreationResult { session, plaintext } = account
            .account
            .create_inbound_session(
                SessionConfig::version_2(),
                curve_from_bytes(their_curve25519),
                &pre_key,
            )
            .map_err(|_| SessionError::SessionCreation)?;

        // SECURITY: vodozemac derives the Olm version from the incoming pre-key
        // message, NOT from the `version_2()` config we passed. A V1
        // (truncated-8-byte-MAC) pre-key message therefore yields a V1 session
        // even on this V2-configured account — a silent V2->V1 downgrade. Reject
        // any non-V2 session before returning the session or its plaintext, so a
        // truncated-MAC inbound session is never handed to the caller.
        if session.session_config() != SessionConfig::version_2() {
            return Err(SessionError::VersionDowngrade);
        }

        Ok((MercurySession { session }, plaintext))
    }

    /// Encrypt `plaintext`, advancing the ratchet. The result is a pre-key
    /// message until the channel is fully established, then a normal message.
    ///
    /// Fails closed: vodozemac's `encrypt` only errors on a non-contributory DH
    /// shared secret, which should not occur for an already-established local
    /// ratchet, but the error is propagated as [`SessionError::Encryption`]
    /// rather than swallowed so a caller can never transmit an empty/bogus
    /// ciphertext in its place.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<SessionCiphertext, SessionError> {
        let message = self.session.encrypt(plaintext)?;
        Ok(SessionCiphertext::from_olm_message(&message))
    }

    /// Decrypt `ct`, advancing the ratchet. Fails closed on tampering, replay
    /// (vodozemac rejects a reused message key), wrong session, or a corrupt
    /// envelope.
    pub fn decrypt(&mut self, ct: &SessionCiphertext) -> Result<Vec<u8>, SessionError> {
        let message = ct.to_olm_message()?;
        self.session
            .decrypt(&message)
            .map_err(|_| SessionError::Decryption)
    }

    /// Whether this session uses Olm v2 (untruncated MAC). Always true for a
    /// session returned by [`MercurySession::establish_outbound`] or
    /// [`MercurySession::establish_inbound`], which reject any non-v2 session;
    /// exposed (the config is public protocol metadata, not a secret) so callers
    /// and tests can attest to the negotiated version.
    pub fn is_version_2(&self) -> bool {
        self.session.session_config() == SessionConfig::version_2()
    }

    /// Serialize and encrypt this session into a pickle string under `key`.
    pub fn to_encrypted_pickle(&self, key: &[u8; 32]) -> String {
        self.session.pickle().encrypt(key)
    }

    /// Decrypt and restore a session from a pickle produced by
    /// [`MercurySession::to_encrypted_pickle`]. Wrong key or corrupt blob yields
    /// [`SessionError::Pickle`].
    pub fn from_encrypted_pickle(blob: &str, key: &[u8; 32]) -> Result<Self, SessionError> {
        let pickle = SessionPickle::from_encrypted(blob, key)?;
        Ok(Self {
            session: Session::from_pickle(pickle),
        })
    }
}

// ===========================================================================
// Continuous post-quantum layer (HybridSession)
// ===========================================================================
//
// `establish_outbound_hybrid` / `establish_inbound_hybrid` above protect only the
// FIRST flight. `HybridSession` extends that protection to EVERY 1:1 message by
// running a forward-secret symmetric ratchet, seeded from the SAME bootstrap
// ML-KEM-768 shared secret, OUTSIDE the inner Olm session. Each message is wrapped
// in an outer XChaCha20-Poly1305 layer keyed by a per-message chain key, so a
// message stays confidential as long as EITHER the classical inner Olm ratchet OR
// ML-KEM-768 holds — for the whole conversation, not just the bootstrap.
//
// What this layer DOES provide:
//   * Post-quantum confidentiality for every message (hybrid with inner Olm).
//   * Forward secrecy on the post-quantum side: each chain key is derived one-way
//     and zeroized after use, so a compromise of current state does not expose
//     past messages' post-quantum keys.
//   * Replay rejection and bounded out-of-order delivery on the outer layer.
//
// What it does NOT provide (the honest residual): POST-COMPROMISE SECURITY on the
// post-quantum side. The chain is seeded once from the bootstrap encapsulation and
// never re-keyed with fresh post-quantum entropy, so an attacker who learns the
// current post-quantum chain state can derive all FUTURE outer keys. The inner Olm
// Double Ratchet keeps its classical forward secrecy AND post-compromise security
// (its Diffie-Hellman ratchet self-heals), so post-compromise the conversation
// recovers classical secrecy — but the post-quantum layer does not self-heal. A
// post-quantum layer WITH post-compromise security needs periodic fresh ML-KEM
// encapsulations woven into the ratchet (a PQXDH-style vodozemac fork), which this
// additive design deliberately does not attempt. This is the precise, scoped
// limitation — not a claim of a full post-quantum Double Ratchet.
//
// This layer is ADDITIVE: it does not modify the Olm session, the classical
// establishment paths, or the first-flight functions.

/// Derive a 32-byte value from a chain key via HKDF-SHA256 *expand* (the chain key
/// is used directly as the pseudo-random key, KDF-CK style). Used to derive both a
/// per-message key (`PQ_CHAIN_MSG_INFO`) and the successor chain key
/// (`PQ_CHAIN_STEP_INFO`) from the current chain key.
fn pq_chain_kdf(chain_key: &[u8; 32], info: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::from_prk(chain_key)
        .expect("a 32-byte chain key is a valid HKDF-SHA256 PRK");
    let mut out = [0u8; 32];
    hkdf.expand(info, &mut out)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    out
}

/// Seed one direction's chain key from the bootstrap ML-KEM shared secret. The
/// extract step is salted by the ML-KEM ciphertext (the transcript both ends
/// share) and the `info` carries a domain + role label, so the two directions get
/// independent chains and neither can be transplanted onto a different handshake.
fn seed_pq_chain(pq_secret: &[u8; 32], mlkem_ciphertext: &[u8], role: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(Some(mlkem_ciphertext), pq_secret);
    let mut info = Vec::with_capacity(PQ_CHAIN_SEED_DOMAIN.len() + role.len());
    info.extend_from_slice(PQ_CHAIN_SEED_DOMAIN);
    info.extend_from_slice(role);
    let mut chain_key = [0u8; 32];
    hkdf.expand(&info, &mut chain_key)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    chain_key
}

/// The transcript binder bound into every continuous-message AAD: SHA-256 of the
/// ML-KEM ciphertext. Public (a hash of public ciphertext), so it is not secret;
/// its job is to tie every outer layer to this specific encapsulation.
fn pq_transcript_binder(mlkem_ciphertext: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(mlkem_ciphertext);
    hasher.finalize().into()
}

/// Associated data for a continuous outer-layer message: domain || binder || index.
/// Binding the index means an outer layer captured at one position cannot be
/// replayed at another, and binding the transcript binder means it cannot be moved
/// to a different session.
fn pq_chain_aad(binder: &[u8; 32], index: u32) -> Vec<u8> {
    let mut aad = Vec::with_capacity(PQ_CHAIN_AAD_DOMAIN.len() + 32 + 4);
    aad.extend_from_slice(PQ_CHAIN_AAD_DOMAIN);
    aad.extend_from_slice(binder);
    aad.extend_from_slice(&index.to_le_bytes());
    aad
}

/// Wrap inner Olm message bytes in the outer post-quantum AEAD under message key
/// `mk`, with a fresh random nonce and the index/transcript bound as AAD. Output is
/// `nonce(24) || ciphertext+tag`.
fn pq_chain_wrap(
    mk: &[u8; 32],
    binder: &[u8; 32],
    index: u32,
    inner_bytes: &[u8],
) -> Result<Vec<u8>, SessionError> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(mk).expect("32-byte key is valid for XChaCha20Poly1305");
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let aad = pq_chain_aad(binder, index);
    let wrapped = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: inner_bytes,
                aad: &aad,
            },
        )
        .map_err(|_| SessionError::HybridChainFailed)?;
    let mut outer = Vec::with_capacity(NONCE_LEN + wrapped.len());
    outer.extend_from_slice(&nonce);
    outer.extend_from_slice(&wrapped);
    Ok(outer)
}

/// Reverse of [`pq_chain_wrap`]: authenticate + decrypt the outer layer under `mk`.
/// Fails closed ([`SessionError::HybridChainFailed`]) on a short frame, a wrong key
/// (wrong index, wrong session, or desynced chain), or any tampering.
fn pq_chain_unwrap(
    mk: &[u8; 32],
    binder: &[u8; 32],
    index: u32,
    outer: &[u8],
) -> Result<Vec<u8>, SessionError> {
    if outer.len() < NONCE_LEN + TAG_LEN {
        return Err(SessionError::HybridChainFailed);
    }
    let (nonce_bytes, wrapped) = outer.split_at(NONCE_LEN);
    let cipher =
        XChaCha20Poly1305::new_from_slice(mk).expect("32-byte key is valid for XChaCha20Poly1305");
    let nonce = XNonce::from_slice(nonce_bytes);
    let aad = pq_chain_aad(binder, index);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: wrapped,
                aad: &aad,
            },
        )
        .map_err(|_| SessionError::HybridChainFailed)
}

/// A continuous post-quantum message: the outer-layer index and the wrapped bytes
/// (`nonce || ciphertext+tag`). The payload is ciphertext, so `Debug`/`Clone` are
/// safe; carry it as the opaque payload of a transport envelope alongside the inner
/// Olm message it protects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridMessage {
    /// The sender's monotonically increasing chain index for this message.
    pub index: u32,
    /// `nonce(24) || XChaCha20-Poly1305(inner Olm message bytes)`.
    pub outer: Vec<u8>,
}

impl HybridMessage {
    /// Canonical wire bytes: `version(1) || index(4 LE) || outer`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + 4 + self.outer.len());
        out.push(HYBRID_WIRE_V1);
        out.extend_from_slice(&self.index.to_le_bytes());
        out.extend_from_slice(&self.outer);
        out
    }

    /// Parse wire bytes produced by [`HybridMessage::to_bytes`]. Never panics; fails closed
    /// ([`SessionError::HybridChainFailed`]) on a wrong version byte or a frame too short to
    /// hold the version, the index, and a minimal outer layer.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SessionError> {
        let min = 1 + 4 + NONCE_LEN + TAG_LEN;
        if bytes.len() < min || bytes[0] != HYBRID_WIRE_V1 {
            return Err(SessionError::HybridChainFailed);
        }
        let index = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        Ok(HybridMessage {
            index,
            outer: bytes[5..].to_vec(),
        })
    }
}

/// One direction's forward-secret send chain. Each call to [`Self::next_key`]
/// yields the current message key and advances (then zeroizes) the chain key.
struct PqSendChain {
    chain_key: [u8; 32],
    index: u32,
}

impl PqSendChain {
    /// Derive the current message key and advance the chain. The old chain key is
    /// zeroized. Returns the message key and the index it is bound to. Fails closed
    /// if the index counter would overflow `u32` (≈4.3 billion messages — rotate the
    /// session long before then) rather than wrapping and reusing an index.
    fn next_key(&mut self) -> Result<([u8; 32], u32), SessionError> {
        let index = self.index;
        let next_index = index
            .checked_add(1)
            .ok_or(SessionError::HybridChainFailed)?;
        let mk = pq_chain_kdf(&self.chain_key, PQ_CHAIN_MSG_INFO);
        let next_chain_key = pq_chain_kdf(&self.chain_key, PQ_CHAIN_STEP_INFO);
        self.chain_key.zeroize();
        self.chain_key = next_chain_key;
        self.index = next_index;
        Ok((mk, index))
    }
}

impl Drop for PqSendChain {
    fn drop(&mut self) {
        self.chain_key.zeroize();
    }
}

/// A not-yet-committed receive-chain advance. The receive chain is only mutated by
/// [`PqRecvChain::commit`] AFTER the message authenticates, so a forged or tampered
/// message — which never authenticates — can neither desync the chain nor pollute
/// the skipped-key cache.
enum PqRecvStep {
    /// The message key came from the skipped-key cache at this index; committing
    /// removes (and zeroizes) it.
    Cached(u32),
    /// The message key required walking the live chain forward; committing installs
    /// the new chain key + index and the keys skipped over along the way.
    Forward {
        new_chain_key: [u8; 32],
        new_index: u32,
        newly_skipped: Vec<(u32, [u8; 32])>,
    },
}

impl Drop for PqRecvStep {
    fn drop(&mut self) {
        // Zeroize any pending secret material. After a successful `commit` the
        // Forward fields are already drained/zeroized, so this only does real work
        // on the discard (authentication-failed) path.
        if let PqRecvStep::Forward {
            new_chain_key,
            newly_skipped,
            ..
        } = self
        {
            new_chain_key.zeroize();
            for (_, key) in newly_skipped.iter_mut() {
                key.zeroize();
            }
        }
    }
}

/// One direction's forward-secret receive chain, with bounded out-of-order
/// tolerance. Keys for messages that arrive ahead of the chain are derived and
/// cached (up to [`PQ_MAX_SKIP`]); already-consumed indices not in the cache are
/// rejected as replays.
struct PqRecvChain {
    chain_key: [u8; 32],
    index: u32,
    skipped: BTreeMap<u32, [u8; 32]>,
}

impl PqRecvChain {
    /// Compute the message key for `target` WITHOUT mutating the chain, returning it
    /// alongside the pending [`PqRecvStep`] that [`Self::commit`] applies on success.
    ///
    /// Fails closed if `target` is an already-consumed index not retained in the
    /// skipped cache (replay) or is more than [`PQ_MAX_SKIP`] ahead of the chain
    /// (too large a gap — bounds the work a peer can force per message).
    fn prepare(&self, target: u32) -> Result<([u8; 32], PqRecvStep), SessionError> {
        if let Some(key) = self.skipped.get(&target) {
            return Ok((*key, PqRecvStep::Cached(target)));
        }
        if target < self.index {
            // Already consumed and not held as a skipped key -> replay.
            return Err(SessionError::HybridChainFailed);
        }
        let gap = target - self.index;
        if gap > PQ_MAX_SKIP {
            return Err(SessionError::HybridChainFailed);
        }
        // Compute the post-commit index BEFORE deriving any key material, so the
        // overflow path returns without leaving a derived message/chain key on the
        // stack unzeroized. (Unreachable for a genuine peer — `next_key` refuses to
        // emit `u32::MAX` — but this keeps forward secrecy airtight even for a
        // forged `target == u32::MAX`.)
        let new_index = target
            .checked_add(1)
            .ok_or(SessionError::HybridChainFailed)?;
        // Walk a LOCAL copy of the chain forward to `target`, collecting the keys we
        // skip over. `self` is untouched until `commit`.
        let mut chain_key = self.chain_key;
        let mut newly_skipped = Vec::new();
        let mut i = self.index;
        while i < target {
            let mk = pq_chain_kdf(&chain_key, PQ_CHAIN_MSG_INFO);
            let next = pq_chain_kdf(&chain_key, PQ_CHAIN_STEP_INFO);
            newly_skipped.push((i, mk));
            chain_key.zeroize();
            chain_key = next;
            i += 1;
        }
        let key = pq_chain_kdf(&chain_key, PQ_CHAIN_MSG_INFO);
        let new_chain_key = pq_chain_kdf(&chain_key, PQ_CHAIN_STEP_INFO);
        chain_key.zeroize();
        Ok((
            key,
            PqRecvStep::Forward {
                new_chain_key,
                new_index,
                newly_skipped,
            },
        ))
    }

    /// Apply a [`PqRecvStep`] produced by [`Self::prepare`] after the message has
    /// authenticated. Takes the step by `&mut` and zeroizes its copies as it moves
    /// them into place, so no secret outlives the call.
    fn commit(&mut self, mut step: PqRecvStep) {
        match &mut step {
            PqRecvStep::Cached(index) => {
                let idx = *index;
                if let Some(mut key) = self.skipped.remove(&idx) {
                    key.zeroize();
                }
            }
            PqRecvStep::Forward {
                new_chain_key,
                new_index,
                newly_skipped,
            } => {
                for (i, key) in newly_skipped.drain(..) {
                    if let Some(mut old) = self.skipped.insert(i, key) {
                        old.zeroize();
                    }
                }
                self.chain_key.zeroize();
                self.chain_key.copy_from_slice(new_chain_key);
                new_chain_key.zeroize();
                self.index = *new_index;
                // Bound the skipped cache: evict the oldest (smallest index) keys
                // until at most PQ_MAX_SKIP remain, zeroizing each evicted key.
                while self.skipped.len() as u32 > PQ_MAX_SKIP {
                    let oldest = match self.skipped.keys().next() {
                        Some(&k) => k,
                        None => break,
                    };
                    if let Some(mut key) = self.skipped.remove(&oldest) {
                        key.zeroize();
                    }
                }
            }
        }
    }
}

impl Drop for PqRecvChain {
    fn drop(&mut self) {
        self.chain_key.zeroize();
        for (_, key) in self.skipped.iter_mut() {
            key.zeroize();
        }
    }
}

/// A 1:1 session whose EVERY message carries continuous post-quantum protection.
///
/// Wraps an inner classical [`MercurySession`] (the Olm Double Ratchet) plus two
/// forward-secret symmetric chains — one per direction — seeded from the bootstrap
/// ML-KEM-768 shared secret. [`Self::initiate`] / [`Self::accept`] establish the
/// session and protect the first flight; [`Self::encrypt`] / [`Self::decrypt`]
/// protect every subsequent message. See the module-level note above for the exact
/// security properties and the honest residual (no post-compromise security on the
/// post-quantum side).
///
/// Secret-bearing; `Debug` is intentionally not derived.
pub struct HybridSession {
    session: MercurySession,
    send: PqSendChain,
    recv: PqRecvChain,
    binder: [u8; 32],
}

impl HybridSession {
    /// Establish the OUTBOUND side toward `peer` and protect `first_message` with
    /// continuous post-quantum protection.
    ///
    /// Verifies the bundle (authenticating the peer's ML-KEM key under its identity
    /// signature), encapsulates to it, seeds both directional chains from the shared
    /// secret, establishes the classical Olm outbound session, and wraps the first
    /// Olm message as chain index 0. Returns the live [`HybridSession`] and a
    /// [`HybridFirstFlight`] to send. Pair with [`HybridSession::accept`] on the peer
    /// (the continuous path is keyed by the chain, so it is NOT interchangeable with
    /// the first-flight-only [`MercurySession::establish_inbound_hybrid`]).
    pub fn initiate(
        account: &LocalSessionAccount,
        peer: &SignedPreKeyBundle,
        first_message: &[u8],
    ) -> Result<(HybridSession, HybridFirstFlight), SessionError> {
        // Authenticate BEFORE touching the peer's post-quantum key.
        peer.verify()?;
        let pq_public = peer.pq_public()?;
        let (mlkem_ciphertext, mut pq_secret) = pq_public.encapsulate();
        let binder = pq_transcript_binder(&mlkem_ciphertext);

        // Initiator sends on i2r, receives on r2i.
        let mut send = PqSendChain {
            chain_key: seed_pq_chain(&pq_secret, &mlkem_ciphertext, PQ_CHAIN_LABEL_I2R),
            index: 0,
        };
        let recv = PqRecvChain {
            chain_key: seed_pq_chain(&pq_secret, &mlkem_ciphertext, PQ_CHAIN_LABEL_R2I),
            index: 0,
            skipped: BTreeMap::new(),
        };
        pq_secret.zeroize();

        // Classical Olm outbound + the first Olm message (re-verifies the bundle).
        let mut session = MercurySession::establish_outbound(account, peer)?;
        let inner = session.encrypt(first_message)?;
        let inner_bytes = inner.to_bytes();

        // Wrap the first Olm message as outer chain index 0.
        let (mut mk, index) = send.next_key()?;
        let outer = pq_chain_wrap(&mk, &binder, index, &inner_bytes);
        mk.zeroize();
        let outer = outer?;

        Ok((
            HybridSession {
                session,
                send,
                recv,
                binder,
            },
            HybridFirstFlight {
                mlkem_ciphertext,
                outer,
            },
        ))
    }

    /// Establish the INBOUND side from a [`HybridFirstFlight`] produced by
    /// [`HybridSession::initiate`], recovering the first plaintext.
    ///
    /// Decapsulates the ML-KEM ciphertext, seeds both directional chains from the
    /// shared secret, unwraps the outer layer over the first Olm message (failing
    /// closed if it does not authenticate — so the inner Olm handshake is never run
    /// on a tampered first flight), and runs the classical inbound establishment.
    pub fn accept(
        account: &mut LocalSessionAccount,
        pq_keypair: &MlKemKeyPair,
        their_curve25519: [u8; 32],
        flight: &HybridFirstFlight,
    ) -> Result<(HybridSession, Vec<u8>), SessionError> {
        let mut pq_secret = pq_keypair
            .decapsulate(&flight.mlkem_ciphertext)
            .map_err(|_| SessionError::HybridChainFailed)?;
        let binder = pq_transcript_binder(&flight.mlkem_ciphertext);

        // Responder sends on r2i, receives on i2r (mirror of the initiator).
        let send = PqSendChain {
            chain_key: seed_pq_chain(&pq_secret, &flight.mlkem_ciphertext, PQ_CHAIN_LABEL_R2I),
            index: 0,
        };
        let mut recv = PqRecvChain {
            chain_key: seed_pq_chain(&pq_secret, &flight.mlkem_ciphertext, PQ_CHAIN_LABEL_I2R),
            index: 0,
            skipped: BTreeMap::new(),
        };
        pq_secret.zeroize();

        // Unwrap the first flight as receive chain index 0 (no commit until it and
        // the inner Olm handshake both succeed).
        let (mut mk, step) = recv.prepare(0)?;
        let inner_result = pq_chain_unwrap(&mk, &binder, 0, &flight.outer);
        mk.zeroize();
        let inner_bytes = inner_result?;
        let inner = SessionCiphertext::from_bytes(&inner_bytes)?;
        let (session, plaintext) =
            MercurySession::establish_inbound(account, their_curve25519, &inner)?;
        recv.commit(step);

        Ok((
            HybridSession {
                session,
                send,
                recv,
                binder,
            },
            plaintext,
        ))
    }

    /// Encrypt `plaintext`: advance the Olm ratchet, then wrap the inner Olm message
    /// in the outer post-quantum layer at the next send-chain index.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<HybridMessage, SessionError> {
        let (mut mk, index) = self.send.next_key()?;
        let inner = self.session.encrypt(plaintext)?;
        let inner_bytes = inner.to_bytes();
        let outer = pq_chain_wrap(&mk, &self.binder, index, &inner_bytes);
        mk.zeroize();
        let outer = outer?;
        Ok(HybridMessage { index, outer })
    }

    /// Decrypt a [`HybridMessage`]: authenticate + strip the outer post-quantum
    /// layer, then decrypt the inner Olm message.
    ///
    /// Fails closed on a tampered outer layer, a replayed index, or an index too far
    /// ahead of the receive chain. The receive chain is advanced ONLY after both the
    /// outer layer and the inner Olm message decrypt successfully, so a forged
    /// message cannot desync the chain. Out-of-order messages within the skip bound
    /// are handled via cached skipped keys.
    pub fn decrypt(&mut self, message: &HybridMessage) -> Result<Vec<u8>, SessionError> {
        let (mut mk, step) = self.recv.prepare(message.index)?;
        let inner_result = pq_chain_unwrap(&mk, &self.binder, message.index, &message.outer);
        mk.zeroize();
        let inner_bytes = inner_result?;
        let inner = SessionCiphertext::from_bytes(&inner_bytes)?;
        let plaintext = self.session.decrypt(&inner)?;
        // Authenticated end-to-end: commit the receive-chain advance.
        self.recv.commit(step);
        Ok(plaintext)
    }

    /// Whether the inner Olm session is v2 (untruncated MAC). Always true for a
    /// session from [`HybridSession::initiate`] / [`HybridSession::accept`].
    pub fn is_version_2(&self) -> bool {
        self.session.is_version_2()
    }
}

/// vodozemac message type code for a pre-key message.
const PRE_KEY_TYPE: u8 = 0;
/// vodozemac message type code for a normal (post-handshake) message.
const NORMAL_TYPE: u8 = 1;

/// An opaque, serializable Olm ciphertext: the message type plus the encrypted
/// body, as produced by [`OlmMessage::to_parts`].
///
/// The body is ciphertext, so `Debug`/`Clone` are safe to log. Designed to be
/// carried as the opaque payload of a [`mercury_message`] envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCiphertext {
    /// 0 = pre-key message, 1 = normal message.
    pub msg_type: u8,
    /// The Olm message body bytes.
    pub body: Vec<u8>,
}

impl SessionCiphertext {
    /// Build from a vodozemac [`OlmMessage`].
    fn from_olm_message(message: &OlmMessage) -> Self {
        let (msg_type, body) = message.to_parts();
        // vodozemac message types are 0 or 1, which fit in a u8.
        SessionCiphertext {
            msg_type: msg_type as u8,
            body,
        }
    }

    /// Reconstruct a vodozemac [`OlmMessage`], rejecting unknown types or a body
    /// that does not decode.
    fn to_olm_message(&self) -> Result<OlmMessage, SessionError> {
        OlmMessage::from_parts(self.msg_type as usize, &self.body)
            .map_err(|_| SessionError::Decryption)
    }

    /// Is this a pre-key (handshake-initiating) message?
    pub fn is_pre_key(&self) -> bool {
        self.msg_type == PRE_KEY_TYPE
    }

    /// Is this a normal (post-handshake) message?
    pub fn is_normal(&self) -> bool {
        self.msg_type == NORMAL_TYPE
    }

    /// Canonical wire bytes: `msg_type (1 byte) || body`. Suitable as the opaque
    /// payload of a [`mercury_message`] envelope.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.body.len());
        out.push(self.msg_type);
        out.extend_from_slice(&self.body);
        out
    }

    /// Parse wire bytes produced by [`SessionCiphertext::to_bytes`]. Never
    /// panics; an empty input is malformed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SessionError> {
        let (&msg_type, body) = bytes.split_first().ok_or(SessionError::Decryption)?;
        Ok(SessionCiphertext {
            msg_type,
            body: body.to_vec(),
        })
    }
}

#[cfg(test)]
mod leg_independence_tests {
    use super::*;

    /// LEG INDEPENDENCE — the marquee hybrid claim: confidentiality requires breaking BOTH legs.
    ///
    /// Here the attacker FULLY recovers the POST-QUANTUM leg — decapsulates the ML-KEM shared secret
    /// and derives the exact outer-AEAD key (`derive_first_flight_key` depends ONLY on the PQ leg by
    /// its signature) — and opens the OUTER PQ AEAD layer. What it recovers is the INNER Olm
    /// ciphertext, NOT the plaintext: the classical Olm layer independently protects it. (The reverse
    /// — Olm compromised, the PQ outer protects — is covered by `the_wrong_pq_keypair_fails_the_flight`
    /// and the tampered-mlkem test, where a wrong/absent PQ secret fails the outer AEAD.) A future
    /// refactor that keyed the outer AEAD from inner Olm state, or dropped the inner layer, would leak
    /// the plaintext here and fail this test — turning the headline claim from prose into a check.
    #[test]
    fn recovering_the_pq_leg_alone_does_not_reveal_the_plaintext() {
        const PLAINTEXT: &[u8] = b"top-secret-hybrid-bootstrap-plaintext";

        let mut bob_account = LocalSessionAccount::new();
        let bob_identity = IdentityKeyPair::generate();
        let bob_pq = MlKemKeyPair::generate();
        let bob_bundle = bob_account.publish_bundle(&bob_identity, &bob_pq.public(), 5);
        let alice = LocalSessionAccount::new();

        let (_a_session, flight) =
            MercurySession::establish_outbound_hybrid(&alice, &bob_bundle, PLAINTEXT).unwrap();

        // Break the PQ leg completely: decapsulate -> the exact outer-AEAD key.
        let ss_pq = bob_pq.decapsulate(&flight.mlkem_ciphertext).unwrap();
        let outer_key = derive_first_flight_key(&ss_pq, &flight.mlkem_ciphertext);

        // Opening the OUTER PQ AEAD now SUCCEEDS (the PQ leg is broken)...
        let (nonce, sealed) = flight.outer.split_at(NONCE_LEN);
        let cipher = XChaCha20Poly1305::new_from_slice(&outer_key)
            .expect("32-byte key is valid for XChaCha20Poly1305");
        let inner_olm = cipher
            .decrypt(
                XNonce::from_slice(nonce),
                Payload {
                    msg: sealed,
                    aad: &first_flight_aad(&flight.mlkem_ciphertext),
                },
            )
            .expect("a recovered PQ leg opens the outer AEAD");

        // ...but the recovered bytes are the INNER Olm ciphertext, NOT the plaintext: the classical
        // leg still protects it. Breaking only the PQ leg yields no plaintext.
        assert_ne!(inner_olm.as_slice(), PLAINTEXT);
        assert!(
            !inner_olm.windows(PLAINTEXT.len()).any(|w| w == PLAINTEXT),
            "breaking the PQ leg alone must NOT reveal the plaintext — the inner Olm leg protects it"
        );
    }
}
