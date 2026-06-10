//! End-to-end relay tests driven through the Axum router with
//! `tower::ServiceExt::oneshot` — no real server, no bound port, no Redis.
//!
//! Every test exercises the authoritative `mercury-core` gates as the only
//! admission path. The relay only ever returns opaque bytes + a content-free
//! status/reason.

use std::sync::{Arc, LazyLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use mercury_keys::{
    IdentityKeyPair, RelayRouteOperation, account_id_from_identity_key, sign_directory_publish,
    sign_relay_route_pop,
};
use mercury_relay::http::{HEADER_IDENTITY_PUB, HEADER_POP_TIMESTAMP, HEADER_ROUTE_POP};
use mercury_relay::{InMemoryQueueStore, NoopPushSender, QueueStore, RelayState, router};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower::ServiceExt;

// --- fixtures ---------------------------------------------------------------

const REPLAY_HEX: &str = "0909090909090909090909090909090909090909090909090909090909090909"; // 32 bytes
const AUTH32_HEX: &str = "0101010101010101010101010101010101010101010101010101010101010101"; // 32 bytes
const TAG_HEX: &str = "1010101010101010101010101010101010101010"; // 20 bytes (16..=128)

/// A fixed test identity (stable for the whole test run). Its account id is the 32-byte route
/// that poll/ack/delete authenticate against — only its holder can drain that route.
static TEST_IDENTITY: LazyLock<IdentityKeyPair> = LazyLock::new(IdentityKeyPair::generate);

fn route_bytes() -> [u8; 32] {
    account_id_from_identity_key(&TEST_IDENTITY.public())
}

/// Lowercase-hex of the authenticated route (the test identity's account id).
fn route_hex() -> String {
    mercury_relay::hex::encode(&route_bytes())
}

fn now_s() -> i64 {
    mercury_relay::store::now_unix_seconds()
}

fn hexn(byte: u8, n: usize) -> String {
    let mut s = String::with_capacity(n * 2);
    for _ in 0..n {
        s.push_str(&format!("{byte:02x}"));
    }
    s
}

fn build() -> (Router, Arc<Mutex<dyn QueueStore>>) {
    let store: Arc<Mutex<dyn QueueStore>> = Arc::new(Mutex::new(InMemoryQueueStore::new()));
    let push = Arc::new(NoopPushSender);
    let state = RelayState::new(Arc::clone(&store), push);
    (router(state), store)
}

/// Like [`build`] but with the in-process waker, so a `/relay/wait` long-poll is woken by a submit.
fn build_with_waker() -> (Router, Arc<Mutex<dyn QueueStore>>) {
    let store: Arc<Mutex<dyn QueueStore>> = Arc::new(Mutex::new(InMemoryQueueStore::new()));
    let push = Arc::new(mercury_relay::InProcessWaker::new());
    let state = RelayState::new(Arc::clone(&store), push);
    (router(state), store)
}

fn submit_body() -> Value {
    json!({
        "route_id": route_hex(),
        "replay_token": REPLAY_HEX,
        "ciphertext": hexn(0x42, 128),
        "sealed_header": hexn(0x05, 96),
        "queue_ttl_s": 300,
        "max_queue_ttl_s": 86400,
        "max_ciphertext_len": 1048576,
        "padding_bucket": 3,
        "outbound_send": {
            "accepted": true,
            "can_send": true,
            "can_persist_ciphertext": true,
            "requires_user_action": false,
            "reason_code": 0
        }
    })
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap()
}

/// A poll/ack/delete request carrying a VALID route-ownership proof: signed by the fixed test
/// identity (which owns `route_hex()`), at the current time.
fn authed(method: &str, uri: &str, body: Option<&Value>) -> Request<Body> {
    authed_at(&TEST_IDENTITY, now_s(), method, uri, body)
}

/// Like [`authed`] but signs the proof with a specific `id` at timestamp `ts`, for tests that
/// prove the relay rejects a wrong identity (route not owned) or a stale/future timestamp. The
/// proof is signed over the 32-byte route in the uri's last segment (a non-32-byte route signs
/// over a zero placeholder, which the relay rejects on its length check regardless).
fn authed_at(
    id: &IdentityKeyPair,
    ts: i64,
    method: &str,
    uri: &str,
    body: Option<&Value>,
) -> Request<Body> {
    // Sign the operation the endpoint actually performs (mirrors the relay's path -> operation
    // mapping) so the proof verifies on the matching endpoint.
    authed_with_op_at(id, ts, route_operation_for_uri(uri), method, uri, body)
}

/// The relay operation a `/relay/{poll,ack,queue}/{route}` uri performs (mirror of the relay's
/// own path mapping); defaults to `Poll` for any other shape.
fn route_operation_for_uri(uri: &str) -> RelayRouteOperation {
    match uri.rsplit('/').nth(1) {
        Some("ack") => RelayRouteOperation::Ack,
        Some("queue") => RelayRouteOperation::Delete,
        _ => RelayRouteOperation::Poll,
    }
}

/// Build an authenticated request whose route-ownership proof is signed for an EXPLICIT
/// `operation` — letting a test deliberately present a proof for the wrong operation.
fn authed_with_op_at(
    id: &IdentityKeyPair,
    ts: i64,
    operation: RelayRouteOperation,
    method: &str,
    uri: &str,
    body: Option<&Value>,
) -> Request<Body> {
    let route_seg = uri.rsplit('/').next().unwrap_or("");
    let route32: [u8; 32] = mercury_relay::hex::decode(route_seg)
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b).ok())
        .unwrap_or([0u8; 32]);
    let sig = sign_relay_route_pop(id, &route32, operation, ts);
    let mut b = Request::builder()
        .method(method)
        .uri(uri)
        .header(
            HEADER_IDENTITY_PUB,
            mercury_relay::hex::encode(id.public().as_bytes()),
        )
        .header(HEADER_POP_TIMESTAMP, ts.to_string())
        .header(HEADER_ROUTE_POP, mercury_relay::hex::encode(&sig));
    match body {
        Some(_) => {
            b = b.header(CONTENT_TYPE, "application/json");
            b.body(Body::from(serde_json::to_vec(body.unwrap()).unwrap()))
                .unwrap()
        }
        None => b.body(Body::empty()).unwrap(),
    }
}

