// Prevent a console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Mercury desktop client (Tauri v2).
//!
//! The webview renders the same React UI as the browser build; a single in-process IPC command
//! (`command`) drives the REAL `mercury_app::AppController` over the live relay. Unlike the dev
//! `mercury-app-server` HTTP shim, there is NO open socket and keys/ciphertext never leave this
//! native process — the webview only ever sees the plaintext the user sends/receives. State
//! survives a restart via the PB-1 encrypted snapshot, sealed under a device key from the OS
//! keychain. No crypto or policy lives here: confidentiality is mercury-client's, admission is
//! mercury-core's, persistence is the audited snapshot.
//!
//! (`forbid(unsafe_code)` is intentionally NOT set on this binary — the Tauri context/handler
//! macros may expand to unsafe in the host crate; the security-critical crates keep their fence.)

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mercury_app::AppController;
use mercury_client::{RelayTransport, Transport};
use mercury_store::MercuryKeychainAdapter;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri_plugin_autostart::MacosLauncher;

/// OS-keychain coordinates for the device root key that seals the at-rest snapshot.
const KEYCHAIN_SERVICE: &str = "com.mercury.messaging";
const KEYCHAIN_ACCOUNT: &str = "device-root";
/// Default relay endpoint (override with `MERCURY_RELAY_URL`, e.g. `http://127.0.0.1:8787` for a
/// local relay during development).
const DEFAULT_RELAY: &str = "https://relay.mercury-messaging.com";
const SNAPSHOT_FILE: &str = "mercury-snapshot.bin";
const WINDOW_ICON: &[u8] = include_bytes!("../icons/128x128.png");

/// The DEFAULT relay's username-directory VRF public key, BAKED into the signed build — real
/// out-of-band pinning (closing the trust-on-first-use bootstrap for default-relay users): a relay
/// that is malicious from first contact can no longer serve its own key, because username proofs
/// are verified against THIS pin, distributed inside the Authenticode-signed installer rather than
/// fetched from the relay. Only applied when no pin exists yet and the DEFAULT relay is in use; a
/// custom MERCURY_RELAY_URL stays trust-on-first-use, honestly (we cannot know its key here).
/// Obtain/verify out-of-band: `curl https://relay.mercury-messaging.com/kt/vrf-key`.
const DEFAULT_RELAY_VRF_KEY_HEX: &str =
    "7bc57cb05e12c126d9763e8c3f1192f84a475db073c3bd8b513754910a87d0fd";
/// The DEFAULT relay's KT LOG public key (Ed25519), baked alongside the VRF key: username lookups
/// must also present a tree head SIGNED by this key. Obtain/verify out-of-band:
/// `curl https://relay.mercury-messaging.com/kt/vrf-key` (`log_public_key`).
const DEFAULT_RELAY_LOG_KEY_HEX: &str =
    "fa5526baa40ce74d6a140893965742caa3c1b8d0f038e9319d627f908ec48e67";

/// Managed app state: the live controller behind a mutex, plus the device key + snapshot path used
/// to persist after every mutating command.
struct AppState {
    controller: Mutex<AppController<RelayTransport>>,
    key: [u8; 32],
    snapshot_path: PathBuf,
    /// A SEPARATE relay handle used only by `wait_for_delivery` to long-poll `/relay/wait`, so a
    /// multi-second wait runs WITHOUT holding the `controller` lock (send/poll stay responsive).
    /// The wait carries a proof pre-signed by the controller, so this handle needs no clock sync.
    wait_transport: RelayTransport,
    /// The relay base URL in use — needed to build the replacement transport on a backup restore.
    relay_url: String,
}

/// Is this the `poll` command? (Its persist is conditional on `take_state_dirty` — see `command`.)
fn is_poll(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("cmd").and_then(|c| c.as_str()).map(|c| c == "poll"))
        .unwrap_or(false)
}

/// Does this command mutate controller state (and therefore warrant a snapshot write)? Read-only
/// commands (`account_id`, `can_send`, `history`, `decisions`, `conversation_flags`,
/// `safety_number`, `backup_export`) skip the re-pickle.
fn is_mutating(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("cmd").and_then(|c| c.as_str()).map(str::to_owned))
        .map(|cmd| {
            matches!(
                cmd.as_str(),
                "send"
                    | "poll"
                    | "add_contact"
                    | "publish_self"
                    | "delete_conversation"
                    | "clear_history"
                    | "delete_for_everyone"
                    | "mute"
                    | "unmute"
                    | "block"
                    | "unblock"
                    | "set_disappearing"
                    | "pairing_code"
                    | "add_by_pairing"
                    | "claim_username"
                    | "add_by_username"
                    | "set_verified"
                    | "unset_verified"
                    | "send_attachment"
                    | "create_group"
                    | "group_send"
                    | "leave_group"
                    | "set_read_receipts"
                    | "set_pinned"
            )
        })
        .unwrap_or(false)
}

