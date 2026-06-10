//! Queue storage seam.
//!
//! [`QueueStore`] is the production-backend seam. The shipped implementation is
//! [`InMemoryQueueStore`], which mirrors the `mercury-core`
//! `PrototypeRelayServer` layout exactly: a `BTreeMap` keyed by `route_id`
//! holding [`QueueRecord`]s plus a `BTreeSet` of seen `replay_token`s.
//!
//! A Redis-backed implementation is the intended production backend but is
//! **deliberately deferred**: untested infra must not land. A real backend
//! implements this same trait (e.g. `RedisQueueStore`) with identical
//! semantics — the HTTP layer and the `mercury-core` gates do not change.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use mercury_core::{RelayQueueItemState, RelaySubmissionDecision};
use tokio::sync::Mutex;

/// How long a terminal record (Expired / Delivered / Deleted) — and its replay-token tombstone —
/// is retained past its `expires_at_s` before being REAPED entirely. Until the horizon, terminal
/// records linger only as content-free audit tombstones (their payloads are already cleared); past
/// it they are removed so the store cannot grow without bound. The horizon comfortably exceeds the
/// gate's `max_queue_ttl_s` (7 days), so a replay token always outlives any item it guards.
pub const TERMINAL_RETENTION_S: i64 = 7 * 86_400;

/// One queued relay item.
///
/// Holds only content-free metadata plus the opaque ciphertext + sealed-header
/// bytes the relay shuttles without interpreting. Layout mirrors
/// `mercury_core::PrototypeRelayQueueItem`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueRecord {
    pub route_id: Vec<u8>,
    pub replay_token: Vec<u8>,
    pub state: RelayQueueItemState,
    pub created_at_s: i64,
    pub expires_at_s: i64,
    pub padding_bucket: i32,
    pub submission: RelaySubmissionDecision,
    /// Whether a delivery acknowledgement has already been recorded. Feeds the
    /// `ack_seen` input of `evaluate_delivery_ack` so a duplicate ack on a
    /// `Delivered` record takes the gate's idempotent `DuplicateAck` path
    /// rather than being mistaken for a fresh ack. Set true once an ack is
    /// accepted; the record then survives as a hash-only audit tombstone.
    pub ack_seen: bool,
    /// Opaque ciphertext. Cleared on delivery / expiry / delete.
    pub ciphertext: Vec<u8>,
    /// Opaque sealed header. Cleared on delivery / expiry / delete.
    pub sealed_header: Vec<u8>,
}

impl QueueRecord {
    pub fn has_payload(&self) -> bool {
        !self.ciphertext.is_empty()
    }
}

/// Production-backend seam for relay queue persistence.
///
/// Mirrors the `PrototypeRelayServer` `BTreeMap`(route_id) + `BTreeSet`(replay
/// token) logic. Implementations must be `Send + Sync` so the store can sit
/// behind an `Arc<Mutex<dyn QueueStore>>` shared across Axum handlers and the
/// sweep task. All methods are content-free: they move opaque bytes and
/// content-free metadata only.
pub trait QueueStore: Send + Sync {
    /// Fetch a clone of the record for `route_id`, if present.
    fn get(&self, route_id: &[u8]) -> Option<QueueRecord>;

    /// Insert (or overwrite) a record, keyed by its `route_id`.
    fn insert(&mut self, record: QueueRecord);

    /// Transition the stored record's state. No-op if absent.
    fn update_state(&mut self, route_id: &[u8], next_state: RelayQueueItemState);

    /// Take (and clear) the stored opaque payload, returning
    /// `(ciphertext, sealed_header)`. Returns empties if absent or already
    /// cleared. Mirrors `std::mem::take` on the prototype item.
    fn take_payload(&mut self, route_id: &[u8]) -> (Vec<u8>, Vec<u8>);

    /// Clear the stored opaque payload without returning it (expiry / delete).
    fn clear_payload(&mut self, route_id: &[u8]);

    /// Mark a record's delivery as acknowledged (hash-only audit tombstone).
    /// No-op if absent. The record's state is left `Delivered` so a duplicate
    /// ack is recognised by the gate via `ack_seen`.
    fn mark_acked(&mut self, route_id: &[u8]);

    /// Whether this `replay_token` has been seen (replay defence).
    fn contains_replay_token(&self, replay_token: &[u8]) -> bool;

