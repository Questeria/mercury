//! Post-quantum asymmetric ratchet for Mercury 1:1 sessions — the break-in-recovery
//! (post-compromise security) layer.
//!
//! [`HybridSession`](crate::HybridSession) (the continuous layer) already makes every
//! 1:1 message post-quantum-confidential and post-quantum-forward-secret, but its PQ
//! chain is seeded ONCE from the session bootstrap and never re-keyed, so it has no
//! post-compromise security on the post-quantum side. This module closes that gap.
//!
//! [`PqRatchet`] is a Double Ratchet (Signal's construction) with the Diffie–Hellman
//! ratchet replaced by an **ML-KEM-768 KEM ratchet**: each time the direction of the
//! conversation turns, the sender performs a FRESH ML-KEM encapsulation to the peer's
//! current ratchet public key and generates a fresh ratchet keypair of its own, mixing
//! the encapsulated secret into a root key. Because each epoch injects fresh
//! post-quantum entropy that an attacker holding only old state cannot reproduce, the
//! conversation **self-heals** after a state compromise — one round-trip after the
//! compromised party sends a fresh ratchet key, the post-quantum keys are again unknown
//! to the attacker. This mirrors how the classical DH ratchet gives Olm its
//! post-compromise security, but against a quantum adversary.
//!
//! Properties of [`PqRatchet`] (post-quantum side only; combine with the inner Olm
//! ratchet via [`HybridRatchetSession`] for the hybrid guarantee):
//!   * Confidentiality + integrity of every message under ML-KEM-768.
//!   * Forward secrecy: per-message keys come from a one-way chain and are zeroized
//!     after use; old chain/root keys are zeroized when superseded.
//!   * Post-compromise security: fresh ML-KEM entropy enters the root every epoch.
//!   * Replay rejection, and bounded out-of-order delivery within AND across epochs
//!     (Signal-style skipped-message-key cache), with a two-phase commit so a forged
//!     message can neither be decrypted nor desync the ratchet.
//!
//! HONEST SCOPE: this is a bespoke protocol ASSEMBLED FROM AUDITED PRIMITIVES (ml-kem
//! 0.3, hkdf-sha256, XChaCha20-Poly1305 — the same RustCrypto primitives used
//! elsewhere in Mercury). It is NOT a formally-verified protocol like MLS, and its
//! post-compromise/forward-secrecy guarantees rest on the standard Double Ratchet
//! security analysis with a KEM substituted for the DH (e.g. the line of work on
//! post-quantum Signal / Apple PQ3), not on an in-crate proof. It is purely ADDITIVE:
//! it does not touch the inner Olm ratchet or any classical/first-flight/continuous
//! path. When layered over Olm by [`HybridRatchetSession`], a message stays confidential
//! as long as EITHER the classical Olm ratchet OR the ML-KEM ratchet holds, with both
//! layers retaining forward secrecy AND post-compromise security.
//!
//! TRUST MODEL: only the INITIAL responder ratchet key (the pre-key bundle's ML-KEM key)
//! is identity-signed. Each subsequent per-epoch ratchet public key is authenticated by
//! CONTINUITY, not a fresh signature: a substituted epoch key is adopted (`peer_pub` is
//! replaced) only AFTER the message carrying it authenticates under a root an attacker
//! cannot forge — peer-key adoption is gated on the outer AEAD succeeding. This is the
//! standard Double Ratchet / Olm trust model; it is not a per-epoch signature scheme.

use std::collections::BTreeMap;

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use mercury_keys::{MlKemKeyPair, MlKemPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::{
    LocalSessionAccount, MercurySession, SessionCiphertext, SessionError, SignedPreKeyBundle,
};

/// ML-KEM-768 public (encapsulation) key length (FIPS 203).
const ML_KEM_PK_LEN: usize = 1184;
/// ML-KEM-768 ciphertext length (FIPS 203).
const ML_KEM_CT_LEN: usize = 1088;
/// XChaCha20-Poly1305 nonce length.
const NONCE_LEN: usize = 24;
/// Poly1305 tag length.
const TAG_LEN: usize = 16;

/// HKDF info for the PUBLIC initial root derivation (binds the transcript; carries no
/// secrecy — security comes from the first encapsulation).
const ROOT_SEED_DOMAIN: &[u8] = b"mercury-pq-ratchet-root-seed-v1";
/// HKDF info for a root step `(root, ss) -> (root', chain_key)`.
const ROOT_STEP_INFO: &[u8] = b"mercury-pq-ratchet-root-step-v1";
/// HKDF info deriving a per-message key from a chain key.
const CHAIN_MSG_INFO: &[u8] = b"mercury-pq-ratchet-chain-msg-v1";
/// HKDF info advancing a chain key.
const CHAIN_STEP_INFO: &[u8] = b"mercury-pq-ratchet-chain-step-v1";
/// Domain for the per-message outer AEAD associated data.
const MSG_AAD_DOMAIN: &[u8] = b"mercury-pq-ratchet-msg-aad-v1";
/// Domain for the epoch identifier `sha256(domain || epoch_public_key)`.
const EPOCH_ID_DOMAIN: &[u8] = b"mercury-pq-ratchet-epoch-id-v1";

/// Max messages that may be skipped within one chain in a single step (bounds the work
/// a peer can force per message). A larger jump is rejected.
pub const RATCHET_MAX_SKIP: u32 = 1024;
/// Max total skipped message keys retained across all epochs (bounds memory). When
/// exceeded, the lowest-keyed entries are evicted (and zeroized).
const RATCHET_MAX_SKIPPED_KEYS: usize = 2048;
/// Wire-format version byte for the ratchet-message codec. Bumped if the layout changes,
/// so an old/foreign frame is rejected (fail-closed) rather than silently misparsed.
const RATCHET_WIRE_V1: u8 = 1;

/// HKDF-SHA256 expand from a 32-byte chain key (used directly as the PRK), KDF-CK style.
fn kdf(prk: &[u8; 32], info: &[u8]) -> [u8; 32] {
    let hkdf =
        Hkdf::<Sha256>::from_prk(prk).expect("a 32-byte chain key is a valid HKDF-SHA256 PRK");
    let mut out = [0u8; 32];
    hkdf.expand(info, &mut out)
        .expect("32 is a valid HKDF-SHA256 output length");
    out
}

/// Root step: mix an encapsulated secret `kem_ss` into `root`, yielding a new root and a
/// fresh chain key. `(root', chain_key) = HKDF(salt=root, ikm=kem_ss)`.
fn root_step(root: &[u8; 32], kem_ss: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hkdf = Hkdf::<Sha256>::new(Some(root), kem_ss);
    let mut okm = [0u8; 64];
    hkdf.expand(ROOT_STEP_INFO, &mut okm)
        .expect("64 is a valid HKDF-SHA256 output length");
    let mut new_root = [0u8; 32];
    let mut chain_key = [0u8; 32];
    new_root.copy_from_slice(&okm[..32]);
    chain_key.copy_from_slice(&okm[32..]);
    okm.zeroize();
    (new_root, chain_key)
}

/// Public initial root: deterministic over the responder's bundle ML-KEM key, so both
/// ends start from the same value. Carries no secrecy on its own.
fn seed_root(responder_pq_pub: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(Some(ROOT_SEED_DOMAIN), responder_pq_pub);
    let mut root = [0u8; 32];
    hkdf.expand(ROOT_SEED_DOMAIN, &mut root)
        .expect("32 is a valid HKDF-SHA256 output length");
    root
}

/// Stable 32-byte identifier of an epoch, derived from its ratchet public key.
fn epoch_id(epoch_pub: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(EPOCH_ID_DOMAIN);
    hasher.update(epoch_pub);
    hasher.finalize().into()
}

/// Associated data binding a message to its epoch + position so it cannot be replayed at
/// another index or transplanted to another epoch: `domain || epoch_id || index ||
/// prev_chain_len`.
fn msg_aad(eid: &[u8; 32], index: u32, prev_chain_len: u32) -> Vec<u8> {
    let mut aad = Vec::with_capacity(MSG_AAD_DOMAIN.len() + 32 + 4 + 4);
    aad.extend_from_slice(MSG_AAD_DOMAIN);
    aad.extend_from_slice(eid);
    aad.extend_from_slice(&index.to_le_bytes());
    aad.extend_from_slice(&prev_chain_len.to_le_bytes());
    aad
}

/// Wrap `inner` under message key `mk`: `nonce(24) || XChaCha20-Poly1305(inner)`.
fn wrap(mk: &[u8; 32], eid: &[u8; 32], index: u32, prev_chain_len: u32, inner: &[u8]) -> Vec<u8> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(mk).expect("32-byte key is valid for XChaCha20Poly1305");
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let aad = msg_aad(eid, index, prev_chain_len);
    let ct = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: inner,
                aad: &aad,
            },
        )
        .expect("XChaCha20-Poly1305 encryption does not fail for a valid key/nonce");
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

