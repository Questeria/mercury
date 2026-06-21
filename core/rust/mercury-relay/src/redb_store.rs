//! A durable, redb-backed [`QueueStore`] so a relay restart does not lose queued items or
//! replay tombstones.
//!
//! The relay's queue holds ALREADY end-to-end-encrypted opaque ciphertext + content-free
//! metadata, so this is a plain durable key/value store. It deliberately does NOT reuse the
//! client's keychain-encrypted `mercury-store`: a server has no user keychain, and there is no
//! plaintext here to protect at rest beyond what the operator's disk/volume encryption covers.
//!
//! OPT-IN: [`InMemoryQueueStore`](crate::InMemoryQueueStore) remains the default; a deployment
//! selects this for durability (the server binary does so when `MERCURY_RELAY_DB` is set).
//!
//! Records are stored under their `route_id` as a JSON [`PersistedRecord`] mirror — the core
//! `RelayQueueItemState` / `RelaySubmissionDecision` types are not `serde`, so we mirror their
//! fields here rather than modify the policy core. Replay tokens are a second table used as a
//! set.
//!
//! Error model: the [`QueueStore`] trait is infallible, but disk I/O is not. Reads fail CLOSED
//! (a storage error reads as "absent"/"not seen"). A write that fails (full/corrupt disk)
//! PANICS the current request rather than silently losing a message — a loud, honest failure;
//! the panic aborts only that request (the relay keeps serving). Making the trait fallible
//! end-to-end is a clean follow-on.
//!
//! Concurrency note: the store is driven through `Arc<Mutex<dyn QueueStore>>` from async Axum
//! handlers, and these methods do SYNCHRONOUS redb I/O, so each queue op briefly blocks a tokio
//! worker thread. Acceptable at single-instance relay scale; offloading via `spawn_blocking` or
//! a dedicated DB thread (paired with the fallible-trait work above) is the scaling follow-on.

use std::path::Path;

use mercury_core::{RelayQueueItemState, RelaySubmissionDecision};
use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::store::{MAX_QUEUE_ENTRIES, QueueRecord, QueueStore, TERMINAL_RETENTION_S};

/// `route_id` -> JSON [`PersistedRecord`].
const RECORDS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("mercury_queue_records_v1");
/// `replay_token` -> 1 (used as a set).
const REPLAY: TableDefinition<&[u8], u8> = TableDefinition::new("mercury_replay_tokens_v1");

const STATE_PENDING: u8 = 1;
const STATE_EXPIRED: u8 = 3;

/// Serializable mirror of [`QueueRecord`] (the core enum/struct fields are not `serde`).
#[derive(Serialize, Deserialize)]
struct PersistedRecord {
    route_id: Vec<u8>,
    replay_token: Vec<u8>,
    state: u8,
    created_at_s: i64,
    expires_at_s: i64,
    padding_bucket: i32,
    submission_accepted: bool,
    submission_reason_code: i32,
    submission_audit_class: i32,
    ack_seen: bool,
    ciphertext: Vec<u8>,
    sealed_header: Vec<u8>,
}

fn state_to_u8(s: RelayQueueItemState) -> u8 {
    match s {
        RelayQueueItemState::Absent => 0,
        RelayQueueItemState::Pending => 1,
        RelayQueueItemState::Delivered => 2,
        RelayQueueItemState::Expired => 3,
        RelayQueueItemState::Deleted => 4,
    }
}

fn state_from_u8(v: u8) -> Option<RelayQueueItemState> {
    Some(match v {
        0 => RelayQueueItemState::Absent,
        1 => RelayQueueItemState::Pending,
        2 => RelayQueueItemState::Delivered,
        3 => RelayQueueItemState::Expired,
        4 => RelayQueueItemState::Deleted,
        _ => return None,
    })
}

impl PersistedRecord {
    fn from_record(r: &QueueRecord) -> Self {
        Self {
            route_id: r.route_id.clone(),
            replay_token: r.replay_token.clone(),
            state: state_to_u8(r.state),
            created_at_s: r.created_at_s,
            expires_at_s: r.expires_at_s,
            padding_bucket: r.padding_bucket,
            submission_accepted: r.submission.accepted,
            submission_reason_code: r.submission.reason_code,
            submission_audit_class: r.submission.audit_class,
            ack_seen: r.ack_seen,
            ciphertext: r.ciphertext.clone(),
            sealed_header: r.sealed_header.clone(),
        }
    }

