#![forbid(unsafe_code)]
//! Mercury client runtime — Milestone 1 (forward-secret ratchet + sealed-sender envelope).
//!
//! This crate ties a Mercury identity, a session account, per-peer
//! [`HybridRatchetSession`]s (the post-quantum Double Ratchet), and a sealed-sender
//! transport envelope into a working end-to-end-encrypted 1:1 messenger client.
//!
//! TWO cryptographic layers protect every message:
//!   * INNER — the hybrid post-quantum ratchet ([`mercury_session`]): the plaintext is
//!     confidential if EITHER ML-KEM-768 OR the classical Olm handshake holds, with
//!     forward secrecy AND post-compromise security.
//!   * OUTER — a sealed-sender envelope ([`mercury_message::seal_message`]) sealed to the
//!     recipient's device key. This is what the transport carries, so the relay sees only
//!     opaque bytes and CANNOT learn the SENDER (the sender's identity travels inside the
//!     sealed envelope). The recipient is still addressed by the route (the relay must
//!     know where to deliver), matching the sealed-sender model.
//!
//! Flow:
//!   * A recipient [`publish_card`](MercuryClient::publish_card)s a [`ContactCard`] (its
//!     signed pre-key bundle + sealed-sender device key) and retains the matching one-time
//!     ML-KEM keypair. The card can be handed over out-of-band, or published to the relay's
//!     prekey directory ([`publish_to_directory`](MercuryClient::publish_to_directory)) so a
//!     peer can discover it by account id ([`fetch_contact`](MercuryClient::fetch_contact),
//!     which re-verifies the card's signature + account-id binding — the untrusted relay is
//!     never taken on faith).
//!   * An initiator [`initiate`](MercuryClient::initiate)s with that card: it opens a
//!     ratchet session, builds the inner first-flight frame (carrying its own device key so
//!     the recipient can reply), seals it to the recipient's device key, and emits the blob.
//!   * The recipient [`receive`](MercuryClient::receive)s the blob: opens the outer
//!     envelope, accepts the session, and recovers the plaintext. Thereafter both ends use
//!     [`send`](MercuryClient::send) / `receive`.
//!
//! HONEST SCOPE (Milestone 1): the OUTER sealed-sender envelope is the CLASSICAL sealed box
//! (X25519 to the device key) — it hides the sender from a classical relay/observer; a
//! post-quantum sealed-sender (hybrid sealed box) is a later refinement, while the message
//! CONTENT is already fully post-quantum via the inner ratchet. The transport route is the
//! recipient account id (the relay learns the recipient, like any delivery service). The
//! outer envelope's policy epoch/sequence are fixed constants — ordering/replay protection
//! comes from the inner ratchet. Encrypted-at-rest persistence (so a conversation survives a
//! restart) is provided by [`MercuryClient::to_encrypted_pickle`] /
//! [`MercuryClient::from_encrypted_pickle`].

pub mod transport;
pub use transport::{
    InMemoryTransport, KtPublicKeys, PollAuth, RelayTransport, Transport, TransportError,
    UsernameClaimOutcome, UsernameLookup,
};

use std::collections::HashMap;

use mercury_core::{ClientMessageEnvelope, ConversationPolicyState, MessageKind, ProtocolSuite};
use mercury_keys::{
    DeviceKeyPair, DevicePublicKey, IdentityKeyPair, IdentityPublicKey, MlKemKeyPair, SafetyNumber,
    account_id_from_identity_key,
};
use mercury_message::{MessageError, from_bytes, open_message, seal_message, to_bytes};
use mercury_session::{
    HybridRatchetSession, LocalSessionAccount, RatchetMessage, SessionError, SignedPreKeyBundle,
};
use serde::{Deserialize, Serialize};

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use zeroize::Zeroize;

/// Inner client-frame wire version. Bumped if the framing changes, so an old/foreign frame
/// is rejected (fail-closed) rather than silently misparsed.
const CLIENT_WIRE_V1: u8 = 1;
/// Frame kind: the first message of a session.
const KIND_FIRST_FLIGHT: u8 = 0;
/// Frame kind: a subsequent message on an established session.
const KIND_NORMAL: u8 = 1;
/// Account-id / Curve25519 / device-key length.
const ID_LEN: usize = 32;
/// Max inner-frame plaintext the outer envelope will seal/open. Sized for the LARGEST legitimate
/// frame: an in-band attachment chunk (384 KiB raw → ~512 KiB base64 + JSON framing) or a
/// single-frame attachment (512 KiB raw → ~683 KiB base64 + framing) — 768 KiB covers both with
/// headroom, and the sealed result stays well inside the relay's 1 MiB ciphertext ceiling. Both
/// sides enforce the same cap on seal AND open (it is conversation policy), so frames between
/// up-to-date clients flow; text and protocol frames are unaffected (kilobytes at most).
const SEAL_MAX_PAYLOAD_LEN: i32 = 786_432;

