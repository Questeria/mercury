#![forbid(unsafe_code)]
//! `mercury-ui-bridge` — a thin HTTP shim that serves the real mercury-core
//! `PlatformDecisionView` fixtures as JSON.
//!
//! Purpose: let the web client (`ui/app`) render decisions produced by the
//! actual Rust core (`mercury-core`, via `mercury-bindings`) instead of its
//! in-app TypeScript simulator. This recomputes NO policy — it returns the
//! exact views `mercury-bindings` already builds from the core's
//! `from_bootstrap` / `from_outbound_send` / `from_client_receive` /
//! `from_policy` constructors. It is a dev/integration tool, not a production
//! transport (production is the FFI/Tauri bridge).
//!
//! Run:  cargo run -p mercury-ui-bridge            (defaults to port 7878)
//!       cargo run -p mercury-ui-bridge -- 9000    (custom port)
//!
//! Endpoints:
//!   GET /health                 -> { status, service }
//!   GET /decisions              -> { decisions: [<scenario names>] }
//!   GET /decision/<scenario>    -> the real PlatformDecisionView JSON, or 404

use std::net::SocketAddr;

use axum::{
    Json, Router,
    extract::{Path, Request},
    http::{StatusCode, header::HeaderValue},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use mercury_bindings::{
    PLATFORM_FIXTURES, mercury_accept_received_ciphertext, mercury_prepare_send,
    platform_fixture_by_name, platform_fixture_view,
};
use mercury_core::{
    ClientReceiveDecision, ClientReceiveReason, OutboundSendDecision, OutboundSendReason,
    PlatformDecisionView,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MERCURY_UI_BRIDGE_PORT").ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(7878);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("mercury-ui-bridge: failed to bind {addr}: {err}"));

    println!("mercury-ui-bridge listening on http://{addr}");
    println!("  GET /health");
    println!("  GET /decisions");
    println!("  GET /decision/<scenario>");

    axum::serve(listener, app())
        .await
        .expect("mercury-ui-bridge: server error");
}

fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/decisions", get(list_decisions))
        .route("/decision/{name}", get(get_decision))
        .layer(middleware::from_fn(allow_cors))
}

/// Permissive CORS so the bridge works whether the client reaches it via the
/// Vite dev proxy (same-origin) or directly from another origin. GET-only and
/// payload-free, so a simple `Access-Control-Allow-Origin` header suffices.
async fn allow_cors(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("access-control-allow-origin", HeaderValue::from_static("*"));
    response
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok", "service": "mercury-ui-bridge" }))
}

async fn list_decisions() -> impl IntoResponse {
    let mut names: Vec<&str> = PLATFORM_FIXTURES.iter().map(|d| d.name).collect();
    names.extend_from_slice(EXTRA_DECISIONS);
    Json(json!({ "decisions": names }))
}

async fn get_decision(Path(name): Path<String>) -> Response {
    // platform_fixture_view returns the real mercury-core PlatformDecisionView
    // for the 10 canonical fixtures; bridge_extra_view constructs additional
    // real decisions via the same pure constructors.
    if let Some(fixture) = platform_fixture_by_name(&name) {
        return Json(platform_fixture_view(fixture)).into_response();
    }
    if let Some(view) = bridge_extra_view(&name) {
        return Json(view).into_response();
    }
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "unknown scenario", "scenario": name })),
    )
        .into_response()
}

/// Additional real-core decisions the bridge constructs directly (beyond the 10
/// canonical fixtures), so the UI can exercise reason variants no fixture
/// covers -- e.g. recipient/sender DEVICE_TRUST_REJECTED. These call the same
/// pure mercury-core constructors (`mercury_prepare_send` /
/// `mercury_accept_received_ciphertext`); the bridge invents no policy, it only
/// supplies decision inputs the core then turns into a view.
const EXTRA_DECISIONS: &[&str] = &[
    "outbound_device_trust_rejected",
    "receive_sender_device_trust_rejected",
];

fn bridge_extra_view(name: &str) -> Option<PlatformDecisionView> {
    match name {
        "outbound_device_trust_rejected" => Some(mercury_prepare_send(OutboundSendDecision {
            accepted: false,
            can_send: false,
            can_persist_ciphertext: false,
            requires_user_action: true,
            reason: OutboundSendReason::DeviceTrustRejected,
        })),
        "receive_sender_device_trust_rejected" => {
            Some(mercury_accept_received_ciphertext(ClientReceiveDecision {
                accepted: false,
                can_decrypt: false,
                can_persist_ciphertext: false,
                can_expose_to_ui: false,
                requires_client_retry: false,
                requires_user_action: true,
                reason: ClientReceiveReason::SenderDeviceTrustRejected,
            }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get_json(uri: &str) -> (StatusCode, serde_json::Value) {
        let response = app()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, value)
    }

    #[tokio::test]
    async fn serves_real_bootstrap_view() {
        let (status, v) = get_json("/decision/bootstrap_accepted").await;
        assert_eq!(status, StatusCode::OK);
        // These come straight from mercury-core, not a simulator.
        assert_eq!(v["source"], "client_bootstrap");
        assert_eq!(v["accepted"], true);
        assert_eq!(v["can_open_message_ui"], true);
        assert_eq!(v["reason_label"], "ACCEPTED");
    }

    #[tokio::test]
    async fn serves_real_receive_ordering_gap() {
        let (status, v) = get_json("/decision/client_receive_ordering_gap").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(v["source"], "client_receive");
        assert_eq!(v["accepted"], false);
        assert_eq!(v["requires_client_retry"], true);
    }

    #[tokio::test]
    async fn lists_all_decisions() {
        let (status, v) = get_json("/decisions").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            v["decisions"].as_array().unwrap().len(),
            PLATFORM_FIXTURES.len() + EXTRA_DECISIONS.len()
        );
    }

    #[tokio::test]
    async fn serves_extra_recipient_device_trust_rejected() {
        let (status, v) = get_json("/decision/outbound_device_trust_rejected").await;
        assert_eq!(status, StatusCode::OK);
        // Real OutboundSendReason variant with no canonical fixture.
        assert_eq!(v["source"], "outbound_send");
        assert_eq!(v["reason_label"], "DEVICE_TRUST_REJECTED");
        assert_eq!(v["accepted"], false);
        assert_eq!(v["can_send"], false);
    }

    #[tokio::test]
    async fn serves_extra_sender_device_trust_rejected() {
        let (status, v) = get_json("/decision/receive_sender_device_trust_rejected").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(v["source"], "client_receive");
        assert_eq!(v["reason_label"], "SENDER_DEVICE_TRUST_REJECTED");
        assert_eq!(v["accepted"], false);
        assert_eq!(v["requires_user_action"], true);
    }

    #[tokio::test]
    async fn unknown_scenario_is_404() {
        let (status, _) = get_json("/decision/does_not_exist").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
