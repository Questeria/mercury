//! Local identity and device key generation for Mercury (Phase 2).
//!
//! Local-only: no network, no session/ratchet protocol. All primitives come
//! from audited RustCrypto crates; nothing here implements crypto by hand.
//!
//! Secret-bearing types (`IdentityKeyPair`, `DeviceKeyPair`) zeroize their key
//! material on drop and redact it from `Debug`.

#![forbid(unsafe_code)]

use core::fmt;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use ml_kem::ml_kem_768::{DecapsulationKey, EncapsulationKey};
use ml_kem::{Ciphertext, Decapsulate, Encapsulate, Kem, KeyExport, MlKem768, Seed, TryKeyInit};
use sha2::{Digest, Sha256, Sha512};
use x25519_dalek::{PublicKey as XPublicKey, StaticSecret};
use zeroize::ZeroizeOnDrop;

const ACCOUNT_ID_DOMAIN: &[u8] = b"mercury-account-id-v1";

/// Errors surfaced when importing public keys, verifying signatures, or
/// performing key agreement.
///
/// Secret-key construction cannot fail; errors only arise from untrusted
/// public bytes, signatures, or a peer-supplied agreement key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyError {
    /// Bytes did not decode to a valid public key (e.g. not on the curve).
    InvalidPublicKey,
    /// A signature failed verification against the given key and message.
    InvalidSignature,
    /// The peer's key produced a non-contributory (low-order) shared secret.
    NonContributoryPeer,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::InvalidPublicKey => f.write_str("invalid public key"),
            KeyError::InvalidSignature => f.write_str("invalid signature"),
            KeyError::NonContributoryPeer => f.write_str("non-contributory peer key"),
        }
    }
}

impl std::error::Error for KeyError {}

/// Long-term Ed25519 identity signing key.
///
/// The underlying `SigningKey` zeroizes on drop (dalek `zeroize` feature).
pub struct IdentityKeyPair(SigningKey);

impl IdentityKeyPair {
    /// Generate a fresh identity key from the OS CSPRNG.
    pub fn generate() -> Self {
        Self::generate_with(&mut rand_core::OsRng)
    }

    /// Generate a fresh identity key from a caller-supplied CSPRNG.
    pub fn generate_with(rng: &mut impl rand_core::CryptoRngCore) -> Self {
        Self(SigningKey::generate(rng))
    }

    /// Sign `message`, returning the raw 64-byte Ed25519 signature.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.0.sign(message).to_bytes()
    }

    /// The public half of this identity key.
    pub fn public(&self) -> IdentityPublicKey {
        IdentityPublicKey(self.0.verifying_key().to_bytes())
    }

    /// Serialize the SECRET signing key to its 32 raw bytes. SECRET-BEARING: the caller
    /// must store these encrypted-at-rest and zeroize the copy after use.
    pub fn to_secret_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Reconstruct an identity key from its 32 secret bytes (any 32 bytes is a valid
    /// Ed25519 seed, so this is infallible).
    pub fn from_secret_bytes(bytes: [u8; 32]) -> Self {
        Self(SigningKey::from_bytes(&bytes))
    }
}

impl fmt::Debug for IdentityKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never expose private scalar bytes; identify by public key only.
        f.debug_struct("IdentityKeyPair")
            .field("public", &self.public())
            .finish_non_exhaustive()
    }
}

/// Ed25519 identity public key (32 bytes).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IdentityPublicKey([u8; 32]);

impl IdentityPublicKey {
    /// Borrow the raw 32-byte encoding.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Decode and validate a 32-byte Ed25519 public key.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self, KeyError> {
        VerifyingKey::from_bytes(&bytes).map_err(|_| KeyError::InvalidPublicKey)?;
        Ok(Self(bytes))
    }

    /// Verify a 64-byte signature over `message` under this key.
    pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> Result<(), KeyError> {
        let key = VerifyingKey::from_bytes(&self.0).map_err(|_| KeyError::InvalidPublicKey)?;
        let sig = Signature::from_bytes(signature);
        key.verify(message, &sig)
            .map_err(|_| KeyError::InvalidSignature)
    }
}

impl fmt::Debug for IdentityPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IdentityPublicKey({})", hex32(&self.0))
    }
}

/// X25519 device key for Diffie-Hellman key agreement.
///
/// `StaticSecret` derives `Zeroize` but does not zeroize on drop by itself, so
/// the wrapper derives `ZeroizeOnDrop` to clear the scalar when dropped.
#[derive(ZeroizeOnDrop)]
pub struct DeviceKeyPair(StaticSecret);

impl DeviceKeyPair {
    /// Generate a fresh device key from the OS CSPRNG.
    pub fn generate() -> Self {
        Self::generate_with(&mut rand_core::OsRng)
    }