/// Reverse of [`wrap`]. Fails closed on a short frame, wrong key, or any tampering.
fn unwrap(
    mk: &[u8; 32],
    eid: &[u8; 32],
    index: u32,
    prev_chain_len: u32,
    outer: &[u8],
) -> Result<Vec<u8>, SessionError> {
    if outer.len() < NONCE_LEN + TAG_LEN {
        return Err(SessionError::RatchetFailed);
    }
    let (nonce_bytes, body) = outer.split_at(NONCE_LEN);
    let cipher =
        XChaCha20Poly1305::new_from_slice(mk).expect("32-byte key is valid for XChaCha20Poly1305");
    let nonce = XNonce::from_slice(nonce_bytes);
    let aad = msg_aad(eid, index, prev_chain_len);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: body,
                aad: &aad,
            },
        )
        .map_err(|_| SessionError::RatchetFailed)
}

/// A post-quantum ratchet message header. Carries the sender's current ratchet public
/// key and the ML-KEM ciphertext that opened the sender's current epoch (repeated on
/// every message of the epoch so a lost first message does not strand the receiver),
/// plus the length of the sender's previous sending chain and this message's index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RatchetHeader {
    /// The sender's current ratchet ML-KEM public key (1184 bytes).
    pub epoch_pub: Vec<u8>,
    /// ML-KEM ciphertext encapsulated to the receiver's previous ratchet key (1088 bytes).
    pub kem_ct: Vec<u8>,
    /// Number of messages the sender sent in its previous sending chain.
    pub prev_chain_len: u32,
    /// This message's index within the sender's current sending chain.
    pub index: u32,
}

/// A post-quantum ratchet message: header plus the wrapped payload (`nonce || AEAD`).
/// The payload is ciphertext, so `Debug`/`Clone` are safe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RatchetMessage {
    /// Routing/ratchet header (not secret).
    pub header: RatchetHeader,
    /// `nonce(24) || XChaCha20-Poly1305(payload)`.
    pub outer: Vec<u8>,
}

impl RatchetMessage {
    /// Canonical wire bytes:
    /// `version(1) || epoch_pub(1184) || kem_ct(1088) || prev_chain_len(4 LE) || index(4 LE) || outer`.
    /// The ML-KEM public key and ciphertext are fixed-length, so all fields sit at fixed
    /// offsets and `outer` is the remainder.
    pub fn to_bytes(&self) -> Vec<u8> {
        let h = &self.header;
        let mut out =
            Vec::with_capacity(1 + ML_KEM_PK_LEN + ML_KEM_CT_LEN + 4 + 4 + self.outer.len());
        out.push(RATCHET_WIRE_V1);
        out.extend_from_slice(&h.epoch_pub);
        out.extend_from_slice(&h.kem_ct);
        out.extend_from_slice(&h.prev_chain_len.to_le_bytes());
        out.extend_from_slice(&h.index.to_le_bytes());
        out.extend_from_slice(&self.outer);
        out
    }

    /// Parse wire bytes produced by [`RatchetMessage::to_bytes`]. Never panics; fails closed
    /// ([`SessionError::RatchetFailed`]) on a wrong version byte or a frame too short to hold
    /// the fixed header and a minimal outer layer. A wrong-length `epoch_pub`/`kem_ct` is
    /// impossible by construction (both are framed at fixed offsets).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SessionError> {
        let fixed = 1 + ML_KEM_PK_LEN + ML_KEM_CT_LEN + 4 + 4;
        let min = fixed + NONCE_LEN + TAG_LEN;
        if bytes.len() < min || bytes[0] != RATCHET_WIRE_V1 {
            return Err(SessionError::RatchetFailed);
        }
        let mut o = 1;
        let epoch_pub = bytes[o..o + ML_KEM_PK_LEN].to_vec();
        o += ML_KEM_PK_LEN;
        let kem_ct = bytes[o..o + ML_KEM_CT_LEN].to_vec();
        o += ML_KEM_CT_LEN;
        let prev_chain_len =
            u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        o += 4;
        let index = u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        o += 4;
        Ok(RatchetMessage {
            header: RatchetHeader {
                epoch_pub,
                kem_ct,
                prev_chain_len,
                index,
            },
            outer: bytes[o..].to_vec(),
        })
    }
}

/// A tentatively-derived skipped message key awaiting commit: `((epoch_id, index), key)`.
type SkippedEntry = (([u8; 32], u32), [u8; 32]);