/// An error from the client runtime. Coarse and free of key/plaintext material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientError {
    /// An underlying session/ratchet operation failed (bad bundle, decryption, replay,
    /// tampered first flight, etc.).
    Session(SessionError),
    /// The outer sealed-sender envelope failed to seal/open or its policy gate rejected
    /// (tampered/foreign envelope, wrong recipient, malformed wire).
    Envelope,
    /// A decoded inner frame was malformed (bad version/kind, too short, or self-addressed).
    Malformed,
    /// A normal message arrived for a peer with no established session.
    UnknownPeer,
    /// A first-flight arrived but this client has no retained one-time key to accept it.
    NoInboundKey,
    /// The encrypted client-state pickle could not be produced or restored (wrong key, or
    /// a corrupt/foreign/truncated blob).
    Pickle,
    /// A directory publish/fetch could not reach the relay (network/transport failure). The
    /// detailed transport error is surfaced by the transport layer, not embedded here.
    Transport,
    /// The directory has no contact card published for the requested account id.
    ContactNotFound,
    /// A fetched contact card's account id does not match the one requested — i.e. the relay
    /// returned a different account's card (a substitution attempt). Fails closed.
    DirectoryMismatch,
    /// A username lookup could not be verified: its key-transparency inclusion proof did not check
    /// out against the directory's VRF key (a forged or equivocating `handle -> account_id`
    /// binding). Fails closed — the binding is NOT trusted.
    UsernameProofInvalid,
    /// A username lookup could not be verified because no directory VRF key was available to check
    /// the proof against (neither pinned nor served). Fails closed rather than trust unverified.
    UsernameUnverifiable,
}

impl core::fmt::Display for ClientError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ClientError::Session(e) => write!(f, "session error: {e}"),
            ClientError::Envelope => f.write_str("sealed-sender envelope failed"),
            ClientError::Malformed => f.write_str("malformed client frame"),
            ClientError::UnknownPeer => f.write_str("no session established with peer"),
            ClientError::NoInboundKey => {
                f.write_str("no retained one-time key to accept a session")
            }
            ClientError::Pickle => f.write_str("client pickle invalid"),
            ClientError::Transport => f.write_str("directory transport failed"),
            ClientError::ContactNotFound => f.write_str("no contact card in the directory"),
            ClientError::DirectoryMismatch => {
                f.write_str("fetched contact card account id mismatch")
            }
            ClientError::UsernameProofInvalid => {
                f.write_str("username transparency proof did not verify")
            }
            ClientError::UsernameUnverifiable => {
                f.write_str("no directory key to verify the username proof")
            }
        }
    }
}

impl std::error::Error for ClientError {}

impl From<SessionError> for ClientError {
    fn from(e: SessionError) -> Self {
        ClientError::Session(e)
    }
}

impl From<MessageError> for ClientError {
    fn from(_: MessageError) -> Self {
        ClientError::Envelope
    }
}

/// A publishable contact card: the recipient's signed pre-key bundle (for the ratchet
/// session) plus its sealed-sender device public key (so a sender can seal the outer
/// envelope to it). Self-identifying via the bundle's account id + identity signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCard {
    /// The signed pre-key bundle that bootstraps the post-quantum ratchet.
    pub bundle: SignedPreKeyBundle,
    /// The recipient's sealed-sender device (X25519) public key.
    pub device_pub: [u8; 32],
}

impl ContactCard {
    /// The account id this card belongs to.
    pub fn account_id(&self) -> [u8; 32] {
        self.bundle.account_id
    }

    /// Serialize the card to JSON (an out-of-band file/QR exchange until a real directory
    /// exists).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("a ContactCard always serializes")
    }

    /// Parse a card from JSON. Fails closed on malformed input. The bundle's signature is
    /// verified later, when a session is initiated to it.
    pub fn from_json(s: &str) -> Result<Self, ClientError> {
        serde_json::from_str(s).map_err(|_| ClientError::Malformed)
    }
}

/// A parsed inner client frame (recovered AFTER opening the outer sealed-sender envelope).
enum ClientFrame {
    /// The first message of a session: the initiator's account id, its Curve25519 key (for
    /// the ratchet accept), its sealed-sender device key (so the recipient can reply), and
    /// the serialized first-flight ratchet message.
    FirstFlight {
        from: [u8; 32],
        curve25519: [u8; 32],
        sender_device_pub: [u8; 32],
        ratchet: Vec<u8>,
    },
    /// A subsequent message: the sender's account id + the serialized ratchet message.
    Normal { from: [u8; 32], ratchet: Vec<u8> },
}

/// Encode a first-flight frame:
/// `v1 || KIND_FIRST_FLIGHT || from(32) || curve(32) || sender_device(32) || ratchet`.
fn encode_first_flight(
    from: &[u8; 32],
    curve25519: &[u8; 32],
    sender_device_pub: &[u8; 32],
    ratchet: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + 3 * ID_LEN + ratchet.len());
    out.push(CLIENT_WIRE_V1);
    out.push(KIND_FIRST_FLIGHT);
    out.extend_from_slice(from);
    out.extend_from_slice(curve25519);
    out.extend_from_slice(sender_device_pub);
    out.extend_from_slice(ratchet);
    out
}

