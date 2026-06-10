//! Durable, redb-backed persistence for the username registry, so a relay restart does not
//! lose claimed handles (closing residual #4 in `docs/USERNAMES.md`).
//!
//! Scope (honest): this persists the **first-claimant-wins guard** — the authoritative
//! `normalized_username -> account_id` map — and nothing more. The AKD key-transparency log
//! itself stays in-memory and is REBUILT at boot by re-registering every persisted binding
//! into a fresh directory (the server binary batches them into ONE epoch via
//! `KtDirectory::register_batch`). Because the directory's VRF key is derived from a persisted
//! seed (`MERCURY_KT_VRF_SEED`), the rebuilt log produces inclusion proofs that still verify
//! against the key clients pinned; what is NOT preserved is epoch history — epochs restart
//! from zero, so consistency proofs spanning a restart are not meaningful. That residual is
//! disclosed in `docs/USERNAMES.md`, not hidden.
//!
//! Like [`crate::redb_store`], this is a plain durable KV: usernames and account ids are
//! public directory data (served to anyone via `GET /username/{name}`), so there is no
//! plaintext to protect at rest beyond what the operator's disk encryption covers.
//!
//! Error model: unlike the infallible `QueueStore` trait, these methods are fallible and the
//! registry FAILS CLOSED on them — a claim whose durable write fails is refused (the HTTP
//! layer returns 503) rather than accepted into memory only, so the in-memory map and the
//! disk file never diverge. A corrupt row (an account id that is not exactly 32 bytes) is an
//! ERROR at load, never truncated or padded into a "valid" id.

use std::path::Path;

use redb::{Database, ReadableTable, TableDefinition};

/// `normalized_username` -> 32-byte account id.
const USERNAMES: TableDefinition<&str, &[u8]> = TableDefinition::new("mercury_usernames_v1");

/// A durable `username -> account_id` store backed by a redb database file. See the module
/// docs for exactly what this does and does not persist.
pub struct DurableUsernameStore {
    db: Database,
}

impl std::fmt::Debug for DurableUsernameStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DurableUsernameStore")
            .finish_non_exhaustive()
    }
}

impl DurableUsernameStore {
    /// Open (creating if absent) a durable username store at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, redb::Error> {
        let db = Database::create(path)?;
        // Materialize the table so a brand-new file has it.
        let txn = db.begin_write()?;
        {
            let _ = txn.open_table(USERNAMES)?;
        }
        txn.commit()?;
        Ok(Self { db })
    }

    /// Every persisted `(normalized_username, account_id)` binding. A value that is not
    /// exactly 32 bytes is a corrupt store and an ERROR (fail closed — never truncate or pad
    /// bytes into something that merely looks like an account id).
    pub fn load_all(&self) -> Result<Vec<(String, [u8; 32])>, redb::Error> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(USERNAMES)?;
        let mut out = Vec::new();
        for entry in table.iter()? {
            let (name, value) = entry?;
            let id: [u8; 32] = value.value().try_into().map_err(|_| {
                redb::Error::Corrupted(format!(
                    "username store: account id for '{}' is {} bytes, expected exactly 32",
                    name.value(),
                    value.value().len()
                ))
            })?;
            out.push((name.value().to_string(), id));
        }
        Ok(out)
    }

    /// Durably persist `name -> id`. The transaction is committed before this returns, so an
    /// `Ok` means the binding survives a crash/restart; an `Err` means NOTHING was persisted
    /// (the caller must refuse the claim — see [`crate::username::ClaimResult::PersistFailed`]).
    pub fn put(&self, name: &str, id: &[u8; 32]) -> Result<(), redb::Error> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(USERNAMES)?;
            table.insert(name, id.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }
}
