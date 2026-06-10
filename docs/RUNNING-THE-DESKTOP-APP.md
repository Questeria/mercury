# Running & packaging the Mercury desktop app

The desktop client (`ui/app/src-tauri`, crate `mercury-desktop`) is a **Tauri v2** shell around the
same React UI as the browser build. The difference is what sits behind the UI:

- **Browser dev demo** → the UI talks over HTTP to a `mercury-app-server` shim that holds the keys.
- **Desktop app** → the UI talks over **in-process IPC** to the real
  `mercury_app::AppController<RelayTransport>` living inside the native process. There is **no open
  socket**; keys and ciphertext never leave the process, and the webview only ever sees the
  plaintext you type or receive.

Both paths drive the identical `AppController::handle_command(json) -> json` surface — the desktop
shell adds no crypto or trust logic of its own. State (identity, device keys, ratchet sessions,
contacts, chat log) is sealed to disk after every message with XChaCha20-Poly1305 under a device
key kept in the **OS keychain**, so it survives a restart.

---

## Prerequisites

| Tool | Version used | Notes |
|------|--------------|-------|
| Rust | stable (1.92) | `rustup` |
| Node.js | v22 | for the frontend build the shell embeds |
| Tauri CLI | v2 (`@tauri-apps/cli`) | already in `ui/app` devDependencies — invoke with `npx tauri` |
| Platform webview | WebView2 (Windows) · WebKitGTK (Linux) · WKWebView (macOS) | Windows 10/11 ship WebView2; on Linux install `webkit2gtk` |

The `mercury-desktop` crate is intentionally **excluded from the Cargo workspace** (see the root
`Cargo.toml`), so the headless `cargo test --workspace` and Linux CI never have to build the webview
toolchain. It builds standalone on a developer machine.

## Run it in development

```sh
cd ui/app
npm install          # first time only
npx tauri dev        # compiles the Rust shell, opens the native window, hot-reloads the UI
```

By default the app points at the production relay `https://relay.mercury-messaging.com`. To run
fully locally against your own relay, start a relay (see
**[RUNNING-THE-LIVE-DEMO.md](RUNNING-THE-LIVE-DEMO.md)** §1 for `mercury-relay-server` on
`127.0.0.1:8787`) and override the endpoint:

```sh
# Windows (PowerShell):  $env:MERCURY_RELAY_URL = "http://127.0.0.1:8787"
# macOS/Linux:           export MERCURY_RELAY_URL=http://127.0.0.1:8787
cd ui/app && npx tauri dev
```

## Build an installer

```sh
cd ui/app
npx tauri build                 # all installers your OS can produce
# or pick one bundle explicitly:
npx tauri build --bundles nsis  # Windows .exe installer (NSIS is auto-downloaded by Tauri)
```

`npx tauri build` runs the frontend build, compiles the shell in release, and packages a native
installer per platform:

| Platform | Bundles | Extra toolchain |
|----------|---------|-----------------|
| Windows | NSIS `.exe`, MSI `.msi` | NSIS is auto-fetched by Tauri; **MSI** additionally needs the WiX toolset installed |
| macOS | `.app`, `.dmg` | Xcode command-line tools |
| Linux | `.deb`, `.AppImage`, `.rpm` | the matching packaging tools (`dpkg`, etc.) |

The artifacts land under `ui/app/src-tauri/target/release/bundle/` (git-ignored).

## Where the app keeps state

| What | Location |
|------|----------|
| Encrypted snapshot | `<app-data-dir>/mercury-snapshot.bin` (Windows `%APPDATA%\com.mercury.messaging\`) |
| Device root key | OS keychain — service `com.mercury.messaging`, account `device-root` (Windows Credential Manager · macOS Keychain · Linux Secret Service) |

The snapshot is ciphertext; the only thing that can open it is the keychain key. Deleting the
keychain entry makes the snapshot unreadable, and the app starts fresh (fail-closed — it never falls
back to plaintext or a default key). Snapshot writes are atomic (temp-file + rename), so a crash
mid-write cannot corrupt or reset an existing account.

---

## What is the operator's job, not the app's (honest boundaries)

The desktop app **compiles and packages** from this repo today. Making it usable by other people
involves steps that are deliberately **outside** what this code does or claims:

1. **Deploy a relay.** The app needs a reachable relay. Stand one up with the kit in
   **[DEPLOY-THE-RELAY.md](DEPLOY-THE-RELAY.md)** (`docker compose up`). The relay only routes
   opaque ciphertext — it never sees plaintext or keys.
2. **Point the DNS.** The app defaults to `https://relay.mercury-messaging.com`. The owner of
   `mercury-messaging.com` must create a DNS record for `relay.` → the relay host, and the relay
   must terminate TLS for that name (the Caddy config in the deploy kit does this automatically).
   To point a build elsewhere, set `MERCURY_RELAY_URL` or change `DEFAULT_RELAY` in
   `src-tauri/src/main.rs`.
3. **Sign the installer.** A bare local `npx tauri build` produces an **unsigned** installer, but the
   **public Windows release is already Authenticode code-signed** in CI (Azure Trusted Signing;
   publisher *Anthony DeMarco* — see [SIGNING.md](SIGNING.md)), so the download from
   `mercury-messaging.com` verifies Status=Valid. macOS Developer ID + notarization remains the
   operator's step once a Mac build is added. Signing requires paid certificates + a verified
   identity — this repo never fakes it.

## Background delivery & its honest residual

The desktop app now delivers in the background while it's **running** — which, by default, it stays:

- **Close-to-tray:** the window's `[X]` hides Mercury to the system tray instead of quitting, so it
  keeps a live connection to the relay and receives in **real time** in the background. Real quit is
  the tray menu's **Quit**.
- **Launch at login (opt-in):** toggle it on in the **Updates** panel and the OS starts Mercury
  minimized in the tray at sign-in — so it's running (and receiving) after a reboot without you
  opening it.
- **Notifications:** a native OS notification fires on a new message when the window isn't focused
  (sender id + count only — **never plaintext**).

Delivery while Mercury is **running** (foreground or tray) is real-time: the client long-polls the
relay's `GET /relay/wait`, which the relay releases the instant a message is enqueued (an in-process
waker), so messages arrive with no fixed-poll lag and far less traffic. For the intended audience it
works without keeping a window open.

The remaining **true** limitation: there is still **no wake of a fully-quit process**. The relay can
only wake a client that is *running* to hold the long-poll; it cannot start a process the user has
**Quit** (with launch-at-login off). In that case inbound messages wait in the relay's encrypted
durable queue (set `MERCURY_RELAY_DB`) and arrive the next time Mercury launches. True
closed-process OS push (Windows WNS; APNs/FCM on mobile) is future work and is the path to mobile.