/// One sending epoch's chain plus the header material repeated on each of its messages.
struct SendChain {
    chain_key: [u8; 32],
    index: u32,
    epoch_pub: Vec<u8>,
    epoch_ct: Vec<u8>,
}

impl Drop for SendChain {
    fn drop(&mut self) {
        self.chain_key.zeroize();
    }
}

/// One receiving epoch's chain.
struct RecvChain {
    chain_key: [u8; 32],
    index: u32,
    epoch_id: [u8; 32],
}

impl Drop for RecvChain {
    fn drop(&mut self) {
        self.chain_key.zeroize();
    }
}

/// A post-quantum Double Ratchet over opaque byte payloads (ML-KEM-768 KEM ratchet).
///
/// Construct with [`PqRatchet::initiator`] (the side that fetched the peer's bundle) or
/// [`PqRatchet::responder`] (the side whose bundle was fetched), then
/// [`PqRatchet::encrypt`] / [`PqRatchet::decrypt`]. Secret-bearing; no `Debug`.
pub struct PqRatchet {
    root: [u8; 32],
    /// Our current ratchet keypair (others encapsulate to its public key). `None` only
    /// for a fresh initiator before its first send.
    self_kp: Option<MlKemKeyPair>,
    /// The peer's current ratchet public key (we encapsulate to it). `None` for a fresh
    /// responder before it has received anything.
    peer_pub: Option<MlKemPublicKey>,
    /// Whether the next [`encrypt`](Self::encrypt) must open a new sending epoch (true
    /// after we learn a new peer ratchet key, or at first send).
    need_send_ratchet: bool,
    send: Option<SendChain>,
    recv: Option<RecvChain>,
    /// Length of our current sending chain, stamped into outgoing headers as the peer's
    /// skip target when it ratchets past this epoch.
    prev_send_count: u32,
    /// Skipped message keys for out-of-order delivery, keyed by `(epoch_id, index)`. The
    /// value pairs the message key with a monotonic insertion sequence, so on overflow the
    /// cache evicts the OLDEST entry (chronological / FIFO) rather than a hash-ordered one
    /// — a pathological reorder then loses the longest-outstanding message first.
    skipped: BTreeMap<([u8; 32], u32), ([u8; 32], u64)>,
    /// Monotonic counter stamped into each skipped entry to give a stable FIFO eviction
    /// order independent of the (hashed) epoch identifier.
    skip_seq: u64,
}

impl Drop for PqRatchet {
    fn drop(&mut self) {
        self.root.zeroize();
        for (_, (key, _)) in self.skipped.iter_mut() {
            key.zeroize();
        }
    }
}

impl PqRatchet {
    /// The INITIATOR side: the party that fetched and verified the peer's signed pre-key
    /// bundle. `responder_pq_pub` is the peer bundle's ML-KEM public key (already
    /// identity-authenticated by the caller). The initiator's first [`encrypt`] opens
    /// epoch 1 by encapsulating to this key.
    pub fn initiator(responder_pq_pub: &MlKemPublicKey) -> PqRatchet {
        let root = seed_root(&responder_pq_pub.to_bytes());
        PqRatchet {
            root,
            self_kp: None,
            peer_pub: Some(responder_pq_pub.clone()),
            need_send_ratchet: true,
            send: None,
            recv: None,
            prev_send_count: 0,
            skipped: BTreeMap::new(),
            skip_seq: 0,
        }
    }

    /// The RESPONDER side: the party whose bundle was fetched. `bundle_pq_keypair` is the
    /// ML-KEM keypair whose public half is in that bundle; it is the responder's initial
    /// ratchet keypair (the initiator's epoch 1 encapsulates to it). The responder cannot
    /// send until it has received the initiator's first message.
    pub fn responder(bundle_pq_keypair: MlKemKeyPair) -> PqRatchet {
        let root = seed_root(&bundle_pq_keypair.public().to_bytes());
        PqRatchet {
            root,
            self_kp: Some(bundle_pq_keypair),
            peer_pub: None,
            need_send_ratchet: true,
            send: None,
            recv: None,
            prev_send_count: 0,
            skipped: BTreeMap::new(),
            skip_seq: 0,
        }
    }

    /// Open a new sending epoch: encapsulate to the peer's current ratchet key, mix the
    /// secret into the root, and generate a fresh ratchet keypair of our own.
    fn send_ratchet(&mut self) -> Result<(), SessionError> {
        let peer = self.peer_pub.as_ref().ok_or(SessionError::RatchetFailed)?;
        let (epoch_ct, mut ss) = peer.encapsulate();
        let (new_root, chain_key) = root_step(&self.root, &ss);
        ss.zeroize();
        let prev = self.send.as_ref().map(|s| s.index).unwrap_or(0);
        let new_kp = MlKemKeyPair::generate();
        let epoch_pub = new_kp.public().to_bytes();
        self.root.zeroize();
        self.root = new_root;
        self.self_kp = Some(new_kp);
        self.prev_send_count = prev;
        self.send = Some(SendChain {
            chain_key,
            index: 0,
            epoch_pub,
            epoch_ct,
        });
        self.need_send_ratchet = false;
        Ok(())
    }

    /// Encrypt `plaintext`, advancing the sending chain (opening a new epoch first if we
    /// have learned a new peer ratchet key since our last send).
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<RatchetMessage, SessionError> {
        if self.need_send_ratchet || self.send.is_none() {
            self.send_ratchet()?;
        }
        let send = self.send.as_mut().expect("send chain established above");
        let index = send.index;
        let next_index = index.checked_add(1).ok_or(SessionError::RatchetFailed)?;
        let mut mk = kdf(&send.chain_key, CHAIN_MSG_INFO);
        let next_ck = kdf(&send.chain_key, CHAIN_STEP_INFO);
        send.chain_key.zeroize();
        send.chain_key = next_ck;
        send.index = next_index;

        let eid = epoch_id(&send.epoch_pub);
        let header = RatchetHeader {
            epoch_pub: send.epoch_pub.clone(),
            kem_ct: send.epoch_ct.clone(),
            prev_chain_len: self.prev_send_count,
            index,
        };
        let outer = wrap(&mk, &eid, index, header.prev_chain_len, plaintext);
        mk.zeroize();
        Ok(RatchetMessage { header, outer })
    }

