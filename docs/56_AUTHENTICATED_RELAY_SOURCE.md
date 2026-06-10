# Authenticated Relay Source Gate

Generated: 2026-05-28

## Status

Mercury now has a core decision boundary for turning an authenticated relay transport source into an inbound-sync input:

```text
AuthenticatedRelaySourceInput
AuthenticatedRelaySourceDecision
AuthenticatedRelaySourceReason
AuthenticatedRelayTransportState
evaluate_authenticated_relay_source(...)
```

This is the backend contract a future desktop, mobile, or service transport adapter must satisfy before background sync treats a relay item as a real pending delivery.

`mercury-bindings` now exposes checked source fixtures and backend commands for this gate.

## Security Rules

- Plaintext notification previews are rejected before polling.
- Plaintext identity metadata is rejected before polling.
- Offline and backoff transport states never run receive-session processing.
- Relay session tickets, device credentials, and server authentication tags must be exactly 32 bytes.
- Server authentication, route-key authentication, and replay-window validation must already be verified by the transport adapter.
- Pending deliveries must expose only an opaque 32-byte route id.
- Poll batches must stay within the same 1 to 100 item window as the inbound sync gate.
- Every outcome has `plaintext_bytes_exposed = false`.

## Inbound Sync Handoff

Accepted decisions produce an `InboundSyncInput` through:

```text
AuthenticatedRelaySourceDecision::into_inbound_sync_input(...)
```

That keeps transport/auth validation separate from bootstrap readiness. The inbound sync gate still decides whether the client can poll the relay and whether a receive session may run.

## Stable Reasons

```text
DELIVERY_READY
IDLE
TRANSPORT_OFFLINE
BACKOFF_REQUIRED
BAD_SESSION_TICKET_LENGTH
BAD_DEVICE_CREDENTIAL_LENGTH
BAD_SERVER_AUTH_TAG_LENGTH
SERVER_AUTHENTICATION_REJECTED
ROUTE_KEY_AUTHENTICATION_REJECTED
REPLAY_WINDOW_REJECTED
PLAINTEXT_IDENTITY_FORBIDDEN
PLAINTEXT_NOTIFICATION_PREVIEW_FORBIDDEN
BAD_POLL_BATCH_LIMIT
BAD_ROUTE_ID_LENGTH
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test authenticated_relay_source --test inbound_sync_gate --test prototype_inbound_sync_session
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers accepted delivery, accepted idle sync, offline/backoff transport mapping, authentication rejection, plaintext metadata rejection, malformed batch/route metadata, and stable transport/reason codes.

## Binding Fixtures

Prototype fixtures:

```text
authenticated_relay_source_delivery_ready
authenticated_relay_source_idle
authenticated_relay_source_auth_rejected
authenticated_relay_source_plaintext_forbidden
```

Backend commands:

```text
run_authenticated_relay_source_delivery_ready
run_authenticated_relay_source_idle
run_authenticated_relay_source_auth_rejected
run_authenticated_relay_source_plaintext_forbidden
```

## Session Integration

The composed inbound sync session now has an authenticated-source entry point:

```text
PrototypeAuthenticatedInboundSyncSessionInput
PrototypeAuthenticatedInboundSyncSessionOutcome
PrototypeInboundSyncSession::run_authenticated_source(...)
```

The checked `inbound_sync_session_*` fixtures use this path and include the source decision alongside the sync/session outcome.
