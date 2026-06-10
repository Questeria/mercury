# Mercury Relay — Deployment Model & Honest Residuals

**Scope:** how the `mercury-relay` server is meant to be run, which security properties it
provides *in code*, and which are **deployment seams** (handled by surrounding infrastructure,
not faked inside the relay). Every claim below is backed by the cited source; nothing here
describes a capability the code does not actually have.

The relay is a **dumb ciphertext router**: it stores and forwards opaque, already-end-to-end-
encrypted bytes plus content-free metadata, and delegates every admission decision to the
authoritative `mercury-core` gates (`core/rust/mercury-relay/src/lib.rs`). It never sees
plaintext.

---

## What the relay enforces in code (Milestone 4)

- **Request body limits** — per-route caps so one request can't make the relay buffer without
  bound: `/directory/publish` 64 KiB, `/relay/submit` 4 MiB
  (`core/rust/mercury-relay/src/http.rs`, `SUBMIT_BODY_LIMIT` / `PUBLISH_BODY_LIMIT`).
- **Route-ownership authentication** on `poll` / `ack` / `delete` — the caller must present an
  Ed25519 proof that it holds the identity key whose account id *is* the route, with a both-
  sided freshness window. This replaced the previously forgeable client-set header booleans
  (`http.rs` `auth_layer` / `route_ownership_proven`; `mercury_keys::verify_relay_route_pop`).
  `submit` is intentionally **open** (a sender is anonymous under sealed-sender).
- **Per-IP flood rate limiting** on the two open endpoints (`/relay/submit`,
  `/directory/publish`) — a fixed-window cap keyed by peer IP, with a bounded tracking table
  (`core/rust/mercury-relay/src/rate.rs`; wired in `http.rs` `rate_limit_layer`).
- **Optional durable storage** — queued items + replay tombstones survive a restart when
  `MERCURY_RELAY_DB` names a redb file; otherwise the default is in-memory
  (`core/rust/mercury-relay/src/redb_store.rs`; selected in
  `src/bin/mercury-relay-server.rs`).

### Environment variables

| Variable | Effect | Default |
|---|---|---|
| `MERCURY_RELAY_ADDR` | bind address | `127.0.0.1:8787` |
| `MERCURY_RELAY_DB` | path to a redb file → durable queue store | unset → in-memory |
| `MERCURY_KT_VRF_SEED` | 64 hex chars → stable KT directory VRF key | unset → ephemeral (dev) |

---

## Deployment seams (NOT implemented in the relay — handled by infrastructure)

These are honestly **out of scope for the relay binary**. They are not faked; they are the
operator's responsibility, with a clear interface where one exists.

### TLS — terminated by a reverse proxy (the relay serves plain HTTP)

The relay binary serves **plain HTTP/1.1** (`axum::serve` in
`src/bin/mercury-relay-server.rs`; there is no `rustls` / TLS code anywhere in
`core/rust/mercury-relay/src`). The intended production topology is a **TLS-terminating reverse
proxy or load balancer** (nginx / Caddy / a cloud LB) in front of the relay, with the relay
bound to a private interface. A direct in-process `rustls` / `axum-server` listener is a
possible alternative seam but is **not implemented** — do not assume the relay speaks TLS on
its own.

**Consequence for rate limiting behind a proxy (read this):** the per-IP limiter keys on the
*direct* peer IP via `ConnectInfo<SocketAddr>` (`http.rs` `client_key`). Behind a reverse
proxy that peer IP is the **proxy's** IP, so *all* clients would share one rate-limit bucket.
Production must therefore either (a) enforce per-client rate limiting **at the proxy**, or
(b) extend `client_key` to read a trusted forwarded-client header (e.g. `X-Forwarded-For`)
set by the proxy — a small follow-on. The in-relay limiter as shipped is a single-source flood
bound for the **direct-connection** (no-proxy) case.

### Recipient wake-ups — the `PushSender` seam