async fn json_of(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn call(router: &Router, req: Request<Body>) -> axum::response::Response {
    router.clone().oneshot(req).await.unwrap()
}

// --- submit -----------------------------------------------------------------

#[tokio::test]
async fn submit_accepted_stores_item() {
    let (app, store) = build();
    let resp = call(&app, post_json("/relay/submit", &submit_body())).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let body = json_of(resp).await;
    assert_eq!(body["accepted"], true);
    assert_eq!(body["reason"], "accepted");

    // Item is stored with its opaque payload intact.
    let route = mercury_relay::hex::decode(&route_hex()).unwrap();
    let rec = store.lock().await.get(&route).unwrap();
    assert!(rec.has_payload());
    assert_eq!(rec.ciphertext.len(), 128);
    assert_eq!(rec.sealed_header.len(), 96);
}

// --- wait (long-poll) -------------------------------------------------------

#[tokio::test]
async fn wait_without_auth_is_unauthorized() {
    let (app, _store) = build();
    let req = Request::builder()
        .method("GET")
        .uri(format!("/relay/wait/{}", route_hex()))
        .body(Body::empty())
        .unwrap();
    let resp = call(&app, req).await;
    // wait reuses the route-ownership middleware (it maps to the Poll proof); no proof -> 401.
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wait_wakes_when_a_message_is_submitted() {
    let (app, _store) = build_with_waker();
    let wait_uri = format!("/relay/wait/{}", route_hex());

    // Park an authenticated long-poll wait (signed with the Poll proof the relay maps `wait` to).
    let app_wait = app.clone();
    let wait_task = tokio::spawn(async move {
        call(&app_wait, authed("GET", &wait_uri, None))
            .await
            .status()
    });

    // Let the wait park, then submit a message for that route — the enqueue must wake the wait.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let resp = call(&app, post_json("/relay/submit", &submit_body())).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    // The parked wait returns promptly (well before its 25s timeout) with 200 OK = "poll now".
    let status = tokio::time::timeout(std::time::Duration::from_secs(5), wait_task)
        .await
        .expect("the wait must wake on submit, long before its timeout")
        .unwrap();
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn submit_rejected_bad_route_id_len() {
    let (app, store) = build();
    let mut body = submit_body();
    // 8 bytes -> below the 16..=128 gate window.
    body["route_id"] = json!(hexn(0x07, 8));
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(resp).await["reason"], "bad_route_id_len");
    // Nothing persisted: the (sub-window) route is absent.
    let route = mercury_relay::hex::decode(&hexn(0x07, 8)).unwrap();
    assert!(store.lock().await.get(&route).is_none());
}

#[tokio::test]
async fn submit_rejected_bad_replay_token_len() {
    let (app, _store) = build();
    let mut body = submit_body();
    body["replay_token"] = json!(hexn(0x09, 16)); // != 32
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(resp).await["reason"], "bad_replay_token_len");
}

#[tokio::test]
async fn submit_rejected_plaintext_identity_via_send_gate() {
    // The DTO has no plaintext-identity field by construction; the relay
    // hard-wires plaintext_identity_fields=0. The only client-controlled
    // admission witness is the outbound_send decision: a rejected send gate
    // must reject the submission (proving the gate is the admission path, not
    // a relay shortcut).
    let (app, store) = build();
    let mut body = submit_body();
    body["outbound_send"] = json!({
        "accepted": false,
        "can_send": false,
        "can_persist_ciphertext": false,
        "requires_user_action": true,
        "reason_code": 4
    });
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(resp).await["reason"], "send_gate_rejected");
    // Nothing persisted: the route is absent.
    let route = mercury_relay::hex::decode(&route_hex()).unwrap();
    assert!(store.lock().await.get(&route).is_none());
}

