//! Axum HTTP surface for the relay.
//!
//! Every endpoint delegates admission to the authoritative `mercury-core`
//! gates and fails closed. The relay only ever moves opaque bytes + content-
//! free metadata; no handler interprets, logs, or returns plaintext, and no
//! handler accepts `plaintext_identity_fields != 0` (the gates reject it and
//! the relay never sets it to anything else).

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{ConnectInfo, DefaultBodyLimit, Path, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use mercury_core::{
    AuthenticatedRelaySourceInput, AuthenticatedRelayTransportState, DeliveryAckInput,
    OutboundSendDecision, OutboundSendReason, RelayQueueInput, RelayQueueItemState,
    RelayQueueOperation, RelaySubmissionInput, evaluate_authenticated_relay_source,
    evaluate_delivery_ack, evaluate_relay_queue, evaluate_relay_submission,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::directory::InMemoryDirectoryStore;
use crate::hex;
use crate::pairing::{self, PairingStore};
use crate::push::PushSender;
use crate::rate::RateLimiter;
use crate::store::{QueueRecord, QueueStore, now_unix_seconds, queue_admits_new_route};

/// Default flood cap for the open endpoints: requests per window per client key.
const DEFAULT_RATE_LIMIT_MAX: u32 = 120;
/// Default rate-limit window, in seconds.
const DEFAULT_RATE_LIMIT_WINDOW_S: i64 = 60;

/// Shared relay state injected into every handler.
#[derive(Clone)]
pub struct RelayState {
    pub store: Arc<Mutex<dyn QueueStore>>,
    pub push: Arc<dyn PushSender>,
    /// Public prekey directory: account id -> opaque, client-verified contact
    /// card. The relay stores and serves these bytes without interpreting them.
    pub directory: Arc<Mutex<InMemoryDirectoryStore>>,
    /// Short-lived, single-use pairing-code broker: a friendlier way to exchange a contact card
    /// than a 64-hex account id. The relay stores opaque cards under random codes and never
    /// interprets them (see [`crate::pairing`]).
    pub pairing: Arc<Mutex<PairingStore>>,
    /// Flood limiter for the OPEN endpoints (`/relay/submit`, `/directory/publish`,
    /// `/pair/publish`, `/pair/fetch`), keyed by peer IP. Bounds a single-source flood; see
    /// [`crate::rate`] for the honest scope.
    pub rate_limiter: Arc<RateLimiter>,
    /// Maximum tolerated delivery -> ack delay, in seconds. Passed verbatim to
    /// `evaluate_delivery_ack`.
    pub max_ack_delay_s: i64,
}

impl RelayState {
    pub fn new(store: Arc<Mutex<dyn QueueStore>>, push: Arc<dyn PushSender>) -> Self {
        Self {
            store,
            push,
            directory: Arc::new(Mutex::new(InMemoryDirectoryStore::new())),
            pairing: Arc::new(Mutex::new(PairingStore::new())),
            rate_limiter: Arc::new(RateLimiter::new(
                DEFAULT_RATE_LIMIT_MAX,
                DEFAULT_RATE_LIMIT_WINDOW_S,
            )),
            max_ack_delay_s: 300,
        }
    }

    /// Override the open-endpoint flood limiter (e.g. a low cap for tests).
    pub fn with_rate_limit(mut self, max_per_window: u32, window_s: i64) -> Self {
        self.rate_limiter = Arc::new(RateLimiter::new(max_per_window, window_s));
        self
    }
}

/// Build the relay router.
///
/// Public endpoints:
/// - `POST /relay/submit` — admit + enqueue opaque ciphertext.
/// - `POST /directory/publish` — store an opaque contact card, authorized by a
///   proof of possession so a card can only be written to its own derived slot.
/// - `GET /directory/fetch/:account_id` — serve an opaque contact card (the
///   client re-verifies it; no auth needed since cards are public key material).
///
/// Transport-authenticated endpoints (guarded by [`auth_layer`]):
/// - `GET /relay/poll/:route_id`
/// - `POST /relay/ack/:route_id`
/// - `DELETE /relay/queue/:route_id`
///
/// Each body-bearing endpoint carries an explicit request-body limit (overriding axum's 2 MiB
/// default) so a single request cannot make the relay buffer an unbounded amount before its
/// length gates run. Each limit is sized to its endpoint's legitimate maximum:
/// - `/directory/publish` is a TIGHTENING (cards are sub-kilobyte), closing the buffering DoS
///   where publish hex-decodes the whole body before the per-card guard.
/// - `/relay/submit` is a deliberate, bounded RAISE from the 2 MiB default — a 1 MiB ciphertext
///   is already 2 MiB of hex, so the default was too tight for the client's declared maximum;
///   4 MiB fits it with headroom while still capping per-request buffering.
///
/// Neither limit is unbounded. A FLOOD of many small requests on the two OPEN endpoints is
/// bounded by a per-IP flood limiter (see `rate_limit_layer` / [`crate::rate`]); a DISTRIBUTED
/// flood across many source IPs still needs upstream (CDN/proxy) protection.
pub fn router(state: RelayState) -> Router {
    let authenticated = Router::new()
        .route("/relay/poll/{route_id}", get(poll))
        .route("/relay/wait/{route_id}", get(wait))
        .route("/relay/ack/{route_id}", post(ack))
        .route("/relay/queue/{route_id}", delete(delete_queue))
        .layer(middleware::from_fn(auth_layer));

    // Flood limiter on the OPEN endpoints, applied OUTSIDE the body limit so a flood is rejected
    // (429) before the body is buffered. Keyed by peer IP (see `rate_limit_layer`).
    let rate_limited = middleware::from_fn_with_state(state.clone(), rate_limit_layer);

    Router::new()
        .route(
            "/relay/submit",
            post(submit)
                .layer(DefaultBodyLimit::max(SUBMIT_BODY_LIMIT))
                .layer(rate_limited.clone()),
        )
        .route(
            "/directory/publish",
            post(directory_publish)
                .layer(DefaultBodyLimit::max(PUBLISH_BODY_LIMIT))
                .layer(rate_limited.clone()),
        )
        .route("/directory/fetch/{account_id}", get(directory_fetch))
        .route(
            "/pair/publish",
            post(pairing_publish)
                .layer(DefaultBodyLimit::max(PUBLISH_BODY_LIMIT))
                .layer(rate_limited.clone()),
        )
        // The fetch endpoint is flood-limited too: a code is single-use over a ~50-bit space, so
        // blind guessing is already impractical, and the cap removes even that thin margin.
        .route("/pair/fetch/{code}", get(pairing_fetch).layer(rate_limited))
        .merge(authenticated)
        .with_state(state)
}

/// Flood limiter middleware for the open endpoints. Keys by peer IP (from `ConnectInfo`, present
/// when the server runs with `into_make_service_with_connect_info`); falls back to a single
/// shared key when connection info is absent (e.g. in-process tests). Returns 429 over the cap.
async fn rate_limit_layer(
    State(state): State<RelayState>,
    request: Request,
    next: Next,
) -> Response {
    let key = client_key(&request);
    if !state.rate_limiter.check(&key) {
        return status_only(StatusCode::TOO_MANY_REQUESTS);
    }
    next.run(request).await
}

/// Whether to trust a forwarded-client header for rate-limit keying. Opt-in via the
/// `MERCURY_TRUSTED_PROXY` env var — set it when (and only when) the relay sits behind a single
/// trusted reverse proxy that appends the real client to `X-Forwarded-For` (the documented Caddy
/// deployment). Cached once: the deployment topology does not change at runtime.
fn trust_forwarded_for() -> bool {
    static TRUST: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *TRUST.get_or_init(|| {
        std::env::var("MERCURY_TRUSTED_PROXY")
            .map(|v| {
                let v = v.trim();
                !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
            })
            .unwrap_or(false)
    })
}

/// The coarse client key for rate limiting.
///
/// Default: the direct peer IP (un-spoofable, but behind a reverse proxy it is the PROXY's IP, so
/// every client collapses into ONE shared bucket — useless as flood control in that topology, and
/// it can starve legitimate users once a handful are active).
///
/// When `MERCURY_TRUSTED_PROXY` is set, key on the RIGHT-MOST entry of `X-Forwarded-For` instead.
/// With exactly one trusted proxy in front, that hop is the address the proxy itself observed for
/// the client and appended — a client cannot forge it (any client-supplied XFF entries sit to its
/// LEFT). We never trust the left-most/aggregate header value, so this is not a spoofing vector;
/// it simply restores per-client keying in the documented behind-Caddy deployment. Falls back to
/// the peer IP when the header is absent or empty.
pub(crate) fn client_key(request: &Request) -> String {
    let forwarded = trust_forwarded_for()
        .then(|| {
            // Use the LAST `X-Forwarded-For` header line (multiple lines are semantically one
            // comma-joined list, in order), then its RIGHT-MOST entry — the hop the trusted proxy
            // appended for the real client. Un-forgeable behind a single appending proxy; any
            // client-supplied entries sit to the left.
            request
                .headers()
                .get_all("x-forwarded-for")
                .iter()
                .next_back()
                .and_then(|v| v.to_str().ok())
                .and_then(|line| line.rsplit(',').map(str::trim).find(|s| !s.is_empty()))
                .map(str::to_string)
        })
        .flatten();
    if let Some(client) = forwarded {
        return client;
    }
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "no-conn".to_string())
}

/// Max `POST /relay/submit` request body, in bytes. The largest legitimate submit is the
/// client's declared ciphertext maximum (~1 MiB) hex-encoded (~2 MiB) plus the sealed header
/// and JSON framing; 4 MiB leaves headroom while still bounding per-request buffering. (Larger
/// "media object" ciphertexts the core gate permits up to its 16 MiB ceiling are out of scope
/// for the plain submit path and would need a chunked/attachment transport.)
const SUBMIT_BODY_LIMIT: usize = 4 * 1024 * 1024;

/// Max `POST /directory/publish` request body, in bytes. A contact card is well under a
/// kilobyte (`directory::MAX_CARD_LEN` is 8 KiB) plus its identity key + proof; 64 KiB is
/// generous headroom yet tight enough that the hex decode of an oversized body is bounded.
const PUBLISH_BODY_LIMIT: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// Submit
// ---------------------------------------------------------------------------

/// The client-forwarded `OutboundSendDecision` (the upstream send-gate result).
/// The relay treats it as an opaque admission witness — it is fed straight into
/// `evaluate_relay_submission`, which re-derives the send-gate reason. The
/// relay never trusts these to *bypass* a gate; a rejected send decision yields
/// a rejected submission.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct OutboundSendDto {
    pub accepted: bool,
    pub can_send: bool,
    pub can_persist_ciphertext: bool,
    pub requires_user_action: bool,
    /// `OutboundSendReason` code (see `mercury_core::OutboundSendReason::code`).
    pub reason_code: i32,
}