    /// Generate a fresh device key from a caller-supplied CSPRNG.
    pub fn generate_with(rng: &mut impl rand_core::CryptoRngCore) -> Self {
        Self(StaticSecret::random_from_rng(rng))
    }

    /// The public half of this device key.
    pub fn public(&self) -> DevicePublicKey {
        DevicePublicKey(XPublicKey::from(&self.0).to_bytes())
    }

    /// Compute the X25519 shared secret with `peer`.
    ///
    /// Rejects non-contributory results: a low-order peer key would force an
    /// all-zero shared secret, so such peers return `NonContributoryPeer`. The
    /// 32-byte output is a raw DH result; callers MUST run it through a KDF
    /// before use as a symmetric key.
    pub fn diffie_hellman(&self, peer: &DevicePublicKey) -> Result<[u8; 32], KeyError> {
        let shared = self.0.diffie_hellman(&XPublicKey::from(peer.0));
        if !shared.was_contributory() {
            return Err(KeyError::NonContributoryPeer);
        }
        Ok(shared.to_bytes())
    }

    /// Serialize the SECRET X25519 key to its 32 raw bytes. SECRET-BEARING: the caller
    /// must store these encrypted-at-rest and zeroize the copy after use.
    pub fn to_secret_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Reconstruct a device key from its 32 secret bytes.
    pub fn from_secret_bytes(bytes: [u8; 32]) -> Self {
        Self(StaticSecret::from(bytes))
    }
}

impl fmt::Debug for DeviceKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceKeyPair")
            .field("public", &self.public())
            .finish_non_exhaustive()
    }
}

/// X25519 device public key (32 bytes).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DevicePublicKey([u8; 32]);

impl DevicePublicKey {
    /// Borrow the raw 32-byte encoding.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Wrap a 32-byte X25519 public key. Every 32-byte string is a syntactically
    /// valid Montgomery-u value, so this is infallible by construction.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for DevicePublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DevicePublicKey({})", hex32(&self.0))
    }
}

/// Derive a stable account id from an identity public key:
/// `SHA-256(b"mercury-account-id-v1" || pubkey_bytes)`.
pub fn account_id_from_identity_key(public: &IdentityPublicKey) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ACCOUNT_ID_DOMAIN);
    hasher.update(public.as_bytes());
    hasher.finalize().into()
}

/// Domain-separation tag for the directory-publish proof of possession.
pub const DIRECTORY_POP_DOMAIN: &[u8] = b"mercury-directory-publish-pop-v1";

/// The exact byte string a directory-publish proof of possession signs:
/// `DIRECTORY_POP_DOMAIN || card`. Binding the card means a captured proof cannot be
/// replayed onto a different card.
fn directory_pop_message(card: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(DIRECTORY_POP_DOMAIN.len() + card.len());
    m.extend_from_slice(DIRECTORY_POP_DOMAIN);
    m.extend_from_slice(card);
    m
}

/// Sign a directory-publish proof of possession: the holder of `identity` authorizes
/// publishing `card` to *its own* directory slot (`account_id_from_identity_key(identity)`).
/// The relay verifies this before storing, so only the identity-key holder can write to a
/// slot — closing directory slot-squatting.
pub fn sign_directory_publish(identity: &IdentityKeyPair, card: &[u8]) -> [u8; 64] {
    identity.sign(&directory_pop_message(card))
}

/// Verify a directory-publish proof of possession. Returns the account id derived from
/// `identity_pub` **iff** `pop_sig` is a valid signature by that identity over `card`. A bad
/// key or a bad/forged signature returns `None` so the caller fails closed. Because the
/// returned account id is derived from the verified identity key, a caller that stores under
/// it cannot be tricked into writing to someone else's slot.
pub fn verify_directory_publish(
    identity_pub: &[u8; 32],
    card: &[u8],
    pop_sig: &[u8; 64],
) -> Option<[u8; 32]> {
    let public = IdentityPublicKey::from_bytes(*identity_pub).ok()?;
    public.verify(&directory_pop_message(card), pop_sig).ok()?;
    Some(account_id_from_identity_key(&public))
}

/// Domain-separation tag for the pairing-code publish proof of possession. Distinct from the
/// directory-publish tag so a directory proof can never be replayed as a pairing publish (and
/// vice-versa), even though both sign over a contact card.
pub const PAIRING_POP_DOMAIN: &[u8] = b"mercury-pairing-publish-pop-v1";

/// The exact byte string a pairing-publish proof of possession signs: `PAIRING_POP_DOMAIN || card`.
fn pairing_pop_message(card: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(PAIRING_POP_DOMAIN.len() + card.len());
    m.extend_from_slice(PAIRING_POP_DOMAIN);
    m.extend_from_slice(card);
    m
}