/// Encode a normal frame: `v1 || KIND_NORMAL || from(32) || ratchet`.
fn encode_normal(from: &[u8; 32], ratchet: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + ID_LEN + ratchet.len());
    out.push(CLIENT_WIRE_V1);
    out.push(KIND_NORMAL);
    out.extend_from_slice(from);
    out.extend_from_slice(ratchet);
    out
}

/// Parse an inner client frame. Never panics; fails closed ([`ClientError::Malformed`]) on
/// a wrong version byte, an unknown kind, or a frame too short for its declared fields.
fn decode(bytes: &[u8]) -> Result<ClientFrame, ClientError> {
    if bytes.len() < 2 || bytes[0] != CLIENT_WIRE_V1 {
        return Err(ClientError::Malformed);
    }
    let mut from = [0u8; 32];
    match bytes[1] {
        KIND_FIRST_FLIGHT => {
            let head = 2 + 3 * ID_LEN;
            if bytes.len() < head {
                return Err(ClientError::Malformed);
            }
            from.copy_from_slice(&bytes[2..2 + ID_LEN]);
            let mut curve = [0u8; 32];
            curve.copy_from_slice(&bytes[2 + ID_LEN..2 + 2 * ID_LEN]);
            let mut device = [0u8; 32];
            device.copy_from_slice(&bytes[2 + 2 * ID_LEN..head]);
            Ok(ClientFrame::FirstFlight {
                from,
                curve25519: curve,
                sender_device_pub: device,
                ratchet: bytes[head..].to_vec(),
            })
        }
        KIND_NORMAL => {
            let head = 2 + ID_LEN;
            if bytes.len() < head {
                return Err(ClientError::Malformed);
            }
            from.copy_from_slice(&bytes[2..head]);
            Ok(ClientFrame::Normal {
                from,
                ratchet: bytes[head..].to_vec(),
            })
        }
        _ => Err(ClientError::Malformed),
    }
}

/// The fixed envelope metadata for the outer sealed-sender layer. Ordering/replay are
/// enforced by the inner ratchet, so the policy epoch/sequence are constant.
fn seal_envelope() -> ClientMessageEnvelope {
    ClientMessageEnvelope {
        version: 1,
        suite: ProtocolSuite::ClassicalDev,
        conversation_id_len: 32,
        sender_account_id_len: 32,
        sender_device_id_len: 32,
        epoch: 1,
        sequence: 1,
        kind: MessageKind::Application,
        payload_len: 0,
        critical_flags: 0,
        noncritical_flags: 0,
    }
}

/// The fixed policy context both sender (seal) and recipient (open) use.
fn seal_context() -> ConversationPolicyState {
    ConversationPolicyState {
        expected_epoch: 1,
        expected_sequence: 1,
        minimum_suite: ProtocolSuite::ClassicalDev,
        max_payload_len: SEAL_MAX_PAYLOAD_LEN,
    }
}

/// Seal an inner frame to a recipient device key, producing the transport blob.
fn seal_to(recipient_device_pub: &[u8; 32], inner: &[u8]) -> Result<Vec<u8>, ClientError> {
    let recipient = DevicePublicKey::from_bytes(*recipient_device_pub);
    let sealed = seal_message(&recipient, inner, seal_envelope(), &seal_context())?;
    Ok(to_bytes(&sealed))
}

/// Open a transport blob with our device key, recovering the inner frame bytes.
fn open_with(device: &DeviceKeyPair, blob: &[u8]) -> Result<Vec<u8>, ClientError> {
    let sealed = from_bytes(blob)?;
    Ok(open_message(device, &sealed, &seal_context())?)
}

/// A Mercury 1:1 messenger client. Secret-bearing; no `Debug`.
pub struct MercuryClient {
    identity: IdentityKeyPair,
    account_id: [u8; 32],
    account: LocalSessionAccount,
    /// Stable sealed-sender device keypair (others seal the outer envelope to its public).
    device: DeviceKeyPair,
    /// Retained one-time ML-KEM keypair matching the most recent published card.
    inbound_pq: Option<MlKemKeyPair>,
    /// Established ratchet sessions, keyed by peer account id.
    sessions: HashMap<[u8; 32], HybridRatchetSession>,
    /// Peers' sealed-sender device public keys, keyed by peer account id (so we can seal
    /// replies/outbound messages to them).
    peer_device: HashMap<[u8; 32], [u8; 32]>,
}

impl Default for MercuryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MercuryClient {
    /// Create a client with a fresh Mercury identity, session account, and device key.
    pub fn new() -> Self {
        let identity = IdentityKeyPair::generate();
        let account_id = account_id_from_identity_key(&identity.public());
        Self {
            identity,
            account_id,
            account: LocalSessionAccount::new(),
            device: DeviceKeyPair::generate(),
            inbound_pq: None,
            sessions: HashMap::new(),
            peer_device: HashMap::new(),
        }
    }

    /// This client's account id.
    pub fn account_id(&self) -> [u8; 32] {
        self.account_id
    }