impl OutboundSendDto {
    fn to_decision(self) -> OutboundSendDecision {
        OutboundSendDecision {
            accepted: self.accepted,
            can_send: self.can_send,
            can_persist_ciphertext: self.can_persist_ciphertext,
            requires_user_action: self.requires_user_action,
            reason: outbound_reason_from_code(self.reason_code),
        }
    }
}

fn outbound_reason_from_code(code: i32) -> OutboundSendReason {
    match code {
        0 => OutboundSendReason::Accepted,
        1 => OutboundSendReason::DeviceTrustRejected,
        2 => OutboundSendReason::RoomMembershipTransitionRejected,
        3 => OutboundSendReason::RoomMembershipEpochNotRotated,
        4 => OutboundSendReason::MessagePolicyRejected,
        // Any unknown / out-of-range code maps to a rejecting reason so the
        // submission gate fails closed rather than silently treating it as
        // accepted.
        _ => OutboundSendReason::LocalStoreSealingRejected,
    }
}

/// `POST /relay/submit` request. Opaque byte fields are lowercase hex.
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitRequest {
    /// Opaque routing handle (hex). 16..=128 bytes per the gate.
    pub route_id: String,
    /// Opaque replay token (hex). Exactly 32 bytes per the gate.
    pub replay_token: String,
    /// Opaque ciphertext (hex). 1..=4 MiB per the gate.
    pub ciphertext: String,
    /// Opaque sealed header (hex). 16..=4096 bytes per the gate.
    pub sealed_header: String,
    pub queue_ttl_s: i32,
    pub max_queue_ttl_s: i32,
    pub max_ciphertext_len: i32,
    pub padding_bucket: i32,
    /// Client-forwarded upstream send-gate decision.
    pub outbound_send: OutboundSendDto,
}