/// Sign a pairing-publish proof of possession: the holder of `identity` authorizes brokering
/// `card` under a short-lived, single-use pairing code. The relay stores the card under a random
/// code (NOT a derived slot), so this proof does not gate *where* the card lands; it proves the
/// publisher actually holds an identity key and signed exactly these card bytes, so the pairing
/// broker cannot be stuffed with un-signed garbage. The fetcher independently re-verifies that the
/// fetched card is self-authenticating before use.
pub fn sign_pairing_publish(identity: &IdentityKeyPair, card: &[u8]) -> [u8; 64] {
    identity.sign(&pairing_pop_message(card))
}

/// Verify a pairing-publish proof of possession. Returns the account id derived from
/// `identity_pub` **iff** `pop_sig` is a valid signature by that identity over `card`; a bad key
/// or a bad/forged signature returns `None` so the caller fails closed.
pub fn verify_pairing_publish(
    identity_pub: &[u8; 32],
    card: &[u8],
    pop_sig: &[u8; 64],
) -> Option<[u8; 32]> {
    let public = IdentityPublicKey::from_bytes(*identity_pub).ok()?;
    public.verify(&pairing_pop_message(card), pop_sig).ok()?;
    Some(account_id_from_identity_key(&public))
}

/// Domain-separation tag for the relay route-ownership proof of possession (poll / ack /
/// delete on a route). Distinct from the directory-publish tag so the two proofs can never be
/// cross-replayed.
pub const RELAY_ROUTE_POP_DOMAIN: &[u8] = b"mercury-relay-route-pop-v1";

/// A relay operation a route-ownership proof authorizes. The proof binds the operation, so a
/// captured proof for one operation (e.g. a read-only `Poll`) cannot be replayed as another
/// (e.g. a destructive `Delete`) within the freshness window. Discriminants are non-zero so a
/// zeroed byte is not a valid operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RelayRouteOperation {
    /// Drain the next queued item — `GET /relay/poll/:route`.
    Poll = 1,
    /// Acknowledge a delivered item — `POST /relay/ack/:route`.
    Ack = 2,
    /// Delete the route's queue — `DELETE /relay/queue/:route`.
    Delete = 3,
}

/// The exact byte string a relay route-ownership proof signs:
/// `RELAY_ROUTE_POP_DOMAIN || route_id(32) || operation(1) || timestamp_s` (8-byte little-endian
/// `i64`). Binding the route ties the proof to one route; binding the operation stops a proof for
/// one endpoint being replayed against another; binding the timestamp lets the relay enforce a
/// freshness window so an old proof cannot be replayed indefinitely.
fn relay_route_pop_message(
    route_id: &[u8; 32],
    operation: RelayRouteOperation,
    timestamp_s: i64,
) -> Vec<u8> {
    let mut m = Vec::with_capacity(RELAY_ROUTE_POP_DOMAIN.len() + 32 + 1 + 8);
    m.extend_from_slice(RELAY_ROUTE_POP_DOMAIN);
    m.extend_from_slice(route_id);
    m.push(operation as u8);
    m.extend_from_slice(&timestamp_s.to_le_bytes());
    m
}

/// Sign a relay route-ownership proof: the holder of `identity` proves it owns `route_id` and is
/// authorizing `operation` at `timestamp_s`. For a 1:1 route the route IS the recipient's account
/// id, so this proves "I am the recipient." The relay verifies it to authorize draining that
/// route's queue (poll / ack / delete) — closing the forgeable header-trust on those endpoints.
pub fn sign_relay_route_pop(
    identity: &IdentityKeyPair,
    route_id: &[u8; 32],
    operation: RelayRouteOperation,
    timestamp_s: i64,
) -> [u8; 64] {
    identity.sign(&relay_route_pop_message(route_id, operation, timestamp_s))
}

/// Verify a relay route-ownership proof. Returns `true` iff `identity_pub` is a valid key, it
/// OWNS the route (`account_id_from_identity_key(identity_pub) == route_id`), and `pop_sig` is a
/// valid signature by it over `(route_id, operation, timestamp_s)`. This does NOT check freshness:
/// only the relay knows wall-clock "now", so it enforces the `timestamp_s` window itself. A proof
/// signed for one `operation` does not verify for another. Fail-closed — a bad key, a non-owned
/// route, the wrong operation, or a bad signature all return `false`.
pub fn verify_relay_route_pop(
    identity_pub: &[u8; 32],
    route_id: &[u8; 32],
    operation: RelayRouteOperation,
    timestamp_s: i64,
    pop_sig: &[u8; 64],
) -> bool {
    let Ok(public) = IdentityPublicKey::from_bytes(*identity_pub) else {
        return false;
    };
    if account_id_from_identity_key(&public) != *route_id {
        return false;
    }
    public
        .verify(
            &relay_route_pop_message(route_id, operation, timestamp_s),
            pop_sig,
        )
        .is_ok()
}

