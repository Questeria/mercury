//! `mercury-relay-server` — a real Axum/Tokio delivery relay.
//!
//! Wires the [`mercury_relay`] router and the key-transparency directory
//! ([`kt_router`]) over a queue store selected at startup, an [`InProcessWaker`]
//! (real-time long-poll wake), and the background expiry sweeper, then serves on
//! a configurable address (`MERCURY_RELAY_ADDR`, default `127.0.0.1:8787`).
//!
//! Queue store: a durable redb file when `MERCURY_RELAY_DB` is set, so queued
//! items and replay tombstones survive a restart; otherwise an
//! [`InMemoryQueueStore`]. The KT directory uses a per-directory VRF key from
//! `MERCURY_KT_VRF_SEED` (64 hex chars; an EPHEMERAL dev key if unset).
//! Username registry: a durable redb file when `MERCURY_USERNAME_DB` is set —
//! claimed handles then survive a restart, with every persisted binding
//! re-registered into the fresh KT directory in ONE epoch at boot (proofs stay
//! verifiable under the stable VRF key; epoch history resets — disclosed in
//! `docs/USERNAMES.md`). Otherwise the registry is in-memory (dev only).
//!
//! The relay is untrusted — it serves only opaque ciphertext and proofs clients
//! verify themselves — and authorizes the two write paths cryptographically:
//! `/directory/publish` requires an Ed25519 proof of possession, so a contact
//! card can only land in the directory slot derived from its signer's identity
//! key; poll / ack / delete require a route-ownership proof of possession
//! verified in `auth_layer`, so the `server_authenticated`,
//! `route_key_authenticated`, and `replay_window_valid` booleans fed to the core
//! admission gate are DERIVED from that verified proof — never supplied by the
//! client or a proxy. Verifying public-key proofs gains the relay no ability to
//! decrypt.
//!
//! Deployment seams (see `docs/RELAY-DEPLOYMENT.md`), deliberately not faked here:
//! - **TLS** — the relay serves plain HTTP behind a TLS-terminating reverse
//!   proxy; behind a proxy the per-IP flood limiter must key on a forwarded-
//!   client header (it currently keys on the direct peer IP).
//! - **Real-time delivery** — [`InProcessWaker`] wakes a long-poll
//!   `GET /relay/wait/{route}` the instant a message is enqueued, so a *running*
//!   client (foreground or tray) receives at once. It is single-instance (the
//!   wake is in-process); a multi-instance deployment, or true closed-process OS
//!   push (APNs/FCM/WNS), implements the same `PushSender` seam.

#![forbid(unsafe_code)]

use std::sync::Arc;
use std::time::Duration;