/// Decision returned to the client. Content-free.
#[derive(Debug, Clone, Serialize)]
pub struct DecisionResponse {
    pub accepted: bool,
    /// Stable machine reason (see `reason` helpers below).
    pub reason: &'static str,
}

/// Successful delivery body: opaque bytes only.
#[derive(Debug, Clone, Serialize)]
pub struct DeliveryResponse {
    pub ciphertext: String,
    pub sealed_header: String,
}

async fn submit(State(state): State<RelayState>, Json(req): Json<SubmitRequest>) -> Response {
    // Decode opaque handles. A decode failure is a malformed request (400);
    // the relay never echoes the offending bytes.
    let (Ok(route_id), Ok(replay_token), Ok(ciphertext), Ok(sealed_header)) = (
        hex::decode(&req.route_id),
        hex::decode(&req.replay_token),
        hex::decode(&req.ciphertext),
        hex::decode(&req.sealed_header),
    ) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(DecisionResponse {
                accepted: false,
                reason: "malformed_request",
            }),
        )
            .into_response();
    };

    let now_s = now_unix_seconds();
    let created_at_s = now_s;
    let expires_at_s = created_at_s.saturating_add(req.queue_ttl_s as i64);

    // 1) Authoritative submission gate, built from byte LENGTHS only.
    //    `plaintext_identity_fields` is hard-wired to 0: the relay never
    //    carries plaintext identity, and the gate forbids non-zero anyway.
    let submission = evaluate_relay_submission(RelaySubmissionInput {
        version: 1,
        outbound_send: req.outbound_send.to_decision(),
        route_id_len: len_to_i32(route_id.len()),
        replay_token_len: len_to_i32(replay_token.len()),
        queue_ttl_s: req.queue_ttl_s,
        max_queue_ttl_s: req.max_queue_ttl_s,
        ciphertext_len: len_to_i32(ciphertext.len()),
        max_ciphertext_len: req.max_ciphertext_len,
        sealed_header_len: len_to_i32(sealed_header.len()),
        plaintext_identity_fields: 0,
        padding_bucket: req.padding_bucket,
    });

    // 2)+3) Authoritative queue gate (Enqueue) AND the persist, under ONE store lock.
    //
    // The read (state + replay-seen), the gate decision, and the insert MUST be a single
    // critical section. If the lock were dropped between them, two concurrent submits to the
    // SAME route could both observe `Absent`, both pass the Enqueue gate, and both reach the
    // insert — the second `insert` (insert-or-overwrite) clobbering the first sender's
    // still-pending ciphertext after that sender already received `202 ACCEPTED`.
    // `/relay/submit` is unauthenticated and the route is the recipient's public id, so that
    // race is a remotely-triggerable, targeted silent message loss. Holding the guard across
    // the whole sequence serializes racing submits: the loser now observes `Pending` and is
    // rejected as `ItemAlreadyPending`. (Mirrors the already-correct single-guard shape in
    // `poll`. `evaluate_relay_queue` is a pure, synchronous function — no `.await` is held
    // across the lock.)
    {
        let mut store = state.store.lock().await;

        let state_now = store
            .get(&route_id)
            .map(|r| r.state)
            .unwrap_or(RelayQueueItemState::Absent);
        let replay_seen = store.contains_replay_token(&replay_token);

        let queue = evaluate_relay_queue(RelayQueueInput {
            operation: RelayQueueOperation::Enqueue,
            submission,
            state: state_now,
            replay_token_seen: replay_seen,
            created_at_s,
            expires_at_s,
            now_s,
        });

        if !(queue.accepted && queue.persist_item) {
            return (
                submit_status(submission.accepted, queue.accepted),
                Json(DecisionResponse {
                    accepted: false,
                    reason: submit_reason(submission.reason_code, &queue),
                }),
            )
                .into_response();
        }

        // Global entry cap (mirrors the directory/pairing/username stores): refuse a brand-new route
        // once the store holds MAX_QUEUE_ENTRIES records, so an unauthenticated remote flood of
        // distinct attacker-chosen route_ids cannot grow the queue (and the durable redb file) without
        // bound. An EXISTING route (state_now != Absent) is always updatable, so a legitimate recipient
        // with a pending item is never blocked — only new route_ids are refused; the background sweeper
        // frees slots as far-past-horizon entries are reaped. Checked under the same store lock.
        if !queue_admits_new_route(state_now == RelayQueueItemState::Absent, store.len()) {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(DecisionResponse {
                    accepted: false,
                    reason: "queue_full",
                }),
            )
                .into_response();
        }

        store.insert_replay_token(&replay_token);
        store.insert(QueueRecord {
            route_id: route_id.clone(),
            replay_token,
            state: queue.next_state,
            created_at_s,
            expires_at_s,
            padding_bucket: req.padding_bucket,
            submission,
            ack_seen: false,
            ciphertext,
            sealed_header,
        });
    }

    // 4) Content-free wake-up (opaque route handle only).
    state.push.notify(&route_id);

    (
        StatusCode::ACCEPTED,
        Json(DecisionResponse {
            accepted: true,
            reason: "accepted",
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Directory (prekey-bundle publish / fetch)
// ---------------------------------------------------------------------------

/// `POST /directory/publish` request. All fields are lowercase hex. The card is opaque to
/// the relay (it is never parsed); the slot is DERIVED from `identity_pub`, and `pop_sig`
/// proves the publisher holds that identity key (see [`crate::directory`]).
#[derive(Debug, Clone, Deserialize)]
pub struct PublishCardRequest {
    /// The publisher's Ed25519 identity public key (hex, 32 bytes). The directory slot is
    /// `account_id_from_identity_key(identity_pub)`, so a publisher can only write its own.
    pub identity_pub: String,
    /// The opaque contact card (hex). The relay never interprets it.
    pub card: String,
    /// Proof of possession (hex, 64 bytes): an Ed25519 signature by `identity_pub` over the
    /// card (`mercury_keys::sign_directory_publish`). Without a valid proof, publish is 403.
    pub pop_sig: String,
}

/// `GET /directory/fetch/:account_id` success body: the opaque card (hex).
#[derive(Debug, Clone, Serialize)]
pub struct CardResponse {
    pub card: String,
}

/// Publish a contact card. The relay authorizes the write via a proof of possession: only
/// the holder of the identity key may publish, and only to the slot derived from that key
/// (closing directory slot-squatting). The card itself stays opaque — the relay verifies the
/// proof over the card bytes, derives the slot, and stores. Fails closed: 400 on malformed
/// hex / wrong-width key or signature, 403 on a missing/invalid proof, 413 on an oversized
/// card.
async fn directory_publish(
    State(state): State<RelayState>,
    Json(req): Json<PublishCardRequest>,
) -> Response {
    // Decode + width-check identity_pub (32) and pop_sig (64); the card is variable-length.
    let parsed = (|| {
        let identity_pub: [u8; 32] = hex::decode(&req.identity_pub)
            .ok()?
            .as_slice()
            .try_into()
            .ok()?;
        let card = hex::decode(&req.card).ok()?;
        let pop_sig: [u8; 64] = hex::decode(&req.pop_sig).ok()?.as_slice().try_into().ok()?;
        Some((identity_pub, card, pop_sig))
    })();
    let Some((identity_pub, card, pop_sig)) = parsed else {
        return (
            StatusCode::BAD_REQUEST,
            Json(DecisionResponse {
                accepted: false,
                reason: "malformed_request",
            }),
        )
            .into_response();
    };

    // Authorize: the proof must be a valid Ed25519 signature by `identity_pub` over the card.
    // The slot is DERIVED from the verified key, so a forged or absent proof cannot write to
    // anyone's slot. A `None` here is a refused (unauthorized) publish.
    let Some(account_id) = mercury_keys::verify_directory_publish(&identity_pub, &card, &pop_sig)
    else {
        return (
            StatusCode::FORBIDDEN,
            Json(DecisionResponse {
                accepted: false,
                reason: "unauthorized",
            }),
        )
            .into_response();
    };

    let stored = {
        let mut dir = state.directory.lock().await;
        dir.publish(&account_id, &card)
    };

    if stored {
        (
            StatusCode::ACCEPTED,
            Json(DecisionResponse {
                accepted: true,
                reason: "accepted",
            }),
        )
            .into_response()
    } else {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(DecisionResponse {
                accepted: false,
                reason: "card_rejected",
            }),
        )
            .into_response()
    }
}

/// Fetch a contact card by account id. Returns the opaque card (200) or 404 if
/// no card has been published for that account id. Malformed hex is a 400.
async fn directory_fetch(
    State(state): State<RelayState>,
    Path(account_id_hex): Path<String>,
) -> Response {
    let Ok(account_id) = hex::decode(&account_id_hex) else {
        return status_only(StatusCode::BAD_REQUEST);
    };

    let card = {
        let dir = state.directory.lock().await;
        dir.fetch(&account_id)
    };

    match card {
        Some(bytes) => (
            StatusCode::OK,
            Json(CardResponse {
                card: hex::encode(&bytes),
            }),
        )
            .into_response(),
        None => status_only(StatusCode::NOT_FOUND),
    }
}

// ---------------------------------------------------------------------------
// Pairing-code broker (short-lived, single-use card exchange)
// ---------------------------------------------------------------------------

/// `POST /pair/publish` request. All fields are lowercase hex. The card is opaque to the relay; the
/// pairing proof of possession (`pop_sig` by `identity_pub` over the card) proves the publisher
/// holds an identity key and signed exactly these bytes (see [`crate::pairing`]).
#[derive(Debug, Clone, Deserialize)]
pub struct PublishPairingRequest {
    /// The publisher's Ed25519 identity public key (hex, 32 bytes).
    pub identity_pub: String,
    /// The opaque contact card (hex). The relay never interprets it.
    pub card: String,
    /// Pairing proof of possession (hex, 64 bytes): `mercury_keys::sign_pairing_publish`.
    pub pop_sig: String,
}

/// `POST /pair/publish` success body: the freshly minted, human-typable pairing code.
#[derive(Debug, Clone, Serialize)]
pub struct PairingCodeResponse {
    pub code: String,
}

/// How many fresh codes to try before giving up if every candidate collides with a live code.
/// Collisions over a ~50-bit space are astronomically unlikely; this bound just guarantees the
/// handler terminates rather than looping forever in a pathological state.
const PAIRING_CODE_MINT_ATTEMPTS: usize = 8;

/// Broker a contact card under a fresh short-lived pairing code. Authorizes the write via a pairing
/// proof of possession (so the broker can't be stuffed with un-signed bytes), mints a random code,
/// and stores the opaque card under it with a few-minutes TTL. Fails closed: 400 on malformed hex /
/// wrong-width key or signature, 403 on a missing/invalid proof, 413 on an oversized card, 503 if
/// the broker is full.
async fn pairing_publish(
    State(state): State<RelayState>,
    Json(req): Json<PublishPairingRequest>,
) -> Response {
    let parsed = (|| {
        let identity_pub: [u8; 32] = hex::decode(&req.identity_pub)
            .ok()?
            .as_slice()
            .try_into()
            .ok()?;
        let card = hex::decode(&req.card).ok()?;
        let pop_sig: [u8; 64] = hex::decode(&req.pop_sig).ok()?.as_slice().try_into().ok()?;
        Some((identity_pub, card, pop_sig))
    })();
    let Some((identity_pub, card, pop_sig)) = parsed else {
        return (
            StatusCode::BAD_REQUEST,
            Json(DecisionResponse {
                accepted: false,
                reason: "malformed_request",
            }),
        )
            .into_response();
    };

    // Authorize: a valid pairing proof by `identity_pub` over the card. A `None` is a refused
    // (unauthorized) broker. The returned account id is unused (storage is by random code, not slot).
    if mercury_keys::verify_pairing_publish(&identity_pub, &card, &pop_sig).is_none() {
        return (
            StatusCode::FORBIDDEN,
            Json(DecisionResponse {
                accepted: false,
                reason: "unauthorized",
            }),
        )
            .into_response();
    }

    let now_s = now_unix_seconds();
    let mut store = state.pairing.lock().await;
    // Mint a fresh, non-colliding code (canonicalized so lookup matches regardless of display form).
    let mut minted = None;
    for _ in 0..PAIRING_CODE_MINT_ATTEMPTS {
        let display = pairing::generate_pairing_code();
        let code = pairing::canonical_code(&display);
        if !store.contains(&code) {
            minted = Some((display, code));
            break;
        }
    }
    let Some((display, code)) = minted else {
        return status_only(StatusCode::SERVICE_UNAVAILABLE);
    };

    if store.put(code, &card, now_s) {
        (
            StatusCode::CREATED,
            Json(PairingCodeResponse { code: display }),
        )
            .into_response()
    } else {
        // Full or (already length-checked) oversized -> the broker declined.
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(DecisionResponse {
                accepted: false,
                reason: "pairing_store_full",
            }),
        )
            .into_response()
    }
}

/// Fetch (and consume) a brokered card by pairing code. Returns the opaque card (200) or 404 if the
/// code is unknown, already used, or expired. The code is canonicalized (hyphen/space-insensitive,
/// case-insensitive) before lookup; the client re-verifies the card before use.
async fn pairing_fetch(State(state): State<RelayState>, Path(code): Path<String>) -> Response {
    let canonical = pairing::canonical_code(&code);
    let now_s = now_unix_seconds();
    let card = {
        let mut store = state.pairing.lock().await;
        store.take(&canonical, now_s)
    };
    match card {
        Some(bytes) => (
            StatusCode::OK,
            Json(CardResponse {
                card: hex::encode(&bytes),
            }),
        )
            .into_response(),
        None => status_only(StatusCode::NOT_FOUND),
    }
}

// ---------------------------------------------------------------------------
// Poll
// ---------------------------------------------------------------------------

async fn poll(State(state): State<RelayState>, Path(route_id_hex): Path<String>) -> Response {
    let Ok(route_id) = hex::decode(&route_id_hex) else {
        return status_only(StatusCode::BAD_REQUEST);
    };

    let now_s = now_unix_seconds();
    let mut store = state.store.lock().await;

    let Some(record) = store.get(&route_id) else {
        // Absent -> the queue gate would say ItemNotQueued; surface as 404.
        return status_only(StatusCode::NOT_FOUND);
    };

    let queue = evaluate_relay_queue(RelayQueueInput {
        operation: RelayQueueOperation::Deliver,
        submission: record.submission,
        state: record.state,
        replay_token_seen: true,
        created_at_s: record.created_at_s,
        expires_at_s: record.expires_at_s,
        now_s,
    });

    if !queue.accepted {
        // Expired/deleted vs already-delivered both map to 410 Gone except a
        // genuinely-absent item (handled above). Pending-but-expired is 410.
        return status_only(StatusCode::GONE);
    }

    // Accepted: transition to Delivered and hand back the opaque payload,
    // clearing it from storage (deliver-once).
    store.update_state(&route_id, queue.next_state);
    let (ciphertext, sealed_header) = store.take_payload(&route_id);

    (
        StatusCode::OK,
        Json(DeliveryResponse {
            ciphertext: hex::encode(&ciphertext),
            sealed_header: hex::encode(&sealed_header),
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Wait (long-poll)
// ---------------------------------------------------------------------------

/// How long `GET /relay/wait/{route}` holds the connection before returning "no delivery yet". The
/// client immediately re-issues the wait, so this only bounds a single held request (and keeps it
/// under typical proxy/idle timeouts). Real-time delivery does not depend on it: an enqueue wakes a
/// parked wait at once.
const WAIT_TIMEOUT_S: u64 = 25;

/// `GET /relay/wait/{route_id}` — long-poll companion of `poll`. Blocks until a delivery is enqueued
/// for the route (PoP-authenticated via `auth_layer`, reusing the Poll proof) or [`WAIT_TIMEOUT_S`]
/// elapses, then returns `200 OK` = a delivery is likely pending (the client should `poll` now) or
/// `204 No Content` = timed out (the client re-waits). It moves NO payload and reveals only
/// "pending vs not" to the verified route owner. When the push backend has no in-process wait
/// (e.g. `NoopPushSender`) it returns after a short delay, so the client degrades to ordinary
/// interval polling instead of hot-looping.
async fn wait(State(state): State<RelayState>, Path(route_id_hex): Path<String>) -> Response {
    let Ok(route_id) = hex::decode(&route_id_hex) else {
        return status_only(StatusCode::BAD_REQUEST);
    };

    match state.push.subscribe(&route_id) {
        Some(notify) => {
            let woken = tokio::time::timeout(
                std::time::Duration::from_secs(WAIT_TIMEOUT_S),
                notify.notified(),
            )
            .await
            .is_ok();
            status_only(if woken {
                StatusCode::OK
            } else {
                StatusCode::NO_CONTENT
            })
        }
        None => {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            status_only(StatusCode::NO_CONTENT)
        }
    }
}

// ---------------------------------------------------------------------------
// Ack
// ---------------------------------------------------------------------------

/// `POST /relay/ack/:route_id` request. Opaque audit handles only.
#[derive(Debug, Clone, Deserialize)]
pub struct AckRequest {
    /// Opaque ack token (hex). Exactly 32 bytes per the gate.
    pub ack_token: String,
    /// Opaque ciphertext digest (hex). Exactly 32 bytes per the gate.
    pub ciphertext_digest: String,
    /// Opaque delivery tag (hex). 16..=128 bytes per the gate.
    pub delivery_tag: String,
    pub delivered_at_s: i64,
    pub acknowledged_at_s: i64,
}

async fn ack(
    State(state): State<RelayState>,
    Path(route_id_hex): Path<String>,
    Json(req): Json<AckRequest>,
) -> Response {
    let (Ok(route_id), Ok(ack_token), Ok(ciphertext_digest), Ok(delivery_tag)) = (
        hex::decode(&route_id_hex),
        hex::decode(&req.ack_token),
        hex::decode(&req.ciphertext_digest),
        hex::decode(&req.delivery_tag),
    ) else {
        return status_only(StatusCode::BAD_REQUEST);
    };

    let mut store = state.store.lock().await;
    let Some(record) = store.get(&route_id) else {
        return status_only(StatusCode::NOT_FOUND);
    };

    // An already-acked record stays in `Delivered` state with `ack_seen` set,
    // so the gate's DuplicateAck path (keyed on queue_state == Delivered &&
    // ack_seen) recognises a repeat ack as idempotent rather than treating it
    // as fresh.
    let decision = evaluate_delivery_ack(DeliveryAckInput {
        queue_state: record.state,
        ack_seen: record.ack_seen,
        delivered_at_s: req.delivered_at_s,
        acknowledged_at_s: req.acknowledged_at_s,
        max_ack_delay_s: state.max_ack_delay_s,
        ack_token_len: len_to_i32(ack_token.len()),
        ciphertext_digest_len: len_to_i32(ciphertext_digest.len()),
        delivery_tag_len: len_to_i32(delivery_tag.len()),
        plaintext_identity_fields: 0,
    });

    if decision.duplicate {
        // Idempotent: already acked. 200 with a content-free marker.
        return (
            StatusCode::OK,
            Json(DecisionResponse {
                accepted: false,
                reason: "duplicate_ack",
            }),
        )
            .into_response();
    }

    if !decision.accepted {
        return (
            StatusCode::CONFLICT,
            Json(DecisionResponse {
                accepted: false,
                reason: ack_reason(&decision),
            }),
        )
            .into_response();
    }

    // Accepted: the gate says delete_queue_record + retain_hash_audit. We
    // clear the opaque payload (already empty after delivery) and mark the
    // record acked, leaving it in `Delivered` state as the hash-only audit
    // tombstone so a duplicate ack is recognised via `ack_seen`.
    if decision.delete_queue_record {
        store.clear_payload(&route_id);
        store.mark_acked(&route_id);
    }

    (
        StatusCode::OK,
        Json(DecisionResponse {
            accepted: true,
            reason: "accepted",
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

async fn delete_queue(
    State(state): State<RelayState>,
    Path(route_id_hex): Path<String>,
) -> Response {
    let Ok(route_id) = hex::decode(&route_id_hex) else {
        return status_only(StatusCode::BAD_REQUEST);
    };

    let now_s = now_unix_seconds();
    let mut store = state.store.lock().await;
    let Some(record) = store.get(&route_id) else {
        return status_only(StatusCode::NOT_FOUND);
    };

    let queue = evaluate_relay_queue(RelayQueueInput {
        operation: RelayQueueOperation::Delete,
        submission: record.submission,
        state: record.state,
        replay_token_seen: true,
        created_at_s: record.created_at_s,
        expires_at_s: record.expires_at_s,
        now_s,
    });

    if !queue.accepted {
        // Already deleted -> idempotent success; other rejects -> 409.
        return status_only(StatusCode::CONFLICT);
    }

    store.clear_payload(&route_id);
    store.update_state(&route_id, queue.next_state);

    (
        StatusCode::OK,
        Json(DecisionResponse {
            accepted: true,
            reason: "accepted",
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Transport-auth middleware
// ---------------------------------------------------------------------------

/// Header names for the route-ownership proof of possession the relay verifies on poll / ack /
/// delete (all hex except the timestamp). The caller proves it holds the identity key whose
/// account id IS the route it is draining — this REPLACES the old forgeable transport-auth
/// handles + pre-verified booleans (which any client could set).
pub const HEADER_IDENTITY_PUB: &str = "x-mercury-identity-pub";
pub const HEADER_POP_TIMESTAMP: &str = "x-mercury-pop-timestamp";
pub const HEADER_ROUTE_POP: &str = "x-mercury-route-pop";

/// How far a proof's timestamp may differ from the relay's clock, in seconds, in EITHER
/// direction (too old AND too far in the future are rejected). 5 minutes tolerates real client
/// clock skew + network latency (the client ALSO stamps proofs with relay-synced time, so this is
/// defense-in-depth); the in-window replay race stays bounded by the single-slot per-route mailbox.
const RELAY_POP_FRESHNESS_WINDOW_S: i64 = 300;

/// Transport-auth admission layer for poll / ack / delete.
///
/// Authenticates the caller with a real ROUTE-OWNERSHIP proof of possession: the request must
/// carry an Ed25519 proof (over the route id + the operation + a fresh timestamp, [`HEADER_ROUTE_POP`]) by the
/// identity key ([`HEADER_IDENTITY_PUB`]) whose account id IS the route being drained — and
/// these endpoints operate on the recipient's OWN account-id route, so only the genuine
/// recipient can drain their queue. The relay verifies the proof + a both-sided freshness
/// window, then runs the authoritative `evaluate_authenticated_relay_source` gate with
/// AUTHENTIC, proof-derived signals (NOT client-set booleans): a valid proof means the source
/// is authenticated, owns the route key, is within the replay window, and presents the 32-byte
/// credential shape the gate requires. On any failure those inputs are false/zero so the gate
/// rejects -> 401. Never exposes plaintext identity (the gate's plaintext inputs are 0).
///
/// Residual (honest): the proof binds `(route_id, operation, timestamp)`, so a captured proof can
/// no longer be replayed ACROSS operations — a poll proof cannot drive an ack/delete. What remains
/// is SAME-operation replay within the freshness window (e.g. a captured poll proof re-presented as
/// another poll, racing the genuine deliver-once drain). It is never an escalation beyond what the
/// route owner could already do to their own queue, and in production the proof travels inside TLS
/// so it is not on the wire to capture. A server-issued, single-use nonce challenge-response would
/// close the residual fully; deferred as a follow-on.
async fn auth_layer(headers: HeaderMap, request: axum::extract::Request, next: Next) -> Response {
    let authentic = route_ownership_proven(&headers, request.uri().path());

    let input = AuthenticatedRelaySourceInput {
        transport: AuthenticatedRelayTransportState::Ready,
        // On a valid proof the source IS authenticated and presents credentials of the
        // gate-required 32-byte shape; we NEVER read these from client-set headers.
        session_ticket_len: if authentic { 32 } else { 0 },
        device_credential_len: if authentic { 32 } else { 0 },
        server_auth_tag_len: if authentic { 32 } else { 0 },
        server_authenticated: authentic,
        route_key_authenticated: authentic,
        replay_window_valid: authentic,
        pending_delivery: false,
        route_id_len: 0,
        poll_batch_limit: 1,
        plaintext_notification_preview_len: 0,
        plaintext_identity_fields: 0,
    };

    let decision = evaluate_authenticated_relay_source(input);
    if !decision.accepted {
        return status_only(StatusCode::UNAUTHORIZED);
    }

    next.run(request).await
}

/// Verify the route-ownership proof for the route addressed by `path` (the last path segment of
/// `/relay/{poll,ack,queue}/{route_id}`). Returns true iff the caller proved it holds the
/// identity key whose account id equals that EXACTLY-32-byte route, with a fresh timestamp.
/// Fail-closed on any malformed / missing / stale / wrong input; never trusts a client boolean.
fn route_ownership_proven(headers: &HeaderMap, path: &str) -> bool {
    // The path is `/relay/{poll,ack,queue}/{route_id}`. Bind BOTH the route id (last segment)
    // and the operation (the segment before it) into the verified proof, so a captured proof for
    // one operation (e.g. a read-only poll) cannot be replayed as another (e.g. a destructive
    // delete) within the freshness window. Fail closed on an unrecognized operation segment.
    let mut segments = path.rsplit('/');
    let Some(route_seg) = segments.next() else {
        return false;
    };
    let operation = match segments.next() {
        // `wait` is the long-poll companion of `poll` (block until a delivery is pending, then the
        // client polls). It is the same read-side trust class, so it reuses the Poll proof — no new
        // operation in the audited `mercury_keys` PoP set.
        Some("poll") | Some("wait") => mercury_keys::RelayRouteOperation::Poll,
        Some("ack") => mercury_keys::RelayRouteOperation::Ack,
        Some("queue") => mercury_keys::RelayRouteOperation::Delete,
        _ => return false,
    };
    // route_id = the last path segment, hex, exactly 32 bytes (an account-id route).
    let Some(route_id) = hex::decode(route_seg)
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b).ok())
    else {
        return false;
    };

    // identity_pub (32) + pop_sig (64) from hex headers; timestamp from a decimal header.
    let Some(identity_pub) =
        decode_header(headers, HEADER_IDENTITY_PUB).and_then(|b| <[u8; 32]>::try_from(b).ok())
    else {
        return false;
    };
    let Some(pop_sig) =
        decode_header(headers, HEADER_ROUTE_POP).and_then(|b| <[u8; 64]>::try_from(b).ok())
    else {
        return false;
    };
    let Some(timestamp_s) = headers
        .get(HEADER_POP_TIMESTAMP)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
    else {
        return false;
    };

    // Both-sided freshness: reject a proof too far in the past OR the future. `saturating_sub`
    // avoids an i64 overflow on an extreme client-supplied timestamp.
    let delta = now_unix_seconds().saturating_sub(timestamp_s);
    if delta > RELAY_POP_FRESHNESS_WINDOW_S || delta < -RELAY_POP_FRESHNESS_WINDOW_S {
        return false;
    }

    mercury_keys::verify_relay_route_pop(&identity_pub, &route_id, operation, timestamp_s, &pop_sig)
}

fn decode_header(headers: &HeaderMap, name: &str) -> Option<Vec<u8>> {
    let value = headers.get(name)?.to_str().ok()?;
    hex::decode(value).ok()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn status_only(status: StatusCode) -> Response {
    status.into_response()
}

fn len_to_i32(len: usize) -> i32 {
    if len > i32::MAX as usize {
        i32::MAX
    } else {
        len as i32
    }
}

/// Map a submission/queue rejection to an HTTP status. Submission-contract
/// failures are 400; queue-state conflicts (duplicate replay, already pending)
/// are 409.
fn submit_status(submission_accepted: bool, queue_accepted: bool) -> StatusCode {
    if !submission_accepted {
        StatusCode::BAD_REQUEST
    } else if !queue_accepted {
        StatusCode::CONFLICT
    } else {
        // Unreachable in practice (caller only enters on a reject), but fail
        // closed.
        StatusCode::BAD_REQUEST
    }
}

/// Stable machine reason string for a submit rejection. Maps the authoritative
/// `mercury_policy` reason codes / `RelayQueueReason` — the relay never invents
/// its own admission verdicts.
fn submit_reason(reason_code: i32, queue: &mercury_core::RelayQueueDecision) -> &'static str {
    use mercury_core::RelayQueueReason as R;
    // Submission-contract failure takes precedence (it gates the queue).
    if reason_code != 0 {
        return match reason_code {
            1 => "bad_version",
            2 => "bad_send_gate_reason",
            3 => "send_gate_rejected",
            4 => "bad_route_id_len",
            5 => "bad_replay_token_len",
            6 => "bad_ttl",
            7 => "ttl_too_long",
            8 => "bad_ciphertext_len",
            9 => "ciphertext_too_large",
            10 => "bad_sealed_header_len",
            11 => "sealed_header_too_large",
            12 => "plaintext_identity_forbidden",
            13 => "bad_padding_bucket",
            _ => "submission_rejected",
        };
    }
    match queue.reason {
        R::Accepted => "accepted",
        R::RelaySubmissionRejected => "submission_rejected",
        R::ReplayTokenDuplicate => "replay_token_duplicate",
        R::BadTimeWindow => "bad_time_window",
        R::ItemNotQueued => "item_not_queued",
        R::ItemAlreadyPending => "item_already_pending",
        R::ItemExpired => "item_expired",
        R::ItemNotExpired => "item_not_expired",
        R::ItemAlreadyDelivered => "item_already_delivered",
        R::ItemAlreadyExpired => "item_already_expired",
        R::ItemAlreadyDeleted => "item_already_deleted",
    }
}

fn ack_reason(decision: &mercury_core::DeliveryAckDecision) -> &'static str {
    use mercury_core::DeliveryAckReason as R;
    match decision.reason {
        R::Accepted => "accepted",
        R::QueueItemNotDelivered => "queue_item_not_delivered",
        R::QueueItemExpired => "queue_item_expired",
        R::QueueItemDeleted => "queue_item_deleted",
        R::DuplicateAck => "duplicate_ack",
        R::BadTimeWindow => "bad_time_window",
        R::AckTooLate => "ack_too_late",
        R::BadAckTokenLength => "bad_ack_token_length",
        R::BadCiphertextDigestLength => "bad_ciphertext_digest_length",
        R::BadDeliveryTagLength => "bad_delivery_tag_length",
        R::PlaintextIdentityForbidden => "plaintext_identity_forbidden",
    }
}
