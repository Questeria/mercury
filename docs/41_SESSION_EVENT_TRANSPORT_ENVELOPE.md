# Session Event Transport Envelope

Generated: 2026-05-28

## Status

Backend session events now have stable transport views:

```text
PrototypeBackendSessionReason::code()
PrototypeBackendSessionReason::label()
PrototypeBackendSessionEventKind::code()
PrototypeBackendSessionEventKind::label()
PrototypeBackendSessionEvent::view()
PrototypeBackendSessionEventView
```

The JSON fixture event shape now uses stable fields:

```json
{
  "sequence": 0,
  "kind_code": 1,
  "kind_label": "session_started",
  "accepted": true,
  "terminal": false,
  "reason_code": 0,
  "reason_label": "completed",
  "plaintext_bytes_exposed": false
}
```

This keeps desktop/mobile bindings from depending on Rust `Debug` formatting for event names.

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_backend_session
cargo test -p mercury-bindings --test prototype_fixtures
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Step

The backend command envelope is documented in `docs/42_BACKEND_COMMAND_ENVELOPE.md`. The next parallel increment should add an AI bridge command fixture that represents a local AI requesting draft assistance through the same envelope while preserving the existing AI grant checks.