    /// Record a `replay_token` as seen.
    fn insert_replay_token(&mut self, replay_token: &[u8]);

    /// Sweep expired pending items: clear their payloads and mark them
    /// `Expired`. Returns the number of items transitioned. Replay tokens are
    /// retained (a tombstone) so a re-submit after expiry is still rejected as
    /// a duplicate — matching the gate's replay semantics.
    fn sweep_expired(&mut self, now_s: i64) -> usize;
}

/// In-memory [`QueueStore`]. Mirrors `PrototypeRelayServer` exactly.
#[derive(Debug, Clone, Default)]
pub struct InMemoryQueueStore {
    items: BTreeMap<Vec<u8>, QueueRecord>,
    replay_tokens: BTreeSet<Vec<u8>>,
}

impl InMemoryQueueStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl QueueStore for InMemoryQueueStore {
    fn get(&self, route_id: &[u8]) -> Option<QueueRecord> {
        self.items.get(route_id).cloned()
    }

    fn insert(&mut self, record: QueueRecord) {
        self.items.insert(record.route_id.clone(), record);
    }

    fn update_state(&mut self, route_id: &[u8], next_state: RelayQueueItemState) {
        if let Some(item) = self.items.get_mut(route_id) {
            item.state = next_state;
        }
    }

    fn take_payload(&mut self, route_id: &[u8]) -> (Vec<u8>, Vec<u8>) {
        match self.items.get_mut(route_id) {
            Some(item) => (
                std::mem::take(&mut item.ciphertext),
                std::mem::take(&mut item.sealed_header),
            ),
            None => (Vec::new(), Vec::new()),
        }
    }

    fn clear_payload(&mut self, route_id: &[u8]) {
        if let Some(item) = self.items.get_mut(route_id) {
            item.ciphertext.clear();
            item.sealed_header.clear();
        }
    }

    fn mark_acked(&mut self, route_id: &[u8]) {
        if let Some(item) = self.items.get_mut(route_id) {
            item.ack_seen = true;
        }
    }

    fn contains_replay_token(&self, replay_token: &[u8]) -> bool {
        self.replay_tokens.contains(replay_token)
    }

    fn insert_replay_token(&mut self, replay_token: &[u8]) {
        self.replay_tokens.insert(replay_token.to_vec());
    }

    fn sweep_expired(&mut self, now_s: i64) -> usize {
        let mut swept = 0;
        // Records (and their replay tombstones) to REAP — far past expiry. Collected first because
        // we cannot remove from the map while iterating it.
        let mut reap_routes: Vec<Vec<u8>> = Vec::new();
        let mut reap_tokens: Vec<Vec<u8>> = Vec::new();
        for (route, item) in self.items.iter_mut() {
            if now_s >= item.expires_at_s.saturating_add(TERMINAL_RETENTION_S) {
                // Far past expiry: reap the record AND its replay token so neither set grows
                // without bound. The original message is long gone; a "replay" this old is moot.
                reap_routes.push(route.clone());
                reap_tokens.push(item.replay_token.clone());
            } else if item.state == RelayQueueItemState::Pending && now_s >= item.expires_at_s {
                // Just expired: clear the payload and tombstone it. The replay token is retained
                // (a re-submit before the reap horizon is still rejected as a duplicate).
                item.state = RelayQueueItemState::Expired;
                item.ciphertext.clear();
                item.sealed_header.clear();
                swept += 1;
            }
        }
        for route in &reap_routes {
            self.items.remove(route);
        }
        for token in &reap_tokens {
            self.replay_tokens.remove(token);
        }
        swept
    }
}

/// Spawn a background sweep task that periodically expires stale pending items.
///
/// The task loops forever on `interval`, calling
/// [`QueueStore::sweep_expired`] with a wall-clock `now_s`. Returns the
/// [`tokio::task::JoinHandle`] so callers may abort it on shutdown.
pub fn spawn_sweeper(
    store: Arc<Mutex<dyn QueueStore>>,
    interval: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let now_s = now_unix_seconds();
            let mut guard = store.lock().await;
            guard.sweep_expired(now_s);
        }
    })
}

