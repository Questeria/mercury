//! The short-lived, single-use pairing-code broker, driven through the Axum router with
//! `tower::ServiceExt::oneshot`. The relay treats the card as opaque bytes; it authorizes a broker
//! with a pairing proof of possession and a fetch consumes the code exactly once.

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use mercury_keys::{IdentityKeyPair, sign_pairing_publish};
use mercury_relay::{InMemoryQueueStore, NoopPushSender, QueueStore, RelayState, hex, router};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower::ServiceExt;

fn build() -> Router {
    let store: Arc<Mutex<dyn QueueStore>> = Arc::new(Mutex::new(InMemoryQueueStore::new()));
    router(RelayState::new(store, Arc::new(NoopPushSender)))
}

/// An opaque "contact card" — the broker never parses it, so arbitrary bytes are a faithful stand-in.
fn fake_card() -> Vec<u8> {
    b"{\"opaque\":\"pairing-card-bytes\"}".to_vec()
}

async fn post_json(app: Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

async fn get(app: Router, uri: &str) -> (StatusCode, Value) {
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

fn publish_body(id: &IdentityKeyPair, card: &[u8]) -> Value {
    let sig = sign_pairing_publish(id, card);
    json!({
        "identity_pub": hex::encode(id.public().as_bytes()),
        "card": hex::encode(card),
        "pop_sig": hex::encode(&sig),
    })
}

#[tokio::test]
async fn publish_then_fetch_round_trips_once() {
    let app = build();
    let id = IdentityKeyPair::generate();
    let card = fake_card();

    let (status, body) = post_json(app.clone(), "/pair/publish", publish_body(&id, &card)).await;
    assert_eq!(status, StatusCode::CREATED);
    let code = body["code"]
        .as_str()
        .expect("a code is returned")
        .to_string();
    assert!(code.contains('-'), "code is displayed grouped: {code}");

    // Fetch once -> the card comes back verbatim.
    let (status, body) = get(app.clone(), &format!("/pair/fetch/{code}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(hex::decode(body["card"].as_str().unwrap()).unwrap(), card);

    // Single-use: a second fetch is 404.
    let (status, _) = get(app, &format!("/pair/fetch/{code}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn fetch_is_case_and_hyphen_insensitive() {
    let app = build();
    let id = IdentityKeyPair::generate();
    let card = fake_card();
    let (_, body) = post_json(app.clone(), "/pair/publish", publish_body(&id, &card)).await;
    let code = body["code"].as_str().unwrap().to_string();

    // Type it with the hyphen stripped and lower-cased — the relay canonicalizes both sides.
    let typed = code.replace('-', "").to_lowercase();
    let (status, body) = get(app, &format!("/pair/fetch/{typed}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(hex::decode(body["card"].as_str().unwrap()).unwrap(), card);
}

#[tokio::test]
async fn publish_with_a_bad_proof_is_unauthorized() {
    let app = build();
    let id = IdentityKeyPair::generate();
    let card = fake_card();
    // A proof over DIFFERENT bytes must not authorize brokering this card.
    let wrong_sig = sign_pairing_publish(&id, b"some other bytes");
    let body = json!({
        "identity_pub": hex::encode(id.public().as_bytes()),
        "card": hex::encode(&card),
        "pop_sig": hex::encode(&wrong_sig),
    });
    let (status, _) = post_json(app, "/pair/publish", body).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn unknown_code_is_404() {
    let app = build();
    let (status, _) = get(app, "/pair/fetch/ZZZZZ-ZZZZZ").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn malformed_publish_is_400() {
    let app = build();
    let body = json!({ "identity_pub": "nothex", "card": "zz", "pop_sig": "00" });
    let (status, _) = post_json(app, "/pair/publish", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