    /// Compute the mutual identity SAFETY NUMBER for out-of-band verification with the holder of
    /// `peer` (a verified contact card). Both sides derive the SAME 60-digit number from each
    /// other's identity key + stable account id (order-independent), so comparing the displayed
    /// numbers over a trusted channel (in person, or a recognizable voice/video call) detects a
    /// man-in-the-middle on the identity keys — the human backstop behind key transparency.
    /// Returns `None` only if the card's identity bytes are malformed, which cannot happen for a
    /// card that passed verification on add (its account id IS the hash of this identity key).
    pub fn safety_number(&self, peer: &ContactCard) -> Option<SafetyNumber> {
        let peer_identity = IdentityPublicKey::from_bytes(peer.bundle.identity_ed25519).ok()?;
        let me_identity = self.identity.public();
        Some(mercury_keys::safety_number(
            &self.account_id,
            &me_identity,
            &peer.account_id(),
            &peer_identity,
        ))
    }

    /// Whether a session is already established with `peer`.
    pub fn has_session(&self, peer: &[u8; 32]) -> bool {
        self.sessions.contains_key(peer)
    }

    /// True when this client holds NO retained one-time inbound key — i.e. the published contact
    /// card's key was consumed by an accepted first flight (or none was ever published), so a
    /// SECOND initiator fetching the old card would fail with [`ClientError::NoInboundKey`]. The
    /// caller should re-publish (mint a fresh card) so new sessions keep working — the app does
    /// this automatically after every accepted session.
    pub fn needs_republish(&self) -> bool {
        self.inbound_pq.is_none()
    }

    /// Sign a route-ownership poll authorization for OUR OWN route (this account id), proving to
    /// the relay that we are the recipient entitled to drain this route's queue. `timestamp_s`
    /// is the current Unix time; the relay enforces a freshness window around it. Pass the
    /// result to [`Transport::poll`].
    pub fn sign_poll_auth(&self, timestamp_s: i64) -> PollAuth {
        let pop_sig = mercury_keys::sign_relay_route_pop(
            &self.identity,
            &self.account_id,
            mercury_keys::RelayRouteOperation::Poll,
            timestamp_s,
        );
        PollAuth {
            identity_pub: *self.identity.public().as_bytes(),
            timestamp_s,
            pop_sig,
        }
    }

    /// Publish a [`ContactCard`] (a fresh signed pre-key bundle + this client's device key)
    /// and retain the matching one-time ML-KEM keypair so a peer can open a session.
    pub fn publish_card(&mut self) -> ContactCard {
        let pq = MlKemKeyPair::generate();
        let bundle = self.account.publish_bundle(&self.identity, &pq.public(), 1);
        self.inbound_pq = Some(pq);
        ContactCard {
            bundle,
            device_pub: *self.device.public().as_bytes(),
        }
    }

    /// Open a session to the holder of `card` and produce the FIRST-FLIGHT blob (inner
    /// ratchet first flight, sealed to the recipient's device key) protecting
    /// `first_message`. Returns `(recipient_account_id, blob)`.
    pub fn initiate(
        &mut self,
        card: &ContactCard,
        first_message: &[u8],
    ) -> Result<([u8; 32], Vec<u8>), ClientError> {
        let peer = card.bundle.account_id;
        let (session, first_flight) =
            HybridRatchetSession::initiate(&self.account, &card.bundle, first_message)?;
        let inner = encode_first_flight(
            &self.account_id,
            &self.account.curve25519_key(),
            self.device.public().as_bytes(),
            &first_flight.to_bytes(),
        );
        let blob = seal_to(&card.device_pub, &inner)?;
        self.sessions.insert(peer, session);
        self.peer_device.insert(peer, card.device_pub);
        Ok((peer, blob))
    }

