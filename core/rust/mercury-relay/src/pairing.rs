//! Short-lived, single-use pairing-code broker.
//!
//! An easier way to connect than copy-pasting a 64-hex-character account id: a user asks the relay
//! to broker their opaque contact card under a short, human-typable CODE with a few-minutes TTL,
//! reads the code aloud / messages it, and the peer enters it to fetch the card ONCE. The code then
//! disappears.
//!
//! Like the directory, the relay is **untrusted** and treats every card as opaque bytes — it never
//! parses, interprets, or verifies a card. Authenticity stays the **client's** job: a card fetched
//! by code is only used after the client re-verifies it is self-authenticating (the bundle's
//! identity signature checks out and the embedded account id is the hash of that identity key, via
//! `mercury-client`'s fetch-and-verify path). So the worst a malicious relay can do is withhold a
//! code or broker a *different* valid card — the fetcher sees exactly which account id they are
//! about to add and confirms it; a relay can never forge a card for an identity it does not control.
//!
//! **Write authorization (proof of possession).** The HTTP layer requires an Ed25519 pairing proof
//! of possession over the card bytes (`mercury_keys::verify_pairing_publish`) before brokering, so
//! the broker cannot be stuffed with un-signed garbage by an unauthenticated flooder. Unlike the
//! directory, the storage slot is a RANDOM code, NOT the proving key's derived slot — a pairing code
//! is meant to be shared out-of-band, so it is deliberately not tied to the account id.
//!
//! Anti-abuse: codes are single-use (deleted on fetch), short-lived ([`PAIRING_TTL_S`]), and the
//! store is capped ([`MAX_PAIRING_ENTRIES`]); expired entries are reaped lazily on access and
//! swept when the store is full. The code space ([`PAIRING_CODE_LEN`] chars over a 32-symbol
//! alphabet ≈ 50 bits) makes blind guessing within a TTL window impractical, and the open endpoints
//! are flood-limited per source.

use std::collections::BTreeMap;

use crate::directory::MAX_CARD_LEN;

/// How long a brokered pairing code lives before it expires, in seconds (10 minutes). Long enough
/// to read a code to someone and have them type it; short enough that an unused code is not a
/// lingering grief/guess target.
pub const PAIRING_TTL_S: i64 = 600;

/// Maximum number of live pairing codes held at once. A publish requires a valid pairing proof of
/// possession, but keypairs are free to mint, so without a cap a griefer could broker unbounded
/// codes; this bounds the footprint (fail-closed — a brand-new code is refused once full, after an
/// expired-entry sweep). Generous for real usage.
pub const MAX_PAIRING_ENTRIES: usize = 100_000;

/// Number of characters in a generated pairing code (excludes the display hyphen). 10 symbols over
/// the 32-symbol [`PAIRING_ALPHABET`] ≈ 50 bits of entropy — blind guessing within a TTL window,
/// even against the flood cap, is negligible against this keyspace.
pub const PAIRING_CODE_LEN: usize = 10;

/// Code alphabet: uppercase letters + digits with the ambiguous glyphs (`0/O`, `1/I`) removed, so a
/// spoken or hand-copied code is unambiguous (`1` is gone, so a lone `L` is unmistakable). Exactly
/// 32 symbols => 5 unbiased bits each (32 divides 256, so the rejection sampler never rejects).
pub const PAIRING_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// One brokered card under a pairing code.
#[derive(Debug, Clone)]
struct PairingEntry {
    card: Vec<u8>,
    expires_at_s: i64,
}

/// In-memory pairing broker: code -> (opaque card, expiry).
///
/// Mirrors the deliberately-simple shape of [`crate::InMemoryDirectoryStore`]: a `BTreeMap` with no
/// interpretation of the card bytes. Entries are short-lived and single-use.
#[derive(Debug, Clone, Default)]
pub struct PairingStore {
    entries: BTreeMap<String, PairingEntry>,
}

impl PairingStore {
    /// A fresh, empty broker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether `code` is currently brokering a (possibly-expired) card. Used by the handler to
    /// avoid overwriting a live code when it generates a fresh one.
    pub fn contains(&self, code: &str) -> bool {
        self.entries.contains_key(code)
    }

    /// Broker `card` under `code` until `now_s + PAIRING_TTL_S`. Returns `false` (storing nothing)
    /// if the card is empty or exceeds [`MAX_CARD_LEN`], or if the store is full even after reaping
    /// expired entries. The relay never interprets the card beyond this length guard.
    pub fn put(&mut self, code: String, card: &[u8], now_s: i64) -> bool {
        if card.is_empty() || card.len() > MAX_CARD_LEN {
            return false;
        }
        // Reap expired entries before enforcing the cap, so transient churn does not lock the broker.
        if self.entries.len() >= MAX_PAIRING_ENTRIES {
            self.prune_expired(now_s);
        }
        // Refuse a brand-new code once full (an existing code is always replaceable). Fail-closed.
        if !self.entries.contains_key(&code) && self.entries.len() >= MAX_PAIRING_ENTRIES {
            return false;
        }
        self.entries.insert(
            code,
            PairingEntry {
                card: card.to_vec(),
                expires_at_s: now_s.saturating_add(PAIRING_TTL_S),
            },
        );
        true
    }

    /// Single-use fetch: remove and return the card for `code` iff it exists and has not expired.
    /// An expired entry is removed and treated as absent. After a successful fetch the code is gone
    /// (deliver-once), so a code cannot be reused or guessed-then-replayed.
    pub fn take(&mut self, code: &str, now_s: i64) -> Option<Vec<u8>> {
        let entry = self.entries.remove(code)?;
        if entry.expires_at_s <= now_s {
            // Expired between brokering and fetch: already removed above; report absent.
            return None;
        }
        Some(entry.card)
    }

