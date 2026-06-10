//! Transport layer: how client message blobs reach a peer.
//!
//! The client runtime is transport-agnostic — it produces/consumes opaque byte blobs
//! ([`crate::MercuryClient::initiate`] / `send` / `receive`). A [`Transport`] moves a
//! blob to a route and polls a route for blobs. Two impls are provided: an in-process
//! [`InMemoryTransport`] (tests/demos) and a [`RelayTransport`] that talks to the real
//! Mercury relay over HTTP.
//!
//! Routing note (Milestone 1): the route is the recipient's account id. This means the
//! relay learns the recipient (no sealed-sender yet) and a route holds one in-flight
//! message at a time (the relay is a single-slot mailbox per route, deliver-once). The
//! sealed-sender outer envelope and multi-message mailbox sequencing are later increments.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// A transport error. Distinct from a crypto/[`crate::ClientError`]: this is about
/// reaching the relay, not protecting the message.
#[derive(Debug, Clone)]
pub enum TransportError {
    /// The relay could not be reached (connection/IO failure).
    Network(String),
    /// The relay reached but refused the request (e.g. submission gate reject).
    Rejected(String),
    /// The relay's response was malformed (bad hex / missing field).
    Malformed,
}

impl core::fmt::Display for TransportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TransportError::Network(e) => write!(f, "relay network error: {e}"),
            TransportError::Rejected(r) => write!(f, "relay rejected the request: {r}"),
            TransportError::Malformed => f.write_str("malformed relay response"),
        }
    }
}

impl std::error::Error for TransportError {}

/// A route-ownership authorization for polling: a proof that the caller holds the identity key
/// whose account id IS the route it is draining (so only the genuine recipient can poll/ack
/// their own queue). Produced by [`crate::MercuryClient::sign_poll_auth`] over
/// [`mercury_keys::sign_relay_route_pop`]. The [`RelayTransport`] sends it as headers for the
/// relay to verify; the in-process [`InMemoryTransport`] has no untrusted boundary and ignores
/// it.
#[derive(Clone, Debug)]
pub struct PollAuth {
    /// The poller's Ed25519 identity public key. The route equals the account id derived from
    /// it, so the relay can confirm the poller owns the route.
    pub identity_pub: [u8; 32],
    /// Unix seconds the proof was signed at; the relay enforces a freshness window.
    pub timestamp_s: i64,
    /// Ed25519 route-ownership signature over `(route_id, timestamp_s)`.
    pub pop_sig: [u8; 64],
}

impl PollAuth {
    /// An unauthenticated placeholder: accepted by the in-memory transport (which has no auth
    /// boundary), rejected by the hardened relay. Use [`crate::MercuryClient::sign_poll_auth`]
    /// to poll the real relay.
    pub fn unauthenticated() -> Self {
        Self {
            identity_pub: [0u8; 32],
            timestamp_s: 0,
            pop_sig: [0u8; 64],
        }
    }
}

/// The outcome of a username claim, as reported by the relay registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsernameClaimOutcome {
    /// A brand-new binding `handle -> our account id` was created.
    Created,
    /// The handle was already bound to OUR account id (idempotent success).
    AlreadyOwned,
    /// The handle is already taken by a DIFFERENT account (first-claimant-wins refusal).
    Taken,
    /// The handle was rejected as illegal, or the proof of possession did not verify.
    Rejected,
    /// The registry is at capacity (or the directory could not commit the claim) — try later.
    Unavailable,
}

/// A raw username lookup as served by the relay: the claimed account id plus the KT inclusion proof
/// the CLIENT must verify against the directory's VRF key before trusting the binding. The proof is
/// carried as its serialized JSON so the transport layer stays free of the AKD proof types; the
/// verifier ([`crate::MercuryClient::resolve_username`]) deserializes + checks it.
#[derive(Clone, Debug)]
pub struct UsernameLookup {
    /// The account id the relay claims the handle resolves to. UNTRUSTED until the proof verifies.
    pub account_id: [u8; 32],
    /// Directory epoch the proof is anchored to.
    pub epoch: u64,
    /// 32-byte AKD root hash at that epoch.
    pub root_hash: Vec<u8>,
    /// The serialized AKD inclusion (`LookupProof`) JSON for label `username:<handle>`.
    pub proof_json: String,
}

/// The directory's pinnable public keys, for trust-on-first-use bootstrap (see `docs/USERNAMES.md`).
#[derive(Clone, Debug)]
pub struct KtPublicKeys {
    /// VRF public key — what an inclusion proof is verified against.
    pub vrf_public_key: Vec<u8>,
    /// 32-byte Ed25519 log public key — what signed tree heads are verified against.
    pub log_public_key: [u8; 32],
}

