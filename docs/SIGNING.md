# Signing the Windows installer

> **STATUS: LIVE (2026-06-06).** Windows installers are Authenticode code-signed via Azure Artifact
> Signing. The current public installer at `mercury-messaging.com/download/Mercury-Setup-Windows-x64.exe`
> verifies **Status=Valid**, publisher **CN=Anthony DeMarco**, issued by **Microsoft ID Verified CS**,
> Microsoft-timestamped. Identity validation is complete, the certificate profile exists, and
> `TRUSTED_SIGNING_PROFILE` is set in GitHub Actions — so tagged releases sign automatically.

Code signing attaches your **verified identity** to the installer. Without it, Windows shows a
SmartScreen *"unknown publisher / Windows protected your PC"* warning on first run (and many people
bail, assuming malware). With it, the warning goes away, the installer shows **your name as publisher**,
and it's tamper-evident. It does **not** change the app's encryption — it's about *trust at install
time*.

## When you actually need it

- **Not needed for:** testing it yourself · a few trusted people you hand it to personally · showing
  investors the **code/repo** (signing is about the *installer*, not the source) · a demo **you** drive
  on your own machine.
- **Needed before:** a **public download push** · **self-serve installs** by people you can't walk
  through the warning · a demo where an **investor installs it themselves**.
- **Lead-time catch:** setup includes a **multi-day identity validation**, so **start ~1–2 weeks before**
  any public/self-install moment. Not urgent today.

## How the CI signs

`.github/workflows/release.yml` signs the installer with **Azure Artifact Signing** *before* the SHA-256
is computed. Auth uses GitHub OIDC through the `mercury-signing-ci` app registration, so Mercury needs
**no** long-lived `AZURE_CLIENT_SECRET`. The signing step is gated on the `TRUSTED_SIGNING_PROFILE` repo
secret (now set); if it were ever absent the build still succeeds, unsigned.

Because Authenticode signing rewrites the installer bytes, the workflow then **re-generates the Tauri
updater `.sig` over the final signed installer** and only then writes `latest.json` — so auto-update stays
valid on signed builds. (Previously the pre-signing `.sig` was dropped and the manifest skipped; that is
fixed.)

## Cutting a signed release

1. Bump `version` in `ui/app/src-tauri/tauri.conf.json` (and `src-tauri/Cargo.toml`).
2. `git tag vX.Y.Z && git push origin vX.Y.Z` — the workflow builds, **Authenticode-signs**, **re-signs the
   updater `.sig` over the signed bytes**, computes the SHA-256, and writes `latest.json`, all in the
   **`mercury-windows-installer`** artifact (four files).
3. Download the artifact and place: `Mercury-Setup-Windows-x64.exe` + `.exe.sha256` → `site/download/`,
   and `latest.json` → `site/updates/`. Bump the `?v=` cache-buster on the site's Download button.
4. Re-upload `site/` to Cloudflare. The signed installer is now the public download (SmartScreen warning
   gone), and installed apps can see the new `latest.json`.

## Alternative: sign locally with signtool

If you'd rather not use CI: install the Trusted Signing client tools, make a `metadata.json`
(`{"Endpoint": "...", "CodeSigningAccountName": "...", "CertificateProfileName": "..."}`), then
`signtool sign /v /fd SHA256 /tr http://timestamp.acs.microsoft.com /td SHA256 /dlib <Azure.CodeSigning.Dlib.dll> /dmdf metadata.json site\download\Mercury-Setup-Windows-x64.exe`,
recompute the `.sha256`, and re-upload.