/// Wall-clock seconds since the Unix epoch. Saturates rather than panicking on
/// a pre-epoch clock.
pub fn now_unix_seconds() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs() as i64,
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mercury_core::{RelaySubmissionDecision, evaluate_relay_submission};

    fn accepted_submission() -> RelaySubmissionDecision {
        // A trivially-accepted submission decision for store-level unit tests.
        RelaySubmissionDecision {
            accepted: true,
            reason_code: 0,
            audit_class: 1,
        }
    }

    fn record(route_id: &[u8], expires_at_s: i64) -> QueueRecord {
        QueueRecord {
            route_id: route_id.to_vec(),
            replay_token: vec![9u8; 32],
            state: RelayQueueItemState::Pending,
            created_at_s: 100,
            expires_at_s,
            padding_bucket: 3,
            submission: accepted_submission(),
            ack_seen: false,
            ciphertext: vec![42u8; 128],
            sealed_header: vec![5u8; 96],
        }
    }

    #[test]
    fn insert_get_and_take_payload() {
        let mut store = InMemoryQueueStore::new();
        let route = [7u8; 32];
        store.insert(record(&route, 400));
        assert_eq!(store.len(), 1);

        let got = store.get(&route).unwrap();
        assert!(got.has_payload());

        let (ct, sh) = store.take_payload(&route);
        assert_eq!(ct, vec![42u8; 128]);
        assert_eq!(sh, vec![5u8; 96]);
        assert!(!store.get(&route).unwrap().has_payload());

        // Second take yields empties.
        let (ct2, sh2) = store.take_payload(&route);
        assert!(ct2.is_empty() && sh2.is_empty());
    }

    #[test]
    fn replay_token_set() {
        let mut store = InMemoryQueueStore::new();
        assert!(!store.contains_replay_token(&[1u8; 32]));
        store.insert_replay_token(&[1u8; 32]);
        assert!(store.contains_replay_token(&[1u8; 32]));
    }

    #[test]
    fn sweep_expires_pending_and_clears_payload() {
        let mut store = InMemoryQueueStore::new();
        store.insert(record(&[7u8; 32], 400));
        store.insert(record(&[8u8; 32], 1000));

        assert_eq!(store.sweep_expired(500), 1);
        let expired = store.get(&[7u8; 32]).unwrap();
        assert_eq!(expired.state, RelayQueueItemState::Expired);
        assert!(!expired.has_payload());
        let live = store.get(&[8u8; 32]).unwrap();
        assert_eq!(live.state, RelayQueueItemState::Pending);
        assert!(live.has_payload());
    }

    #[test]
    fn sweep_reaps_terminal_records_and_their_replay_tokens_past_the_horizon() {
        let mut store = InMemoryQueueStore::new();
        let route = [7u8; 32];
        store.insert(record(&route, 400)); // replay_token = [9u8; 32], expires_at_s = 400
        store.insert_replay_token(&[9u8; 32]);

        // Just past expiry: tombstoned (Expired), payload cleared, record + token RETAINED.
        assert_eq!(store.sweep_expired(500), 1);
        assert_eq!(
            store.get(&route).unwrap().state,
            RelayQueueItemState::Expired
        );
        assert!(store.contains_replay_token(&[9u8; 32]));
        assert_eq!(store.len(), 1);

        // Far past expiry (beyond the retention horizon): the record AND its replay token are reaped.
        store.sweep_expired(400 + TERMINAL_RETENTION_S + 1);
        assert!(store.get(&route).is_none());
        assert!(!store.contains_replay_token(&[9u8; 32]));
        assert!(store.is_empty());
    }

    #[test]
    fn submission_decision_is_the_real_gate() {
        // Sanity: the store stores whatever the real gate decides; we don't
        // fabricate acceptance anywhere in the HTTP path.
        use mercury_core::{OutboundSendDecision, OutboundSendReason, RelaySubmissionInput};
        let decision = evaluate_relay_submission(RelaySubmissionInput {
            version: 1,
            outbound_send: OutboundSendDecision {
                accepted: true,
                can_send: true,
                can_persist_ciphertext: true,
                requires_user_action: false,
                reason: OutboundSendReason::Accepted,
            },
            route_id_len: 32,
            replay_token_len: 32,
            queue_ttl_s: 300,
            max_queue_ttl_s: 86400,
            ciphertext_len: 128,
            max_ciphertext_len: 1048576,
            sealed_header_len: 96,
            plaintext_identity_fields: 0,
            padding_bucket: 3,
        });
        assert!(decision.accepted);
    }
}