    /// Encrypt `plaintext` to an established `peer`, returning the sealed blob to send.
    pub fn send(&mut self, peer: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, ClientError> {
        let from = self.account_id;
        let recipient_device = *self.peer_device.get(peer).ok_or(ClientError::UnknownPeer)?;
        let session = self
            .sessions
            .get_mut(peer)
            .ok_or(ClientError::UnknownPeer)?;
        let msg = session.encrypt(plaintext)?;
        let inner = encode_normal(&from, &msg.to_bytes());
        seal_to(&recipient_device, &inner)
    }

    /// Process an inbound blob: open the outer sealed-sender envelope, then accept a new
    /// session (first flight) or decrypt on an established one. Returns
    /// `(sender_account_id, plaintext)`. Fails closed throughout.
    pub fn receive(&mut self, blob: &[u8]) -> Result<([u8; 32], Vec<u8>), ClientError> {
        let inner = open_with(&self.device, blob)?;
        match decode(&inner)? {
            ClientFrame::FirstFlight {
                from,
                curve25519,
                sender_device_pub,
                ratchet,
            } => {
                // A flight that claims to come from ourselves is invalid; reject it before
                // consuming the one-time inbound key.
                if from == self.account_id {
                    return Err(ClientError::Malformed);
                }
                let msg = RatchetMessage::from_bytes(&ratchet)?;
                let pq = self.inbound_pq.take().ok_or(ClientError::NoInboundKey)?;
                let (session, plaintext) =
                    HybridRatchetSession::accept(&mut self.account, pq, curve25519, &msg)?;
                self.sessions.insert(from, session);
                self.peer_device.insert(from, sender_device_pub);
                Ok((from, plaintext))
            }
            ClientFrame::Normal { from, ratchet } => {
                let msg = RatchetMessage::from_bytes(&ratchet)?;
                let session = self
                    .sessions
                    .get_mut(&from)
                    .ok_or(ClientError::UnknownPeer)?;
                let plaintext = session.decrypt(&msg)?;
                Ok((from, plaintext))
            }
        }
    }
}

/// Directory operations: discover a peer's contact card through the (untrusted) relay
/// instead of exchanging it out-of-band. Publishing is authorized by an Ed25519 proof of
/// possession so a card can only be written to the slot derived from its signer's identity key
/// (the relay closes directory slot-squatting); the card itself is also self-authenticating.
/// Fetching is the security boundary — every fetched card is re-verified locally before it can
/// establish a session.
impl MercuryClient {
    /// Publish this client's contact card to the directory under its account id, so a peer
    /// who knows that account id can fetch it. The card carries the identity-signed pre-key
    /// bundle + sealed-sender device key; the relay stores it without trust.
    pub fn publish_to_directory(&mut self, transport: &dyn Transport) -> Result<(), ClientError> {
        let card = self.publish_card();
        let bytes = card.to_json().into_bytes();
        // Prove possession of our identity key so the relay authorizes writing to OUR slot
        // (the slot is derived from this key) — nobody else can publish over us.
        let pop_sig = mercury_keys::sign_directory_publish(&self.identity, &bytes);
        let identity_pub = *self.identity.public().as_bytes();
        transport
            .publish_card(&identity_pub, &bytes, &pop_sig)
            .map_err(|_| ClientError::Transport)
    }

    /// Fetch a peer's contact card from the directory and VERIFY it before returning. The
    /// relay is never trusted: this fails closed if the relay has no card
    /// ([`ClientError::ContactNotFound`]), returns a malformed/foreign card
    /// ([`ClientError::Malformed`]), returns a card whose account id is not the one requested
    /// ([`ClientError::DirectoryMismatch`] — a relay substitution attempt), or returns a card
    /// whose bundle fails its identity-signature / account-id-binding check
    /// ([`ClientError::Session`]). Only a card that passes all of these is returned, and it is
    /// then safe to pass to [`MercuryClient::initiate`] (which re-verifies the bundle anyway).
    pub fn fetch_contact(
        &self,
        transport: &dyn Transport,
        account_id: &[u8; 32],
    ) -> Result<ContactCard, ClientError> {
        let bytes = transport
            .fetch_card(account_id)
            .map_err(|_| ClientError::Transport)?
            .ok_or(ClientError::ContactNotFound)?;
        let json = std::str::from_utf8(&bytes).map_err(|_| ClientError::Malformed)?;
        let card = ContactCard::from_json(json)?;

        // 1. Identity binding to the REQUEST: the returned card must be for exactly the
        //    account id we asked for. Without this, a malicious relay could hand back some
        //    other user's (perfectly valid) card and we would establish a session with the
        //    wrong person.
        if card.account_id() != *account_id {
            return Err(ClientError::DirectoryMismatch);
        }

        // 2. Self-authentication: the bundle's Ed25519 identity signature must verify AND its
        //    account id must equal the hash of the signing identity key. This is what lets the
        //    relay stay untrusted — it cannot forge or alter a card without the identity key.
        card.bundle.verify().map_err(ClientError::Session)?;

        Ok(card)
    }

    /// Verify a fetched card is self-authenticating (identity signature + account-id binding) and
    /// return it, WITHOUT checking it against a requested account id. Used by the pairing-code path,
    /// where the account id is not known in advance — the caller learns it from the returned card.
    fn verify_card_bytes(&self, bytes: &[u8]) -> Result<ContactCard, ClientError> {
        let json = std::str::from_utf8(bytes).map_err(|_| ClientError::Malformed)?;
        let card = ContactCard::from_json(json)?;
        card.bundle.verify().map_err(ClientError::Session)?;
        Ok(card)
    }
}

/// Pairing-code + username (key-transparency) discovery: friendlier ways to reach a peer than a
/// 64-hex account id. Both paths re-verify everything client-side — the relay stays untrusted.
impl MercuryClient {
    /// Broker this client's contact card under a short-lived, single-use PAIRING CODE and return the
    /// code (which the user shares out-of-band). Mirrors [`MercuryClient::publish_to_directory`] but
    /// via the pairing broker; the publish is authorized by a pairing proof of possession.
    pub fn publish_pairing(&mut self, transport: &dyn Transport) -> Result<String, ClientError> {
        let card = self.publish_card();
        let bytes = card.to_json().into_bytes();
        let pop_sig = mercury_keys::sign_pairing_publish(&self.identity, &bytes);
        let identity_pub = *self.identity.public().as_bytes();
        transport
            .publish_pairing(&identity_pub, &bytes, &pop_sig)
            .map_err(|_| ClientError::Transport)
    }

