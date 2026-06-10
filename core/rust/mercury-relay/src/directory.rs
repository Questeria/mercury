//! Prekey-bundle directory.
//!
//! A public store mapping an account id to that account's opaque, self-
//! authenticating **contact card** (a signed pre-key bundle + sealed-sender
//! device key, serialized by the client). It removes the out-of-band card
//! exchange the thin-slice (Milestone 1) relied on: a client publishes its card
//! once, and any peer who knows its account id can fetch it.
//!
//! The relay is **untrusted** and treats every card as opaque bytes — it never
//! parses, interprets, or verifies the card contents. Authenticity is the
//! **client's** job: a fetched card is only safe after the client (1) checks the
//! bundle's Ed25519 identity signature and that the account id embedded in the
//! card equals the one derived from that identity key (`mercury-session`
//! `SignedPreKeyBundle::verify`), and (2) checks that the card's account id
//! equals the one it asked for. Because a valid card is signed by an identity
//! key that *hashes to* its own account id, a malicious relay cannot forge or
//! substitute a card for an account it does not control — the worst it can do is
//! withhold a card or serve a stale one (an availability problem), which key-
//! transparency registration closes in a later increment.
//!
//! **Write authorization (proof of possession).** Although the relay does not
//! parse the card, it DOES authorize each publish: the HTTP layer requires an
//! Ed25519 proof of possession over the card bytes and stores the card only
//! under the slot DERIVED from the proving identity key
//! (`mercury_keys::verify_directory_publish`). So a publisher can only write to
//! its *own* slot — directory slot-squatting is closed. This verifies PUBLIC key
//! material only; it grants the relay no ability to decrypt messages.
//!
//! Known residuals: (a) **availability** — the relay, being untrusted, can still
//! withhold a card or serve a stale one, and a slot's own key-holder can overwrite
//! their own card; key-transparency registration (a later increment) makes such
//! equivocation/rollback detectable. (b) **lookup metadata** — the relay learns
//! which account ids exist and who fetches whom; private contact discovery is a
//! later concern. Both are inherent to a plain directory and documented, not hidden.

use std::collections::BTreeMap;

/// Maximum stored card size, in bytes. A signed pre-key bundle (account id +
/// curve25519 + ed25519 + ML-KEM-768 public key + signature) plus a device key
/// is well under a kilobyte; this bound is generous headroom while still capping
/// a griefing publisher's per-slot footprint.
pub const MAX_CARD_LEN: usize = 8 * 1024;

/// Maximum number of distinct directory slots (account ids) held at once. A publish requires a
/// valid Ed25519 proof of possession over the slot derived from the publishing key, but keypairs
/// are free to mint, so without a cap a griefer could publish unbounded distinct slots. Updates to
/// an EXISTING slot are always allowed; only a brand-new slot is refused once the directory is
/// full — generous headroom for real usage while bounding the footprint (fail-closed, like the
/// rate limiter's `MAX_TRACKED_KEYS`).
pub const MAX_DIRECTORY_SLOTS: usize = 1_000_000;

/// In-memory prekey directory: account id (bytes) -> opaque card bytes.
///
/// Mirrors the deliberately-simple shape of [`crate::InMemoryQueueStore`]: a
/// `BTreeMap` with no interpretation of the values. A durable backend (the
/// relay-hardening milestone) implements the same publish/fetch semantics.
#[derive(Debug, Clone, Default)]
pub struct InMemoryDirectoryStore {
    cards: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl InMemoryDirectoryStore {
    /// A fresh, empty directory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Store (or replace) the opaque `card` for `account_id`. Returns `false`
    /// (storing nothing) if the card is empty or exceeds [`MAX_CARD_LEN`]; the
    /// relay never interprets the bytes beyond this length guard.
    pub fn publish(&mut self, account_id: &[u8], card: &[u8]) -> bool {
        if card.is_empty() || card.len() > MAX_CARD_LEN {
            return false;
        }
        // Bound the number of distinct slots: refuse a brand-new slot once full, but always allow a
        // key-holder to update their OWN existing slot (no lock-out of established accounts).
        let account = account_id.to_vec();
        if !self.cards.contains_key(&account) && self.cards.len() >= MAX_DIRECTORY_SLOTS {
            return false;
        }
        self.cards.insert(account, card.to_vec());
        true
    }

    /// Fetch a clone of the opaque card for `account_id`, if present.
    pub fn fetch(&self, account_id: &[u8]) -> Option<Vec<u8>> {
        self.cards.get(account_id).cloned()
    }

    /// Number of published cards.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Whether the directory is empty.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_then_fetch_round_trips() {
        let mut dir = InMemoryDirectoryStore::new();
        let id = [7u8; 32];
        let card = vec![0xabu8; 256];
        assert!(dir.publish(&id, &card));
        assert_eq!(dir.fetch(&id), Some(card));
        assert_eq!(dir.len(), 1);
    }

    #[test]
    fn fetch_missing_is_none() {
        let dir = InMemoryDirectoryStore::new();
        assert_eq!(dir.fetch(&[1u8; 32]), None);
    }

    #[test]
    fn publish_replaces_an_existing_slot() {
        let mut dir = InMemoryDirectoryStore::new();
        let id = [7u8; 32];
        assert!(dir.publish(&id, &[1u8; 64]));
        assert!(dir.publish(&id, &[2u8; 64]));
        assert_eq!(dir.fetch(&id), Some(vec![2u8; 64]));
        assert_eq!(dir.len(), 1);
    }

    #[test]
    fn empty_and_oversized_cards_are_rejected() {
        let mut dir = InMemoryDirectoryStore::new();
        let id = [7u8; 32];
        assert!(!dir.publish(&id, &[]));
        assert!(!dir.publish(&id, &vec![0u8; MAX_CARD_LEN + 1]));
        assert!(dir.is_empty());
        // The boundary value is accepted.
        assert!(dir.publish(&id, &vec![0u8; MAX_CARD_LEN]));
    }
}