#[tokio::test]
async fn submit_rejected_ciphertext_too_large() {
    let (app, _store) = build();
    let mut body = submit_body();
    // Declared cap below the actual payload length.
    body["max_ciphertext_len"] = json!(64);
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(resp).await["reason"], "ciphertext_too_large");
}

#[tokio::test]
async fn submit_rejected_ttl_too_long() {
    let (app, _store) = build();
    let mut body = submit_body();
    body["max_queue_ttl_s"] = json!(700000); // > 604800
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(resp).await["reason"], "ttl_too_long");
}

#[tokio::test]
async fn submit_duplicate_replay_token_rejected() {
    let (app, _store) = build();
    // First submit accepted.
    assert_eq!(
        call(&app, post_json("/relay/submit", &submit_body()))
            .await
            .status(),
        StatusCode::ACCEPTED
    );
    // Second submit, different route id, same replay token -> rejected.
    let mut body = submit_body();
    body["route_id"] = json!(hexn(0x33, 32));
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(json_of(resp).await["reason"], "replay_token_duplicate");
}

// --- poll -------------------------------------------------------------------

#[tokio::test]
async fn poll_pending_returns_payload_then_clears() {
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;

    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_of(resp).await;
    assert_eq!(body["ciphertext"], hexn(0x42, 128));
    assert_eq!(body["sealed_header"], hexn(0x05, 96));

    // Second poll: already delivered, payload cleared -> 410 Gone.
    let resp2 = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp2.status(), StatusCode::GONE);
}

#[tokio::test]
async fn route_is_reusable_after_delivery_for_a_conversation() {
    // Regression for the conversation-killer at the HTTP layer (the exact deployed path): after a
    // message is delivered, the SAME route must accept the NEXT message and deliver its own
    // payload. Before the fix the second submit was rejected `item_already_delivered` and the
    // route stayed dead until the 7-day terminal-retention reaper — so no two-message conversation
    // was possible over the live relay.
    let (app, _store) = build();

    // msg1: submit then poll (deliver) -> route slot becomes a Delivered tombstone.
    assert_eq!(
        call(&app, post_json("/relay/submit", &submit_body()))
            .await
            .status(),
        StatusCode::ACCEPTED
    );
    let p1 = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(p1.status(), StatusCode::OK);
    assert_eq!(json_of(p1).await["ciphertext"], hexn(0x42, 128));

    // msg2: SAME route, fresh replay token + a distinct payload. MUST be accepted (slot re-armed).
    let mut body2 = submit_body();
    body2["replay_token"] = json!(hexn(0x77, 32));
    body2["ciphertext"] = json!(hexn(0x24, 128));
    assert_eq!(
        call(&app, post_json("/relay/submit", &body2))
            .await
            .status(),
        StatusCode::ACCEPTED,
        "a delivered route must accept the next message (the conversation-killer regression)"
    );

    // msg2 delivers independently, carrying ITS ciphertext (not a stale msg1 payload).
    let p2 = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(p2.status(), StatusCode::OK);
    assert_eq!(json_of(p2).await["ciphertext"], hexn(0x24, 128));
}

