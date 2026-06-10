# Session Event Transcript

Generated: 2026-05-28

## Status

`PrototypeBackendSession` now records a plaintext-free event transcript for each run:

```text
PrototypeBackendSessionEventKind
PrototypeBackendSessionEvent
PrototypeBackendSessionEventView
PrototypeBackendSession::events()
```

Each event records:

- sequence number
- operation kind
- accepted flag
- terminal flag
- stable event kind code/label
- stable session reason code/label
- `plaintext_bytes_exposed`, always false in the prototype transcript

## Event Flow

The happy path emits:

```text
SessionStarted
BootstrapChecked
LocalStoreSealEvaluated
LocalStoreWriteEvaluated
RelaySubmitEvaluated
RelayDeliveryEvaluated
LocalStoreOpenEvaluated
AiParticipantEvaluated
SessionFinished
```

Rejected flows stop at the terminal failed event. For example, blocked bootstrap stops at `BootstrapChecked`, relay rejection stops at `RelaySubmitEvaluated`, and AI rejection stops at `AiParticipantEvaluated`.

## Fixtures

Backend session fixtures now include an `events` array. This gives UI and integration agents a deterministic operation timeline without exposing plaintext:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype backend_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype backend_session_ai_rejected
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_backend_session
cargo test -p mercury-bindings --test prototype_fixtures
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Step

The session event transport envelope is documented in `docs/41_SESSION_EVENT_TRANSPORT_ENVELOPE.md`. The next parallel increment should add a small backend command/action envelope so UI, desktop, mobile, and AI bridge layers can request deterministic session operations through one stable shape.