    fn into_record(self) -> Option<QueueRecord> {
        Some(QueueRecord {
            route_id: self.route_id,
            replay_token: self.replay_token,
            state: state_from_u8(self.state)?,
            created_at_s: self.created_at_s,
            expires_at_s: self.expires_at_s,
            padding_bucket: self.padding_bucket,
            submission: RelaySubmissionDecision {
                accepted: self.submission_accepted,
                reason_code: self.submission_reason_code,
                audit_class: self.submission_audit_class,
            },
            ack_seen: self.ack_seen,
            ciphertext: self.ciphertext,
            sealed_header: self.sealed_header,
        })
    }
}

/// A durable [`QueueStore`] backed by a redb database file.
pub struct RedbQueueStore {
    db: Database,
}

impl RedbQueueStore {
    /// Open (creating if absent) a durable queue store at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, redb::Error> {
        let db = Database::create(path)?;
        // Materialize the tables so a brand-new file has them.
        let txn = db.begin_write()?;
        {
            let _ = txn.open_table(RECORDS)?;
            let _ = txn.open_table(REPLAY)?;
        }
        txn.commit()?;
        Ok(Self { db })
    }

    /// The raw JSON bytes for `route_id`, or None (also None on any storage error: fail closed).
    fn record_bytes(&self, route_id: &[u8]) -> Option<Vec<u8>> {
        let txn = self.db.begin_read().ok()?;
        let table = txn.open_table(RECORDS).ok()?;
        let v = table.get(route_id).ok()??;
        Some(v.value().to_vec())
    }

    /// Persist `bytes` under `route_id` (panics on a storage error — see the module error model).
    fn put(&self, route_id: &[u8], bytes: &[u8]) {
        let txn = self.db.begin_write().expect("redb begin_write");
        {
            let mut table = txn.open_table(RECORDS).expect("open records table");
            table.insert(route_id, bytes).expect("redb insert record");
        }
        txn.commit().expect("redb commit");
    }

    /// Read-modify-write the record for `route_id` via `f` (no-op if absent or undecodable).
    fn modify(&self, route_id: &[u8], f: impl FnOnce(&mut PersistedRecord)) {
        let Some(bytes) = self.record_bytes(route_id) else {
            return;
        };
        let Ok(mut rec) = serde_json::from_slice::<PersistedRecord>(&bytes) else {
            return;
        };
        f(&mut rec);
        let out = serde_json::to_vec(&rec).expect("a queue record always serializes");
        self.put(route_id, &out);
    }
}

impl QueueStore for RedbQueueStore {
    fn get(&self, route_id: &[u8]) -> Option<QueueRecord> {
        let bytes = self.record_bytes(route_id)?;
        serde_json::from_slice::<PersistedRecord>(&bytes)
            .ok()
            .and_then(PersistedRecord::into_record)
    }

    fn insert(&mut self, record: QueueRecord) {
        // Retire the superseded record's replay token on overwrite (see InMemoryQueueStore::insert): a
        // route is recycled per message, so without this the REPLAY table grows without bound as old
        // tokens are orphaned. Insert the record and remove the old token in ONE write txn (atomic);
        // guard old != new so the just-inserted token is never dropped.
        let old_token = self.get(&record.route_id).map(|r| r.replay_token);
        let bytes = serde_json::to_vec(&PersistedRecord::from_record(&record))
            .expect("a queue record always serializes");
        let txn = self.db.begin_write().expect("redb begin_write");
        {
            let mut records = txn.open_table(RECORDS).expect("open records table");
            records
                .insert(record.route_id.as_slice(), bytes.as_slice())
                .expect("redb insert record");
        }
        if let Some(old) = old_token
            && old != record.replay_token
        {
            let mut replay = txn.open_table(REPLAY).expect("open replay table");
            let _ = replay.remove(old.as_slice());
        }
        txn.commit().expect("redb commit");
    }

    fn len(&self) -> usize {
        // Count the RECORDS table. Fail closed (mirrors `record_bytes`): an unreadable store is
        // treated as full so the entry cap refuses NEW routes rather than admitting unbounded growth.
        let Ok(txn) = self.db.begin_read() else {
            return MAX_QUEUE_ENTRIES;
        };
        let Ok(table) = txn.open_table(RECORDS) else {
            return MAX_QUEUE_ENTRIES;
        };
        table.len().map(|n| n as usize).unwrap_or(MAX_QUEUE_ENTRIES)
    }

    fn update_state(&mut self, route_id: &[u8], next_state: RelayQueueItemState) {
        let s = state_to_u8(next_state);
        self.modify(route_id, |r| r.state = s);
    }