    /// Fetch and VERIFY a peer's contact card brokered under a pairing `code`. Fails closed if the
    /// code is unknown/expired/used ([`ClientError::ContactNotFound`]), or the brokered card is
    /// malformed / does not self-authenticate. The returned card's `account_id()` is who the caller
    /// is about to add — the UI should surface it for confirmation.
    pub fn fetch_by_pairing(
        &self,
        transport: &dyn Transport,
        code: &str,
    ) -> Result<ContactCard, ClientError> {
        let bytes = transport
            .fetch_pairing(code)
            .map_err(|_| ClientError::Transport)?
            .ok_or(ClientError::ContactNotFound)?;
        self.verify_card_bytes(&bytes)
    }

    /// Claim `username` (any case / surrounding space) for THIS account, first-claimant-wins. Signs
    /// a username-claim proof of possession over the normalized handle so the relay can bind it to
    /// our account id. Returns the relay's outcome; an illegal handle is reported as
    /// [`UsernameClaimOutcome::Rejected`] without a network round-trip.
    pub fn claim_username(
        &self,
        transport: &dyn Transport,
        username: &str,
    ) -> Result<UsernameClaimOutcome, ClientError> {
        let Some(name) = mercury_keys::normalize_username(username) else {
            return Ok(UsernameClaimOutcome::Rejected);
        };
        let pop_sig = mercury_keys::sign_username_claim(&self.identity, &name);
        let identity_pub = *self.identity.public().as_bytes();
        transport
            .claim_username(&identity_pub, &name, &pop_sig)
            .map_err(|_| ClientError::Transport)
    }

