//! HTTP shim tests, driven through the Axum router with `tower::ServiceExt::oneshot` — no
//! bound port. Two routers sharing an in-memory transport hold a full conversation purely over
//! `POST /command`, proving the dev shim faithfully exposes the controller.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use mercury_app::AppController;
use mercury_app_server::router;
use mercury_client::InMemoryTransport;
use serde_json::Value;
use tower::ServiceExt;

async fn get(app: &Router, uri: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn command(app: &Router, body: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/command")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn health_is_ok() {
    let app = router(AppController::new(InMemoryTransport::new()));
    let (status, body) = get(&app, "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "mercury-app-server");
}

#[tokio::test]
async fn command_account_id_round_trips() {
    let app = router(AppController::new(InMemoryTransport::new()));
    let (status, body) = command(&app, r#"{"cmd":"account_id"}"#).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    let id = body["result"]["account_id"].as_str().unwrap();
    assert_eq!(id.len(), 64); // 32-byte hex
}

#[tokio::test]
async fn a_malformed_command_is_an_in_band_error_not_a_5xx() {
    let app = router(AppController::new(InMemoryTransport::new()));
    // The body was read fine, so HTTP is 200; the command-level failure is reported in-band.
    let (status, body) = command(&app, "not json at all").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], false);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn two_shims_hold_a_conversation_over_http() {
    // Two controllers (each behind its own router) share an in-memory transport, and exchange
    // a real end-to-end-encrypted message purely over POST /command.
    let transport = InMemoryTransport::new();
    let alice = router(AppController::new(transport.clone()));
    let bob = router(AppController::new(transport.clone()));

    let alice_id = command(&alice, r#"{"cmd":"account_id"}"#).await.1["result"]["account_id"]
        .as_str()
        .unwrap()
        .to_string();
    let bob_id = command(&bob, r#"{"cmd":"account_id"}"#).await.1["result"]["account_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Bob publishes; Alice adds + sends — all over HTTP.
    assert_eq!(
        command(&bob, r#"{"cmd":"publish_self"}"#).await.1["ok"],
        true
    );
    let add = format!(r#"{{"cmd":"add_contact","account_id":"{bob_id}"}}"#);
    assert_eq!(command(&alice, &add).await.1["ok"], true);
    let send = format!(r#"{{"cmd":"send","peer":"{bob_id}","text":"hi over http"}}"#);
    assert_eq!(command(&alice, &send).await.1["ok"], true);

    // Bob polls over HTTP and sees the decrypted message from Alice.
    let inbox = command(&bob, r#"{"cmd":"poll"}"#).await.1;
    let msgs = inbox["result"]["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0]["text"], "hi over http");
    assert_eq!(msgs[0]["direction"], "incoming");
    assert_eq!(msgs[0]["peer"], alice_id);
}
