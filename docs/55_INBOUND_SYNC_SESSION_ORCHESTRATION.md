# Inbound Sync Session Orchestration

Generated: 2026-05-28

## Status

Mercury now has a composed inbound sync session in `mercury-core`:

```text
PrototypeInboundSyncSession
PrototypeInboundSyncSessionInput
PrototypeInboundSyncSessionOutcome
PrototypeInboundSyncSessionReason
PrototypeInboundSyncSessionEvent
PrototypeInboundSyncSessionEventKind
```

The session composes the authenticated relay source gate, inbound sync gate, and receive-session prototype. It lets background sync produce one plaintext-free operation transcript before any UI, notification preview, or local plaintext surface is involved.

## Flow

```text
authenticated relay source gate
  -> inbound sync gate
  -> receive session, only when sync can run receive
  -> combined event transcript
```

Accepted idle sync stops safely without receive-side effects. Rejected source or sync states stop before relay polling or local persistence. Receive rejection is surfaced as the terminal session event.

## Event Transcript

Stable event labels:

```text
sync_gate_evaluated
receive_started
relay_submit_evaluated
relay_delivery_evaluated
delivery_ack_evaluated
client_receive_evaluated
local_store_write_evaluated
receive_finished
sync_finished
```

Every event includes stable kind/reason codes, labels, terminal state, acceptance state, and `plaintext_bytes_exposed = false`. Fixture output also includes the authenticated relay source decision that produced the sync input.

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_inbound_sync_session
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers delivery-ready sync plus receive, idle sync without receive side effects, sync rejection before receive, source-auth rejection before receive, receive rejection as a terminal event, and stable reason/event codes and labels. Binding tests pin simulator, command, bridge, and checked JSON fixture output.

## Binding Fixtures

Prototype fixtures:

```text
inbound_sync_session_happy_path
inbound_sync_session_idle
inbound_sync_session_sync_rejected
inbound_sync_session_receive_rejected
```

Backend commands:

```text
run_inbound_sync_session_happy_path
run_inbound_sync_session_idle
run_inbound_sync_session_sync_rejected
run_inbound_sync_session_receive_rejected
```

## Next Backend Step

The authenticated relay source gate and binding fixtures are documented in `docs/56_AUTHENTICATED_RELAY_SOURCE.md`.