    /// Insert a skipped message key, enforcing the global cache bound by evicting +
    /// zeroizing the OLDEST (lowest insertion sequence) entries on overflow, so a
    /// pathological reorder loses the longest-outstanding key first (FIFO).
    fn store_skipped(&mut self, eid: [u8; 32], index: u32, key: [u8; 32]) {
        let seq = self.skip_seq;
        self.skip_seq = self.skip_seq.wrapping_add(1);
        if let Some((mut old, _)) = self.skipped.insert((eid, index), (key, seq)) {
            old.zeroize();
        }
        while self.skipped.len() > RATCHET_MAX_SKIPPED_KEYS {
            let victim = self
                .skipped
                .iter()
                .min_by_key(|(_, (_, s))| *s)
                .map(|(k, _)| *k);
            match victim {
                Some(k) => {
                    if let Some((mut key, _)) = self.skipped.remove(&k) {
                        key.zeroize();
                    }
                }
                None => break,
            }
        }
    }

    /// Decrypt a [`RatchetMessage`]. Fails closed and advances NO state on a replay, a
    /// too-large gap, a tampered/foreign header or payload, or an un-decapsulatable epoch.
    pub fn decrypt(&mut self, message: &RatchetMessage) -> Result<Vec<u8>, SessionError> {
        let header = &message.header;
        // Reject malformed header sizes before any crypto.
        if header.epoch_pub.len() != ML_KEM_PK_LEN || header.kem_ct.len() != ML_KEM_CT_LEN {
            return Err(SessionError::RatchetFailed);
        }
        let inc_eid = epoch_id(&header.epoch_pub);

        // 1. A skipped (out-of-order) message we already derived a key for.
        if self.skipped.contains_key(&(inc_eid, header.index)) {
            let mut mk = self
                .skipped
                .get(&(inc_eid, header.index))
                .expect("checked present")
                .0;
            let result = unwrap(
                &mk,
                &inc_eid,
                header.index,
                header.prev_chain_len,
                &message.outer,
            );
            mk.zeroize();
            let pt = result?;
            if let Some((mut k, _)) = self.skipped.remove(&(inc_eid, header.index)) {
                k.zeroize();
            }
            return Ok(pt);
        }

        let same_epoch = self
            .recv
            .as_ref()
            .map(|r| r.epoch_id == inc_eid)
            .unwrap_or(false);

        if same_epoch {
            self.decrypt_same_epoch(&inc_eid, header, &message.outer)
        } else {
            self.decrypt_new_epoch(&inc_eid, header, &message.outer)
        }
    }

    /// Decrypt a message belonging to the CURRENT receiving epoch (no KEM step).
    fn decrypt_same_epoch(
        &mut self,
        inc_eid: &[u8; 32],
        header: &RatchetHeader,
        outer: &[u8],
    ) -> Result<Vec<u8>, SessionError> {
        let recv = self.recv.as_ref().expect("same_epoch implies a recv chain");
        if header.index < recv.index {
            // Already consumed and not retained as skipped -> replay.
            return Err(SessionError::RatchetFailed);
        }
        let gap = header.index - recv.index;
        if gap > RATCHET_MAX_SKIP {
            return Err(SessionError::RatchetFailed);
        }
        let new_index = header
            .index
            .checked_add(1)
            .ok_or(SessionError::RatchetFailed)?;

        // Walk a LOCAL copy of the chain to the target, collecting skipped keys. `self`
        // is untouched until the AEAD authenticates.
        let mut ck = recv.chain_key;
        let mut pending: Vec<(u32, [u8; 32])> = Vec::new();
        let mut i = recv.index;
        while i < header.index {
            let mk = kdf(&ck, CHAIN_MSG_INFO);
            let next = kdf(&ck, CHAIN_STEP_INFO);
            pending.push((i, mk));
            ck.zeroize();
            ck = next;
            i += 1;
        }
        let mut mk = kdf(&ck, CHAIN_MSG_INFO);
        let new_ck = kdf(&ck, CHAIN_STEP_INFO);
        ck.zeroize();

        let result = unwrap(&mk, inc_eid, header.index, header.prev_chain_len, outer);
        mk.zeroize();
        let pt = match result {
            Ok(pt) => pt,
            Err(e) => {
                for (_, mut k) in pending {
                    k.zeroize();
                }
                let mut new_ck = new_ck;
                new_ck.zeroize();
                return Err(e);
            }
        };

        // Authenticated: commit the chain advance + any skipped keys.
        for (idx, key) in pending {
            self.store_skipped(*inc_eid, idx, key);
        }
        let recv = self.recv.as_mut().expect("recv chain still present");
        recv.chain_key.zeroize();
        recv.chain_key = new_ck;
        recv.index = new_index;
        Ok(pt)
    }

    /// Decrypt a message that opens a NEW receiving epoch: skip the tail of the current
    /// receiving chain, perform the KEM receiving ratchet step, then derive the key — all
    /// computed locally and committed only after the AEAD authenticates.
    fn decrypt_new_epoch(
        &mut self,
        inc_eid: &[u8; 32],
        header: &RatchetHeader,
        outer: &[u8],
    ) -> Result<Vec<u8>, SessionError> {
        // Parse the peer's new ratchet key up front; a malformed key fails closed.
        let new_peer_pub = MlKemPublicKey::from_bytes(&header.epoch_pub)
            .map_err(|_| SessionError::RatchetFailed)?;
        let self_kp = self.self_kp.as_ref().ok_or(SessionError::RatchetFailed)?;

        // Bound the new-chain target index BEFORE deriving any key material, so an
        // over-bound or counter-overflowing index returns without leaving a derived
        // chain/message key on the stack. (The bound also caps the forward-walk work.)
        if header.index > RATCHET_MAX_SKIP {
            return Err(SessionError::RatchetFailed);
        }
        let new_index = header
            .index
            .checked_add(1)
            .ok_or(SessionError::RatchetFailed)?;

        // Skip the remaining keys of the CURRENT receiving chain up to the sender's
        // declared previous-chain length, retaining them (keyed by the OLD epoch). The
        // `filter` runs the block only when there is a tail to skip.
        let mut old_skipped: Vec<SkippedEntry> = Vec::new();
        if let Some(recv) = self
            .recv
            .as_ref()
            .filter(|r| header.prev_chain_len > r.index)
        {
            let gap = header.prev_chain_len - recv.index;
            if gap > RATCHET_MAX_SKIP {
                return Err(SessionError::RatchetFailed);
            }
            let mut ck = recv.chain_key;
            let mut i = recv.index;
            while i < header.prev_chain_len {
                let mk = kdf(&ck, CHAIN_MSG_INFO);
                let next = kdf(&ck, CHAIN_STEP_INFO);
                old_skipped.push(((recv.epoch_id, i), mk));
                ck.zeroize();
                ck = next;
                i += 1;
            }
            ck.zeroize();
        }

        // KEM receiving ratchet step: decapsulate with our CURRENT secret key.
        let mut ss = match self_kp.decapsulate(&header.kem_ct) {
            Ok(ss) => ss,
            Err(_) => {
                for (_, mut k) in old_skipped {
                    k.zeroize();
                }
                return Err(SessionError::RatchetFailed);
            }
        };
        let (new_root, chain0) = root_step(&self.root, &ss);
        ss.zeroize();

        // Walk the NEW receiving chain to the target index.
        let mut ck = chain0;
        let mut new_skipped: Vec<SkippedEntry> = Vec::new();
        let mut i = 0u32;
        while i < header.index {
            let mk = kdf(&ck, CHAIN_MSG_INFO);
            let next = kdf(&ck, CHAIN_STEP_INFO);
            new_skipped.push(((*inc_eid, i), mk));
            ck.zeroize();
            ck = next;
            i += 1;
        }
        let mut mk = kdf(&ck, CHAIN_MSG_INFO);
        let new_ck = kdf(&ck, CHAIN_STEP_INFO);
        ck.zeroize();

        let result = unwrap(&mk, inc_eid, header.index, header.prev_chain_len, outer);
        mk.zeroize();
        let pt = match result {
            Ok(pt) => pt,
            Err(e) => {
                // Discard ALL tentatively-derived material; advance no state.
                let mut new_root = new_root;
                new_root.zeroize();
                let mut new_ck = new_ck;
                new_ck.zeroize();
                for (_, mut k) in old_skipped {
                    k.zeroize();
                }
                for (_, mut k) in new_skipped {
                    k.zeroize();
                }
                return Err(e);
            }
        };

        // Authenticated: commit the ratchet step.
        for (key, value) in old_skipped {
            self.store_skipped(key.0, key.1, value);
        }
        for (key, value) in new_skipped {
            self.store_skipped(key.0, key.1, value);
        }
        self.root.zeroize();
        self.root = new_root;
        self.recv = Some(RecvChain {
            chain_key: new_ck,
            index: new_index,
            epoch_id: *inc_eid,
        });
        self.peer_pub = Some(new_peer_pub);
        // We have a new peer ratchet key: our next send must open a new epoch.
        self.need_send_ratchet = true;
        Ok(pt)
    }
}

