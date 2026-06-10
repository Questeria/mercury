# Update channel (auto-update manifest)

> **Status: dormant.** Updates currently ship via **manual website re-download** (the site's Download
> button). No `latest.json` is hosted here yet, so the in-app auto-updater stays silent. This is the
> runbook for the auto-update channel when we switch it on — see
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
4. Re-deploy the site. Installed apps see the new `version` on next launch and **update themselves — no
   reinstall.**

`latest.json` (and any binaries dropped here) are git-ignored — they're generated per-release and
placed at deploy time.