/// Moves opaque client blobs between peers via routes (recipient account ids in 1a/1b),
/// and carries the public prekey directory (publish/fetch a contact card by account id).
pub trait Transport {
    /// Submit `blob` for delivery to `route`.
    fn submit(&self, route: &[u8; 32], blob: &[u8]) -> Result<(), TransportError>;
    /// Poll `route` for the next available blob, if any. `auth` is the route-ownership proof
    /// the real relay verifies (the in-memory transport ignores it).
    fn poll(&self, route: &[u8; 32], auth: &PollAuth) -> Result<Option<Vec<u8>>, TransportError>;
    /// Publish an opaque contact `card` to the directory, authorized by a proof of possession
    /// (`pop_sig` = an Ed25519 signature by `identity_pub` over the card, from
    /// [`mercury_keys::sign_directory_publish`]). The directory slot is DERIVED from
    /// `identity_pub`, so a publisher can only ever write its own slot.
    fn publish_card(
        &self,
        identity_pub: &[u8; 32],
        card: &[u8],
        pop_sig: &[u8; 64],
    ) -> Result<(), TransportError>;
    /// Fetch the opaque contact card published for `account_id`, if any. The caller MUST
    /// verify the returned card before use (see [`crate::MercuryClient::fetch_contact`]).
    fn fetch_card(&self, account_id: &[u8; 32]) -> Result<Option<Vec<u8>>, TransportError>;

    /// Current time (Unix seconds) to stamp a route-ownership poll proof with. Defaults to the
    /// local clock; [`RelayTransport`] overrides it with the RELAY's clock so a skewed local clock
    /// can't push the proof outside the relay's freshness window (the cause of a spurious
    /// "transport failed"). The in-memory transport has no clock boundary, so the default is fine.
    fn proof_time(&self) -> i64 {
        local_unix_seconds()
    }

    /// Long-poll for a pending delivery on `route`: block until the relay signals a delivery is
    /// likely pending, or a bounded server-side timeout elapses, then return `true` ("poll now") or
    /// `false` ("timed out / not supported — re-wait or fall back to interval polling"). `auth` is
    /// the SAME route-ownership proof [`poll`](Transport::poll) uses (the relay maps `/relay/wait`
    /// to the Poll operation). A network error returns `Ok(false)` rather than erroring, so a
    /// delivery loop degrades to polling instead of stalling. Default: `Ok(false)` immediately —
    /// transports without a long-poll endpoint (the in-memory + dev shims) leave pacing to the
    /// caller.
    fn wait(&self, _route: &[u8; 32], _auth: &PollAuth) -> Result<bool, TransportError> {
        Ok(false)
    }

    /// Broker an opaque contact `card` under a short-lived, single-use PAIRING CODE, authorized by a
    /// pairing proof of possession (`mercury_keys::sign_pairing_publish`). Returns the human-typable
    /// code the peer enters to fetch the card once. Default: unsupported (fail-closed).
    fn publish_pairing(
        &self,
        _identity_pub: &[u8; 32],
        _card: &[u8],
        _pop_sig: &[u8; 64],
    ) -> Result<String, TransportError> {
        Err(TransportError::Rejected(
            "pairing not supported".to_string(),
        ))
    }

    /// Fetch (and consume) the opaque contact card brokered under `code`, if any. The caller MUST
    /// re-verify the returned card before use. Default: nothing brokered.
    fn fetch_pairing(&self, _code: &str) -> Result<Option<Vec<u8>>, TransportError> {
        Ok(None)
    }

    /// Claim `username` (first-claimant-wins) for our account, authorized by a username-claim proof
    /// of possession (`mercury_keys::sign_username_claim` over the NORMALIZED handle). Default:
    /// unsupported (fail-closed).
    fn claim_username(
        &self,
        _identity_pub: &[u8; 32],
        _username: &str,
        _pop_sig: &[u8; 64],
    ) -> Result<UsernameClaimOutcome, TransportError> {
        Err(TransportError::Rejected(
            "usernames not supported".to_string(),
        ))
    }

    /// Look up `username` -> account id + KT inclusion proof. The result is UNTRUSTED until the
    /// proof is verified against the pinned VRF key. Default: no registry (None).
    fn lookup_username(&self, _username: &str) -> Result<Option<UsernameLookup>, TransportError> {
        Ok(None)
    }

    /// Fetch the directory's pinnable KT public keys, for first-use pinning. Default: no directory.
    fn kt_public_keys(&self) -> Result<Option<KtPublicKeys>, TransportError> {
        Ok(None)
    }