    /// Resolve `username` to a VERIFIED contact card via key transparency. The lookup's inclusion
    /// proof is checked against `pinned_vrf` (the directory VRF key the caller pinned) — or, if
    /// `None`, against the key the relay serves now (TRUST-ON-FIRST-USE; the caller should pin
    /// `vrf_used` afterwards and pass it on later calls so a relay that LATER rotates its key is
    /// detected). Only after the proof verifies is the resolved account id used to fetch a contact
    /// card, which is INDEPENDENTLY self-authenticating. Fails closed at every step.
    pub fn resolve_username(
        &self,
        transport: &dyn Transport,
        username: &str,
        pinned_vrf: Option<&[u8]>,
        pinned_log: Option<&[u8; 32]>,
    ) -> Result<ResolvedUsername, ClientError> {
        let name = mercury_keys::normalize_username(username).ok_or(ClientError::Malformed)?;
        let lookup = transport
            .lookup_username(&name)
            .map_err(|_| ClientError::Transport)?
            .ok_or(ClientError::ContactNotFound)?;

        // When a LOG key is pinned (baked into the signed build for the default relay), the served
        // head must verify under it AND the resolved binding must be proven against THAT signed
        // head. The inclusion proof below is checked against `lookup.root_hash` at `lookup.epoch`,
        // so those must EXACTLY equal the `(tree_size, root)` the genuine log signed — otherwise the
        // pin proves nothing: a malicious log-key holder could serve a real signed head alongside a
        // FORGED lookup root + matching proof, and an `epoch <= tree_size` check would wave it
        // through. Require an exact match. (A username write racing between the lookup and STH
        // fetches trips this and fails CLOSED; the caller simply retries against the newer head.
        // This is signed-head binding, NOT independent witnesses — a log-key holder can still
        // equivocate per-client; disclosed in docs/USERNAMES.md.)
        if let Some(log_key) = pinned_log {
            let (tree_size, root, timestamp_s, sig) = transport
                .kt_signed_tree_head()
                .map_err(|_| ClientError::Transport)?
                .ok_or(ClientError::UsernameUnverifiable)?;
            let sth = mercury_kt::SignedTreeHead {
                tree_size,
                root_hash: root,
                timestamp_s,
            };
            if !mercury_kt::verify_signed_tree_head(log_key, &sth, &sig) {
                return Err(ClientError::UsernameProofInvalid);
            }
            let lookup_root: [u8; 32] = lookup
                .root_hash
                .as_slice()
                .try_into()
                .map_err(|_| ClientError::UsernameProofInvalid)?;
            if lookup.epoch != tree_size || lookup_root != root {
                return Err(ClientError::UsernameProofInvalid);
            }
        }

        // Choose the VRF key to verify against: the pinned one (strong), else the served one (TOFU).
        let (vrf_key, tofu) = match pinned_vrf {
            Some(k) => (k.to_vec(), false),
            None => {
                let keys = transport
                    .kt_public_keys()
                    .map_err(|_| ClientError::Transport)?
                    .ok_or(ClientError::UsernameUnverifiable)?;
                (keys.vrf_public_key, true)
            }
        };

        // Deserialize the served inclusion proof + reconstruct the checkpoint.
        let proof: mercury_kt::LookupProof = serde_json::from_str(&lookup.proof_json)
            .map_err(|_| ClientError::UsernameProofInvalid)?;
        let root: [u8; 32] = lookup
            .root_hash
            .as_slice()
            .try_into()
            .map_err(|_| ClientError::UsernameProofInvalid)?;
        let checkpoint = mercury_kt::EpochHash(lookup.epoch, root);
        let label = username_kt_label(&name);

        // The trust boundary: the proof must bind `username:<name> -> account_id` under the VRF key.
        if mercury_kt::verify_inclusion(&vrf_key, &checkpoint, &label, &lookup.account_id, proof)
            != mercury_kt::KeyTransparencyProofStatus::Verified
        {
            return Err(ClientError::UsernameProofInvalid);
        }

        // Verified: now fetch the contact card for the resolved account id (self-authenticating).
        let card = self.fetch_contact(transport, &lookup.account_id)?;
        Ok(ResolvedUsername {
            card,
            account_id: lookup.account_id,
            vrf_used: vrf_key,
            tofu,
        })
    }
}

/// A username resolved + cryptographically verified via key transparency.
#[derive(Clone, Debug)]
pub struct ResolvedUsername {
    /// The verified contact card the handle resolves to (safe to [`MercuryClient::initiate`] to).
    pub card: ContactCard,
    /// The account id the handle resolved to (= `card.account_id()`).
    pub account_id: [u8; 32],
    /// The VRF public key the inclusion proof was verified against. On a trust-on-first-use resolve
    /// the caller should PIN this and pass it back on later calls so a key rotation is detected.
    pub vrf_used: Vec<u8>,
    /// True iff this resolve trusted the relay-served VRF key on first use (no key was pinned).
    pub tofu: bool,
}

/// The KT log label a username is committed under. Kept in lock-step with the relay's
/// `mercury_relay::kt_username_label` (a `username:` prefix); duplicated here so the client crate
/// does not depend on the relay crate.
fn username_kt_label(normalized_username: &str) -> String {
    format!("username:{normalized_username}")
}

/// XChaCha20-Poly1305 nonce length for the at-rest pickle.
const PICKLE_NONCE_LEN: usize = 24;
/// Poly1305 tag length.
const PICKLE_TAG_LEN: usize = 16;

/// Serializable snapshot of a [`MercuryClient`]'s full state. Secret-bearing; the plaintext
/// form is sealed before it leaves [`MercuryClient::to_encrypted_pickle`], and zeroized.
#[derive(Serialize, Deserialize)]
struct ClientSnapshot {
    identity: [u8; 32],
    device: [u8; 32],
    inbound_pq_seed: Option<Vec<u8>>,
    account_pickle: String,
    sessions: Vec<([u8; 32], Vec<u8>)>,
    peer_device: Vec<([u8; 32], [u8; 32])>,
}

impl Drop for ClientSnapshot {
    fn drop(&mut self) {
        self.identity.zeroize();
        self.device.zeroize();
        if let Some(seed) = self.inbound_pq_seed.as_mut() {
            seed.zeroize();
        }
        // account_pickle + the session pickles are already encrypted.
    }
}

impl MercuryClient {
    /// Serialize the FULL client state to an encrypted-at-rest pickle under `key`, so a
    /// conversation survives a restart. Identity, device, the retained one-time key, the
    /// session account, every established ratchet session, and the peer-device map are all
    /// captured; the secrets are sealed with XChaCha20-Poly1305 under `key`.
    pub fn to_encrypted_pickle(&self, key: &[u8; 32]) -> Vec<u8> {
        let snapshot = ClientSnapshot {
            identity: self.identity.to_secret_bytes(),
            device: self.device.to_secret_bytes(),
            inbound_pq_seed: self.inbound_pq.as_ref().map(|k| k.to_secret_bytes()),
            account_pickle: self.account.to_encrypted_pickle(key),
            sessions: self
                .sessions
                .iter()
                .map(|(id, s)| (*id, s.to_encrypted_pickle(key)))
                .collect(),
            peer_device: self.peer_device.iter().map(|(a, d)| (*a, *d)).collect(),
        };
        let mut plain = serde_json::to_vec(&snapshot).expect("the client snapshot serializes");
        let blob = pickle_seal(key, &plain);
        plain.zeroize();
        blob
    }

