# Mercury Launch-Readiness Integration Audit

**Date:** 2026-06-01
**Method:** read-only evidence survey across the relay, FFI bridge, UI, core orchestration, and the end-to-end crypto/identity/store/KT path (five independent mapping passes, cross-checked).
**Scope:** "what is wired into a live message flow today vs. what is built-but-unconsumed," to make the launch plan exact before writing integration code.

> **⚠️ SUPERSEDED (2026-06-05).** This is the pre-integration baseline. The launch plan below has since
> been executed: a working desktop client (`mercury-desktop` → `mercury-app` → `mercury-client`) now
> sends real end-to-end 1:1 messages through the Fly.io-deployed relay, and the relay has
> route-ownership auth + TLS + durable redb storage. Read the "MISSING / SIMULATED / no client"
> findings below as the 2026-06-01 snapshot, **not** current state.

---

## Headline finding

**Every subsystem works in isolation; none are wired into a live message flow.** Mercury is a
**policy brain + a set of unconsumed crypto islands + an opaque relay + a decision-view UI**, with
no integration glue tying them into "send a message to a user." The *only* place a real message is
sealed → crosses the relay → is opened is a **single in-process integration test**,
`core/rust/mercury-relay/tests/spine_roundtrip.rs` — which is therefore the **blueprint** for the
launch path.

Structural proof (dependency graph): the only shipping binaries — `mercury-relay`,
`mercury-ui-bridge`, `mercury-bindings` — depend at **runtime** only on `mercury-core` (+ `mercury-kt`
for the relay). They pull `mercury-message`/`mercury-mls`/`mercury-sealedbox`/`mercury-media` in as
**`[dev-dependencies]`** (test-only). **`mercury-session` and `mercury-store` are imported by no other
crate at all.**

---

## Integration map (wired / partial / missing, with evidence)

### Relay server — `mercury-relay`  →  REAL & runnable (security stubbed)
- **WIRED:** axum router with `POST /relay/submit`, `GET /relay/poll/{route_id}`, `POST /relay/ack/{route_id}`,
  `DELETE /relay/queue/{route_id}` + KT proof routes; real binary binds `127.0.0.1:8787`
  (`src/bin/mercury-relay-server.rs:32-81`, `src/http.rs:61-72`). Submit/poll/ack run the **real**
  `mercury-core` admission gates (`evaluate_relay_submission`/`_queue`/`evaluate_delivery_ack`); poll
  has deliver-once semantics (takes the payload). Carries **opaque ciphertext only** — the crypto crates
  are dev-deps; the binary cannot open ciphertext. Replay protection + expiry sweeper are real.
  Run: `cargo run -p mercury-relay --bin mercury-relay-server` (env `MERCURY_RELAY_ADDR`, `MERCURY_KT_VRF_SEED`).
- **STUBBED (launch blockers):**
  - **Forgeable auth** — the auth gate's `server_authenticated`/`route_key_authenticated`/`replay_window_valid`
    booleans are read straight from client-settable HTTP headers (`src/http.rs:477-479`). Any client can
    send `x-mercury-server-authenticated: true` and pass. Submit has **no auth at all**.
  - **No TLS** (plain TCP), **no rate-limit / body-limit** on the open `/submit` (flood / storage-exhaustion vector).
  - **In-memory storage only** (`src/store.rs:8-11`) — all queued messages + replay tombstones lost on restart.
  - **Push is a no-op** (`src/push.rs:19-27`); KT directory starts empty (no publish pipeline).

### FFI bridge — `mercury-bindings` / `mercury-ffi` / `mercury-ui-bridge`  →  policy-only, NO messaging crypto
- **WIRED:** real C-ABI (`mercury_ffi_handle_bridge_request`, ABI v1, C header) and a dev HTTP shim,
  exposing **policy decisions / store-policy / audit / fixtures** (≈292 prototype + 259 backend-command
  fixtures + 4 policy-view constructors).
- **MISSING:** **zero** live messaging crypto. No `encrypt`/`decrypt`/`establish_session`/`send`/`receive`
  is exported; the FFI *forbids* plaintext payloads (`PlaintextPayloadForbidden`, bindings `lib.rs:4302-4312`).
  None of the bridge crates depend on `mercury-session`. The session engine is bridged nowhere.

### UI — `ui/app` (React 19 + Vite + TS)  →  polished shell, fully simulated
- **WIRED:** chat-style UI shell (rooms, thread, composer, inspector); a clean `MercuryBinding` swap seam;
  an HTTP binding that fetches **decision-view fixtures** from `mercury-ui-bridge`.