/// A 1:1 session whose every message carries BOTH the classical Olm Double Ratchet
/// (inner) AND the post-quantum KEM Double Ratchet (outer).
///
/// This is the strongest 1:1 channel Mercury offers: a message stays confidential as long
/// as EITHER ML-KEM-768 OR the classical Olm handshake holds, and BOTH layers provide
/// forward secrecy AND post-compromise security (break-in recovery). The outer PQ ratchet
/// is additive — the inner [`MercurySession`] is the unmodified, audited classical Olm
/// ratchet, so even a flaw in this bespoke outer layer cannot drop confidentiality below
/// the classical guarantee (the outer layer fails closed).
///
/// Establish with [`HybridRatchetSession::initiate`] / [`HybridRatchetSession::accept`]
/// (which also protect the bootstrap message), then [`encrypt`](Self::encrypt) /
/// [`decrypt`](Self::decrypt). Secret-bearing; no `Debug`.
pub struct HybridRatchetSession {
    session: MercurySession,
    ratchet: PqRatchet,
}

impl HybridRatchetSession {
    /// Establish the OUTBOUND side toward `peer` and protect `first_message` with both
    /// ratchets. Verifies the bundle (authenticating the peer's ML-KEM ratchet key under
    /// its identity signature), starts the classical Olm session, and sends the first
    /// message wrapped by the post-quantum ratchet's epoch 1.
    pub fn initiate(
        account: &LocalSessionAccount,
        peer: &SignedPreKeyBundle,
        first_message: &[u8],
    ) -> Result<(HybridRatchetSession, RatchetMessage), SessionError> {
        peer.verify()?;
        let peer_pq = peer.pq_public()?;
        let mut session = MercurySession::establish_outbound(account, peer)?;
        let mut ratchet = PqRatchet::initiator(&peer_pq);
        // Encrypt the first Olm (pre-key) message, then wrap it in the PQ ratchet's epoch 1.
        let inner = session.encrypt(first_message)?;
        let outer = ratchet.encrypt(&inner.to_bytes())?;
        Ok((HybridRatchetSession { session, ratchet }, outer))
    }

    /// Establish the INBOUND side from a [`RatchetMessage`] produced by
    /// [`HybridRatchetSession::initiate`], recovering the first plaintext. Decapsulation
    /// + the outer AEAD authenticate the bootstrap before the inner Olm handshake runs.
    pub fn accept(
        account: &mut LocalSessionAccount,
        bundle_pq_keypair: MlKemKeyPair,
        their_curve25519: [u8; 32],
        first: &RatchetMessage,
    ) -> Result<(HybridRatchetSession, Vec<u8>), SessionError> {
        let mut ratchet = PqRatchet::responder(bundle_pq_keypair);
        let inner_bytes = ratchet.decrypt(first)?;
        let inner = SessionCiphertext::from_bytes(&inner_bytes)?;
        let (session, plaintext) =
            MercurySession::establish_inbound(account, their_curve25519, &inner)?;
        Ok((HybridRatchetSession { session, ratchet }, plaintext))
    }

    /// Encrypt `plaintext`: advance the inner Olm ratchet, then wrap the Olm ciphertext in
    /// the outer post-quantum ratchet.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<RatchetMessage, SessionError> {
        let inner = self.session.encrypt(plaintext)?;
        self.ratchet.encrypt(&inner.to_bytes())
    }

    /// Decrypt a [`RatchetMessage`]: strip the outer post-quantum ratchet, then decrypt the
    /// inner Olm message. Fails closed if either layer rejects.
    pub fn decrypt(&mut self, message: &RatchetMessage) -> Result<Vec<u8>, SessionError> {
        let inner_bytes = self.ratchet.decrypt(message)?;
        let inner = SessionCiphertext::from_bytes(&inner_bytes)?;
        self.session.decrypt(&inner)
    }

    /// Whether the inner Olm session is v2 (untruncated MAC). Always true for a session
    /// from [`initiate`](Self::initiate) / [`accept`](Self::accept).
    pub fn is_version_2(&self) -> bool {
        self.session.is_version_2()
    }

    /// Serialize this session to an ENCRYPTED pickle under `key`, for encrypted-at-rest
    /// persistence so a conversation survives a restart. Binds the inner Olm session (its
    /// own encrypted pickle) and the post-quantum ratchet state (serialized, then sealed
    /// with XChaCha20-Poly1305 under the same key). SECRET-BEARING output — store it
    /// encrypted-at-rest; `key` is the at-rest protection key.
    pub fn to_encrypted_pickle(&self, key: &[u8; 32]) -> Vec<u8> {
        let session_pickle = self.session.to_encrypted_pickle(key).into_bytes();
        let mut snapshot = serde_json::to_vec(&self.ratchet.to_snapshot())
            .expect("the ratchet snapshot always serializes");
        let ratchet_blob = pickle_seal(key, &snapshot);
        snapshot.zeroize();
        let mut out = Vec::with_capacity(4 + session_pickle.len() + ratchet_blob.len());
        out.extend_from_slice(&(session_pickle.len() as u32).to_le_bytes());
        out.extend_from_slice(&session_pickle);
        out.extend_from_slice(&ratchet_blob);
        out
    }

    /// Restore a session from [`HybridRatchetSession::to_encrypted_pickle`]. Never panics;
    /// fails closed ([`SessionError::Pickle`]) on a wrong key or a corrupt/foreign blob.
    pub fn from_encrypted_pickle(blob: &[u8], key: &[u8; 32]) -> Result<Self, SessionError> {
        if blob.len() < 4 {
            return Err(SessionError::Pickle);
        }
        let sp_len = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
        let rest = &blob[4..];
        if rest.len() < sp_len {
            return Err(SessionError::Pickle);
        }
        let (session_bytes, ratchet_blob) = rest.split_at(sp_len);
        let session_pickle =
            std::str::from_utf8(session_bytes).map_err(|_| SessionError::Pickle)?;
        let session = MercurySession::from_encrypted_pickle(session_pickle, key)?;
        let mut snapshot = pickle_open(key, ratchet_blob)?;
        let snap: RatchetSnapshot =
            serde_json::from_slice(&snapshot).map_err(|_| SessionError::Pickle)?;
        snapshot.zeroize();
        let ratchet = PqRatchet::from_snapshot(snap)?;
        Ok(HybridRatchetSession { session, ratchet })
    }
}