Recipient wake-ups go through the `PushSender` trait (`core/rust/mercury-relay/src/push.rs`). The
shipped server (`mercury-relay-server`) uses `InProcessWaker`: it wakes a long-poll
`GET /relay/wait/{route}` the instant a message is enqueued, so a **running** client (foreground or
tray) receives in real time rather than on a fixed poll interval. This is single-instance (the wake
is in-process); a multi-instance deployment needs shared pub/sub, and waking a **fully-quit** process
(true closed-process OS push) implements the same seam over **APNs / FCM / WNS** — external services
requiring credentials and infrastructure that is **out of scope for this repository**; it is not
faked. Every wake is content-free by construction (only the opaque `route_id`; the
`evaluate_authenticated_relay_source` gate independently forbids any plaintext preview).

---

## Honest residual list (Milestone 4 and adjacent)

Carried forward verbatim from the per-increment audits; none is hidden.

**Authentication (poll/ack/delete):**
- *Same-operation in-window replay.* The route-ownership proof binds `(route_id, operation,
  timestamp)`, so a captured proof can no longer be replayed ACROSS operations — a poll proof
  cannot drive an `ack`/`delete`. What remains is SAME-operation replay within the ±30 s window
  (e.g. a captured poll proof re-presented as another poll, racing the genuine deliver-once
  drain); same-operation `ack`/`delete` replays are idempotent no-ops. Never an escalation beyond
  what the route owner could do to their own queue, and in production the proof travels inside TLS
  so it is not on the wire to capture. A server-issued, single-use nonce challenge-response closes
  the residual fully (`http.rs` `auth_layer` doc).

**Rate limiting (submit/publish):**
- *Distributed flood.* The per-IP limiter bounds a single source; a flood from many source IPs
  needs upstream (CDN / load-balancer) DDoS protection.
- *Behind a proxy.* Keys on the direct peer IP (the proxy's IP behind a proxy) — see the TLS
  section above; production keys on a forwarded-client header at the proxy.
- *Capacity fail-closed.* When the bounded tracking table is full of active windows, new source
  keys are denied (fail-closed, to cap the limiter's own memory) rather than tracked.

**Durable storage (when enabled):**
- *Panic on write.* The `QueueStore` trait is infallible; a redb write failure (full/corrupt
  disk) panics the current request rather than silently losing a message. Reads fail closed.
  A fallible trait end-to-end is the clean follow-on (`redb_store.rs` doc).
- *Synchronous I/O under the async mutex.* The sync store runs behind `Arc<Mutex<dyn
  QueueStore>>` in async handlers, so redb disk I/O briefly blocks a tokio worker per queue op
  — acceptable at single-instance relay scale; `spawn_blocking` / a dedicated DB thread is the
  scaling follow-on.
- *Not encrypted at rest.* The redb file is **not** itself encrypted — by design: the queue
  holds already-E2E-encrypted opaque ciphertext + content-free metadata, so at-rest protection
  is the operator's disk/volume encryption (this is NOT the client's keychain-encrypted
  `mercury-store`).

**Adjacent (other milestones):**
- *UI visual verification (M3).* The application controller (`mercury-app`), its JSON command
  surface, the dev HTTP shim (`mercury-app-server`), and the typed React messaging binding
  (`ui/app/src/mercury/messaging.ts`) are built and tested headlessly, but wiring the chat
  view onto the real backend and confirming two browser windows chat end-to-end requires a
  human (recipe in the `messaging.ts` header). **Update (2026-06-05):** the Tauri desktop client now
  drives the real `AppController<RelayTransport>` over in-process IPC end-to-end; this residual
  referred to the browser dev shim.
- *Directory transparency (M2).* The prekey directory provides trustless publish (proof of
  possession) + client-side re-verification; key-transparency *inclusion-proof binding* (to
  detect a relay serving a stale/withheld card) and classical MLS-group discovery are deferred,
  documented residuals.
- *Classical sealed-sender (from M1).* The outer sealed-sender envelope is classical X25519; the
  message **content** is already post-quantum via the inner ratchet. A post-quantum sealed
  sender is a later refinement.

---

## CI note

The CI **Policy** job (Rust workspace tests, proptests, Python vector checkers) is green. The
**Helix** job fails only because its private `Questeria/helix` git submodule cannot be cloned by
the default `GITHUB_TOKEN`; add a `HELIX_CHECKOUT_TOKEN` repo secret with read access to that
repo (the checkout step is already wired for it in `.github/workflows/ci.yml`). This is an
infrastructure-access gap, unrelated to the relay code.