/// The ONE IPC bridge. Runs a JSON command against the real `AppController` off the UI thread (the
/// relay transport does blocking I/O), then persists on any mutating command. Returns the SAME
/// `{ok,result|error}` JSON envelope the dev HTTP shim returns, so the UI binding is unchanged.
#[tauri::command]
async fn command(state: tauri::State<'_, Arc<AppState>>, body: String) -> Result<String, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut ctrl = state.controller.lock().expect("controller mutex poisoned");
        let out = ctrl.handle_command(&body);
        // Decide whether to re-seal the snapshot. Every mutating command persists unconditionally,
        // EXCEPT `poll` — which runs ~1/s for the app's lifetime, so re-sealing the whole snapshot
        // on an IDLE poll is pure waste (CPU + disk churn that grows with history size). A poll
        // re-seals ONLY when it actually changed durable state, reported authoritatively by the
        // controller via `take_state_dirty()`. That is strictly more correct than the old
        // "messages + updated both empty" heuristic: a control-only first-flight (e.g. a group
        // invite) ACCEPTS a session + CONSUMES a one-time prekey while surfacing neither a message
        // nor a dirty conversation — the heuristic skipped it, losing the session + re-opening the
        // prekey on the next quit/restart. `take_state_dirty()` is called on every poll so the flag
        // is always cleared, never leaking into a later command.
        // `take_state_dirty()` is read (and cleared) on EVERY command: poll() sets it on a durable
        // change, and so does mark_read when it sends a read watermark (advancing the ratchet) —
        // mark_read is otherwise not a "mutating" command, so without this it would not persist and
        // a read-then-quit could rewind the on-disk send index.
        let state_changed = ctrl.take_state_dirty();
        let persist = if is_poll(&body) {
            state_changed
        } else {
            is_mutating(&body) || state_changed
        };
        if persist {
            // Crash-consistent persist: write a temp file then atomically rename it over the live
            // snapshot. A torn or failed write therefore can NEVER corrupt or reset the existing
            // snapshot (the account's identity + sessions); on any error the previous good snapshot
            // and the in-memory conversation are both left untouched. `fs::rename` replaces the
            // destination on both Windows (MoveFileEx REPLACE_EXISTING) and Unix.
            let blob = ctrl.to_encrypted_snapshot(&state.key);
            let tmp = state.snapshot_path.with_extension("tmp");
            let persisted = std::fs::write(&tmp, &blob)
                .and_then(|()| std::fs::rename(&tmp, &state.snapshot_path));
            if let Err(e) = persisted {
                eprintln!("mercury-desktop: snapshot persist failed (kept previous snapshot): {e}");
                let _ = std::fs::remove_file(&tmp);
            }
        }
        out
    })
    .await
    .map_err(|e| format!("command task failed: {e}"))
}

/// Long-poll the relay for a pending delivery (real-time receive). Returns `true` when a delivery is
/// likely pending (the UI should then run a `poll` command) or `false` on timeout/none. The route +
/// proof are taken under a BRIEF controller lock, which is released BEFORE the wait, so the up-to-25s
/// hold never blocks `send`/`poll`. Runs off the UI thread (blocking ureq). Falls back to `false` on
/// any error so the UI's delivery loop degrades to interval polling instead of stalling.
#[tauri::command]
async fn wait_for_delivery(state: tauri::State<'_, Arc<AppState>>) -> Result<bool, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let auth = {
            let ctrl = state.controller.lock().expect("controller mutex poisoned");
            ctrl.poll_auth()
        }; // controller lock released here, before the long-poll
        match auth {
            Some((route, auth)) => state.wait_transport.wait(&route, &auth).unwrap_or(false),
            None => false,
        }
    })
    .await
    .map_err(|e| format!("wait task failed: {e}"))
}