    /// Fetch the directory's current LOG-SIGNED tree head: `(tree_size, root_hash, timestamp_s,
    /// log_signature)`. The caller verifies the signature against its PINNED log key
    /// (`mercury_kt::verify_signed_tree_head`) — serving it confers nothing. Default: none.
    fn kt_signed_tree_head(
        &self,
    ) -> Result<Option<(u64, [u8; 32], i64, [u8; 64])>, TransportError> {
        Ok(None)
    }
}

/// In-process transport: a shared per-route FIFO. Cloning shares the same queues, so two
/// clients in one test/demo can exchange blobs without a network. Multi-slot (unlike the
/// real relay's single-slot route), so it also exercises multi-message + reorder paths.
/// Per-route FIFO queues shared between cloned [`InMemoryTransport`] handles.
type RouteQueues = Arc<Mutex<HashMap<[u8; 32], VecDeque<Vec<u8>>>>>;
/// Published contact cards (account id -> opaque card) shared between cloned handles.
type CardStore = Arc<Mutex<HashMap<[u8; 32], Vec<u8>>>>;
/// Brokered pairing cards (code -> opaque card) shared between cloned handles.
type PairingCards = Arc<Mutex<HashMap<String, Vec<u8>>>>;
/// Username registry (normalized handle -> account id) shared between cloned handles.
type UsernameBindings = Arc<Mutex<HashMap<String, [u8; 32]>>>;

#[derive(Clone, Default)]
pub struct InMemoryTransport {
    queues: RouteQueues,
    cards: CardStore,
    pairing: PairingCards,
    usernames: UsernameBindings,
}

impl InMemoryTransport {
    /// A fresh, empty in-memory transport.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Transport for InMemoryTransport {
    fn submit(&self, route: &[u8; 32], blob: &[u8]) -> Result<(), TransportError> {
        let mut q = self.queues.lock().expect("transport mutex not poisoned");
        q.entry(*route).or_default().push_back(blob.to_vec());
        Ok(())
    }

    fn poll(&self, route: &[u8; 32], _auth: &PollAuth) -> Result<Option<Vec<u8>>, TransportError> {
        // In-process: no untrusted boundary, so the route-ownership proof is not needed here.
        let mut q = self.queues.lock().expect("transport mutex not poisoned");
        Ok(q.get_mut(route).and_then(|d| d.pop_front()))
    }

    fn publish_card(
        &self,
        identity_pub: &[u8; 32],
        card: &[u8],
        pop_sig: &[u8; 64],
    ) -> Result<(), TransportError> {
        // Faithful relay double: authorize via the SAME proof the real relay checks, and store
        // the card under the slot derived from the verified identity key. A bad proof is
        // rejected (so tests exercise the auth path), never silently stored.
        let account_id = mercury_keys::verify_directory_publish(identity_pub, card, pop_sig)
            .ok_or_else(|| TransportError::Rejected("unauthorized".to_string()))?;
        let mut c = self.cards.lock().expect("transport mutex not poisoned");
        c.insert(account_id, card.to_vec());
        Ok(())
    }

    fn fetch_card(&self, account_id: &[u8; 32]) -> Result<Option<Vec<u8>>, TransportError> {
        let c = self.cards.lock().expect("transport mutex not poisoned");
        Ok(c.get(account_id).cloned())
    }

    fn publish_pairing(
        &self,
        identity_pub: &[u8; 32],
        card: &[u8],
        pop_sig: &[u8; 64],
    ) -> Result<String, TransportError> {
        // Faithful double: authorize with the SAME pairing proof the real relay checks.
        mercury_keys::verify_pairing_publish(identity_pub, card, pop_sig)
            .ok_or_else(|| TransportError::Rejected("unauthorized".to_string()))?;
        // Mint a random code (the in-memory double's own scheme; the real relay's alphabet does not
        // matter to a test). Single-use, like the relay.
        let mut raw = [0u8; 8];
        getrandom::getrandom(&mut raw).map_err(|e| TransportError::Network(e.to_string()))?;
        let code = hex_encode(&raw).to_uppercase();
        self.pairing
            .lock()
            .expect("transport mutex not poisoned")
            .insert(code.clone(), card.to_vec());
        Ok(code)
    }

    fn fetch_pairing(&self, code: &str) -> Result<Option<Vec<u8>>, TransportError> {
        // Single-use: remove on read, matching the relay's deliver-once broker.
        Ok(self
            .pairing
            .lock()
            .expect("transport mutex not poisoned")
            .remove(&code.to_uppercase()))
    }