use mercury_kt::KtDirectory;
use mercury_relay::{
    DurableUsernameStore, InMemoryQueueStore, InProcessWaker, KtState, RelayState,
    UsernameRegistry, hex, kt_router, kt_username_label, router, spawn_sweeper,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("MERCURY_RELAY_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".to_string());

    // Durable queue store if MERCURY_RELAY_DB names a redb file path (queued items + replay
    // tombstones then survive a restart); otherwise the default in-memory store.
    let store: Arc<Mutex<dyn mercury_relay::QueueStore>> = match std::env::var("MERCURY_RELAY_DB") {
        Ok(path) => {
            eprintln!("mercury-relay-server: durable queue store at {path}");
            let durable = mercury_relay::RedbQueueStore::open(&path)
                .map_err(|e| format!("opening MERCURY_RELAY_DB '{path}': {e}"))?;
            Arc::new(Mutex::new(durable))
        }
        Err(_) => Arc::new(Mutex::new(InMemoryQueueStore::new())),
    };
    // In-process waker: a long-poll `GET /relay/wait/{route}` is woken the instant a message is
    // enqueued, so a running client receives in real time. Single-instance (the wake is
    // in-process); a multi-instance deployment needs shared pub/sub or the APNs/FCM bridge.
    let push = Arc::new(InProcessWaker::new());

    // Background expiry sweep: clear payloads of pending items past their TTL.
    let _sweeper = spawn_sweeper(Arc::clone(&store), Duration::from_secs(30));

    // Host the key-transparency directory. Production sets MERCURY_KT_VRF_SEED to
    // a persisted 32-byte key so the pinnable public key is stable across
    // restarts; without it we generate an EPHEMERAL key (dev only).
    let mut kt_dir = match std::env::var("MERCURY_KT_VRF_SEED") {
        Ok(seed_hex) => {
            let seed: [u8; 32] = hex::decode(&seed_hex)
                .map_err(|_| "MERCURY_KT_VRF_SEED must be hex")?
                .as_slice()
                .try_into()
                .map_err(|_| "MERCURY_KT_VRF_SEED must be 32 bytes (64 hex chars)")?;
            KtDirectory::with_vrf_seed(seed).await?
        }
        Err(_) => {
            // Durable usernames (MERCURY_USERNAME_DB) imply a PRODUCTION deployment. An ephemeral
            // per-boot VRF key would silently break every pinned client after a restart, so fail
            // CLOSED rather than warn-and-continue.
            if std::env::var("MERCURY_USERNAME_DB").is_ok() {
                return Err("MERCURY_USERNAME_DB is set (durable/production usernames) but MERCURY_KT_VRF_SEED is unset — refusing to start with an ephemeral VRF key that would break pinned clients across restarts. Set MERCURY_KT_VRF_SEED to a persisted 32-byte hex key (openssl rand -hex 32).".into());
            }
            eprintln!(
                "warning: MERCURY_KT_VRF_SEED unset — using an EPHEMERAL VRF key (pinned proofs will not verify across restarts; set it in production)"
            );
            KtDirectory::new().await?
        }
    };
    // Durable username registry if MERCURY_USERNAME_DB names a redb file path: claimed handles
    // survive a restart. The persisted bindings are re-registered into the fresh KT directory in
    // ONE epoch (the AKD log itself is in-memory and rebuilt at boot; proofs remain verifiable
    // because the VRF key is stable, but epoch history resets — see docs/USERNAMES.md).
    let kt_state = match std::env::var("MERCURY_USERNAME_DB") {
        Ok(path) => {
            let store = DurableUsernameStore::open(&path)
                .map_err(|e| format!("opening MERCURY_USERNAME_DB '{path}': {e}"))?;
            let bindings = store
                .load_all()
                .map_err(|e| format!("loading MERCURY_USERNAME_DB '{path}': {e}"))?;
            if !bindings.is_empty() {
                // Re-commit every restored binding into the fresh log in a SINGLE epoch
                // (register_batch rejects an empty batch, hence the guard).
                let updates: Vec<(String, Vec<u8>)> = bindings
                    .iter()
                    .map(|(name, id)| (kt_username_label(name), id.to_vec()))
                    .collect();
                kt_dir
                    .register_batch(&updates)
                    .await
                    .map_err(|e| format!("re-registering persisted usernames into KT: {e}"))?;
            }
            eprintln!(
                "mercury-relay-server: durable username store at {path} — restored {} binding(s) into the KT log",
                bindings.len()
            );
            let registry = UsernameRegistry::with_durable(store)
                .map_err(|e| format!("building username registry from '{path}': {e}"))?;
            KtState::with_registry(kt_dir, registry)
        }
        Err(_) => KtState::new(kt_dir),
    };

    let state = RelayState::new(Arc::clone(&store), push);
    let app = router(state).merge(kt_router(kt_state));

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("mercury-relay-server listening on {addr}");

    // Provide peer connection info so the open-endpoint flood limiter can key by source IP.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

/// Wait for Ctrl-C, then begin graceful shutdown.
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    eprintln!("mercury-relay-server shutting down");
}