/// Persist the controller's current state to the snapshot file (crash-consistent temp+rename, the
/// same discipline as the per-command autosave).
fn persist_snapshot(state: &AppState, ctrl: &AppController<RelayTransport>) -> Result<(), String> {
    let blob = ctrl.to_encrypted_snapshot(&state.key);
    let tmp = state.snapshot_path.with_extension("tmp");
    std::fs::write(&tmp, &blob)
        .and_then(|()| std::fs::rename(&tmp, &state.snapshot_path))
        .map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            format!("snapshot persist failed: {e}")
        })
}

/// Save bytes (base64 from the webview) into the user's Downloads folder under a sanitized name,
/// returning the full path. Used for backup files and received attachments — the webview cannot
/// write files itself, and this keeps the surface to ONE well-understood destination (no caller-
/// chosen paths crossing the IPC boundary).
#[tauri::command]
async fn save_file(
    app: tauri::AppHandle,
    name: String,
    data_b64: String,
) -> Result<String, String> {
    use base64::Engine as _;
    use tauri::Manager as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data_b64)
        .map_err(|_| "invalid file data".to_string())?;
    // Final path component only — a hostile suggested name must never traverse directories.
    let base = name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_start_matches('.');
    let safe: String = base
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        .take(120)
        .collect();
    // Strip trailing dots/spaces — Windows silently drops them, which could defeat the no-clobber
    // check below or resolve to an unintended name.
    let safe = safe.trim_end_matches([' ', '.']).to_string();
    // Reject Windows RESERVED device names (CON, PRN, AUX, NUL, COM1-9, LPT1-9), matched on the
    // stem case-insensitively: opening one writes to a device, not a file in Downloads.
    let stem_upper = safe
        .rsplit_once('.')
        .map_or(safe.as_str(), |(s, _)| s)
        .to_ascii_uppercase();
    let reserved = matches!(stem_upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem_upper.len() == 4
            && (stem_upper.starts_with("COM") || stem_upper.starts_with("LPT"))
            && matches!(stem_upper.as_bytes()[3], b'1'..=b'9'));
    let safe = if safe.is_empty() || reserved {
        "mercury-file.bin".to_string()
    } else {
        safe
    };
    let dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("no Downloads folder: {e}"))?;
    // Never clobber an existing file: append " (n)" before the extension until free.
    let mut path = dir.join(&safe);
    let mut n = 1u32;
    while path.exists() {
        let (stem, ext) = match safe.rsplit_once('.') {
            Some((s, e)) if !s.is_empty() => (s.to_string(), format!(".{e}")),
            _ => (safe.clone(), String::new()),
        };
        path = dir.join(format!("{stem} ({n}){ext}"));
        n += 1;
    }
    std::fs::write(&path, &bytes).map_err(|e| format!("could not save: {e}"))?;
    Ok(path.display().to_string())
}

/// Restore an account from a passphrase-sealed backup file: build a fresh controller from the
/// backup, REPLACE the live one, and persist it under the device key. Fail-closed: a wrong
/// passphrase or damaged file changes nothing. The UI reloads after a successful restore.
#[tauri::command]
async fn restore_backup(
    state: tauri::State<'_, Arc<AppState>>,
    data_b64: String,
    passphrase: String,
) -> Result<(), String> {
    use base64::Engine as _;
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&data_b64)
            .map_err(|_| "invalid backup file".to_string())?;
        let restored = AppController::import_backup(
            RelayTransport::new(state.relay_url.clone()),
            &bytes,
            &passphrase,
        )
        .map_err(|e| e.to_string())?;
        // Persist FIRST (under the device key), then swap — a failed write leaves the old account
        // fully intact in memory and on disk.
        persist_snapshot(&state, &restored)?;
        let mut ctrl = state.controller.lock().expect("controller mutex poisoned");
        *ctrl = restored;
        Ok(())
    })
    .await
    .map_err(|e| format!("restore task failed: {e}"))?
}