/// Seal a serialized pickle blob with XChaCha20-Poly1305 under the at-rest key.
///
/// No AAD is bound: this is a single-key, single-context at-rest blob (unlike the live
/// message path, which binds a header as AAD). The 4-byte `sp_len` framing prefix written by
/// [`HybridRatchetSession::to_encrypted_pickle`] sits OUTSIDE this AEAD, so it is not
/// authenticated — but tampering with it cannot do worse than fail closed: a length-guard
/// precedes the `split_at` in `from_encrypted_pickle`, and the two halves are each their own
/// authenticated ciphertext (the Olm pickle's MAC and this AEAD's tag), so any manipulation
/// yields [`SessionError::Pickle`] with no panic.
fn pickle_seal(key: &[u8; 32], plaintext: &[u8]) -> Vec<u8> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(key).expect("32-byte key is valid for XChaCha20Poly1305");
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ct = cipher
        .encrypt(&nonce, plaintext)
        .expect("XChaCha20-Poly1305 encryption does not fail for a valid key/nonce");
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

/// Reverse of [`pickle_seal`]. Fails closed on a short frame, wrong key, or tampering.
fn pickle_open(key: &[u8; 32], blob: &[u8]) -> Result<Vec<u8>, SessionError> {
    if blob.len() < NONCE_LEN + TAG_LEN {
        return Err(SessionError::Pickle);
    }
    let (nonce_bytes, ct) = blob.split_at(NONCE_LEN);
    let cipher =
        XChaCha20Poly1305::new_from_slice(key).expect("32-byte key is valid for XChaCha20Poly1305");
    cipher
        .decrypt(XNonce::from_slice(nonce_bytes), ct)
        .map_err(|_| SessionError::Pickle)
}

/// Serializable snapshot of a [`SendChain`].
#[derive(Serialize, Deserialize)]
struct SendChainSnap {
    chain_key: [u8; 32],
    index: u32,
    epoch_pub: Vec<u8>,
    epoch_ct: Vec<u8>,
}

impl Drop for SendChainSnap {
    fn drop(&mut self) {
        self.chain_key.zeroize();
    }
}

/// Serializable snapshot of a [`RecvChain`].
#[derive(Serialize, Deserialize)]
struct RecvChainSnap {
    chain_key: [u8; 32],
    index: u32,
    epoch_id: [u8; 32],
}

impl Drop for RecvChainSnap {
    fn drop(&mut self) {
        self.chain_key.zeroize();
    }
}

/// Serializable snapshot of the whole [`PqRatchet`] state. Secret-bearing; zeroizes on drop.
#[derive(Serialize, Deserialize)]
struct RatchetSnapshot {
    root: [u8; 32],
    /// The current ratchet keypair as its 64-byte ML-KEM seed (see
    /// `MlKemKeyPair::to_secret_bytes`).
    self_kp_seed: Option<Vec<u8>>,
    /// The peer's current ratchet public key bytes.
    peer_pub: Option<Vec<u8>>,
    need_send_ratchet: bool,
    send: Option<SendChainSnap>,
    recv: Option<RecvChainSnap>,
    prev_send_count: u32,
    /// The skipped-key cache flattened to `(epoch_id, index, message_key, insertion_seq)`.
    skipped: Vec<([u8; 32], u32, [u8; 32], u64)>,
    skip_seq: u64,
}

impl Drop for RatchetSnapshot {
    fn drop(&mut self) {
        self.root.zeroize();
        if let Some(seed) = self.self_kp_seed.as_mut() {
            seed.zeroize();
        }
        for (_, _, mk, _) in self.skipped.iter_mut() {
            mk.zeroize();
        }
        // The send/recv chain keys zeroize via their own snapshot Drop impls.
    }
}

impl PqRatchet {
    /// Capture the ratchet state into a serializable snapshot.
    fn to_snapshot(&self) -> RatchetSnapshot {
        RatchetSnapshot {
            root: self.root,
            self_kp_seed: self.self_kp.as_ref().map(|k| k.to_secret_bytes()),
            peer_pub: self.peer_pub.as_ref().map(|p| p.to_bytes()),
            need_send_ratchet: self.need_send_ratchet,
            send: self.send.as_ref().map(|s| SendChainSnap {
                chain_key: s.chain_key,
                index: s.index,
                epoch_pub: s.epoch_pub.clone(),
                epoch_ct: s.epoch_ct.clone(),
            }),
            recv: self.recv.as_ref().map(|r| RecvChainSnap {
                chain_key: r.chain_key,
                index: r.index,
                epoch_id: r.epoch_id,
            }),
            prev_send_count: self.prev_send_count,
            skipped: self
                .skipped
                .iter()
                .map(|((e, i), (k, s))| (*e, *i, *k, *s))
                .collect(),
            skip_seq: self.skip_seq,
        }
    }

