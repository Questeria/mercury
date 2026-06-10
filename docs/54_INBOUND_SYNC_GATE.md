# Inbound Sync Gate

Generated: 2026-05-28

## Status

Mercury now has a deterministic inbound sync gate in `mercury-core`:

```text
InboundSyncInput
InboundSyncDecision
InboundSyncReason
InboundSyncSourceState
evaluate_inbound_sync(...)
```

This is a pre-UI background-sync boundary. It decides whether a desktop or mobile client may poll the relay and hand a pending delivery to receive-session processing.

## Security Rules

- Plaintext notification previews are rejected before bootstrap or transport state is considered.
- Bootstrap must expose `can_start_sync = true` before any relay poll is allowed.
- Offline and backoff transport states require network retry without opening the receive path.
- Authentication rejection requires user action.
- A pending delivery must carry a 32-byte route id before the receive session can run.
- Every outcome has `plaintext_bytes_exposed = false`.

## Stable Reasons

```text
DELIVERY_READY
IDLE
BOOTSTRAP_BLOCKED
TRANSPORT_OFFLINE
TRANSPORT_AUTH_REJECTED
BACKOFF_REQUIRED
BAD_POLL_BATCH_LIMIT
BAD_ROUTE_ID_LENGTH
PLAINTEXT_NOTIFICATION_PREVIEW_FORBIDDEN
```

`IDLE` is an accepted state: the sync source may be polled and replay checkpoints may be updated, but no receive session should run because there is no pending delivery.

## Verification

Run:

```powershell
cargo test -p mercury-core --test inbound_sync_gate
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers delivery-ready, idle, bootstrap-blocked, plaintext-preview rejection, transport retry/user-action branches, bad batch limits, bad route ids, and stable reason codes/labels. The binding tests pin simulator, command, bridge, and checked JSON fixture output.

## Binding Fixtures

Prototype fixtures:

```text
inbound_sync_delivery_ready
inbound_sync_idle
inbound_sync_bootstrap_blocked
inbound_sync_transport_offline
inbound_sync_plaintext_preview_forbidden
```

Backend commands:

```text
run_inbound_sync_delivery_ready
run_inbound_sync_idle
run_inbound_sync_bootstrap_blocked
run_inbound_sync_transport_offline
run_inbound_sync_plaintext_preview_forbidden
```

## Next Backend Step

The composed inbound sync session is documented in `docs/55_INBOUND_SYNC_SESSION_ORCHESTRATION.md`, and the authenticated relay source gate that feeds this sync boundary is documented in `docs/56_AUTHENTICATED_RELAY_SOURCE.md`.
