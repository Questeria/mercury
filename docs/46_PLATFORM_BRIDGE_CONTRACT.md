# Platform Bridge Contract

Generated: 2026-05-28

## Status

Mercury now has a narrow JSON-shaped platform bridge contract in `mercury-bindings`.

The bridge accepts:

```text
request_id
operation
target
plaintext_payload_len
```

The request ID must be 32 bytes. The bridge rejects plaintext payloads before target lookup. Accepted requests return a `bridge` decision envelope plus a `body` generated from existing verified backend surfaces.

## Operations

Stable operation labels:

```text
platform_fixture
prototype_fixture
backend_command
```

These map to:

- checked `PlatformDecisionView` fixtures
- prototype backend fixture payloads
- backend command envelopes

## Response Shape

Every response has:

```text
bridge
body
```

`bridge` includes:

```text
request_id_len
operation_code
operation_label
target
accepted
reason_code
reason_label
plaintext_payload_len
```

Rejected requests always return `body = null`.

## Simulator Support

Use:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --bridge backend_command run_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --bridge-json "{\"request_id\":\"0123456789abcdef0123456789abcdef\",\"operation\":\"platform_fixture\",\"target\":\"bootstrap_accepted\"}"
```

## Security Rules

- Platform shells should call this bridge rather than lower-level policy functions.
- UI and shell code should branch on `bridge.accepted`, capability booleans, and stable labels.
- Plaintext payloads are not accepted through the bridge.
- Unknown operations and targets do not return fallback bodies.
- The bridge is a contract layer, not a production FFI ABI yet.

## Verification

Run:

```powershell
cargo test -p mercury-bindings --test platform_bridge
cargo test -p mercury-bindings --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

The local-store unlock gate is documented in `docs/47_LOCAL_STORE_UNLOCK_GATE.md`.