    /// Reconstruct a ratchet from a snapshot. Fails closed ([`SessionError::Pickle`]) on a
    /// malformed key.
    fn from_snapshot(snap: RatchetSnapshot) -> Result<Self, SessionError> {
        let self_kp = match snap.self_kp_seed.as_ref() {
            Some(b) => Some(MlKemKeyPair::from_secret_bytes(b).map_err(|_| SessionError::Pickle)?),
            None => None,
        };
        let peer_pub = match snap.peer_pub.as_ref() {
            Some(b) => Some(MlKemPublicKey::from_bytes(b).map_err(|_| SessionError::Pickle)?),
            None => None,
        };
        let send = snap.send.as_ref().map(|s| SendChain {
            chain_key: s.chain_key,
            index: s.index,
            epoch_pub: s.epoch_pub.clone(),
            epoch_ct: s.epoch_ct.clone(),
        });
        let recv = snap.recv.as_ref().map(|r| RecvChain {
            chain_key: r.chain_key,
            index: r.index,
            epoch_id: r.epoch_id,
        });
        let mut skipped = BTreeMap::new();
        for (e, i, k, s) in snap.skipped.iter() {
            skipped.insert((*e, *i), (*k, *s));
        }
        Ok(PqRatchet {
            root: snap.root,
            self_kp,
            peer_pub,
            need_send_ratchet: snap.need_send_ratchet,
            send,
            recv,
            prev_send_count: snap.prev_send_count,
            skipped,
            skip_seq: snap.skip_seq,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (PqRatchet, PqRatchet) {
        let bob_kp = MlKemKeyPair::generate();
        let bob_pub = bob_kp.public();
        let alice = PqRatchet::initiator(&bob_pub);
        let bob = PqRatchet::responder(bob_kp);
        (alice, bob)
    }

    #[test]
    fn first_message_round_trips() {
        let (mut alice, mut bob) = pair();
        let m = alice.encrypt(b"hello pq ratchet").unwrap();
        assert_eq!(bob.decrypt(&m).unwrap(), b"hello pq ratchet");
    }

    #[test]
    fn bidirectional_multi_epoch_round_trips() {
        let (mut alice, mut bob) = pair();
        // Many turns, each direction change forces a fresh KEM epoch.
        for i in 0..20u32 {
            let a = alice.encrypt(format!("a{i}").as_bytes()).unwrap();
            assert_eq!(bob.decrypt(&a).unwrap(), format!("a{i}").as_bytes());
            let b = bob.encrypt(format!("b{i}").as_bytes()).unwrap();
            assert_eq!(alice.decrypt(&b).unwrap(), format!("b{i}").as_bytes());
        }
    }

    #[test]
    fn multiple_messages_per_epoch_round_trip() {
        let (mut alice, mut bob) = pair();
        let m0 = alice.encrypt(b"a0").unwrap();
        let m1 = alice.encrypt(b"a1").unwrap();
        let m2 = alice.encrypt(b"a2").unwrap();
        // All in one sending epoch (no reply in between): same epoch_pub.
        assert_eq!(m0.header.epoch_pub, m1.header.epoch_pub);
        assert_eq!(m1.header.epoch_pub, m2.header.epoch_pub);
        assert_eq!(bob.decrypt(&m0).unwrap(), b"a0");
        assert_eq!(bob.decrypt(&m1).unwrap(), b"a1");
        assert_eq!(bob.decrypt(&m2).unwrap(), b"a2");
    }

    #[test]
    fn each_epoch_uses_a_fresh_encapsulation() {
        let (mut alice, mut bob) = pair();
        let a0 = alice.encrypt(b"x").unwrap();
        bob.decrypt(&a0).unwrap();
        let b0 = bob.encrypt(b"y").unwrap();
        alice.decrypt(&b0).unwrap();
        let a1 = alice.encrypt(b"z").unwrap(); // new epoch (Alice learned Bob's key)
        // Different epochs -> different ratchet public keys and ciphertexts.
        assert_ne!(a0.header.epoch_pub, a1.header.epoch_pub);
        assert_ne!(a0.header.kem_ct, a1.header.kem_ct);
    }

    #[test]
    fn crossing_ratchets_both_send_before_receiving() {
        // Concurrent sends: each side encrypts before decrypting the other's latest. The
        // KEM lock-step must still decrypt everything. The safety argument: a side
        // generates a fresh ratchet keypair only on a send that FOLLOWS receiving a new
        // epoch, and a new epoch's receive chain is established (via decapsulation with
        // the current secret) during that receive — BEFORE the secret is replaced on the
        // next send — so the secret needed to open an in-flight epoch is never dropped
        // early.
        let (mut alice, mut bob) = pair();
        let a0 = alice.encrypt(b"a0").unwrap();
        bob.decrypt(&a0).unwrap();

        // Bob opens his reply epoch and sends two; Alice (not yet aware of Bob's key)
        // sends another in her original epoch — all before any of them are delivered.
        let b0 = bob.encrypt(b"b0").unwrap();
        let b1 = bob.encrypt(b"b1").unwrap();
        let a1 = alice.encrypt(b"a1").unwrap();
        assert_eq!(
            a0.header.epoch_pub, a1.header.epoch_pub,
            "a1 is still epoch A1"
        );

        // Deliver crossing.
        assert_eq!(alice.decrypt(&b0).unwrap(), b"b0"); // Alice: new epoch from Bob
        assert_eq!(bob.decrypt(&a1).unwrap(), b"a1"); // Bob: still on Alice's first epoch
        assert_eq!(alice.decrypt(&b1).unwrap(), b"b1"); // Alice: same (Bob) epoch

        // The conversation continues correctly through further ratchets.
        let a2 = alice.encrypt(b"a2").unwrap(); // Alice ratchets (she now knows Bob's key)
        assert_ne!(a2.header.epoch_pub, a1.header.epoch_pub);
        assert_eq!(bob.decrypt(&a2).unwrap(), b"a2");
        let b2 = bob.encrypt(b"b2").unwrap(); // Bob ratchets
        assert_eq!(alice.decrypt(&b2).unwrap(), b"b2");
    }

    #[test]
    fn the_root_evolves_every_epoch_post_compromise_mechanism() {
        // Post-compromise security rests on fresh ML-KEM entropy entering the root each
        // epoch. Capture the root across several epochs and confirm it keeps changing to
        // values seeded by fresh encapsulations (so an attacker holding an old root cannot
        // predict a later one).
        let (mut alice, mut bob) = pair();
        let a0 = alice.encrypt(b"1").unwrap();
        let r_after_a0 = alice.root;
        bob.decrypt(&a0).unwrap();
        let b0 = bob.encrypt(b"2").unwrap();
        alice.decrypt(&b0).unwrap();
        let r_after_b0 = alice.root;
        let a1 = alice.encrypt(b"3").unwrap();
        let r_after_a1 = alice.root;
        bob.decrypt(&a1).unwrap();
        // Each ratchet step changed Alice's root.
        assert_ne!(r_after_a0, r_after_b0);
        assert_ne!(r_after_b0, r_after_a1);
        assert_ne!(r_after_a0, r_after_a1);
    }

    #[test]
    fn out_of_order_within_an_epoch() {
        let (mut alice, mut bob) = pair();
        let m0 = alice.encrypt(b"zero").unwrap();
        let m1 = alice.encrypt(b"one").unwrap();
        let m2 = alice.encrypt(b"two").unwrap();
        assert_eq!(bob.decrypt(&m2).unwrap(), b"two"); // skips 0,1 into cache
        assert_eq!(bob.decrypt(&m0).unwrap(), b"zero"); // from cache
        assert_eq!(bob.decrypt(&m1).unwrap(), b"one"); // from cache
    }

    #[test]
    fn out_of_order_across_an_epoch_boundary() {
        // A message from a previous epoch arriving after the next epoch has begun must
        // still decrypt (via the cross-epoch skipped-key cache).
        let (mut alice, mut bob) = pair();
        let a0 = alice.encrypt(b"a0").unwrap();
        let a1 = alice.encrypt(b"a1").unwrap();
        bob.decrypt(&a0).unwrap();
        let b0 = bob.encrypt(b"b0").unwrap();
        alice.decrypt(&b0).unwrap();
        // Alice opens a new epoch; its header declares prev_chain_len so Bob can skip a1.
        let a2 = alice.encrypt(b"a2").unwrap();
        assert_ne!(a2.header.epoch_pub, a0.header.epoch_pub);
        assert_eq!(bob.decrypt(&a2).unwrap(), b"a2"); // ratchet step; a1 skipped into cache
        assert_eq!(bob.decrypt(&a1).unwrap(), b"a1"); // late, from the previous epoch's cache
    }

    #[test]
    fn replay_is_rejected() {
        let (mut alice, mut bob) = pair();
        let m = alice.encrypt(b"once").unwrap();
        assert_eq!(bob.decrypt(&m).unwrap(), b"once");
        assert!(matches!(bob.decrypt(&m), Err(SessionError::RatchetFailed)));
    }

    #[test]
    fn a_tampered_payload_is_rejected_without_desync() {
        let (mut alice, mut bob) = pair();
        let good = alice.encrypt(b"good").unwrap();
        let mut bad = good.clone();
        let last = bad.outer.len() - 1;
        bad.outer[last] ^= 0x01;
        assert!(matches!(
            bob.decrypt(&bad),
            Err(SessionError::RatchetFailed)
        ));
        // No desync: the genuine message still decrypts.
        assert_eq!(bob.decrypt(&good).unwrap(), b"good");
    }

    #[test]
    fn a_forged_new_epoch_message_does_not_desync_the_ratchet() {
        // A forgery that ROUTES to the new-epoch path (a novel epoch_pub the receiver has
        // never seen) must be rejected by the AAD binding AND leave the ratchet able to
        // open the GENUINE new epoch — the two-phase commit must not adopt the foreign
        // peer key, advance the root, or pollute the skipped cache.
        let (mut alice, mut bob) = pair();
        let a0 = alice.encrypt(b"a0").unwrap();
        bob.decrypt(&a0).unwrap();
        let b0 = bob.encrypt(b"b0").unwrap();
        alice.decrypt(&b0).unwrap();
        let a1 = alice.encrypt(b"a1").unwrap(); // a genuine new epoch from Alice

        // Forge: substitute a novel epoch public key (routes to decrypt_new_epoch) while
        // leaving the ciphertext — the epoch_id in the AAD no longer matches, so the AEAD
        // must fail even though decapsulation yields a usable secret.
        let mut forged = a1.clone();
        let foreign = MlKemKeyPair::generate();
        forged.header.epoch_pub = foreign.public().to_bytes();
        assert!(matches!(
            bob.decrypt(&forged),
            Err(SessionError::RatchetFailed)
        ));

        // No desync: the genuine new epoch still opens and the channel keeps working.
        assert_eq!(bob.decrypt(&a1).unwrap(), b"a1");
        let b1 = bob.encrypt(b"b1").unwrap();
        assert_eq!(alice.decrypt(&b1).unwrap(), b"b1");
    }

    #[test]
    fn a_tampered_kem_ciphertext_is_rejected() {
        let (mut alice, mut bob) = pair();
        let mut m = alice.encrypt(b"secret").unwrap();
        m.header.kem_ct[0] ^= 0x01; // wrong shared secret -> wrong keys -> AEAD fails
        assert!(matches!(bob.decrypt(&m), Err(SessionError::RatchetFailed)));
    }

    #[test]
    fn a_tampered_epoch_key_is_rejected() {
        let (mut alice, mut bob) = pair();
        let mut m = alice.encrypt(b"secret").unwrap();
        m.header.epoch_pub[0] ^= 0x01; // changes epoch_id (AAD) + decaps target
        assert!(matches!(bob.decrypt(&m), Err(SessionError::RatchetFailed)));
    }

    #[test]
    fn a_wrong_length_header_is_rejected() {
        let (mut alice, mut bob) = pair();
        let mut m = alice.encrypt(b"secret").unwrap();
        m.header.epoch_pub.truncate(1183);
        assert!(matches!(bob.decrypt(&m), Err(SessionError::RatchetFailed)));
    }

    #[test]
    fn a_responder_cannot_send_before_receiving() {
        let bob_kp = MlKemKeyPair::generate();
        let mut bob = PqRatchet::responder(bob_kp);
        // No peer ratchet key yet -> fail closed.
        assert!(matches!(
            bob.encrypt(b"early"),
            Err(SessionError::RatchetFailed)
        ));
    }

    #[test]
    fn an_index_too_far_ahead_is_rejected() {
        let (mut alice, mut bob) = pair();
        let a0 = alice.encrypt(b"real").unwrap();
        bob.decrypt(&a0).unwrap();
        // Forge a within-epoch index way beyond the skip bound.
        let mut forged = alice.encrypt(b"x").unwrap();
        forged.header.index = RATCHET_MAX_SKIP + 50;
        assert!(matches!(
            bob.decrypt(&forged),
            Err(SessionError::RatchetFailed)
        ));
    }

    #[test]
    fn two_independent_sessions_differ() {
        // Randomized encapsulation: the same plaintext under two fresh pairs differs.
        let (mut a1, mut _b1) = pair();
        let (mut a2, mut _b2) = pair();
        let m1 = a1.encrypt(b"same").unwrap();
        let m2 = a2.encrypt(b"same").unwrap();
        assert_ne!(m1.outer, m2.outer);
        assert_ne!(m1.header.epoch_pub, m2.header.epoch_pub);
    }
}