    fn claim_username(
        &self,
        identity_pub: &[u8; 32],
        username: &str,
        pop_sig: &[u8; 64],
    ) -> Result<UsernameClaimOutcome, TransportError> {
        let Some(name) = mercury_keys::normalize_username(username) else {
            return Ok(UsernameClaimOutcome::Rejected);
        };
        let Some(account_id) = mercury_keys::verify_username_claim(identity_pub, &name, pop_sig)
        else {
            return Ok(UsernameClaimOutcome::Rejected);
        };
        let mut reg = self.usernames.lock().expect("transport mutex not poisoned");
        match reg.get(&name) {
            Some(existing) if *existing == account_id => Ok(UsernameClaimOutcome::AlreadyOwned),
            Some(_) => Ok(UsernameClaimOutcome::Taken),
            None => {
                reg.insert(name, account_id);
                Ok(UsernameClaimOutcome::Created)
            }
        }
    }

    fn lookup_username(&self, username: &str) -> Result<Option<UsernameLookup>, TransportError> {
        let Some(name) = mercury_keys::normalize_username(username) else {
            return Ok(None);
        };
        let reg = self.usernames.lock().expect("transport mutex not poisoned");
        Ok(reg.get(&name).map(|account_id| UsernameLookup {
            account_id: *account_id,
            epoch: 0,
            root_hash: vec![0u8; 32],
            // The in-memory double does NOT run an AKD log, so it cannot produce a real inclusion
            // proof. This sentinel does not deserialize to a `LookupProof`, so `resolve_username`
            // FAILS CLOSED over the double — the real proof path is covered by the relay integration
            // test. (The double is faithful for pairing + the claim/registry logic, not for KT.)
            proof_json: "null".to_string(),
        }))
    }

    // `kt_public_keys` falls through to the default (`None`): the double has no KT directory, so a
    // username resolve over it has no key to verify against and correctly fails closed.
}

// ---------------------------------------------------------------------------
// HTTP relay transport
// ---------------------------------------------------------------------------

/// Relay submission framing constants (mirrors the values the spine test uses).
const SEALED_HEADER_PLACEHOLDER_LEN: usize = 96;

/// Talks to the real Mercury relay over HTTP (`POST /relay/submit`, `GET /relay/poll`).
pub struct RelayTransport {
    base_url: String,
    /// Relay-minus-local clock offset (seconds), learned from the relay's signed-tree-head
    /// timestamp. `i64::MIN` = not yet synced. Lets [`Transport::proof_time`] stamp the
    /// route-ownership proof with the relay's clock, so client/relay skew can't reject it.
    clock_offset: std::sync::atomic::AtomicI64,
}

impl RelayTransport {
    /// Target a relay at `base_url`, e.g. `http://127.0.0.1:8787`.
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut base_url = base_url.into();
        while base_url.ends_with('/') {
            base_url.pop();
        }
        Self {
            base_url,
            clock_offset: std::sync::atomic::AtomicI64::new(i64::MIN),
        }
    }

    /// Best-effort: read the relay's current Unix time from its unauthenticated signed tree head
    /// (`GET /kt/sth`) and cache `relay_now - local_now`. On any failure the offset stays unsynced
    /// and `proof_time()` falls back to the local clock.
    fn sync_clock_from_relay(&self) {
        let url = format!("{}/kt/sth", self.base_url);
        let Ok(resp) = ureq::get(&url).call() else {
            return;
        };
        let Ok(text) = resp.into_string() else {
            return;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            return;
        };
        if let Some(relay_s) = v.get("timestamp_s").and_then(|t| t.as_i64()) {
            self.clock_offset.store(
                relay_s - local_unix_seconds(),
                std::sync::atomic::Ordering::Relaxed,
            );
        }
    }
}

