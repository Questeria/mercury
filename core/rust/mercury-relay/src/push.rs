//! Content-free recipient wake seam.
//!
//! After a successful enqueue the relay may wake a recipient. The notification is **content-free**:
//! it carries only the opaque `route_id` handle, never a preview, sender identity, or any
//! plaintext. The gate `evaluate_authenticated_relay_source` independently forbids any plaintext
//! notification preview.
//!
//! Two implementations ship:
//! - [`NoopPushSender`] — drops every notification (the historical default; kept for tests and as
//!   the seam a future APNs/FCM bridge would implement for true closed-process push).
//! - [`InProcessWaker`] — wakes long-poll `GET /relay/wait/{route}` listeners the instant a message
//!   is enqueued, so a *running* client (foreground or minimized-to-tray) receives in real time
//!   instead of on a fixed poll interval. Single-instance only (the wake is in-process); a
//!   multi-instance deployment needs a shared pub/sub or the APNs/FCM bridge.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::Notify;

/// Maximum distinct routes tracked for in-process waking — a HARD cap on the waker's memory. When
/// the map is full it first drops idle entries (no parked waiter); if it is still full (every entry
/// has a parked waiter) a brand-new route is REFUSED rather than inserted past the cap. Refusing is
/// lossless: the queue store is the source of truth, so an untracked route simply misses the instant
/// wake and is delivered on the recipient's next poll (the client polls every loop iteration).
const MAX_WAIT_ROUTES: usize = 100_000;

/// Content-free recipient wake-up seam.
pub trait PushSender: Send + Sync {
    /// Notify the holder of `route_id` that a delivery is pending. Must not transmit any content
    /// beyond the opaque route handle.
    fn notify(&self, route_id: &[u8]);

    /// Subscribe to in-process wake-ups for `route_id` — the handle a long-poll `/relay/wait`
    /// awaits. Returns `None` if this sender has no in-process wait support (e.g.
    /// [`NoopPushSender`]); the caller then falls back to a short delay and ordinary polling.
    /// Default: `None`.
    fn subscribe(&self, _route_id: &[u8]) -> Option<Arc<Notify>> {
        None
    }
}

/// No-op [`PushSender`]. Drops every notification and supports no in-process waiting.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopPushSender;

impl PushSender for NoopPushSender {
    fn notify(&self, _route_id: &[u8]) {
        // Intentionally empty: this sender provides neither push nor in-process wait.
    }
}

/// In-process [`PushSender`] that wakes long-poll `GET /relay/wait/{route}` listeners the instant a
/// message is enqueued for their route. Content-free: only the opaque route handle is ever used as
/// a key — no preview, sender, or plaintext.
#[derive(Default)]
pub struct InProcessWaker {
    routes: Mutex<HashMap<Vec<u8>, Arc<Notify>>>,
}

impl InProcessWaker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get-or-create the wake handle for `route_id`, enforcing the [`MAX_WAIT_ROUTES`] hard cap.
    /// `None` when the map is full of parked-waiter entries (a new route is refused, losslessly —
    /// see the cap's docs).
    fn entry(&self, route_id: &[u8]) -> Option<Arc<Notify>> {
        let mut routes = self.routes.lock().expect("waker mutex not poisoned");
        if let Some(existing) = routes.get(route_id) {
            return Some(existing.clone());
        }
        // Creating a NEW route: if the map is full, first drop idle entries (held only by the map,
        // i.e. no parked waiter). If it is STILL full, refuse — a true hard cap. (Dropping an idle
        // entry with a pending permit, or refusing a new route, only costs a poll interval of
        // latency, never a queued message: the queue store is authoritative.)
        if routes.len() >= MAX_WAIT_ROUTES {
            routes.retain(|_, n| Arc::strong_count(n) > 1);
            if routes.len() >= MAX_WAIT_ROUTES {
                return None;
            }
        }
        let handle = Arc::new(Notify::new());
        routes.insert(route_id.to_vec(), handle.clone());
        Some(handle)
    }
}

impl PushSender for InProcessWaker {
    fn notify(&self, route_id: &[u8]) {
        // `notify_one` stores a permit if no waiter is parked yet, so a wake that races just ahead
        // of a `/relay/wait` subscribe is not lost — the next `notified()` returns immediately. If
        // the route is untracked because the cap was hit, this is a no-op (delivered on next poll).
        if let Some(handle) = self.entry(route_id) {
            handle.notify_one();
        }
    }

    fn subscribe(&self, route_id: &[u8]) -> Option<Arc<Notify>> {
        self.entry(route_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn notify_before_subscribe_is_not_lost() {
        let waker = InProcessWaker::new();
        let route = b"route-a";
        // A message enqueued before the client parks its wait: the permit is stored.
        waker.notify(route);
        let handle = waker.subscribe(route).unwrap();
        // notified() returns immediately because the permit is already waiting.
        let woken = tokio::time::timeout(Duration::from_millis(200), handle.notified())
            .await
            .is_ok();
        assert!(
            woken,
            "a notify that preceded the subscribe must still wake it"
        );
    }

    #[tokio::test]
    async fn subscribe_then_notify_wakes_the_waiter() {
        let waker = Arc::new(InProcessWaker::new());
        let route = b"route-b";
        let handle = waker.subscribe(route).unwrap();
        let w2 = Arc::clone(&waker);
        let task = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(2), handle.notified())
                .await
                .is_ok()
        });
        // Give the waiter a moment to park, then wake it.
        tokio::time::sleep(Duration::from_millis(20)).await;
        w2.notify(route);
        assert!(
            task.await.unwrap(),
            "an enqueue must wake a parked /relay/wait"
        );
    }

    #[tokio::test]
    async fn distinct_routes_are_independent() {
        let waker = InProcessWaker::new();
        waker.notify(b"route-x");
        // route-y has no pending wake: its notified() should time out.
        let handle_y = waker.subscribe(b"route-y").unwrap();
        let woken = tokio::time::timeout(Duration::from_millis(100), handle_y.notified())
            .await
            .is_ok();
        assert!(!woken, "a wake for one route must not wake another");
    }

    #[test]
    fn noop_has_no_in_process_wait() {
        assert!(NoopPushSender.subscribe(b"route").is_none());
    }
}
