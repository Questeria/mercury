# Update channel (auto-update manifest)

> **Status: LIVE.** A signed `latest.json` is hosted here and the in-app auto-updater is active — it
> **checks** this endpoint on launch and, when a newer **signed** release is available, prompts the user
> to install it (one click, in-app — no manual reinstall). Manual re-download from the site's Download
> button remains available too. Runbook below — see
> [`../../docs/AUTO-UPDATE.md`](../../docs/AUTO-UPDATE.md).

The desktop app's auto-updater polls **`https://mercury-messaging.com/updates/latest.json`** on
launch. Host `latest.json` **here** (this folder deploys to `/updates/` on Cloudflare). The installer
it points to is served from `/download/Mercury-Setup-Windows-x64.exe` — the same file the website's
Download button uses — so there's a single installer.

## Publish an update
1. Bump `version` in `ui/app/src-tauri/tauri.conf.json`, commit, then tag:
   `git tag v0.1.1 && git push origin v0.1.1`.
2. The release workflow builds a **signed** installer, its `.sig`, and a ready **`latest.json`**
   (requires the `TAURI_SIGNING_PRIVATE_KEY` secrets — see [`../../docs/AUTO-UPDATE.md`](../../docs/AUTO-UPDATE.md)).
3. Download the run's `mercury-windows-installer` artifact and place:
   - `Mercury-Setup-Windows-x64.exe` (+ `.sha256`) → `site/download/`
   - `latest.json` → `site/updates/`
4. Re-deploy the site. Installed apps see the new `version` on next launch and **prompt the user to
   install it (one click; no manual reinstall).**

`latest.json` (and any binaries dropped here) are git-ignored — they're generated per-release and
placed at deploy time.