    /// Drop every entry whose TTL has passed as of `now_s`.
    pub fn prune_expired(&mut self, now_s: i64) {
        self.entries.retain(|_, e| e.expires_at_s > now_s);
    }

    /// Number of brokered codes (including any not-yet-reaped expired ones).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the broker holds no codes.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Generate a fresh pairing code: [`PAIRING_CODE_LEN`] characters drawn uniformly from
/// [`PAIRING_ALPHABET`] using the OS CSPRNG, with a single hyphen inserted at the midpoint for
/// readability (e.g. `K7MNP-QR4ST`). Rejection-samples the random bytes so every symbol is uniform
/// over the alphabet (no modulo bias). The hyphen is display-only and is stripped before lookup.
pub fn generate_pairing_code() -> String {
    let alphabet = PAIRING_ALPHABET;
    let n = alphabet.len() as u8;
    // Largest multiple of `n` that fits in a byte; bytes at/above it are rejected to avoid bias.
    let limit = (256 / alphabet.len() * alphabet.len()) as u16;
    let mut out = String::with_capacity(PAIRING_CODE_LEN + 1);
    let mid = PAIRING_CODE_LEN / 2;
    let mut produced = 0usize;
    while produced < PAIRING_CODE_LEN {
        let mut b = [0u8; 1];
        // getrandom is the same OS CSPRNG the rest of the crate uses; a failure here is
        // unrecoverable for code generation, so propagate it as a panic (never a weak code).
        getrandom::getrandom(&mut b).expect("OS CSPRNG available for pairing-code generation");
        if (b[0] as u16) < limit {
            if produced == mid {
                out.push('-');
            }
            out.push(alphabet[(b[0] % n) as usize] as char);
            produced += 1;
        }
    }
    out
}

/// Strip the display hyphen(s) and uppercase, yielding the canonical lookup key for a typed code.
/// Both broker and fetch key on this canonical form, so a user may type the code with or without
/// the hyphen and in any case.
pub fn canonical_code(typed: &str) -> String {
    typed
        .chars()
        .filter(|c| *c != '-' && !c.is_whitespace())
        .flat_map(|c| c.to_uppercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphabet_is_32_unambiguous_symbols() {
        // 32 symbols => exactly 5 bits each. The ambiguous pairs 0/O and 1/I are removed; L stays
        // (with 1 and I gone, an uppercase L is unmistakable), which is what gets us to 32.
        assert_eq!(PAIRING_ALPHABET.len(), 32, "alphabet must be 32 symbols");
        for bad in [b'0', b'O', b'1', b'I'] {
            assert!(
                !PAIRING_ALPHABET.contains(&bad),
                "ambiguous glyph {} must be excluded",
                bad as char
            );
        }
        // All symbols distinct.
        let mut sorted = PAIRING_ALPHABET.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            PAIRING_ALPHABET.len(),
            "symbols must be distinct"
        );
    }

    #[test]
    fn put_then_take_round_trips_once() {
        let mut store = PairingStore::new();
        let now = 1_000;
        assert!(store.put("CODE1".to_string(), b"card-bytes", now));
        assert_eq!(store.take("CODE1", now + 1), Some(b"card-bytes".to_vec()));
        // Single-use: a second fetch finds nothing.
        assert_eq!(store.take("CODE1", now + 1), None);
        assert!(store.is_empty());
    }

    #[test]
    fn expired_code_is_not_returned() {
        let mut store = PairingStore::new();
        let now = 1_000;
        assert!(store.put("CODE2".to_string(), b"card", now));
        // Past the TTL -> absent (and removed).
        assert_eq!(store.take("CODE2", now + PAIRING_TTL_S + 1), None);
        assert!(store.is_empty());
    }

    #[test]
    fn empty_and_oversized_cards_are_rejected() {
        let mut store = PairingStore::new();
        assert!(!store.put("C".to_string(), &[], 0));
        assert!(!store.put("C".to_string(), &vec![0u8; MAX_CARD_LEN + 1], 0));
        assert!(store.is_empty());
        // The boundary value is accepted.
        assert!(store.put("C".to_string(), &vec![0u8; MAX_CARD_LEN], 0));
    }

    #[test]
    fn prune_drops_only_expired() {
        let mut store = PairingStore::new();
        store.put("LIVE".to_string(), b"a", 0);
        store.put("DEAD".to_string(), b"b", 0);
        // Advance time so DEAD's entry (put at 0) is well past TTL, then re-broker LIVE fresh.
        store.put("LIVE".to_string(), b"a", PAIRING_TTL_S);
        store.prune_expired(PAIRING_TTL_S + 1);
        assert!(store.contains("LIVE"));
        assert!(!store.contains("DEAD"));
    }

    #[test]
    fn generated_codes_have_the_right_shape_and_alphabet() {
        let code = generate_pairing_code();
        // One hyphen, PAIRING_CODE_LEN symbols.
        assert_eq!(code.matches('-').count(), 1);
        let canon = canonical_code(&code);
        assert_eq!(canon.len(), PAIRING_CODE_LEN);
        assert!(
            canon.bytes().all(|b| PAIRING_ALPHABET.contains(&b)),
            "every symbol must be from the alphabet: {canon}"
        );
    }

    #[test]
    fn canonical_code_strips_hyphens_whitespace_and_uppercases() {
        assert_eq!(canonical_code("k7m-np qr"), "K7MNPQR");
    }

    #[test]
    fn generated_codes_are_distinct() {
        // Not a randomness test, just a sanity check that we are not returning a constant.
        let a = generate_pairing_code();
        let b = generate_pairing_code();
        assert_ne!(a, b);
    }
}
