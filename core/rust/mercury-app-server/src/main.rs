#![forbid(unsafe_code)]
//! `mercury-app-server` binary — a DEV-ONLY loopback HTTP shim that drives a real
//! [`mercury_app::AppController`] over the live relay, for browser-based UI development and as the
//! driver for an AI participant (Claude-over-subscription) talking to users inside Mercury.
//!
//! Run:  cargo run -p mercury-app-server                  (port 7879, relay 127.0.0.1:8787)
//!       cargo run -p mercury-app-server -- 9100          (custom port)
//!       MERCURY_RELAY_URL=https://relay.mercury-messaging.com cargo run -p mercury-app-server
//!
//! It binds LOOPBACK ONLY and holds message keys — a development tool, never a production
//! transport (see the crate docs). Production exposure is Tauri in-process IPC / WASM.
//!
//! ## Persistent identity
//!
//! The controller's identity + sessions + log are persisted to an encrypted snapshot
//! (`MERCURY_APP_SNAPSHOT`, default `mercury-ai-snapshot.bin`) sealed under a 32-byte key
//! (`MERCURY_APP_KEY_FILE`, default `mercury-ai-key.bin`), saved periodically and reloaded on
//! start. Without this, a restart minted a brand-new identity (new account id) and orphaned any
//! inbound messages already queued at the relay for the old keys. Both files hold secrets — keep
//! them local and out of version control.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use mercury_app::AppController;
use mercury_app_server::router_with_handle;
use mercury_client::RelayTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let relay =
        std::env::var("MERCURY_RELAY_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".to_string());
    let port: u16 = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MERCURY_APP_SERVER_PORT").ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(7879);

    // Persistent identity (see the module docs): key file + encrypted snapshot, both local.
    let key_path = PathBuf::from(
        std::env::var("MERCURY_APP_KEY_FILE").unwrap_or_else(|_| "mercury-ai-key.bin".into()),
    );
    let snapshot_path = PathBuf::from(
        std::env::var("MERCURY_APP_SNAPSHOT").unwrap_or_else(|_| "mercury-ai-snapshot.bin".into()),
    );
    ensure_parent_dir(&key_path);
    ensure_parent_dir(&snapshot_path);
    let key = load_or_create_key(&key_path)?;

    // Restore the persistent identity if a readable snapshot exists; otherwise mint a new one.
    let controller = match std::fs::read(&snapshot_path) {
        Ok(blob) => {
            match AppController::from_encrypted_snapshot(
                RelayTransport::new(relay.clone()),
                &blob,
                &key,
            ) {
                Ok(c) => {
                    eprintln!(
                        "mercury-app-server: restored persistent identity from {}",
                        snapshot_path.display()
                    );
                    c
                }
                Err(_) => {
                    eprintln!(
                        "mercury-app-server: snapshot at {} is unreadable under the current key — starting a fresh identity",
                        snapshot_path.display()
                    );
                    AppController::new(RelayTransport::new(relay.clone()))
                }
            }
        }
        Err(_) => {
            eprintln!("mercury-app-server: no snapshot yet — creating a new persistent identity");
            AppController::new(RelayTransport::new(relay.clone()))
        }
    };

    let (app, ctrl) = router_with_handle(controller);

    // Persist (identity + sessions + log) periodically so the identity is stable across restarts
    // and queued inbound messages stay decryptable. Crash-consistent: temp file then atomic rename.
    {
        let ctrl = Arc::clone(&ctrl);
        let snap = snapshot_path.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(3));
            loop {
                tick.tick().await; // fires immediately on the first tick, then every 3s
                let blob = {
                    let guard = ctrl.lock().await;
                    guard.to_encrypted_snapshot(&key)
                };
                let tmp = snap.with_extension("tmp");
                if std::fs::write(&tmp, &blob)
                    .and_then(|()| std::fs::rename(&tmp, &snap))
                    .is_err()
                {
                    let _ = std::fs::remove_file(&tmp);
                }
            }
        });
    }

    // Loopback ONLY: this shim holds message keys; never bind a public interface.
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("mercury-app-server (DEV-ONLY) on http://{addr}  relay={relay}");
    eprintln!(
        "  persistent identity: key={} snapshot={}",
        key_path.display(),
        snapshot_path.display()
    );
    eprintln!("  GET /health ; POST /command  (body = a JSON command)");
    eprintln!("  WARNING: holds message keys; loopback only; not a production transport.");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Load a 32-byte key from `path`, or generate + persist one on first run.
fn load_or_create_key(path: &Path) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(key) = <[u8; 32]>::try_from(bytes.as_slice()) {
            return Ok(key);
        }
    }
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key)?;
    std::fs::write(path, key)?;
    Ok(key)
}

fn ensure_parent_dir(path: &Path) {
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(dir);
        }
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    eprintln!("mercury-app-server shutting down");
}