- **SIMULATED:** every message is hard-coded in a TS simulator (`src/mercury/simulator.ts:170-280`);
  "send/receive" are `setTimeout`/`setInterval` fakes (`useMercuryThread.ts:75-193`); default mode is
  `"sim"`; the production FFI binding **throws** ("not wired yet", `binding.ts:81-95`). No crypto, keys,
  storage, socket, or Tauri in the UI. The binding contract is 3 decision-shaped methods — too narrow to
  carry messages.

### Core — `mercury-core`  →  policy/state engine only (no transport execution)
- Does **not** depend on `mercury-session`/`-sealedbox`/`-mls`/`-keys`; contains zero references to them.
- `Client*` types carry **lengths/states → permission verdicts** (`can_send`, `can_decrypt`), never encrypt.
- `Prototype*Session` types are **simulation harnesses** (in-memory/local-file fakes, XOR test-crypto)
  that compose the gates to test the decision flow — not a runnable client.
- The one real crypto arm is **at-rest local-store sealing** (`MercuryLocalStoreV1CryptoProvider`,
  XChaCha20-Poly1305, `lib.rs:10533`) — *not* transport/message crypto. No `(plaintext, recipient) → ciphertext`
  exists anywhere in core.

### End-to-end crypto / identity / store / KT
- **1:1 path — MISSING:** `mercury-session` (`HybridRatchetSession`, `establish_*`, `encrypt`/`decrypt`)
  is consumed by nobody outside its own tests + decode fuzzers.
- **Sealed-message path — PARTIAL (test-only):** proven once in `spine_roundtrip.rs` — a message sealed
  client-side, submitted to the relay as opaque hex, polled back, opened — all in one test process.
- **Group path — PARTIAL (test-only):** same single test; the MLS Welcome rides the generic opaque queue;
  there is no `/mls/welcome` route; the "welcome outbox" is a policy *reason code*, not infra.
- **Identity / prekey distribution — PARTIAL data, MISSING transport:** key types + `MercurySession::publish_bundle`
  exist, but `publish_bundle` **mints in-memory only** (`mercury-session/src/lib.rs:254`); there is **no HTTP
  client anywhere in the Rust core** and **no bundle publish/fetch endpoint** on the relay. Registration is absent.
- **Persistence — MISSING in runtime:** `mercury-store` is a real redb-backed encrypted-at-rest store but is
  imported by no one (a *dead* dependency of `mercury-session`); session pickles are never persisted.
- **Key transparency — PARTIAL:** proof-serving wired into the relay + tested, but the directory starts empty
  (no registration pipeline) and no client fetches + verifies a peer bundle against it.

---

## The launch plan (grounded in the above)

**Milestone 1 — the thin slice (lift `spine_roundtrip.rs` into real processes).**
Create a **`mercury-client` runtime crate** + a **`mercury-cli` binary** that does, over the real HTTP relay,
what the spine test does in-process — giving `mercury-session`/`mercury-store`/`mercury-message` their first
real consumers. Components: identity (`mercury-keys`), 1:1 sessions (`mercury-session`
`HybridRatchetSession`), transport envelope (`mercury-message` seal/open), an HTTP client to the relay
(`/relay/submit` + `/relay/poll` — **no HTTP client exists in the core yet; this is a known add**), policy
gating via `mercury-core`. Bundle exchange out-of-band (file) for the thinnest slice. **Deliverable:** two
clients exchange a real E2E-encrypted 1:1 message through the running relay.

**Milestone 2 — real accounts & directory.** Prekey-bundle directory (publish/fetch endpoint) + KT registration
so clients fetch *and verify* peer bundles; classical MLS groups wired the same way; encrypted-at-rest
persistence across restart (wire `mercury-store`).

**Milestone 3 — connect the UI.** Widen the FFI bridge with real send/receive/session ops (today it is
decision-only and forbids plaintext); replace the UI simulator + the throwing FFI binding; pick **Tauri**
(desktop, OS key storage) or **WASM** for the binding.

**Milestone 4 — relay hardening (security blockers above).** Real submitter/poller auth (terminate
mTLS/session tickets — today's header-trust is forgeable), TLS, durable storage, rate-limiting/body-limit on
`/submit`, real push.

**Milestone 5 — launch readiness.** E2E + fuzz + pen-test against the running app; signed/reproducible
releases; CI running `cargo audit` + tests.