// === Username (key-transparency-backed handle) claim proof of possession ===

/// Minimum and maximum username length, in characters (= bytes, since usernames are ASCII).
pub const USERNAME_MIN_LEN: usize = 3;
pub const USERNAME_MAX_LEN: usize = 20;

/// Normalize + validate a human-typed username into its canonical registry form, or `None` if it
/// is not a legal username. Canonical form is lowercase ASCII; the legal charset is `[a-z0-9_]`
/// with the FIRST character a letter (`[a-z]`), and length [`USERNAME_MIN_LEN`]..=[`USERNAME_MAX_LEN`].
///
/// Restricting to ASCII `[a-z0-9_]` and lowercasing is a deliberate, conservative anti-confusable
/// measure: it removes case-only collisions and the large Unicode homoglyph attack surface (so
/// `аlice` with a Cyrillic `а` is simply rejected, not silently confused with `alice`). Both the
/// claimer (when signing the claim proof) and any looker-up MUST normalize through this function so
/// they agree on the exact label committed to the transparency log.
pub fn normalize_username(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let lower = trimmed.to_ascii_lowercase();
    let len = lower.len();
    if !(USERNAME_MIN_LEN..=USERNAME_MAX_LEN).contains(&len) {
        return None;
    }
    let mut chars = lower.chars();
    // First character must be an ASCII letter (so a handle never starts with a digit/underscore,
    // and is never an all-numeric string that could be confused with an account id).
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return None,
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return None;
    }
    Some(lower)
}

/// Domain-separation tag for the username-claim proof of possession. Distinct from every other
/// proof tag so a claim proof can never be cross-replayed as a directory/pairing/route proof.
pub const USERNAME_CLAIM_POP_DOMAIN: &[u8] = b"mercury-username-claim-pop-v1";

/// The exact byte string a username-claim proof signs: `USERNAME_CLAIM_POP_DOMAIN || username`,
/// where `username` is the NORMALIZED form. Binding the username stops a captured claim proof for
/// one handle being replayed to claim a different handle; the account id is bound implicitly
/// because it is derived from the signing identity key.
fn username_claim_message(normalized_username: &str) -> Vec<u8> {
    let mut m = Vec::with_capacity(USERNAME_CLAIM_POP_DOMAIN.len() + normalized_username.len());
    m.extend_from_slice(USERNAME_CLAIM_POP_DOMAIN);
    m.extend_from_slice(normalized_username.as_bytes());
    m
}

/// Sign a username-claim proof: the holder of `identity` authorizes binding `normalized_username`
/// to ITS OWN account id (`account_id_from_identity_key(identity)`) in the transparency log. The
/// caller MUST pass a string already run through [`normalize_username`] so the signed label matches
/// what the relay commits and a looker-up later checks.
pub fn sign_username_claim(identity: &IdentityKeyPair, normalized_username: &str) -> [u8; 64] {
    identity.sign(&username_claim_message(normalized_username))
}

/// Verify a username-claim proof. Returns the account id derived from `identity_pub` **iff**
/// `normalized_username` is a legal username AND `pop_sig` is a valid signature by that identity
/// over it. A bad key, an illegal username, or a bad/forged signature returns `None` so the relay
/// fails closed. The returned account id is what the relay binds the handle to — derived from the
/// verified key, so a claimer can only ever bind a handle to its own account.
pub fn verify_username_claim(
    identity_pub: &[u8; 32],
    normalized_username: &str,
    pop_sig: &[u8; 64],
) -> Option<[u8; 32]> {
    // Re-normalize defensively: a claim must be over a canonical username, never a raw one.
    if normalize_username(normalized_username).as_deref() != Some(normalized_username) {
        return None;
    }
    let public = IdentityPublicKey::from_bytes(*identity_pub).ok()?;
    public
        .verify(&username_claim_message(normalized_username), pop_sig)
        .ok()?;
    Some(account_id_from_identity_key(&public))
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        s.push(char::from_digit((b & 0x0f) as u32, 16).unwrap());
    }
    s
}

// === Post-quantum device key (ML-KEM-768, FIPS 203) ===

/// Copy a 32-byte ML-KEM shared secret into a fixed array. The caller is
/// responsible for feeding it through a KDF (it is a raw KEM output, never used
/// directly as a symmetric key).
fn shared_secret_bytes(shared: impl AsRef<[u8]>) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(shared.as_ref());
    out
}