/// Erase this device's Mercury account: remove the OS-keychain device key, then the encrypted
/// snapshot, then RESTART into a clean first run. Order is deliberate and fail-closed: the key is
/// deleted FIRST — if that fails we stop and delete nothing, so the account stays whole and openable
/// and the user can simply retry (never a half-wiped state). Only once the key is gone (which makes
/// the ciphertext permanently unopenable) do we remove the snapshot file. The restart discards the
/// in-memory controller, so no later persist can resurrect the account. This is LOCAL erasure (this
/// device + its at-rest data); server traces and peer copies are handled by the account-delete flow.
#[tauri::command]
async fn delete_account(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        // Hold the controller lock so no concurrent command can re-seal a snapshot mid-erase.
        let _guard = state.controller.lock().expect("controller mutex poisoned");
        let keychain = MercuryKeychainAdapter::os(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);
        keychain.delete_root_key().map_err(|e| {
            format!(
                "could not remove the device key from the OS keychain ({e}); nothing was deleted — \
                 unlock your system keychain and try again"
            )
        })?;
        // Key is gone (ciphertext now unopenable); remove the snapshot + any half-written temp.
        let _ = std::fs::remove_file(&state.snapshot_path);
        let _ = std::fs::remove_file(state.snapshot_path.with_extension("tmp"));
        Ok(())
    })
    .await
    .map_err(|e| format!("erase task failed: {e}"))??;
    // Relaunch into a clean first run; the live in-memory account is dropped without re-persisting.
    app.restart();
    #[allow(unreachable_code)]
    Ok(())
}

/// Fetch the device root key from the OS keychain, provisioning a fresh random one ONLY on a
/// genuine first run (`NotFound`).
///
/// CRITICAL: any OTHER keychain failure (vault locked, access denied, Secret Service not up yet at
/// login) is TRANSIENT and must NOT mint a new key — doing so would re-seal, and on the next write
/// OVERWRITE, the snapshot under a key that cannot open the existing one, irreversibly losing the
/// account's identity + message history. So we fail loudly and let the user retry instead.
fn device_key(keychain: &MercuryKeychainAdapter) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    match keychain.root_key() {
        Ok(k) => Ok(k),
        // Genuine first run: no key has ever been provisioned. Mint + store one.
        Err(mercury_store::KeychainError::NotFound) => {
            let mut k = [0u8; 32];
            getrandom::getrandom(&mut k)?;
            keychain.provision_root_key(&k)?;
            Ok(k)
        }
        // Transient backend failure: refuse to start fresh and clobber an existing account.
        Err(e) => Err(format!(
            "cannot read the device key from the OS keychain: {e}. Refusing to start fresh and \
             overwrite your existing Mercury account. Unlock your system keychain / credential \
             store and relaunch Mercury."
        )
        .into()),
    }
}

/// Decode a 64-char lowercase-hex string to 32 bytes; None on any malformation (the baked pin is
/// a compile-time constant, so a None here would be a build defect — we simply skip pinning).
fn decode_hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn set_default_window_icon(app: &tauri::App) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    match tauri::image::Image::from_bytes(WINDOW_ICON) {
        Ok(icon) => {
            if let Err(e) = window.set_icon(icon) {
                eprintln!("mercury-desktop: failed to set window icon: {e}");
            }
        }
        Err(e) => eprintln!("mercury-desktop: failed to load window icon: {e}"),
    }
}

