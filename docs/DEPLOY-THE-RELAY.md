# Deploy the Mercury relay (≈10-minute runbook)

> **The production relay is already live on Fly.io at `relay.mercury-messaging.com`.** This runbook is
> for **redeploying it** or **self-hosting your own**. Two supported paths: **(A) Fly.io** — what the
> production relay actually runs (config: [`deploy/fly.toml`](../deploy/fly.toml)) — and **(B) self-host
> with Docker + Caddy** on your own Linux host. Pick one; both end at the same `/kt/sth` liveness check.

This stands up the only internet-facing piece of Mercury — the **relay** — with automatic HTTPS,
so the desktop app (which defaults to `https://relay.mercury-messaging.com`) has something to talk
to. The Fly.io config is in [`deploy/fly.toml`](../deploy/fly.toml); the self-host kit is in
[`deploy/`](../deploy): a multi-stage `Dockerfile`, a `docker-compose.yml`, and a `Caddyfile`.

**What the relay is:** a dumb opaque-ciphertext router. It stores and forwards already-end-to-end-
encrypted bytes plus content-free metadata, and delegates every admission decision to the
`mercury-core` gates. **It never sees plaintext or keys** (full security model + honest residuals:
[RELAY-DEPLOYMENT.md](RELAY-DEPLOYMENT.md)).

## Path A — deploy on Fly.io (production)

This is what `relay.mercury-messaging.com` actually runs. Install the `flyctl` CLI, then run **every
command from the repo root** (`C:\Projects\Mercury`) so the Docker build context is the repo root and
the Dockerfile's `COPY core/rust` paths resolve:

```sh
fly auth login
fly apps create mercury-relay                                 # pick another name if taken
fly volume create relay_data -r iad -s 1 -a mercury-relay     # 1 GB durable queue (same region)
fly secrets set MERCURY_KT_VRF_SEED=$(openssl rand -hex 32) -a mercury-relay   # stable KT seed (fail-closed)
fly deploy -c deploy/fly.toml
fly certs add relay.mercury-messaging.com -a mercury-relay    # then add the DNS record it prints
```

Fly fronts the relay with HTTPS on 443 (`force_https`, `internal_port = 8787`), keeps ≥1 machine
running (never scale the relay to zero), and mounts a 1 GB volume at `/data` for the durable redb
queue. Verify: `curl -fsS https://relay.mercury-messaging.com/kt/sth` → HTTP 200 + a signed tree head
(see step 4 below). Redeploy after changes with `fly deploy -c deploy/fly.toml`.

---

## Path B — self-host with Docker + Caddy

The kit in [`deploy/`](../deploy) runs the relay behind Caddy (auto-TLS) on your own Linux host.

## Topology

```
  desktop app ──HTTPS──▶ Caddy (:443, auto-TLS) ──HTTP──▶ relay (:8787, private)
                                                              └─▶ /data/relay.redb (durable queue)
```

Caddy is the only thing published to the host. The relay binds plain HTTP on the private compose
network and is never exposed directly — exactly the "TLS-terminating reverse proxy in front of a
privately-bound relay" model the relay is designed for.

## Prerequisites

- A Linux host with **Docker** + the **Docker Compose** plugin, with **ports 80 and 443** open to the internet.
- Control of DNS for **mercury-messaging.com**.
- This repository checked out on the host.

## Steps

### 1. Generate a stable KT VRF seed

The Key-Transparency directory needs a stable VRF key, or its identity resets on every restart. The
compose file **refuses to start** without one (fail-closed).

```sh
cd deploy
cp .env.example .env
# set a freshly generated 64-hex seed in the .env you just created:
sed -i "s/^MERCURY_KT_VRF_SEED=.*/MERCURY_KT_VRF_SEED=$(openssl rand -hex 32)/" .env
```

Keep `.env` secret (it is git-ignored) and keep the seed stable across redeploys.

### 2. Point DNS at the host

Create a DNS record so the relay name resolves to this host's public IP:

```
relay.mercury-messaging.com.   A    <this host's public IPv4>
# (and/or AAAA for IPv6)
```

Wait for it to propagate (`dig +short relay.mercury-messaging.com` should return your IP). Caddy
cannot obtain a certificate until this resolves and ports 80/443 are reachable.

### 3. Bring it up

```sh
cd deploy
docker compose up -d --build
```

The first build compiles the relay's Rust dependency tree from scratch (several minutes). When it
finishes, Caddy automatically requests a Let's Encrypt certificate for `relay.mercury-messaging.com`.

### 4. Verify it's live over TLS

There is no dedicated `/health` route; the unauthenticated **`GET /kt/sth`** (Key-Transparency
signed tree head) is the liveness probe — a fresh directory returns a valid epoch-0 head:

```sh
curl -fsS https://relay.mercury-messaging.com/kt/sth
# -> {"tree_size":0,"root_hash":"…","timestamp_s":…,"log_signature":"…"}   (HTTP 200)
```

A 200 with that JSON means: DNS resolves, Caddy's TLS works, and the relay is serving. The desktop
app — which already defaults to `https://relay.mercury-messaging.com` — now has a working backend.

## Operating notes

- **Logs / status:** `docker compose logs -f relay` · `docker compose ps`.
- **Durable queue:** queued ciphertext lives in the `relay-data` volume (`/data/relay.redb`). Back
  up that volume to preserve undelivered messages across host moves.
- **Certificates:** Caddy stores them in the `caddy-data` volume — keep it to avoid re-issuing on
  every recreate (and to stay under Let's Encrypt rate limits).
- **Deploy elsewhere:** to use a different hostname, change the site name in `deploy/Caddyfile` and
  set `DEFAULT_RELAY` in `ui/app/src-tauri/src/main.rs` (or build the client with `MERCURY_RELAY_URL`).

## Honest residuals (carried from RELAY-DEPLOYMENT.md — not hidden)

- **Rate limiting behind the proxy.** Resolved for this single-trusted-proxy deployment: the relay
  keys its per-IP flood limiter on the direct peer by default, but `docker-compose.yml` sets
  `MERCURY_TRUSTED_PROXY=1` so it instead keys on the right-most `X-Forwarded-For` hop (the address
  Caddy observed for the client and appended — un-forgeable behind a sole front door). Per-client
  flood control therefore works behind Caddy. For large-scale internet exposure, still add a CDN/LB
  with DDoS protection. (Only enable `MERCURY_TRUSTED_PROXY` when a trusted proxy actually fronts the
  relay — without one, a client could spoof the header.)
- **Real-time delivery while running; no wake of a fully-quit process.** The server uses
  `InProcessWaker`: a long-poll `GET /relay/wait` is released the instant a message is enqueued, so a
  *running* client (foreground or tray) receives in real time. The relay cannot wake a client that
  is not running — a fully-**Quit** app's messages wait in the durable queue until it relaunches.
  True closed-process OS push (APNs/FCM/WNS) is a documented future over the same `PushSender` seam.
- **Queue not encrypted at rest.** `relay.redb` holds already-E2E-encrypted opaque ciphertext +
  content-free metadata; at-rest protection is the operator's disk/volume encryption, by design.
- **Single instance.** This kit runs one relay; horizontal scaling (shared store, multiple relays)
  is future work.