/// An ML-KEM-768 (FIPS 203) post-quantum KEM keypair — the PQ half of Mercury's
/// hybrid sealed box. It pairs with the classical X25519 [`DeviceKeyPair`]; the
/// two shared secrets are combined in the sealed box's KDF, so a message stays
/// secure as long as EITHER primitive holds (defense against a future quantum
/// adversary who breaks X25519, and against a flaw in the newer ML-KEM).
///
/// The decapsulation (secret) key zeroizes on drop (ml-kem `zeroize` feature).
pub struct MlKemKeyPair {
    decapsulation: DecapsulationKey,
    encapsulation: EncapsulationKey,
}

// The decapsulation (secret) key MUST wipe on drop. `MlKemKeyPair` cannot itself
// derive `ZeroizeOnDrop` (its public `EncapsulationKey` field is not `Zeroize`),
// so it relies on ml-kem's own `ZeroizeOnDrop for DecapsulationKey` (the
// `zeroize` feature). This compile-time assertion FAILS THE BUILD if a future
// ml-kem version or feature change silently drops that guarantee.
const _: fn() = || {
    fn assert_zeroize_on_drop<T: ZeroizeOnDrop>() {}
    assert_zeroize_on_drop::<DecapsulationKey>();
};

impl MlKemKeyPair {
    /// Generate a fresh ML-KEM-768 keypair from the OS CSPRNG.
    pub fn generate() -> Self {
        let (decapsulation, encapsulation) = MlKem768::generate_keypair();
        Self {
            decapsulation,
            encapsulation,
        }
    }

    /// The public (encapsulation) key others encapsulate to.
    pub fn public(&self) -> MlKemPublicKey {
        MlKemPublicKey(self.encapsulation.clone())
    }

    /// Decapsulate a `ciphertext` produced by [`MlKemPublicKey::encapsulate`]
    /// into the 32-byte shared secret. ML-KEM decapsulation never fails on a
    /// well-formed ciphertext (implicit rejection yields an unrelated secret for
    /// a bad one, which then fails the downstream AEAD); a wrong-length
    /// ciphertext is rejected here as malformed.
    pub fn decapsulate(&self, ciphertext: &[u8]) -> Result<[u8; 32], KeyError> {
        let ct =
            Ciphertext::<MlKem768>::try_from(ciphertext).map_err(|_| KeyError::InvalidPublicKey)?;
        Ok(shared_secret_bytes(self.decapsulation.decapsulate(&ct)))
    }

    /// Serialize the SECRET keypair to its 64-byte FIPS-203 seed. The full keypair is
    /// deterministically reconstructed from it. SECRET-BEARING: the caller must store
    /// these bytes encrypted-at-rest and zeroize the copy after use.
    pub fn to_secret_bytes(&self) -> Vec<u8> {
        self.decapsulation.to_bytes().to_vec()
    }

    /// Reconstruct a keypair from its 64-byte seed (see [`MlKemKeyPair::to_secret_bytes`]).
    /// Never panics; fails closed ([`KeyError::InvalidPublicKey`]) on a wrong-length seed.
    pub fn from_secret_bytes(bytes: &[u8]) -> Result<Self, KeyError> {
        let seed = Seed::try_from(bytes).map_err(|_| KeyError::InvalidPublicKey)?;
        let decapsulation = DecapsulationKey::from_seed(seed);
        let encapsulation = decapsulation.encapsulation_key().clone();
        Ok(Self {
            decapsulation,
            encapsulation,
        })
    }
}

impl fmt::Debug for MlKemKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never expose secret-key material; identify by the public key length.
        f.debug_struct("MlKemKeyPair")
            .field("public_len", &self.public().to_bytes().len())
            .finish_non_exhaustive()
    }
}

/// An ML-KEM-768 public (encapsulation) key — the pinnable / wire form others
/// encapsulate a shared secret to.
#[derive(Clone)]
pub struct MlKemPublicKey(EncapsulationKey);

impl MlKemPublicKey {
    /// Serialize to the canonical FIPS-203 encapsulation-key bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    /// Parse from bytes, rejecting a malformed key.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeyError> {
        EncapsulationKey::new_from_slice(bytes)
            .map(MlKemPublicKey)
            .map_err(|_| KeyError::InvalidPublicKey)
    }

    /// Encapsulate to this key, returning `(ciphertext, shared_secret)` where the
    /// 32-byte shared secret matches what the holder's [`MlKemKeyPair::decapsulate`]
    /// recovers. Fresh randomness per call.
    pub fn encapsulate(&self) -> (Vec<u8>, [u8; 32]) {
        let (ciphertext, shared) = self.0.encapsulate();
        (ciphertext.to_vec(), shared_secret_bytes(shared))
    }
}

impl fmt::Debug for MlKemPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MlKemPublicKey({} bytes)", self.to_bytes().len())
    }
}

#[cfg(test)]
mod ml_kem_tests {
    use super::*;