    /// Restore a client from [`MercuryClient::to_encrypted_pickle`]. Never panics; fails
    /// closed ([`ClientError::Pickle`]) on a wrong key or a corrupt/foreign blob.
    pub fn from_encrypted_pickle(blob: &[u8], key: &[u8; 32]) -> Result<Self, ClientError> {
        let mut plain = pickle_open(key, blob)?;
        let snap: ClientSnapshot =
            serde_json::from_slice(&plain).map_err(|_| ClientError::Pickle)?;
        plain.zeroize();

        let identity = IdentityKeyPair::from_secret_bytes(snap.identity);
        let account_id = account_id_from_identity_key(&identity.public());
        let device = DeviceKeyPair::from_secret_bytes(snap.device);
        let account = LocalSessionAccount::from_encrypted_pickle(&snap.account_pickle, key)
            .map_err(|_| ClientError::Pickle)?;
        let inbound_pq = match snap.inbound_pq_seed.as_ref() {
            Some(b) => Some(MlKemKeyPair::from_secret_bytes(b).map_err(|_| ClientError::Pickle)?),
            None => None,
        };
        let mut sessions = HashMap::new();
        for (id, session_blob) in snap.sessions.iter() {
            let session = HybridRatchetSession::from_encrypted_pickle(session_blob, key)
                .map_err(|_| ClientError::Pickle)?;
            sessions.insert(*id, session);
        }
        let peer_device = snap.peer_device.iter().map(|(a, d)| (*a, *d)).collect();

        Ok(MercuryClient {
            identity,
            account_id,
            account,
            device,
            inbound_pq,
            sessions,
            peer_device,
        })
    }
}

/// Seal a serialized client snapshot with XChaCha20-Poly1305 under the at-rest key.
fn pickle_seal(key: &[u8; 32], plaintext: &[u8]) -> Vec<u8> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(key).expect("32-byte key is valid for XChaCha20Poly1305");
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ct = cipher
        .encrypt(&nonce, plaintext)
        .expect("XChaCha20-Poly1305 encryption does not fail for a valid key/nonce");
    let mut out = Vec::with_capacity(PICKLE_NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

/// Reverse of [`pickle_seal`]. Fails closed on a short frame, wrong key, or tampering.
fn pickle_open(key: &[u8; 32], blob: &[u8]) -> Result<Vec<u8>, ClientError> {
    if blob.len() < PICKLE_NONCE_LEN + PICKLE_TAG_LEN {
        return Err(ClientError::Pickle);
    }
    let (nonce_bytes, ct) = blob.split_at(PICKLE_NONCE_LEN);
    let cipher =
        XChaCha20Poly1305::new_from_slice(key).expect("32-byte key is valid for XChaCha20Poly1305");
    cipher
        .decrypt(XNonce::from_slice(nonce_bytes), ct)
        .map_err(|_| ClientError::Pickle)
}

/// Seal an arbitrary at-rest blob with XChaCha20-Poly1305 under a 32-byte device key — the same
/// construction used by the client pickle (a fresh random nonce is prepended to the ciphertext).
/// For callers that persist their own state layered above a [`MercuryClient`] (e.g. an application
/// controller's snapshot, whose plaintext message log must be sealed at rest). The crypto lives
/// here, not in the caller. `key` must come from a real device secret (OS keychain); never persist it.
pub fn seal_blob(key: &[u8; 32], plaintext: &[u8]) -> Vec<u8> {
    pickle_seal(key, plaintext)
}

/// Reverse of [`seal_blob`]. Fails closed ([`ClientError::Pickle`]) on a short frame, the wrong key,
/// or any tampering.
pub fn open_blob(key: &[u8; 32], blob: &[u8]) -> Result<Vec<u8>, ClientError> {
    pickle_open(key, blob)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_is_fail_closed() {
        assert!(decode(&[]).is_err());
        assert!(decode(&[CLIENT_WIRE_V1]).is_err()); // too short for a kind
        assert!(decode(&[2u8, KIND_NORMAL, 0, 0]).is_err()); // wrong version
        assert!(decode(&[CLIENT_WIRE_V1, 9]).is_err()); // unknown kind
        assert!(decode(&[CLIENT_WIRE_V1, KIND_FIRST_FLIGHT, 0, 0]).is_err()); // short first-flight
        // A well-formed normal frame round-trips.
        let n = encode_normal(&[7u8; 32], &[1, 2, 3, 4]);
        assert!(matches!(decode(&n), Ok(ClientFrame::Normal { .. })));
    }

    #[test]
    fn a_self_addressed_first_flight_is_rejected_without_burning_the_key() {
        // White-box: a genuinely-sealed envelope whose INNER first-flight claims `from` ==
        // the receiver must be rejected before the one-time inbound key is consumed.
        let mut bob = MercuryClient::new();
        let card = bob.publish_card();
        let b_id = bob.account_id();
        let inner = encode_first_flight(&b_id, &[0u8; 32], &[0u8; 32], &[0u8; 200]);
        let blob = seal_to(&card.device_pub, &inner).expect("seal");
        assert!(matches!(bob.receive(&blob), Err(ClientError::Malformed)));

        // The retained key survived: a genuine inbound session still establishes.
        let mut alice = MercuryClient::new();
        let (_r, flight) = alice.initiate(&card, b"genuine after self-flight").unwrap();
        assert_eq!(
            bob.receive(&flight).unwrap().1,
            b"genuine after self-flight"
        );
    }

    #[test]
    fn safety_number_is_mutual_and_order_independent() {
        // Both parties derive the SAME 60-digit number from each other's contact card, regardless
        // of who computes it — the property that makes an out-of-band comparison meaningful (and
        // what the UI verification screen compares).
        let mut alice = MercuryClient::new();
        let mut bob = MercuryClient::new();
        let card_a = alice.publish_card();
        let card_b = bob.publish_card();

        let sn_alice = alice
            .safety_number(&card_b)
            .expect("alice derives a safety number");
        let sn_bob = bob
            .safety_number(&card_a)
            .expect("bob derives a safety number");

        assert_eq!(
            sn_alice.digits(),
            sn_bob.digits(),
            "both sides must see the same safety number"
        );
        assert_eq!(sn_alice.digits().len(), 60);
        assert_ne!(
            sn_alice.digits(),
            "0".repeat(60),
            "not the degenerate all-zero number"
        );
    }
}
