#![forbid(unsafe_code)]
//! `mercury-app-server` — a DEV-ONLY HTTP shim over the [`mercury_app`] controller.
//!
//! It exposes one in-memory [`AppController`] over two endpoints so a browser-based UI can
//! drive REAL end-to-end-encrypted messaging in development:
//!
//!   GET  /health   -> `{ "status": "ok", "service": "mercury-app-server" }`
//!   POST /command  -> the request body is a JSON command (see
//!                     [`AppController::handle_command`]); the response is the JSON envelope
//!                     `{ "ok": ..., "result"|"error": ... }`. Always HTTP 200 when the body
//!                     was read: command-level failures are reported IN-BAND as `ok:false`
//!                     (JSON-RPC style), so the UI has one uniform error channel.
//!
//! ## DEV-ONLY — read this
//!
//! This process **holds the message keys** (the controller's identity + ratchet sessions) and
//! lets anything that can reach the port send/read messages as that identity. It is therefore
//! bound to **loopback only** and is a development/integration tool, NOT a production
//! transport. The production exposure is **Tauri in-process IPC** (no open socket) or WASM
//! (same-origin, in-browser) — both drive the very same [`AppController::handle_command`]
//! surface, so swapping this shim out changes nothing above it. (Mirrors how
//! `mercury-ui-bridge` is a dev decision-view shim, not the shipping path.)

use std::sync::Arc;

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{Next, from_fn},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mercury_app::AppController;
use mercury_client::Transport;
use tokio::sync::Mutex;

/// Shared server state: the single controller behind an async mutex.
pub struct AppState<T: Transport> {
    controller: Arc<Mutex<AppController<T>>>,
}

// Manual `Clone` (derive would demand `T: Clone`, which the controller does not need —
// only the `Arc` is cloned).
impl<T: Transport> Clone for AppState<T> {
    fn clone(&self) -> Self {
        Self {
            controller: Arc::clone(&self.controller),
        }
    }
}

/// Build the shim router around `controller`. Generic over the transport so the binary uses
/// the real HTTP relay while tests use the in-memory double.
pub fn router<T: Transport + Send + 'static>(controller: AppController<T>) -> Router {
    router_with_handle(controller).0
}

/// Like [`router`], but also returns the shared `Arc<Mutex<AppController>>` so the caller can
/// observe/persist controller state out-of-band (e.g. the binary's periodic snapshot saver, which
/// keeps the AI participant's identity + sessions stable across restarts).
pub fn router_with_handle<T: Transport + Send + 'static>(
    controller: AppController<T>,
) -> (Router, Arc<Mutex<AppController<T>>>) {
    let controller = Arc::new(Mutex::new(controller));
    let state = AppState {
        controller: Arc::clone(&controller),
    };
    let router = Router::new()
        .route("/health", get(health))
        .route("/command", post(command::<T>))
        .with_state(state)
        // DEV-ONLY: the Vite dev server drives this shim from a different origin (port), so we
        // answer the CORS preflight and tag responses. See `dev_cors`.
        .layer(from_fn(dev_cors));
    (router, controller)
}

/// DEV-ONLY permissive CORS. The browser UI is served by the Vite dev server on a different
/// origin (port) and makes cross-origin `fetch`es to this loopback shim; this answers the
/// preflight (`OPTIONS`) and tags every response. Wildcard (`*`) is acceptable ONLY because the
/// shim is a loopback development tool that already trusts any local caller (it holds the
/// message keys) — it is NOT a production transport. Production exposure is Tauri in-process IPC
/// (same-origin, no CORS).
async fn dev_cors(request: Request, next: Next) -> Response {
    if request.method() == Method::OPTIONS {
        let mut response = StatusCode::NO_CONTENT.into_response();
        set_cors(response.headers_mut());
        return response;
    }
    let mut response = next.run(request).await;
    set_cors(response.headers_mut());
    response
}

fn set_cors(headers: &mut HeaderMap) {
    headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    headers.insert(
        "access-control-allow-methods",
        HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    headers.insert(
        "access-control-allow-headers",
        HeaderValue::from_static("content-type"),
    );
}

async fn health() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"status":"ok","service":"mercury-app-server"}"#,
    )
}

/// Run one JSON command against the controller. The body is taken as a UTF-8 string (axum
/// returns 400 for a non-UTF-8 body and 413 past its default body limit); a malformed/unknown
/// command is reported in-band by [`AppController::handle_command`] as `ok:false`.
async fn command<T: Transport + Send + 'static>(
    State(state): State<AppState<T>>,
    body: String,
) -> Response {
    // Serialize access to the single controller. NOTE (dev-only scope): `handle_command` is
    // synchronous, and with the real `RelayTransport` it performs BLOCKING `ureq` I/O while
    // this lock is held, on a tokio worker thread (no `spawn_blocking`). That is fine for a
    // single-controller loopback dev shim — it cannot deadlock and at worst parks one worker
    // per in-flight relay call — but any move to multi-controller or a production-adjacent
    // exposure must switch to an async transport or offload via `spawn_blocking` first.
    let json = {
        let mut controller = state.controller.lock().await;
        controller.handle_command(&body)
    };
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json,
    )
        .into_response()
}