    #[test]
    fn encapsulate_decapsulate_round_trips() {
        let kp = MlKemKeyPair::generate();
        let pubkey = kp.public();
        let (ciphertext, sender_secret) = pubkey.encapsulate();
        let recipient_secret = kp.decapsulate(&ciphertext).unwrap();
        assert_eq!(
            sender_secret, recipient_secret,
            "both sides derive the same secret"
        );
        assert_eq!(sender_secret.len(), 32);
    }

    #[test]
    fn public_key_serialization_round_trips() {
        let kp = MlKemKeyPair::generate();
        let bytes = kp.public().to_bytes();
        let reparsed = MlKemPublicKey::from_bytes(&bytes).unwrap();
        // The reparsed key encapsulates to a secret the original keypair opens.
        let (ciphertext, sender_secret) = reparsed.encapsulate();
        assert_eq!(kp.decapsulate(&ciphertext).unwrap(), sender_secret);
    }

    #[test]
    fn distinct_keypairs_do_not_share_secrets() {
        // A ciphertext for one keypair must not decapsulate to the sender's
        // secret under a different keypair (implicit rejection -> different bytes).
        let kp_a = MlKemKeyPair::generate();
        let kp_b = MlKemKeyPair::generate();
        let (ciphertext, sender_secret) = kp_a.public().encapsulate();
        let wrong = kp_b.decapsulate(&ciphertext).unwrap();
        assert_ne!(wrong, sender_secret);
    }

    #[test]
    fn malformed_ciphertext_is_rejected() {
        let kp = MlKemKeyPair::generate();
        assert_eq!(kp.decapsulate(&[0u8; 7]), Err(KeyError::InvalidPublicKey));
        assert!(kp.decapsulate(&[]).is_err());
    }

    #[test]
    fn malformed_public_key_is_rejected() {
        assert_eq!(
            MlKemPublicKey::from_bytes(&[0u8; 10]).err(),
            Some(KeyError::InvalidPublicKey)
        );
    }
}

// === Identity safety number (manual / QR out-of-band verification backstop) ===

/// Version tag mixed into every fingerprint preimage, so a future format change
/// produces disjoint numbers rather than silently colliding with this one.
const SAFETY_NUMBER_VERSION: u16 = 1;
/// Iterated-hash rounds when deriving a party fingerprint. Key-stretches the
/// short displayed digits against precomputation (mirrors Signal's default).
const SAFETY_NUMBER_ITERATIONS: u32 = 5200;
/// Bytes kept from each party's 64-byte iterated hash (240 bits -> 30 digits).
const FINGERPRINT_LEN: usize = 30;

/// A mutual identity safety number for OUT-OF-BAND verification: 60 decimal
/// digits that BOTH parties compute identically from each other's identity
/// public key + stable account id. If the digits match on both devices there is
/// no man-in-the-middle on the identity keys -- the human-verifiable backstop
/// behind key transparency that feeds `ManualVerificationState::Verified` in the
/// device-trust gate.
///
/// Order-independent by construction: the two per-party fingerprints are
/// concatenated in canonical (sorted) order, so each side computes the SAME
/// number regardless of who is "local" vs "remote".
#[derive(Clone)]
pub struct SafetyNumber {
    digits: String,
    /// Raw `lo(30) || hi(30)` fingerprint bytes -- what a scannable QR encodes.
    /// Transport only: a direct byte comparison is STRICTLY STRONGER than (and
    /// agrees with) the displayed-digit comparison, but the canonical identity of
    /// a safety number is its digits (see `PartialEq` / [`SafetyNumber::matches`]).
    fingerprints: [u8; 2 * FINGERPRINT_LEN],
}

impl PartialEq for SafetyNumber {
    /// Equality is the DISPLAYED number: two safety numbers are equal iff their
    /// digits match -- exactly what a human compares and what feeds the
    /// device-trust gate. (The raw `fingerprints` are QR transport, not identity;
    /// for honestly-derived numbers equal digits and equal fingerprints coincide.)
    fn eq(&self, other: &Self) -> bool {
        self.digits == other.digits
    }
}

impl Eq for SafetyNumber {}

impl SafetyNumber {
    /// The 60-digit number as a plain string (no grouping).
    pub fn digits(&self) -> &str {
        &self.digits
    }

    /// The raw fingerprint bytes a QR code would carry for camera comparison.
    pub fn scannable_bytes(&self) -> &[u8; 2 * FINGERPRINT_LEN] {
        &self.fingerprints
    }

    /// True iff two safety numbers display the same digits -- the out-of-band
    /// match check. This is exactly the comparison a human makes, so a
    /// programmatic `matches()` cannot accept a pair the user would see as
    /// different (and vice-versa).
    pub fn matches(&self, other: &SafetyNumber) -> bool {
        self.digits == other.digits
    }