    fn take_payload(&mut self, route_id: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let Some(bytes) = self.record_bytes(route_id) else {
            return (Vec::new(), Vec::new());
        };
        let Ok(mut rec) = serde_json::from_slice::<PersistedRecord>(&bytes) else {
            return (Vec::new(), Vec::new());
        };
        let ciphertext = std::mem::take(&mut rec.ciphertext);
        let sealed_header = std::mem::take(&mut rec.sealed_header);
        let out = serde_json::to_vec(&rec).expect("a queue record always serializes");
        self.put(route_id, &out);
        (ciphertext, sealed_header)
    }

    fn clear_payload(&mut self, route_id: &[u8]) {
        self.modify(route_id, |r| {
            r.ciphertext.clear();
            r.sealed_header.clear();
        });
    }

    fn mark_acked(&mut self, route_id: &[u8]) {
        self.modify(route_id, |r| r.ack_seen = true);
    }

    fn contains_replay_token(&self, replay_token: &[u8]) -> bool {
        // Fail CLOSED for the replay check: a storage error reads as "seen" so a possible
        // duplicate is REJECTED rather than risk accepting a replay. (For the queue-record reads
        // above, "absent" is the safe failure direction; for the replay set it is the opposite —
        // "seen" is the conservative answer.)
        let Ok(txn) = self.db.begin_read() else {
            return true;
        };
        let Ok(table) = txn.open_table(REPLAY) else {
            return true;
        };
        match table.get(replay_token) {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => true,
        }
    }

    fn insert_replay_token(&mut self, replay_token: &[u8]) {
        let txn = self.db.begin_write().expect("redb begin_write");
        {
            let mut table = txn.open_table(REPLAY).expect("open replay table");
            table
                .insert(replay_token, 1u8)
                .expect("redb insert replay token");
        }
        txn.commit().expect("redb commit");
    }

    fn sweep_expired(&mut self, now_s: i64) -> usize {
        // The sweeper is a LONG-LIVED task; unlike the per-request handlers it must NOT panic on a
        // transient storage error — that would permanently disable TTL enforcement for the life of
        // the process. So every redb step here degrades to "skip this sweep, retry next tick"
        // instead of `.expect()`. redb is transactional: returning before `commit()` lands nothing.
        let Ok(txn) = self.db.begin_write() else {
            return 0;
        };
        let mut swept = 0usize;
        // Replay tokens whose records are being reaped — removed from the REPLAY table below so it,
        // too, stays bounded (mirrors `TERMINAL_RETENTION_S` reaping in the in-memory store).
        let mut reap_tokens: Vec<Vec<u8>> = Vec::new();
        {
            let Ok(mut records) = txn.open_table(RECORDS) else {
                return 0;
            };
            // Pass 1 (read-only): decide tombstone-updates vs reaps. We cannot mutate the table
            // while iterating it, so collect first. `flatten()` skips any errored/undecodable row.
            let mut updates: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
            let mut reap_routes: Vec<Vec<u8>> = Vec::new();
            {
                let Ok(iter) = records.iter() else {
                    return 0;
                };
                for (k, v) in iter.flatten() {
                    let Ok(mut rec) = serde_json::from_slice::<PersistedRecord>(v.value()) else {
                        continue;
                    };
                    if now_s >= rec.expires_at_s.saturating_add(TERMINAL_RETENTION_S) {
                        // Far past expiry: reap the record AND its replay token (bounds both tables).
                        reap_routes.push(k.value().to_vec());
                        reap_tokens.push(rec.replay_token.clone());
                    } else if rec.state == STATE_PENDING && now_s >= rec.expires_at_s {
                        // Just expired: tombstone + clear payload; retain the replay token.
                        rec.state = STATE_EXPIRED;
                        rec.ciphertext.clear();
                        rec.sealed_header.clear();
                        if let Ok(bytes) = serde_json::to_vec(&rec) {
                            updates.push((k.value().to_vec(), bytes));
                        }
                    }
                }
            }
            // Pass 2: apply tombstones, then reap records.
            for (k, bytes) in updates {
                if records.insert(k.as_slice(), bytes.as_slice()).is_ok() {
                    swept += 1;
                }
            }
            for k in &reap_routes {
                let _ = records.remove(k.as_slice());
            }
        } // records table dropped before opening REPLAY

        // Reap the replay tokens for the records removed above (the loop is a no-op when none).
        if let Ok(mut replay) = txn.open_table(REPLAY) {
            for t in &reap_tokens {
                let _ = replay.remove(t.as_slice());
            }
        }
        let _ = txn.commit();
        swept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(route: &[u8]) -> QueueRecord {
        QueueRecord {
            route_id: route.to_vec(),
            replay_token: vec![9u8; 32],
            state: RelayQueueItemState::Pending,
            created_at_s: 100,
            expires_at_s: 400,
            padding_bucket: 3,
            submission: RelaySubmissionDecision {
                accepted: true,
                reason_code: 0,
                audit_class: 1,
            },
            ack_seen: false,
            ciphertext: vec![42u8; 128],
            sealed_header: vec![5u8; 96],
        }
    }

    #[test]
    fn an_item_and_replay_tombstone_survive_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.redb");
        let route = [7u8; 32];
        let token = [9u8; 32];

        {
            let mut store = RedbQueueStore::open(&path).unwrap();
            store.insert(record(&route));
            store.insert_replay_token(&token);
            assert!(store.get(&route).unwrap().has_payload());
        } // store dropped -> db file closed

        // Reopen the SAME path: the item + its payload + the replay tombstone are still there.
        let store = RedbQueueStore::open(&path).unwrap();
        let rec = store.get(&route).expect("the item survives a restart");
        assert_eq!(rec.ciphertext, vec![42u8; 128]);
        assert_eq!(rec.sealed_header, vec![5u8; 96]);
        assert_eq!(rec.state, RelayQueueItemState::Pending);
        assert!(!rec.ack_seen && rec.padding_bucket == 3);
        assert!(store.contains_replay_token(&token));
        assert!(!store.contains_replay_token(&[1u8; 32]));
    }

