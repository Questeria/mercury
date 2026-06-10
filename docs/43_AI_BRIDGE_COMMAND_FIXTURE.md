# AI Bridge Command Fixture

Generated: 2026-05-28

## Status

Mercury now has a deterministic command fixture for a local AI participant requesting draft assistance through the same backend command envelope used by session runs.

The command is:

```text
local_ai_draft_assist
```

It maps to:

```text
actor_kind = local_ai
command_kind = run_local_ai_draft_assist
result.surface = prototype_ai_participant
```

## Security Shape

The command envelope keeps AI access narrow:

- remote AI actors are rejected
- local AI actors cannot run backend session commands
- plaintext command payloads are rejected
- accepted AI draft commands do not emit a session event stream
- accepted AI draft commands set `can_request_ai_draft = true` and `can_run_session = false`

This is only a deterministic prototype fixture. It does not grant a production AI agent access to message plaintext, group keys, or user identity material.

## Simulator Support

Use:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command local_ai_draft_assist
```

The JSON includes:

```text
command
result
```

`command` carries stable actor, command, reason, and capability fields. `result` carries the existing AI participant backend fixture.

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Step

The non-UI readiness gate report is documented in `docs/44_NON_UI_BACKLOG_READY_FOR_UI.md`.