    /// Display form: 12 space-separated groups of 5 digits.
    pub fn grouped(&self) -> String {
        let mut out = String::with_capacity(self.digits.len() + 11);
        for (i, ch) in self.digits.chars().enumerate() {
            if i > 0 && i.is_multiple_of(5) {
                out.push(' ');
            }
            out.push(ch);
        }
        out
    }
}

impl fmt::Debug for SafetyNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // A safety number is not secret, but keep Debug compact + grouped.
        write!(f, "SafetyNumber({})", self.grouped())
    }
}

/// One party's iterated identity fingerprint: `SHA-512`, `ITERATIONS` rounds over
/// `version || identity_key || account_id` (re-mixing the identity key each
/// round), keeping the first [`FINGERPRINT_LEN`] bytes.
fn party_fingerprint(account_id: &[u8; 32], identity: &IdentityPublicKey) -> [u8; FINGERPRINT_LEN] {
    let key = identity.as_bytes();
    let mut hash = {
        let mut h = Sha512::new();
        h.update(SAFETY_NUMBER_VERSION.to_be_bytes());
        h.update(key);
        h.update(account_id);
        h.finalize()
    };
    for _ in 1..SAFETY_NUMBER_ITERATIONS {
        let mut h = Sha512::new();
        h.update(hash);
        h.update(key);
        hash = h.finalize();
    }
    let mut out = [0u8; FINGERPRINT_LEN];
    out.copy_from_slice(&hash[..FINGERPRINT_LEN]);
    out
}

/// Encode a fingerprint as decimal digits: each 5-byte chunk's 40-bit big-endian
/// value taken mod 100000 and zero-padded to 5 digits (6 chunks -> 30 digits).
fn encode_digits(fingerprint: &[u8; FINGERPRINT_LEN]) -> String {
    let mut s = String::with_capacity(FINGERPRINT_LEN / 5 * 5);
    for chunk in fingerprint.chunks_exact(5) {
        let mut value: u64 = 0;
        for &b in chunk {
            value = (value << 8) | u64::from(b);
        }
        s.push_str(&format!("{:05}", value % 100_000));
    }
    s
}

/// Compute the mutual identity safety number for two parties from their identity
/// public keys + stable account ids. Order-independent: both parties get the
/// same number regardless of argument order (the per-party fingerprints are
/// concatenated lowest-first). Matching numbers on both devices => no MITM on
/// the identity keys.
pub fn safety_number(
    account_id_a: &[u8; 32],
    identity_a: &IdentityPublicKey,
    account_id_b: &[u8; 32],
    identity_b: &IdentityPublicKey,
) -> SafetyNumber {
    let fa = party_fingerprint(account_id_a, identity_a);
    let fb = party_fingerprint(account_id_b, identity_b);
    let (lo, hi) = if fa <= fb { (fa, fb) } else { (fb, fa) };

    let mut fingerprints = [0u8; 2 * FINGERPRINT_LEN];
    fingerprints[..FINGERPRINT_LEN].copy_from_slice(&lo);
    fingerprints[FINGERPRINT_LEN..].copy_from_slice(&hi);

    SafetyNumber {
        digits: format!("{}{}", encode_digits(&lo), encode_digits(&hi)),
        fingerprints,
    }
}

#[cfg(test)]
mod safety_number_tests {
    use super::*;

    fn party() -> ([u8; 32], IdentityPublicKey) {
        let id = IdentityKeyPair::generate().public();
        (account_id_from_identity_key(&id), id)
    }

    #[test]
    fn both_parties_compute_the_same_number_regardless_of_order() {
        let (a_id, a_key) = party();
        let (b_id, b_key) = party();

        // Party A computes (self, peer); party B computes (self, peer) -> the
        // argument order is swapped, yet the number must be identical.
        let from_a = safety_number(&a_id, &a_key, &b_id, &b_key);
        let from_b = safety_number(&b_id, &b_key, &a_id, &a_key);

        assert_eq!(from_a, from_b);
        assert!(from_a.matches(&from_b));
        assert_eq!(from_a.digits(), from_b.digits());
    }

    #[test]
    fn safety_number_is_60_grouped_decimal_digits() {
        let (a_id, a_key) = party();
        let (b_id, b_key) = party();
        let sn = safety_number(&a_id, &a_key, &b_id, &b_key);

        assert_eq!(sn.digits().len(), 60);
        assert!(sn.digits().bytes().all(|b| b.is_ascii_digit()));
        // 12 groups of 5 digits, single-space separated.
        let grouped = sn.grouped();
        let groups: Vec<&str> = grouped.split(' ').collect();
        assert_eq!(groups.len(), 12);
        assert!(groups.iter().all(|g| g.len() == 5));
    }

