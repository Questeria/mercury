# Production AI Connector Gate

Generated: 2026-05-28

## Status

Mercury now has a production-facing AI connector gate in `mercury-core`:

```text
AiConnectorRuntimeKind
AiConnectorInput
AiConnectorDecision
AiConnectorReason
evaluate_ai_connector(...)
```

This is not a model runtime implementation. It is the contract a future local model runner, user-hosted connector, or explicitly allowed remote connector must pass before an AI participant can receive selected context and emit a draft.

## Accepted Connector

The accepted path is intentionally narrow:

- AI participant request must already be accepted by `AiParticipantRequest`
- action must be draft-capable, not send, tool, retention, or training
- runtime must be explicitly selected by the user
- model must be explicitly selected by the user
- connector must be authenticated
- model integrity must be verified
- high-security rooms require local-device runtime
- context and draft output must have 32-byte audit digests
- plaintext bridge fields must be zero
- prompt retention, training, direct send, and tool execution must be disabled

Accepted output enables:

```text
can_call_model = true
can_emit_draft = true
requires_user_review = true
```

Accepted output always keeps:

```text
can_send_message = false
can_use_tool = false
forbids_prompt_retention = true
forbids_training = true
plaintext_bytes_exposed = false
```

## Runtime Classes

Stable runtime labels:

```text
local_device
user_hosted_local_network
remote_provider
development_stub
```

Remote runtimes are rejected unless `allow_remote_runtime = true`. Development stubs are rejected unless `allow_development_runtime = true`. High-security rooms reject every runtime except `local_device`.

## Rejection Classes

Stable rejection labels:

```text
PARTICIPANT_REJECTED
DRAFT_ACTION_REQUIRED
RUNTIME_NOT_USER_SELECTED
MODEL_NOT_USER_SELECTED
DEVELOPMENT_RUNTIME_FORBIDDEN
REMOTE_RUNTIME_FORBIDDEN
HIGH_SECURITY_REQUIRES_LOCAL_RUNTIME
CONNECTOR_AUTHENTICATION_MISSING
MODEL_INTEGRITY_UNVERIFIED
CONTEXT_DIGEST_REQUIRED
DRAFT_OUTPUT_DIGEST_REQUIRED
PLAINTEXT_BRIDGE_FORBIDDEN
PROMPT_RETENTION_FORBIDDEN
TRAINING_FORBIDDEN
DIRECT_SEND_FORBIDDEN
TOOL_EXECUTION_FORBIDDEN
```

## Checked Fixtures

Prototype fixtures:

```text
ai_connector_local_draft_ready
ai_connector_remote_forbidden
ai_connector_plaintext_bridge_rejected
ai_connector_retention_rejected
ai_connector_user_selection_required
```

These fixtures expose ready, remote-runtime rejection, plaintext bridge rejection, prompt-retention rejection, and missing user runtime selection states through the simulator.

## Verification

Run:

```powershell
cargo test -p mercury-core --test ai_connector_gate
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype ai_connector_local_draft_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers accepted local draft flow, participant rejection propagation, non-draft action rejection, user/model selection requirements, runtime class restrictions, high-security local-only routing, authentication, model integrity, digest-only bridge, prompt retention, training, direct send, tool execution, and stable codes/labels.

## Next Backend Step

Implement the real model execution adapter behind this gate, keeping the connector draft-only until a separate confirmed-send path has its own explicit review and signing contract.