#[tokio::test]
async fn poll_non_32_byte_route_is_unauthorized() {
    // The authenticated endpoints require an account-id (32-byte) route; a shorter route cannot
    // be a proven account-id route, so the auth layer rejects it before any queue lookup.
    let (app, _store) = build();
    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", hexn(0xaa, 16)), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn poll_absent_owned_route_is_not_found() {
    // Polling our OWN route (valid proof) with nothing queued is a clean 404, not a 401 — auth
    // succeeds, the queue is simply empty.
    let (app, _store) = build();
    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// --- ack --------------------------------------------------------------------

fn ack_body() -> Value {
    json!({
        "ack_token": AUTH32_HEX,
        "ciphertext_digest": AUTH32_HEX,
        "delivery_tag": TAG_HEX,
        "delivered_at_s": 100,
        "acknowledged_at_s": 130
    })
}

#[tokio::test]
async fn ack_delivered_item_accepted_then_duplicate() {
    let (app, store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;
    // Move it to Delivered.
    assert_eq!(
        call(
            &app,
            authed("GET", &format!("/relay/poll/{}", route_hex()), None)
        )
        .await
        .status(),
        StatusCode::OK
    );

    let resp = call(
        &app,
        authed(
            "POST",
            &format!("/relay/ack/{}", route_hex()),
            Some(&ack_body()),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_of(resp).await["accepted"], true);

    // Record survives as a hash-only audit tombstone: still Delivered, acked,
    // payload cleared.
    let route = mercury_relay::hex::decode(&route_hex()).unwrap();
    {
        let guard = store.lock().await;
        let rec = guard.get(&route).unwrap();
        assert_eq!(rec.state, mercury_core::RelayQueueItemState::Delivered);
        assert!(rec.ack_seen);
        assert!(!rec.has_payload());
    }

    // Duplicate ack is idempotent (200, not accepted, duplicate marker).
    let dup = call(
        &app,
        authed(
            "POST",
            &format!("/relay/ack/{}", route_hex()),
            Some(&ack_body()),
        ),
    )
    .await;
    assert_eq!(dup.status(), StatusCode::OK);
    assert_eq!(json_of(dup).await["reason"], "duplicate_ack");
}

#[tokio::test]
async fn ack_on_pending_item_rejected() {
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;
    // No poll -> still Pending. Ack must be rejected by the gate.
    let resp = call(
        &app,
        authed(
            "POST",
            &format!("/relay/ack/{}", route_hex()),
            Some(&ack_body()),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(json_of(resp).await["reason"], "queue_item_not_delivered");
}

// --- delete -----------------------------------------------------------------

#[tokio::test]
async fn delete_then_idempotent() {
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;

    let resp = call(
        &app,
        authed("DELETE", &format!("/relay/queue/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_of(resp).await["accepted"], true);

    // Deleting again -> the gate rejects ItemAlreadyDeleted -> 409.
    let again = call(
        &app,
        authed("DELETE", &format!("/relay/queue/{}", route_hex()), None),
    )
    .await;
    assert_eq!(again.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_non_32_byte_route_is_unauthorized() {
    let (app, _store) = build();
    let resp = call(
        &app,
        authed("DELETE", &format!("/relay/queue/{}", hexn(0xbb, 16)), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_poll_proof_is_rejected_on_the_delete_endpoint() {
    // Operation-binding: a proof signed for Poll, presented to DELETE /relay/queue/{route} (the
    // SAME owner, route, and a fresh timestamp), must be rejected — so a captured poll proof
    // cannot be replayed to destroy the owner's queue within the freshness window.
    let (app, _store) = build();
    let wrong_op = authed_with_op_at(
        &TEST_IDENTITY,
        now_s(),
        RelayRouteOperation::Poll,
        "DELETE",
        &format!("/relay/queue/{}", route_hex()),
        None,
    );
    let resp = call(&app, wrong_op).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    // Sanity: the correctly-bound Delete proof on the same endpoint passes auth (not a 401).
    let right_op = call(
        &app,
        authed("DELETE", &format!("/relay/queue/{}", route_hex()), None),
    )
    .await;
    assert_ne!(right_op.status(), StatusCode::UNAUTHORIZED);
}

// --- transport auth ---------------------------------------------------------

#[tokio::test]
async fn poll_without_auth_headers_is_unauthorized() {
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;
    let req = Request::builder()
        .method("GET")
        .uri(format!("/relay/poll/{}", route_hex()))
        .body(Body::empty())
        .unwrap();
    let resp = call(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ack_without_auth_headers_is_unauthorized() {
    let (app, _store) = build();
    let req = post_json(&format!("/relay/ack/{}", route_hex()), &ack_body());
    let resp = call(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_without_auth_headers_is_unauthorized() {
    let (app, _store) = build();
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/relay/queue/{}", route_hex()))
        .body(Body::empty())
        .unwrap();
    let resp = call(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn poll_with_wrong_identity_is_unauthorized() {
    // A caller that does NOT own the route cannot drain it: it signs a structurally-valid proof
    // with its own key, but its account id is not the route, so verification fails -> 401.
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;
    let other = IdentityKeyPair::generate();
    let req = authed_at(
        &other,
        now_s(),
        "GET",
        &format!("/relay/poll/{}", route_hex()),
        None,
    );
    assert_eq!(call(&app, req).await.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn poll_with_stale_timestamp_is_unauthorized() {
    // A proof whose timestamp is far in the PAST is outside the freshness window -> 401.
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;
    let req = authed_at(
        &TEST_IDENTITY,
        now_s() - 600,
        "GET",
        &format!("/relay/poll/{}", route_hex()),
        None,
    );
    assert_eq!(call(&app, req).await.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn poll_with_future_timestamp_is_unauthorized() {
    // A proof whose timestamp is far in the FUTURE is also rejected (the freshness window is
    // both-sided) -> 401.
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;
    let req = authed_at(
        &TEST_IDENTITY,
        now_s() + 600,
        "GET",
        &format!("/relay/poll/{}", route_hex()),
        None,
    );
    assert_eq!(call(&app, req).await.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_rejected_on_malformed_proof() {
    // A route-ownership proof with a wrong-length signature (not 64 bytes) is malformed and
    // rejected -> 401, with no panic on the bad input.
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;
    let req = Request::builder()
        .method("GET")
        .uri(format!("/relay/poll/{}", route_hex()))
        .header(HEADER_IDENTITY_PUB, hexn(0x01, 32))
        .header(HEADER_POP_TIMESTAMP, now_s().to_string())
        .header(HEADER_ROUTE_POP, hexn(0x00, 16)) // not 64 bytes
        .body(Body::empty())
        .unwrap();
    assert_eq!(call(&app, req).await.status(), StatusCode::UNAUTHORIZED);
}

// --- content / identity guarantee ------------------------------------------

#[tokio::test]
async fn relay_only_returns_opaque_bytes_and_status() {
    // The submit DTO structurally has no plaintext-identity field, so no
    // request path can carry plaintext_identity_fields != 0; the relay
    // hard-wires it to 0 before every gate call. Here we additionally confirm
    // the delivery response contains ONLY opaque hex byte fields + nothing
    // resembling identity/plaintext.
    let (app, _store) = build();
    call(&app, post_json("/relay/submit", &submit_body())).await;
    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    let body = json_of(resp).await;
    let obj = body.as_object().unwrap();
    // Exactly two keys, both opaque byte blobs.
    assert_eq!(obj.len(), 2);
    assert!(obj.contains_key("ciphertext"));
    assert!(obj.contains_key("sealed_header"));
    for forbidden in [
        "identity",
        "sender",
        "plaintext",
        "preview",
        "user",
        "from",
        "to",
    ] {
        assert!(
            !obj.contains_key(forbidden),
            "delivery response must not carry {forbidden}"
        );
    }
}

// --- directory (prekey publish / fetch) -------------------------------------

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

/// A valid authenticated-publish body: identity_pub + card + a proof of possession the relay
/// will accept, storing the card under the slot derived from `identity`.
fn publish_body(identity: &IdentityKeyPair, card: &[u8]) -> Value {
    let pop = sign_directory_publish(identity, card);
    json!({
        "identity_pub": mercury_relay::hex::encode(identity.public().as_bytes()),
        "card": mercury_relay::hex::encode(card),
        "pop_sig": mercury_relay::hex::encode(&pop),
    })
}

/// The hex directory slot an identity's card lands in.
fn slot_of(identity: &IdentityKeyPair) -> String {
    mercury_relay::hex::encode(&account_id_from_identity_key(&identity.public()))
}

#[tokio::test]
async fn directory_publish_then_fetch_round_trips() {
    let (app, _store) = build();
    let identity = IdentityKeyPair::generate();
    let card = vec![0xabu8; 256];
    let body = publish_body(&identity, &card);

    let resp = call(&app, post_json("/directory/publish", &body)).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    assert_eq!(json_of(resp).await["accepted"], true);

    // The card lands in the slot DERIVED from the identity key (not a client-chosen one).
    let resp = call(
        &app,
        get(&format!("/directory/fetch/{}", slot_of(&identity))),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_of(resp).await;
    // The relay returns exactly the opaque card it was given, nothing else.
    assert_eq!(body["card"], hexn(0xab, 256));
    assert_eq!(body.as_object().unwrap().len(), 1);
}

#[tokio::test]
async fn directory_publish_replaces_an_existing_slot() {
    let (app, _store) = build();
    let identity = IdentityKeyPair::generate();
    let first = publish_body(&identity, &[0x11u8; 64]);
    let second = publish_body(&identity, &[0x22u8; 64]);
    assert_eq!(
        call(&app, post_json("/directory/publish", &first))
            .await
            .status(),
        StatusCode::ACCEPTED
    );
    assert_eq!(
        call(&app, post_json("/directory/publish", &second))
            .await
            .status(),
        StatusCode::ACCEPTED
    );
    let resp = call(
        &app,
        get(&format!("/directory/fetch/{}", slot_of(&identity))),
    )
    .await;
    assert_eq!(json_of(resp).await["card"], hexn(0x22, 64));
}

#[tokio::test]
async fn directory_fetch_missing_is_404() {
    let (app, _store) = build();
    let resp = call(&app, get(&format!("/directory/fetch/{}", hexn(0x44, 32)))).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn directory_publish_rejects_an_invalid_proof() {
    // A proof over a DIFFERENT card than the one submitted must be refused (403), and nothing
    // is stored.
    let (app, _store) = build();
    let identity = IdentityKeyPair::generate();
    let signed_card = vec![0xabu8; 64];
    let pop = sign_directory_publish(&identity, &signed_card);
    let body = json!({
        "identity_pub": mercury_relay::hex::encode(identity.public().as_bytes()),
        "card": mercury_relay::hex::encode(&[0xcdu8; 64]), // not what was signed
        "pop_sig": mercury_relay::hex::encode(&pop),
    });
    let resp = call(&app, post_json("/directory/publish", &body)).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    assert_eq!(json_of(resp).await["reason"], "unauthorized");
    let resp = call(
        &app,
        get(&format!("/directory/fetch/{}", slot_of(&identity))),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn directory_publish_cannot_squat_another_identitys_slot() {
    // The key property: an attacker cannot write to a victim's slot. It signs a card with its
    // OWN key but claims the victim's identity_pub; verification under the victim key fails, so
    // the publish is refused and the victim's slot stays empty.
    let (app, _store) = build();
    let victim = IdentityKeyPair::generate();
    let attacker = IdentityKeyPair::generate();
    let card = vec![0x66u8; 64];
    let pop = sign_directory_publish(&attacker, &card);
    let body = json!({
        "identity_pub": mercury_relay::hex::encode(victim.public().as_bytes()),
        "card": mercury_relay::hex::encode(&card),
        "pop_sig": mercury_relay::hex::encode(&pop),
    });
    let resp = call(&app, post_json("/directory/publish", &body)).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let resp = call(&app, get(&format!("/directory/fetch/{}", slot_of(&victim)))).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn directory_publish_oversized_card_is_rejected() {
    // MAX_CARD_LEN is 8 KiB; a (validly-proven) card one byte over is rejected, nothing stored.
    let (app, _store) = build();
    let identity = IdentityKeyPair::generate();
    let body = publish_body(&identity, &[0xabu8; 8 * 1024 + 1]);
    let resp = call(&app, post_json("/directory/publish", &body)).await;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(json_of(resp).await["reason"], "card_rejected");
    let resp = call(
        &app,
        get(&format!("/directory/fetch/{}", slot_of(&identity))),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn directory_publish_malformed_hex_is_400() {
    let (app, _store) = build();
    let body = json!({
        "identity_pub": "zznothex",
        "card": hexn(0xab, 64),
        "pop_sig": hexn(0x00, 64),
    });
    let resp = call(&app, post_json("/directory/publish", &body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(resp).await["reason"], "malformed_request");
}

#[tokio::test]
async fn directory_publish_wrong_width_key_or_sig_is_400() {
    let (app, _store) = build();
    // 16-byte identity_pub -> not a 32-byte Ed25519 key.
    let body = json!({
        "identity_pub": hexn(0x01, 16),
        "card": hexn(0xab, 64),
        "pop_sig": hexn(0x00, 64),
    });
    assert_eq!(
        call(&app, post_json("/directory/publish", &body))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    // 32-byte pop_sig -> not a 64-byte signature.
    let body = json!({
        "identity_pub": hexn(0x01, 32),
        "card": hexn(0xab, 64),
        "pop_sig": hexn(0x00, 32),
    });
    assert_eq!(
        call(&app, post_json("/directory/publish", &body))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn directory_fetch_malformed_account_id_is_400() {
    let (app, _store) = build();
    let resp = call(&app, get("/directory/fetch/nothex")).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// --- request body limits ----------------------------------------------------

#[tokio::test]
async fn directory_publish_body_over_the_limit_is_413() {
    // A body larger than the 64 KiB publish limit is rejected by the body-limit layer BEFORE
    // the handler runs (no valid proof needed to trigger it). This bounds the hex decode of an
    // oversized publish, closing the buffering DoS.
    let (app, _store) = build();
    let body = json!({
        "identity_pub": hexn(0x01, 32),
        "card": hexn(0xab, 40 * 1024), // 40 KiB -> 80 KiB hex -> body > 64 KiB
        "pop_sig": hexn(0x00, 64),
    });
    let resp = call(&app, post_json("/directory/publish", &body)).await;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn submit_body_over_the_limit_is_413() {
    // A submit body over the 4 MiB limit is rejected by the body-limit layer.
    let (app, _store) = build();
    let mut body = submit_body();
    body["ciphertext"] = json!(hexn(0x42, 2_200_000)); // ~2.2 MiB -> ~4.4 MiB hex -> body > 4 MiB
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn submit_accepts_a_body_larger_than_the_publish_limit() {
    // ~200 KiB body: over the directory publish limit (64 KiB) but well under the submit limit,
    // proving the limits are scoped per-endpoint (submit is not capped at the publish bound).
    let (app, _store) = build();
    let mut body = submit_body();
    body["ciphertext"] = json!(hexn(0x42, 100 * 1024)); // 100 KiB -> 200 KiB hex
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
}

// --- flood rate limiting (open endpoints) -----------------------------------

#[tokio::test]
async fn submit_flood_is_rate_limited() {
    // With a low cap, requests beyond it (same source key) return 429 BEFORE the handler runs.
    // oneshot carries no connection info, so all requests share the fallback key.
    let store: Arc<Mutex<dyn QueueStore>> = Arc::new(Mutex::new(InMemoryQueueStore::new()));
    let state = RelayState::new(store, Arc::new(NoopPushSender)).with_rate_limit(3, 60);
    let app = router(state);

    // The first 3 distinct submits are accepted (each a fresh route + replay token).
    for i in 0..3u8 {
        let mut body = submit_body();
        body["route_id"] = json!(hexn(0x40 + i, 32));
        body["replay_token"] = json!(hexn(0x10 + i, 32));
        let resp = call(&app, post_json("/relay/submit", &body)).await;
        assert_eq!(
            resp.status(),
            StatusCode::ACCEPTED,
            "submit {i} should pass"
        );
    }
    // The 4th within the window is rate-limited.
    let mut body = submit_body();
    body["route_id"] = json!(hexn(0x50, 32));
    body["replay_token"] = json!(hexn(0x20, 32));
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn directory_publish_flood_is_rate_limited() {
    let store: Arc<Mutex<dyn QueueStore>> = Arc::new(Mutex::new(InMemoryQueueStore::new()));
    let state = RelayState::new(store, Arc::new(NoopPushSender)).with_rate_limit(2, 60);
    let app = router(state);
    let id = IdentityKeyPair::generate();

    for _ in 0..2 {
        let resp = call(
            &app,
            post_json("/directory/publish", &publish_body(&id, &[0xabu8; 64])),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }
    let resp = call(
        &app,
        post_json("/directory/publish", &publish_body(&id, &[0xabu8; 64])),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}