    #[test]
    fn a_substituted_identity_key_changes_the_number() {
        // A man-in-the-middle swapping one identity key (the attack the safety
        // number exists to catch) must produce a different, non-matching number.
        let (a_id, a_key) = party();
        let (b_id, b_key) = party();
        let (_, mitm_key) = party();

        let honest = safety_number(&a_id, &a_key, &b_id, &b_key);
        let attacked = safety_number(&a_id, &a_key, &b_id, &mitm_key);

        assert_ne!(honest, attacked);
        assert!(!honest.matches(&attacked));
    }

    #[test]
    fn account_id_is_bound_not_just_the_key() {
        // Same identity key but a different stable account id -> different number
        // (the id is part of the fingerprint preimage).
        let (_, key) = party();
        let other_id = [9u8; 32];
        let (peer_id, peer_key) = party();

        let n1 = safety_number(
            &account_id_from_identity_key(&key),
            &key,
            &peer_id,
            &peer_key,
        );
        let n2 = safety_number(&other_id, &key, &peer_id, &peer_key);
        assert_ne!(n1, n2);
    }
}

#[cfg(test)]
mod pairing_pop_tests {
    use super::*;

    #[test]
    fn valid_pairing_proof_returns_signer_account_id() {
        let id = IdentityKeyPair::generate();
        let card = b"an opaque contact card".to_vec();
        let sig = sign_pairing_publish(&id, &card);
        assert_eq!(
            verify_pairing_publish(id.public().as_bytes(), &card, &sig),
            Some(account_id_from_identity_key(&id.public()))
        );
    }

    #[test]
    fn pairing_proof_is_bound_to_the_card() {
        let id = IdentityKeyPair::generate();
        let sig = sign_pairing_publish(&id, b"card A");
        // A proof over card A must not verify over card B.
        assert_eq!(
            verify_pairing_publish(id.public().as_bytes(), b"card B", &sig),
            None
        );
    }

    #[test]
    fn a_directory_proof_does_not_verify_as_a_pairing_proof() {
        // Domain separation: the SAME card signed for the directory must not be accepted by the
        // pairing verifier (and vice-versa).
        let id = IdentityKeyPair::generate();
        let card = b"shared card".to_vec();
        let dir_sig = sign_directory_publish(&id, &card);
        assert_eq!(
            verify_pairing_publish(id.public().as_bytes(), &card, &dir_sig),
            None
        );
        let pair_sig = sign_pairing_publish(&id, &card);
        assert_eq!(
            verify_directory_publish(id.public().as_bytes(), &card, &pair_sig),
            None
        );
    }
}

#[cfg(test)]
mod username_tests {
    use super::*;

    #[test]
    fn normalize_lowercases_and_trims() {
        assert_eq!(normalize_username("  Alice  ").as_deref(), Some("alice"));
        assert_eq!(normalize_username("Bob_99").as_deref(), Some("bob_99"));
    }

    #[test]
    fn normalize_rejects_illegal_usernames() {
        assert_eq!(normalize_username("ab"), None); // too short
        assert_eq!(normalize_username(&"a".repeat(21)), None); // too long
        assert_eq!(normalize_username("9lives"), None); // must start with a letter
        assert_eq!(normalize_username("_hidden"), None); // must start with a letter
        assert_eq!(normalize_username("has space"), None); // illegal char
        assert_eq!(normalize_username("emoji😀x"), None); // non-ASCII
        assert_eq!(normalize_username("аlice"), None); // Cyrillic 'а' homoglyph — rejected
        assert_eq!(normalize_username("bad-dash"), None); // dash not allowed
    }

    #[test]
    fn valid_username_claim_returns_signer_account_id() {
        let id = IdentityKeyPair::generate();
        let name = normalize_username("Alice").unwrap();
        let sig = sign_username_claim(&id, &name);
        assert_eq!(
            verify_username_claim(id.public().as_bytes(), &name, &sig),
            Some(account_id_from_identity_key(&id.public()))
        );
    }

    #[test]
    fn username_claim_is_bound_to_the_handle() {
        let id = IdentityKeyPair::generate();
        let sig = sign_username_claim(&id, "alice");
        // A claim proof for "alice" must not verify for "bob".
        assert_eq!(
            verify_username_claim(id.public().as_bytes(), "bob", &sig),
            None
        );
    }

    #[test]
    fn username_claim_rejects_non_canonical_handle() {
        let id = IdentityKeyPair::generate();
        // Even with a valid signature over the raw bytes, verification requires the canonical form.
        let sig = id.sign(&username_claim_message("Alice"));
        assert_eq!(
            verify_username_claim(id.public().as_bytes(), "Alice", &sig),
            None
        );
    }
}
