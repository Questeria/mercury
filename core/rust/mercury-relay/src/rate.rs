//! A small fixed-window rate limiter for the relay's OPEN endpoints.
//!
//! `POST /relay/submit` and `POST /directory/publish` are intentionally unauthenticated (a
//! sender is anonymous under sealed-sender; a publisher proves possession only of its own
//! slot), so without a limiter a single source can flood them. This bounds the request rate
//! per coarse client key (the peer IP, when the server is run with connection info).
//!
//! HONEST SCOPE — what this does and does NOT do:
//! - It bounds a SINGLE-SOURCE flood: at most `max_per_window` requests per `window_s` seconds
//!   per key. Real and testable.
//! - It is NOT a defence against a DISTRIBUTED flood from many source IPs (that needs upstream
//!   DDoS protection at a CDN/load balancer), and it is NOT per-SENDER (the sealed-sender
//!   identity is unknown to the relay by design) — it is per-IP. Behind a reverse proxy the
//!   peer IP is the proxy's, so production keys on a forwarded-client header at the proxy.
//! - The tracking table is BOUNDED (`MAX_TRACKED_KEYS`): once full of active windows it denies
//!   new keys (fail-closed) rather than grow memory without limit — so the limiter cannot
//!   itself become a memory-exhaustion vector.

use std::collections::HashMap;
use std::sync::Mutex;

/// Maximum number of distinct keys tracked at once. Bounds the limiter's own memory: a flood of
/// distinct source keys prunes expired windows first, then (if still full of ACTIVE windows) is
/// denied rather than allowed to grow the table without limit.
const MAX_TRACKED_KEYS: usize = 100_000;

#[derive(Debug, Clone, Copy)]
struct Window {
    start_s: i64,
    count: u32,
}

/// Fixed-window limiter: at most `max_per_window` requests per `window_s` seconds per key.
pub struct RateLimiter {
    max_per_window: u32,
    window_s: i64,
    state: Mutex<HashMap<String, Window>>,
}

impl RateLimiter {
    /// A limiter allowing `max_per_window` requests per `window_s` seconds per key. `window_s`
    /// is clamped to at least 1 second.
    pub fn new(max_per_window: u32, window_s: i64) -> Self {
        Self {
            max_per_window,
            window_s: window_s.max(1),
            state: Mutex::new(HashMap::new()),
        }
    }

    /// Record a request from `key` at wall-clock `now_s`; returns `true` if it is within the cap
    /// for the current window (allow), `false` if the cap is exceeded or the tracking table is
    /// full of active windows (deny). Deterministic in `now_s` so it is directly testable.
    pub fn check_at(&self, key: &str, now_s: i64) -> bool {
        let mut state = self.state.lock().expect("rate limiter mutex not poisoned");

        if let Some(w) = state.get_mut(key) {
            if now_s.saturating_sub(w.start_s) >= self.window_s {
                // The previous window elapsed: start a fresh one.
                w.start_s = now_s;
                w.count = 0;
            }
            if w.count >= self.max_per_window {
                return false;
            }
            w.count += 1;
            return true;
        }

        // A new key. Keep the table bounded: prune expired windows, then cap.
        if state.len() >= MAX_TRACKED_KEYS {
            state.retain(|_, w| now_s.saturating_sub(w.start_s) < self.window_s);
            if state.len() >= MAX_TRACKED_KEYS {
                return false;
            }
        }
        state.insert(
            key.to_string(),
            Window {
                start_s: now_s,
                count: 1,
            },
        );
        true
    }

    /// [`check_at`](Self::check_at) using the relay's wall clock.
    pub fn check(&self, key: &str) -> bool {
        self.check_at(key, crate::store::now_unix_seconds())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_the_cap_then_denies_within_a_window() {
        let rl = RateLimiter::new(3, 60);
        assert!(rl.check_at("a", 1000));
        assert!(rl.check_at("a", 1001));
        assert!(rl.check_at("a", 1002));
        // 4th within the window is denied.
        assert!(!rl.check_at("a", 1003));
    }

    #[test]
    fn the_window_resets() {
        let rl = RateLimiter::new(2, 60);
        assert!(rl.check_at("a", 1000));
        assert!(rl.check_at("a", 1010));
        assert!(!rl.check_at("a", 1020)); // over cap, same window
        // Past the window from the start (1000 + 60): a fresh window.
        assert!(rl.check_at("a", 1060));
        assert!(rl.check_at("a", 1061));
        assert!(!rl.check_at("a", 1062));
    }

    #[test]
    fn keys_are_independent() {
        let rl = RateLimiter::new(1, 60);
        assert!(rl.check_at("a", 1000));
        assert!(!rl.check_at("a", 1000));
        // A different key has its own budget.
        assert!(rl.check_at("b", 1000));
        assert!(!rl.check_at("b", 1000));
    }

    #[test]
    fn window_s_is_clamped_to_at_least_one() {
        let rl = RateLimiter::new(1, 0);
        assert!(rl.check_at("a", 1000));
        assert!(!rl.check_at("a", 1000)); // same second, over cap
        assert!(rl.check_at("a", 1001)); // next second -> new window
    }

    #[test]
    fn an_extreme_timestamp_does_not_panic() {
        let rl = RateLimiter::new(1, 60);
        assert!(rl.check_at("a", i64::MIN));
        let _ = rl.check_at("a", i64::MAX);
        let _ = rl.check_at("b", i64::MIN);
    }
}