impl Transport for RelayTransport {
    fn submit(&self, route: &[u8; 32], blob: &[u8]) -> Result<(), TransportError> {
        let mut replay = [0u8; 32];
        getrandom::getrandom(&mut replay).map_err(|e| TransportError::Network(e.to_string()))?;

        let body = serde_json::json!({
            "route_id": hex_encode(route),
            "replay_token": hex_encode(&replay),
            "ciphertext": hex_encode(blob),
            "sealed_header": hex_encode(&[0x05u8; SEALED_HEADER_PLACEHOLDER_LEN]),
            "queue_ttl_s": 300,
            "max_queue_ttl_s": 86400,
            "max_ciphertext_len": 1_048_576,
            "padding_bucket": 3,
            "outbound_send": {
                "accepted": true,
                "can_send": true,
                "can_persist_ciphertext": true,
                "requires_user_action": false,
                "reason_code": 0
            }
        });
        let body = serde_json::to_string(&body).expect("submit body serializes");

        let url = format!("{}/relay/submit", self.base_url);
        match ureq::post(&url)
            .set("content-type", "application/json")
            .send_string(&body)
        {
            // Any 2xx (the relay returns 202 ACCEPTED) means the message was queued.
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(_, resp)) => {
                let reason = resp
                    .into_string()
                    .ok()
                    .and_then(|s| {
                        serde_json::from_str::<serde_json::Value>(&s)
                            .ok()
                            .and_then(|v| {
                                v.get("reason").and_then(|r| r.as_str()).map(String::from)
                            })
                    })
                    .unwrap_or_else(|| "rejected".to_string());
                Err(TransportError::Rejected(reason))
            }
            Err(e) => Err(TransportError::Network(e.to_string())),
        }
    }

    fn poll(&self, route: &[u8; 32], auth: &PollAuth) -> Result<Option<Vec<u8>>, TransportError> {
        let url = format!("{}/relay/poll/{}", self.base_url, hex_encode(route));
        let resp = match ureq::get(&url)
            // Route-ownership proof (the hardened relay verifies these): the poller proves it
            // holds the identity key whose account id is this route.
            .set("x-mercury-identity-pub", &hex_encode(&auth.identity_pub))
            .set("x-mercury-pop-timestamp", &auth.timestamp_s.to_string())
            .set("x-mercury-route-pop", &hex_encode(&auth.pop_sig))
            .call()
        {
            Ok(resp) => resp,
            // 404 (nothing queued) / 410 (delivered or expired) -> no message available.
            Err(ureq::Error::Status(404, _)) | Err(ureq::Error::Status(410, _)) => return Ok(None),
            Err(ureq::Error::Status(_, _)) => {
                return Err(TransportError::Rejected("poll refused".to_string()));
            }
            Err(e) => return Err(TransportError::Network(e.to_string())),
        };

        let text = resp
            .into_string()
            .map_err(|e| TransportError::Network(e.to_string()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| TransportError::Malformed)?;
        let ciphertext_hex = value
            .get("ciphertext")
            .and_then(|c| c.as_str())
            .ok_or(TransportError::Malformed)?;
        Ok(Some(hex_decode(ciphertext_hex)?))
    }

    fn wait(&self, route: &[u8; 32], auth: &PollAuth) -> Result<bool, TransportError> {
        let url = format!("{}/relay/wait/{}", self.base_url, hex_encode(route));
        // The relay holds this request open until a delivery is enqueued for the route or its
        // ~25s server-side timeout; allow more than that here so a legitimate hold is not aborted
        // as a client timeout. Any error (incl. timeout) -> Ok(false): the caller then polls /
        // re-waits, so a transient blip degrades the loop to polling instead of stalling it.
        match ureq::get(&url)
            .timeout(std::time::Duration::from_secs(40))
            // Same route-ownership proof as poll (the relay maps `/relay/wait` to the Poll op).
            .set("x-mercury-identity-pub", &hex_encode(&auth.identity_pub))
            .set("x-mercury-pop-timestamp", &auth.timestamp_s.to_string())
            .set("x-mercury-route-pop", &hex_encode(&auth.pop_sig))
            .call()
        {
            // 200 = a delivery is likely pending (poll now); 204 (also 2xx) = timed out (re-wait).
            Ok(resp) => Ok(resp.status() == 200),
            Err(_) => Ok(false),
        }
    }

    fn publish_card(
        &self,
        identity_pub: &[u8; 32],
        card: &[u8],
        pop_sig: &[u8; 64],
    ) -> Result<(), TransportError> {
        let body = serde_json::json!({
            "identity_pub": hex_encode(identity_pub),
            "card": hex_encode(card),
            "pop_sig": hex_encode(pop_sig),
        });
        let body = serde_json::to_string(&body).expect("publish body serializes");

        let url = format!("{}/directory/publish", self.base_url);
        match ureq::post(&url)
            .set("content-type", "application/json")
            .send_string(&body)
        {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(_, resp)) => {
                let reason = resp
                    .into_string()
                    .ok()
                    .and_then(|s| {
                        serde_json::from_str::<serde_json::Value>(&s)
                            .ok()
                            .and_then(|v| {
                                v.get("reason").and_then(|r| r.as_str()).map(String::from)
                            })
                    })
                    .unwrap_or_else(|| "rejected".to_string());
                Err(TransportError::Rejected(reason))
            }
            Err(e) => Err(TransportError::Network(e.to_string())),
        }
    }

    fn fetch_card(&self, account_id: &[u8; 32]) -> Result<Option<Vec<u8>>, TransportError> {
        let url = format!(
            "{}/directory/fetch/{}",
            self.base_url,
            hex_encode(account_id)
        );
        let resp = match ureq::get(&url).call() {
            Ok(resp) => resp,
            // 404 -> no card published for this account id.
            Err(ureq::Error::Status(404, _)) => return Ok(None),
            Err(ureq::Error::Status(_, _)) => {
                return Err(TransportError::Rejected("fetch refused".to_string()));
            }
            Err(e) => return Err(TransportError::Network(e.to_string())),
        };

        let text = resp
            .into_string()
            .map_err(|e| TransportError::Network(e.to_string()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| TransportError::Malformed)?;
        let card_hex = value
            .get("card")
            .and_then(|c| c.as_str())
            .ok_or(TransportError::Malformed)?;
        Ok(Some(hex_decode(card_hex)?))
    }

    fn publish_pairing(
        &self,
        identity_pub: &[u8; 32],
        card: &[u8],
        pop_sig: &[u8; 64],
    ) -> Result<String, TransportError> {
        let body = serde_json::json!({
            "identity_pub": hex_encode(identity_pub),
            "card": hex_encode(card),
            "pop_sig": hex_encode(pop_sig),
        });
        let url = format!("{}/pair/publish", self.base_url);
        match ureq::post(&url)
            .set("content-type", "application/json")
            .send_string(&serde_json::to_string(&body).expect("pairing body serializes"))
        {
            Ok(resp) => {
                let text = resp
                    .into_string()
                    .map_err(|e| TransportError::Network(e.to_string()))?;
                let value: serde_json::Value =
                    serde_json::from_str(&text).map_err(|_| TransportError::Malformed)?;
                value
                    .get("code")
                    .and_then(|c| c.as_str())
                    .map(String::from)
                    .ok_or(TransportError::Malformed)
            }
            Err(ureq::Error::Status(_, resp)) => Err(TransportError::Rejected(reason_of(resp))),
            Err(e) => Err(TransportError::Network(e.to_string())),
        }
    }

    fn fetch_pairing(&self, code: &str) -> Result<Option<Vec<u8>>, TransportError> {
        // Canonicalize client-side to the same shape the relay uses (uppercase ASCII alphanumerics,
        // hyphen/space stripped) BEFORE building the URL, so a typed code can never carry path-special
        // characters into the request. An empty result after canonicalization is simply not found.
        let code: String = code
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .flat_map(|c| c.to_uppercase())
            .collect();
        if code.is_empty() {
            return Ok(None);
        }
        let url = format!("{}/pair/fetch/{}", self.base_url, code);
        let resp = match ureq::get(&url).call() {
            Ok(resp) => resp,
            Err(ureq::Error::Status(404, _)) => return Ok(None),
            Err(ureq::Error::Status(_, _)) => {
                return Err(TransportError::Rejected(
                    "pairing fetch refused".to_string(),
                ));
            }
            Err(e) => return Err(TransportError::Network(e.to_string())),
        };
        let text = resp
            .into_string()
            .map_err(|e| TransportError::Network(e.to_string()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| TransportError::Malformed)?;
        let card_hex = value
            .get("card")
            .and_then(|c| c.as_str())
            .ok_or(TransportError::Malformed)?;
        Ok(Some(hex_decode(card_hex)?))
    }

    fn claim_username(
        &self,
        identity_pub: &[u8; 32],
        username: &str,
        pop_sig: &[u8; 64],
    ) -> Result<UsernameClaimOutcome, TransportError> {
        let body = serde_json::json!({
            "identity_pub": hex_encode(identity_pub),
            "username": username,
            "pop_sig": hex_encode(pop_sig),
        });
        let url = format!("{}/username/claim", self.base_url);
        match ureq::post(&url)
            .set("content-type", "application/json")
            .send_string(&serde_json::to_string(&body).expect("claim body serializes"))
        {
            Ok(resp) => {
                // 2xx: the body's `created` flag distinguishes a fresh bind from an idempotent one.
                let created = resp
                    .into_string()
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| v.get("created").and_then(|c| c.as_bool()))
                    .unwrap_or(false);
                Ok(if created {
                    UsernameClaimOutcome::Created
                } else {
                    UsernameClaimOutcome::AlreadyOwned
                })
            }
            Err(ureq::Error::Status(409, _)) => Ok(UsernameClaimOutcome::Taken),
            Err(ureq::Error::Status(400, _)) | Err(ureq::Error::Status(403, _)) => {
                Ok(UsernameClaimOutcome::Rejected)
            }
            Err(ureq::Error::Status(503, _)) => Ok(UsernameClaimOutcome::Unavailable),
            Err(ureq::Error::Status(_, resp)) => Err(TransportError::Rejected(reason_of(resp))),
            Err(e) => Err(TransportError::Network(e.to_string())),
        }
    }

    fn lookup_username(&self, username: &str) -> Result<Option<UsernameLookup>, TransportError> {
        // Defense-in-depth: only a canonical `[a-z0-9_]` handle is ever placed in the URL path, so a
        // raw caller cannot inject path-traversal / URL-special characters. `resolve_username` already
        // normalizes; re-normalizing here closes the low-level entry point too. A non-canonical handle
        // is treated as unresolvable (no such username) rather than built into a request.
        let Some(username) = mercury_keys::normalize_username(username) else {
            return Ok(None);
        };
        let url = format!("{}/username/{}", self.base_url, username);
        let resp = match ureq::get(&url).call() {
            Ok(resp) => resp,
            Err(ureq::Error::Status(404, _)) => return Ok(None),
            Err(ureq::Error::Status(_, _)) => {
                return Err(TransportError::Rejected(
                    "username lookup refused".to_string(),
                ));
            }
            Err(e) => return Err(TransportError::Network(e.to_string())),
        };
        let text = resp
            .into_string()
            .map_err(|e| TransportError::Network(e.to_string()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| TransportError::Malformed)?;
        let account_id_hex = value
            .get("account_id")
            .and_then(|c| c.as_str())
            .ok_or(TransportError::Malformed)?;
        let account_id: [u8; 32] = hex_decode(account_id_hex)?
            .try_into()
            .map_err(|_| TransportError::Malformed)?;
        let epoch = value
            .get("epoch")
            .and_then(|e| e.as_u64())
            .ok_or(TransportError::Malformed)?;
        let root_hash = hex_decode(
            value
                .get("root_hash")
                .and_then(|r| r.as_str())
                .ok_or(TransportError::Malformed)?,
        )?;
        // Carry the proof object through as its serialized JSON; the verifier deserializes it.
        let proof_json = value
            .get("proof")
            .map(|p| p.to_string())
            .ok_or(TransportError::Malformed)?;
        Ok(Some(UsernameLookup {
            account_id,
            epoch,
            root_hash,
            proof_json,
        }))
    }

    fn kt_signed_tree_head(
        &self,
    ) -> Result<Option<(u64, [u8; 32], i64, [u8; 64])>, TransportError> {
        let url = format!("{}/kt/sth", self.base_url);
        let resp = match ureq::get(&url).call() {
            Ok(resp) => resp,
            Err(ureq::Error::Status(404, _)) => return Ok(None),
            Err(ureq::Error::Status(_, _)) => {
                return Err(TransportError::Rejected("sth fetch refused".to_string()));
            }
            Err(e) => return Err(TransportError::Network(e.to_string())),
        };
        let text = resp
            .into_string()
            .map_err(|e| TransportError::Network(e.to_string()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| TransportError::Malformed)?;
        let tree_size = value
            .get("tree_size")
            .and_then(|v| v.as_u64())
            .ok_or(TransportError::Malformed)?;
        let root: [u8; 32] = hex_decode(
            value
                .get("root_hash")
                .and_then(|v| v.as_str())
                .ok_or(TransportError::Malformed)?,
        )?
        .try_into()
        .map_err(|_| TransportError::Malformed)?;
        let timestamp_s = value
            .get("timestamp_s")
            .and_then(|v| v.as_i64())
            .ok_or(TransportError::Malformed)?;
        let sig: [u8; 64] = hex_decode(
            value
                .get("log_signature")
                .and_then(|v| v.as_str())
                .ok_or(TransportError::Malformed)?,
        )?
        .try_into()
        .map_err(|_| TransportError::Malformed)?;
        Ok(Some((tree_size, root, timestamp_s, sig)))
    }

    fn kt_public_keys(&self) -> Result<Option<KtPublicKeys>, TransportError> {
        let url = format!("{}/kt/vrf-key", self.base_url);
        let resp = match ureq::get(&url).call() {
            Ok(resp) => resp,
            Err(ureq::Error::Status(404, _)) => return Ok(None),
            Err(ureq::Error::Status(_, _)) => {
                return Err(TransportError::Rejected(
                    "vrf-key fetch refused".to_string(),
                ));
            }
            Err(e) => return Err(TransportError::Network(e.to_string())),
        };
        let text = resp
            .into_string()
            .map_err(|e| TransportError::Network(e.to_string()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| TransportError::Malformed)?;
        let vrf_public_key = hex_decode(
            value
                .get("vrf_public_key")
                .and_then(|v| v.as_str())
                .ok_or(TransportError::Malformed)?,
        )?;
        let log_public_key: [u8; 32] = hex_decode(
            value
                .get("log_public_key")
                .and_then(|v| v.as_str())
                .ok_or(TransportError::Malformed)?,
        )?
        .try_into()
        .map_err(|_| TransportError::Malformed)?;
        Ok(Some(KtPublicKeys {
            vrf_public_key,
            log_public_key,
        }))
    }

    fn proof_time(&self) -> i64 {
        // Stamp the poll proof with the relay's clock so a skewed local clock never pushes it
        // outside the relay's freshness window. Sync once (lazily), then reuse the cached offset.
        if self.clock_offset.load(std::sync::atomic::Ordering::Relaxed) == i64::MIN {
            self.sync_clock_from_relay();
        }
        let off = self.clock_offset.load(std::sync::atomic::Ordering::Relaxed);
        local_unix_seconds() + if off == i64::MIN { 0 } else { off }
    }
}

/// Extract the relay's machine `reason` string from an error response body, or a generic fallback.
fn reason_of(resp: ureq::Response) -> String {
    resp.into_string()
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("reason").and_then(|r| r.as_str()).map(String::from))
        .unwrap_or_else(|| "rejected".to_string())
}

/// Local wall clock in Unix seconds (0 if the system clock predates the epoch).
fn local_unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Lowercase-hex encode.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Hex-decode (lower- or upper-case); fails closed on odd length or a non-hex
/// byte.
///
/// Indexes `s.as_bytes()` rather than slicing the `&str` by index, so a
/// malicious relay response carrying a multi-byte UTF-8 character fails closed
/// with `Malformed` instead of panicking on a char boundary. (The relay's own
/// `mercury_relay::hex::decode` is byte-indexed for the same reason; this is the
/// client-side mirror.)
fn hex_decode(s: &str) -> Result<Vec<u8>, TransportError> {
    let bytes = s.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err(TransportError::Malformed);
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

/// One hex nibble from an ASCII byte; `Malformed` on any non-hex byte (which
/// includes every byte of a multi-byte UTF-8 character).
fn hex_nibble(c: u8) -> Result<u8, TransportError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(TransportError::Malformed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_decode_round_trips() {
        let bytes = [0x00u8, 0x0f, 0xa5, 0xff, 0x10];
        assert_eq!(hex_decode(&hex_encode(&bytes)).unwrap(), bytes);
    }

    #[test]
    fn hex_decode_accepts_upper_and_lower_case() {
        assert_eq!(hex_decode("ffAB").unwrap(), vec![0xff, 0xab]);
    }

    #[test]
    fn hex_decode_rejects_odd_length() {
        assert!(matches!(hex_decode("abc"), Err(TransportError::Malformed)));
    }

    #[test]
    fn hex_decode_rejects_non_hex() {
        assert!(matches!(hex_decode("zz"), Err(TransportError::Malformed)));
    }

    // Regression (cert round-2 finding): a malicious relay response whose hex
    // field carries a multi-byte UTF-8 character — even total byte length, so it
    // survives the length check and crosses a 2-byte slice boundary — must fail
    // closed with `Malformed`, NEVER panic on a char boundary.
    #[test]
    fn hex_decode_rejects_multibyte_utf8_without_panicking() {
        // "a€" = 0x61 + 3 bytes (U+20AC) = 4 bytes total (even length).
        assert!(matches!(hex_decode("a€"), Err(TransportError::Malformed)));
        // "€€" = 6 bytes (even length), every byte non-hex.
        assert!(matches!(hex_decode("€€"), Err(TransportError::Malformed)));
    }

    // Regression: the exact malicious-relay path — a poll/fetch JSON response
    // with a non-ASCII ciphertext string must decode to `Malformed`, not panic.
    #[test]
    fn malicious_relay_non_ascii_ciphertext_is_malformed_not_panic() {
        let text = r#"{"ciphertext":"a€"}"#;
        let value: serde_json::Value = serde_json::from_str(text).unwrap();
        let ciphertext = value.get("ciphertext").and_then(|c| c.as_str()).unwrap();
        assert!(matches!(
            hex_decode(ciphertext),
            Err(TransportError::Malformed)
        ));
    }
}