    #[test]
    fn sweep_expires_pending_past_ttl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("q.redb");
        let mut store = RedbQueueStore::open(&path).unwrap();
        store.insert(record(&[7u8; 32])); // expires_at_s = 400
        store.insert({
            let mut r = record(&[8u8; 32]);
            r.expires_at_s = 1000;
            r
        });

        assert_eq!(store.sweep_expired(500), 1);
        let expired = store.get(&[7u8; 32]).unwrap();
        assert_eq!(expired.state, RelayQueueItemState::Expired);
        assert!(!expired.has_payload());
        let live = store.get(&[8u8; 32]).unwrap();
        assert_eq!(live.state, RelayQueueItemState::Pending);
        assert!(live.has_payload());
    }

    #[test]
    fn sweep_reaps_terminal_records_and_replay_tokens_past_the_horizon() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("q.redb");
        let mut store = RedbQueueStore::open(&path).unwrap();
        let route = [7u8; 32];
        store.insert(record(&route)); // expires_at_s = 400, replay_token = [9u8; 32]
        store.insert_replay_token(&[9u8; 32]);

        // Just past expiry: tombstoned, payload cleared, record + replay token RETAINED.
        assert_eq!(store.sweep_expired(500), 1);
        assert_eq!(
            store.get(&route).unwrap().state,
            RelayQueueItemState::Expired
        );
        assert!(store.contains_replay_token(&[9u8; 32]));

        // Far past expiry: the record AND its replay token are reaped, bounding both tables.
        store.sweep_expired(400 + TERMINAL_RETENTION_S + 1);
        assert!(store.get(&route).is_none());
        assert!(!store.contains_replay_token(&[9u8; 32]));
    }

    #[test]
    fn take_payload_clears_and_then_yields_empties() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("q.redb");
        let mut store = RedbQueueStore::open(&path).unwrap();
        store.insert(record(&[7u8; 32]));

        let (ct, sh) = store.take_payload(&[7u8; 32]);
        assert_eq!(ct, vec![42u8; 128]);
        assert_eq!(sh, vec![5u8; 96]);
        assert!(!store.get(&[7u8; 32]).unwrap().has_payload());

        let (ct2, sh2) = store.take_payload(&[7u8; 32]);
        assert!(ct2.is_empty() && sh2.is_empty());
    }

    #[test]
    fn mark_acked_and_update_state_persist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("q.redb");
        let mut store = RedbQueueStore::open(&path).unwrap();
        store.insert(record(&[7u8; 32]));
        store.update_state(&[7u8; 32], RelayQueueItemState::Delivered);
        store.mark_acked(&[7u8; 32]);
        let rec = store.get(&[7u8; 32]).unwrap();
        assert_eq!(rec.state, RelayQueueItemState::Delivered);
        assert!(rec.ack_seen);
    }
}
