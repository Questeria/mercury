# Backend Session Orchestration

Generated: 2026-05-28

## Status

Mercury now has a deterministic non-UI backend session runner in `mercury-core`:

```text
PrototypeBackendSession
PrototypeBackendSessionInput
PrototypeBackendSessionOutcome
PrototypeBackendSessionReason
```

The session runner composes the current prototype surfaces in one flow:

```text
bootstrap accepted
local-store seal
local encrypted-store write
relay submit
relay deliver once
local-store open
AI draft participant decision
```

It is intentionally deterministic and in-memory. It is not a production networking, storage, or cryptographic runtime.

## Security Properties Exercised

- blocked bootstrap stops before crypto, storage, relay, and AI side effects
- rejected local-store seal requests stop before relay or AI work
- relay rejection can happen after encrypted local persistence without delivery
- delivered relay payloads are cleared from relay storage after delivery
- local-store open validates delivered sealed bytes before AI use
- AI requests record digest-only audit metadata and do not expose plaintext in the fixture surface
- each session run records a plaintext-free operation event transcript

## Fixture

The checked prototype fixture includes the happy-path session state:

```text
fixtures/prototypes/backend_session_happy_path.json
fixtures/prototypes/backend_session_bootstrap_blocked.json
fixtures/prototypes/backend_session_relay_rejected.json
fixtures/prototypes/backend_session_ai_rejected.json
```

It is also available through the simulator:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype backend_session_happy_path
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_backend_session
cargo test -p mercury-bindings --test prototype_fixtures
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Step

Session event transcript scaffolding is documented in `docs/40_SESSION_EVENT_TRANSCRIPT.md`. The next parallel increment should add a small transport envelope for backend session events so future desktop/mobile bindings can stream operation updates without depending on Rust enum formatting.
