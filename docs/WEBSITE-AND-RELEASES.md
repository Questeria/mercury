# Website & releases — hosting the download site (private repo)

`Questeria/mercury` is a **private** repository, which rules out GitHub Pages (private repos can't
publish Pages on the free plan) and GitHub Release downloads (private-repo release assets require a
login). So the public download site is hosted on a **third-party static host** — these instructions
use **Cloudflare Pages**; [Netlify](#netlify-equivalent) works the same way. The repo stays private.

Three pieces, all in this repo:

| Piece | What it is |
|-------|------------|
| `site/` | The static landing page (`index.html` + `style.css`). Pure static files — host them anywhere. |
| `site/download/` | Where the installer + `.sha256` go at deploy time. **Git-ignored** (the binary is never committed). |
| `.github/workflows/release.yml` | On a version tag (or manual run), builds the Windows installer + SHA-256 and uploads them as the **`mercury-windows-installer`** workflow artifact. |

The page's **Download for Windows** button links to the same-origin path
`/download/Mercury-Setup-Windows-x64.exe`, so the installer is served from the very same host as the
page.

---

## Updating the app (current flow — manual website redownload)

For now Mercury updates by **re-downloading the installer from the site**, not via the in-app
auto-updater (that path stays wired but dormant until we host `updates/latest.json`). Each release:

1. **I build + stage it** (done for the current version): the new installer is copied to
   `site/download/Mercury-Setup-Windows-x64.exe` (+ its `.sha256`), and the **Download** button's
   version label and its `?v=<version>` cache-buster in `site/index.html` are bumped, then committed.
2. **You re-upload `site/` to Cloudflare** — the same deploy you did the first time (dashboard
   drag-drop, or `wrangler pages deploy site`). This publishes the new page + the new installer.
3. **Redownload from the website.** Open `https://mercury-messaging.com`, click **Download for
   Windows** — the bumped `?v=` guarantees you get the new bytes, not a cached old copy.
4. **Run the installer** over your existing install. Your identity + chats survive (they live in the
   OS keychain + the encrypted app-data snapshot, not the install folder).

> **Why the `?v=`:** the installer keeps a stable filename, so without a changing URL a browser or
> CDN can hand back the previously-cached download. Bumping `?v=0.1.3 → ?v=0.1.4` each release forces
> a fresh fetch. If a download ever looks stale, hard-refresh (Ctrl-F5) or purge the Cloudflare cache.

---

## One-time operator setup (yours — I can't do these for you)

### 1. Put the domain on Cloudflare
Add `mercury-messaging.com` to a Cloudflare account and point your registrar's nameservers at the
two Cloudflare assigns. (Cloudflare DNS is what makes apex + automatic TLS painless.)

### 2. Get the installer + checksum
Either let CI build it — push a tag and download the artifact:
```sh
git tag v0.1.0 && git push origin v0.1.0     # runs release.yml on a Windows runner
# then: Actions -> the run -> download artifact "mercury-windows-installer"
```
…or build locally (verified working): `cd ui/app && npx tauri build --bundles nsis` plus a
`Get-FileHash` for the `.sha256`. Put **both** files in `site/download/`:
```
site/download/Mercury-Setup-Windows-x64.exe
site/download/Mercury-Setup-Windows-x64.exe.sha256
```
(That folder is git-ignored — the binary lives only in your local copy + the deployed host.)

### 3. Deploy the page + installer to Cloudflare Pages
Direct upload keeps the repo private (no need to grant Cloudflare repo access):
```sh
npm install -g wrangler
wrangler login                                  # or set CLOUDFLARE_API_TOKEN + CLOUDFLARE_ACCOUNT_ID
wrangler pages deploy site --project-name=mercury
```
This uploads everything in `site/` — the page and `site/download/` — to a `*.pages.dev` URL.
(Cloudflare Pages free tier allows files up to 25 MiB; the installer is ~2.4 MiB.)

### 4. Point the apex domain at the Pages project
In the Cloudflare dashboard → your Pages project → **Custom domains** → add
`mercury-messaging.com`. Cloudflare wires the apex (CNAME-flattening) and issues TLS automatically.

> **Don't collide with the relay.** The relay ([DEPLOY-THE-RELAY.md](DEPLOY-THE-RELAY.md)) uses the
> **subdomain** `relay.mercury-messaging.com` → your relay host. The **apex**
> `mercury-messaging.com` goes to Cloudflare Pages. Different records; no conflict.

### 5. Code signing status
The Windows installer is now code-signed by Azure Artifact Signing via GitHub Actions OIDC. The
published `.sha256` remains available for integrity checks.

---

## Netlify equivalent
After placing the installer in `site/download/`:
```sh
npm install -g netlify-cli
netlify deploy --prod --dir=site
```
Then add `mercury-messaging.com` as a custom domain in the Netlify dashboard. Same model: static
`site/` (page + installer) served on the apex, repo stays private.

---

## Honest status of the site

- **Not live until you do steps 1–4.** Committing these files publishes nothing; the page is built
  and verified (serves correctly as a web root) but un-deployed.
- **The download 404s until you place the installer** in `site/download/` and deploy (step 2–3).
- **Windows-only and code-signed.** The current public artifact is signed and ships a matching SHA-256.
- **The page has no "open source" claim or GitHub links** — the repo is private, so those would be
  false / dead for the public. If you later make the code public, tell me and I'll restore the
  open-source framing + source links (you can still host the page on Cloudflare either way).
