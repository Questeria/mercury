# In-app auto-update (no reinstall)

> **Current status (2026-06-05): deferred — updates ship via manual website re-download.**
> To update today, users download the new installer from `mercury-messaging.com` and reinstall
> (see [WEBSITE-AND-RELEASES.md](WEBSITE-AND-RELEASES.md) → "Updating the app"). The in-app
> auto-updater described below is fully wired (public key embedded, plugin on) but **dormant**: no
> `latest.json` is hosted, so the launch check 404s and stays silent. This doc is the runbook for
> switching auto-update on when we're ready.

The desktop app ships with Tauri's updater **active**: your public key is embedded, the plugin is on,
and on launch it polls `https://mercury-messaging.com/updates/latest.json` and — if a newer version is
published — downloads + installs it (applied next launch), **no manual reinstall**. Two operator pieces
remain to make updates actually flow: the **CI signing secrets** (so releases are signed) and
**hosting `latest.json`** at that endpoint. Until a manifest is hosted, the check 404s and stays silent.

> One-time caveat: auto-update only works *forward* from a build that already contains the updater — so
> the current build (and the next install) is manual; every update after is seamless.

## CI publish pipeline (wired)
`.github/workflows/release.yml` already builds the installer, **signs the update bundle** when the
signing secrets are present, computes the SHA-256, and **generates `latest.json`** — all in the
`mercury-windows-installer` artifact. Switch on signing by adding two repo secrets (Settings → Secrets
and variables → Actions):

- `TAURI_SIGNING_PRIVATE_KEY` — the contents of your `mercury-update.key` (the private key from
  `tauri signer generate`).
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — the password you set for it.

**Publish flow:** bump `version` in `tauri.conf.json` → `git tag vX.Y.Z && git push origin vX.Y.Z` →
download the run's artifact → put `Mercury-Setup-Windows-x64.exe` (+ `.sha256`) in `site/download/` and
`latest.json` in `site/updates/` → re-deploy the site. Installed apps self-update on next launch.

> Authenticode caveat: if you also enable Windows code signing (Azure Trusted Signing), integrate it
> into the tauri build so the exe is Authenticode-signed *before* the update `.sig` is computed — a
> post-build re-sign of the exe would invalidate the `.sig`. Don't run both as separate post-build steps.

The manual/local steps below remain valid as reference (the keypair + `tauri.conf.json` config are
already done in this repo).

## Activate it (operator steps)

### 1. Generate an update-signing keypair
```sh
npx tauri signer generate -w mercury-update.key
```
This prints a **public key** and writes a **private key** (with a password you set). **Keep the private
key secret** — it signs your update bundles. (This is separate from Authenticode code signing.)

### 2. Configure the updater in `ui/app/src-tauri/tauri.conf.json`
Add a `plugins.updater` block with your public key + the manifest URL:
```json
"plugins": {
  "updater": {
    "endpoints": ["https://mercury-messaging.com/updates/latest.json"],
    "pubkey": "<paste the public key from step 1>"
  }
}
```
Also add the updater capability — already done (`updater:default` in `capabilities/default.json`).

### 2b. Turn the plugin on
In `ui/app/src-tauri/src/main.rs`, uncomment the registration line — it is **off by default** because
the plugin panics at startup if the `plugins.updater` config above is missing:
```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

### 3. Build signed update artifacts
Set the signing key as env vars, then build:
```sh
# PowerShell:
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content mercury-update.key -Raw)
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<your password>"
cd ui/app; npx tauri build
```
This produces the installer **plus** a detached signature (`*.sig`) for the update bundle. (In CI, add
those two env vars as secrets and the release workflow does it automatically.)

### 4. Host the update manifest + bundle
Put `latest.json` where your `endpoints` URL points (`/updates/` on your Cloudflare site) and the
installer at the `url` it references (`/download/Mercury-Setup-Windows-x64.exe` — the same stable file
the website's Download button serves, so there's a single installer):
```json
{
  "version": "0.1.4",
  "notes": "What changed",
  "pub_date": "2026-06-05T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<contents of the .sig file>",
      "url": "https://mercury-messaging.com/download/Mercury-Setup-Windows-x64.exe"
    }
  }
}
```

Done — installed apps now self-update on launch when you publish a newer `version` in `latest.json`.

## How it behaves now (unconfigured)
`checkForUpdates()` (in `ui/app/src/mercury/updater.ts`) runs on startup only inside the desktop app,
catches any "not configured / offline / no update" error, and stays silent. No user impact until you
complete the steps above.