/// Bring the main window back from the tray: show, un-minimize, focus.
fn reveal_main(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn main() {
    tauri::Builder::default()
        // Single-instance MUST be the first plugin. A second launch — including a clicked
        // `mercury://add/<id>` invite while Mercury is already running — is routed here (so we reveal
        // the existing window instead of opening a 2nd copy); with the "deep-link" feature the URL is
        // also delivered to the deep-link open-url listeners (handled in the frontend).
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            reveal_main(app);
        }))
        // Click-to-add: receive `mercury://add/<id>` invite opens (scheme registered via
        // tauri.conf.json at install + register_all() at runtime below).
        .plugin(tauri_plugin_deep_link::init())
        // Auto-update: requires the `plugins.updater` config (endpoints + pubkey) in
        // tauri.conf.json, which is now present. See docs/AUTO-UPDATE.md.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            set_default_window_icon(app);

            // Register the `mercury://` scheme to THIS executable at runtime so click-to-add works
            // on a dev/portable run too (the installer also registers it via tauri.conf.json).
            // Best-effort — a failure just means invite links won't auto-open until the app is
            // installed; pasting the id/link still works regardless.
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Err(e) = app.deep_link().register_all() {
                    eprintln!("mercury-desktop: could not register the mercury:// scheme: {e}");
                }
            }

            let relay =
                std::env::var("MERCURY_RELAY_URL").unwrap_or_else(|_| DEFAULT_RELAY.to_string());

            let data_dir = app.path().app_data_dir().expect("resolve app-data dir");
            std::fs::create_dir_all(&data_dir).ok();
            let snapshot_path = data_dir.join(SNAPSHOT_FILE);

            let keychain = MercuryKeychainAdapter::os(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);
            let key = device_key(&keychain)?;

            // A separate relay handle dedicated to the long-poll wait (`wait_for_delivery`), so the
            // wait never shares — or holds — the controller's lock. Built before the controller
            // because the controller move-consumes `relay` in its fresh-start arm below.
            let wait_transport = RelayTransport::new(relay.clone());

            // Restore the controller from the encrypted snapshot if one exists and opens; else fresh.
            let mut controller = match std::fs::read(&snapshot_path) {
                Ok(blob) => match AppController::from_encrypted_snapshot(
                    RelayTransport::new(relay.clone()),
                    &blob,
                    &key,
                ) {
                    Ok(c) => c,
                    Err(_) => {
                        // The snapshot exists but won't open under the current device key. Do NOT
                        // let the next mutating write clobber it — PRESERVE it (rename aside) so the
                        // account can be recovered if the key becomes readable again, then start
                        // fresh. (With the `device_key` fix above, a transient keychain error never
                        // reaches here — it aborts startup — so this path means genuine corruption
                        // or a real key change.)
                        let aside = snapshot_path.with_extension("unreadable");
                        match std::fs::rename(&snapshot_path, &aside) {
                            Ok(()) => eprintln!(
                                "mercury-desktop: snapshot unreadable under the current key — preserved at {} and starting fresh",
                                aside.display()
                            ),
                            Err(e) => eprintln!(
                                "mercury-desktop: snapshot unreadable and could not be preserved ({e}) — starting fresh"
                            ),
                        }
                        AppController::new(RelayTransport::new(relay.clone()))
                    }
                },
                Err(_) => AppController::new(RelayTransport::new(relay.clone())),
            };

            // Out-of-band VRF pin for the DEFAULT relay: the key ships inside this signed build,
            // so username proofs never trust-on-first-use the relay's own answer. A custom relay
            // keeps TOFU (we cannot know its key); an already-pinned account is never overwritten.
            if relay == DEFAULT_RELAY {
                if let Some(baked) = decode_hex32(DEFAULT_RELAY_VRF_KEY_HEX) {
                    controller.pin_kt_vrf_if_absent(&baked);
                }
                if let Some(baked) = decode_hex32(DEFAULT_RELAY_LOG_KEY_HEX) {
                    controller.pin_kt_log_if_absent(&baked);
                }
            }

            app.manage(Arc::new(AppState {
                controller: Mutex::new(controller),
                key,
                snapshot_path,
                wait_transport,
                relay_url: relay,
            }));

            // Close-to-tray: the window's [X] HIDES it instead of quitting, so Mercury keeps
            // polling the relay and raising notifications in the background. Real quit is the tray
            // menu's "Quit". (First real step toward offline delivery; truly-closed-process push
            // still needs autostart + a background helper — tracked as follow-up.)
            if let Some(win) = app.get_webview_window("main") {
                let hide_target = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = hide_target.hide();
                    }
                });
            }

            // Launched at login with `--minimized` (autostart): start hidden in the tray instead of
            // popping the window open on every boot.
            if std::env::args().any(|a| a == "--minimized") {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }

            // System-tray icon: left-click or "Open Mercury" restores the window; "Quit" exits.
            let show = MenuItem::with_id(app, "show", "Open Mercury", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Mercury", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let tray_icon =
                tauri::image::Image::from_bytes(WINDOW_ICON).expect("decode embedded tray icon");
            TrayIconBuilder::with_id("mercury-tray")
                .icon(tray_icon)
                .tooltip("Mercury")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => reveal_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        reveal_main(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            command,
            wait_for_delivery,
            save_file,
            restore_backup,
            delete_account
        ])
        .build(tauri::generate_context!())
        .expect("error while building the Mercury desktop app")
        .run(|app_handle, event| {
            // Belt-and-suspenders durability: flush the encrypted snapshot as the process exits
            // (tray "Quit" -> app.exit(0), or the event loop ending), so ANY durable mutation not
            // already persisted by the per-command path cannot be lost on quit. Best-effort; the
            // temp+rename in persist_snapshot keeps the previous good snapshot on any write error.
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) {
                if let Some(state) = app_handle.try_state::<Arc<AppState>>() {
                    let app_state = state.inner().clone();
                    if let Ok(ctrl) = app_state.controller.lock() {
                        let _ = persist_snapshot(&app_state, &ctrl);
                    }
                }
            }
        });
}
