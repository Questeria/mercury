//! Username registry: a human-readable handle -> account-id directory, first-claimant-wins, backed
//! by the relay's key-transparency log.
//!
//! This struct is only the **first-claimant-wins guard**: the authoritative `username -> account_id`
//! map plus a cap. The cryptographic commitment lives in the KT log — the HTTP `claim` handler, on a
//! fresh claim, ALSO publishes `(kt_username_label(name) -> account_id)` into the
//! [`crate::kt::KtState`] directory, so a later lookup is served with an AKD inclusion proof the
//! client verifies against the directory's pinned VRF key. Keeping the guard and the KT publish in
//! one critical section is what makes first-claimant-wins atomic.
//!
//! Trust model (honest): the relay is untrusted. A client that has pinned the directory's VRF key
//! out-of-band can detect a relay that forges or rotates a `handle -> account_id` binding (the
//! inclusion proof fails under the pinned key). The looked-up account id is then used to fetch a
//! contact card from the directory, which is INDEPENDENTLY self-authenticating (its account id is
//! the hash of its identity key). Residuals (documented in `docs/USERNAMES.md`, not hidden): without
//! independent witnesses + gossip the relay can still *equivocate* (show different logs to different
//! clients) or *withhold*; and first-use VRF-key pinning is trust-on-first-use unless the key is
//! pinned out-of-band. The handle charset is restricted to ASCII `[a-z0-9_]` (see
//! `mercury_keys::normalize_username`) to cut the Unicode-confusable surface.

use std::collections::BTreeMap;

use crate::username_store::DurableUsernameStore;

/// Maximum number of registered usernames held at once. A claim requires a valid Ed25519
/// proof of possession, but keypairs are free to mint, so this caps the registry footprint
/// (fail-closed — a brand-new handle is refused once full; an existing owner re-confirming
/// their own handle is always allowed).
pub const MAX_USERNAMES: usize = 1_000_000;

/// The label a username is committed under in the KT log. Namespaced with a `username:` prefix so
/// handle entries never collide with any other label space the directory may host (e.g. device
/// keys). Both `claim` (publish) and lookup (proof) MUST use this exact label.
pub fn kt_username_label(normalized_username: &str) -> String {
    format!("username:{normalized_username}")
}

/// The outcome of a [`UsernameRegistry::claim`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimResult {
    /// A brand-new binding was created (the caller MUST now commit it to the KT log).
    Created,
    /// The handle is already bound to THIS account id — idempotent success (no KT re-publish).
    AlreadyOwned,
    /// The handle is already bound to a DIFFERENT account id — first-claimant-wins refusal.
    Taken,
    /// The registry is at capacity and this is a brand-new handle — refused (fail-closed).
    Full,
    /// The durable write-through failed: the binding was recorded NEITHER on disk NOR in the
    /// in-memory map (fail-closed — the two never diverge). The handle remains unclaimed and
    /// the claim can be retried; the HTTP layer maps this to 503 SERVICE_UNAVAILABLE.
    PersistFailed,
}

/// Username registry: normalized handle -> account id (32 bytes), optionally written through
/// to a durable [`DurableUsernameStore`] so claims survive a relay restart.
///
/// Mirrors the deliberately-simple shape of [`crate::InMemoryDirectoryStore`]. Without a
/// durable store ([`UsernameRegistry::new`]) it is in-memory only (tests / dev); the server
/// binary attaches one via [`UsernameRegistry::with_durable`] when `MERCURY_USERNAME_DB` is
/// set. The KT log remains the cryptographic source of truth a client verifies.
#[derive(Debug, Default)]
pub struct UsernameRegistry {
    bindings: BTreeMap<String, [u8; 32]>,
    /// Write-through persistence for the first-claimant-wins guard; `None` = in-memory only.
    durable: Option<DurableUsernameStore>,
}

impl UsernameRegistry {
    /// A fresh, empty, in-memory-only registry (claims do NOT survive a restart).
    pub fn new() -> Self {
        Self::default()
    }

    /// A registry backed by `store`: every persisted binding is loaded into the map, and every
    /// future [`UsernameRegistry::claim`] that creates a binding is durably committed BEFORE it
    /// is visible in memory (fail-closed: a failed disk write refuses the claim, so memory and
    /// disk never diverge). Errors if the store cannot be read or holds a corrupt row.
    pub fn with_durable(store: DurableUsernameStore) -> Result<Self, redb::Error> {
        let mut bindings = BTreeMap::new();
        for (name, id) in store.load_all()? {
            bindings.insert(name, id);
        }
        Ok(Self {
            bindings,
            durable: Some(store),
        })
    }

    /// The account id bound to `normalized_username`, if any.
    pub fn lookup(&self, normalized_username: &str) -> Option<[u8; 32]> {
        self.bindings.get(normalized_username).copied()
    }

    /// Attempt to bind `normalized_username` to `account_id`, first-claimant-wins. The caller is
    /// responsible for having validated/normalized the handle (via `mercury_keys::normalize_username`)
    /// and verified the claimant owns `account_id` (via `mercury_keys::verify_username_claim`). On
    /// [`ClaimResult::Created`] the caller MUST publish the binding into the KT log; the other
    /// variants require no KT mutation.
    ///
    /// When a durable store is attached, a fresh binding is committed to disk BEFORE it is
    /// inserted into the in-memory map; if that write fails the claim is refused with
    /// [`ClaimResult::PersistFailed`] and the map is untouched (fail-closed — a claim is never
    /// "accepted" in a form that a restart would silently revoke).
    pub fn claim(&mut self, normalized_username: &str, account_id: [u8; 32]) -> ClaimResult {
        match self.bindings.get(normalized_username) {
            Some(existing) if *existing == account_id => ClaimResult::AlreadyOwned,
            Some(_) => ClaimResult::Taken,
            None => {
                if self.bindings.len() >= MAX_USERNAMES {
                    return ClaimResult::Full;
                }
                if let Some(store) = &self.durable {
                    if store.put(normalized_username, &account_id).is_err() {
                        return ClaimResult::PersistFailed;
                    }
                }
                self.bindings
                    .insert(normalized_username.to_string(), account_id);
                ClaimResult::Created
            }
        }
    }

    /// Number of registered usernames.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_claim_creates_then_idempotent_for_same_owner() {
        let mut reg = UsernameRegistry::new();
        let alice = [1u8; 32];
        assert_eq!(reg.claim("alice", alice), ClaimResult::Created);
        assert_eq!(reg.lookup("alice"), Some(alice));
        // The same owner re-confirming their handle is idempotent, not a re-create.
        assert_eq!(reg.claim("alice", alice), ClaimResult::AlreadyOwned);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn a_different_account_cannot_take_a_claimed_handle() {
        let mut reg = UsernameRegistry::new();
        assert_eq!(reg.claim("alice", [1u8; 32]), ClaimResult::Created);
        // First-claimant-wins: a different account is refused, and the binding is unchanged.
        assert_eq!(reg.claim("alice", [2u8; 32]), ClaimResult::Taken);
        assert_eq!(reg.lookup("alice"), Some([1u8; 32]));
    }

    #[test]
    fn lookup_missing_is_none() {
        let reg = UsernameRegistry::new();
        assert_eq!(reg.lookup("nobody"), None);
    }

    #[test]
    fn kt_label_is_namespaced() {
        assert_eq!(kt_username_label("alice"), "username:alice");
    }
}
