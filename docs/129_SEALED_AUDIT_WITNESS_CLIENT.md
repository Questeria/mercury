# Sealed Audit Witness Client Gate

Generated: 2026-05-28

## Purpose

Mercury now has a production-shaped gate for witness client operation after a sealed audit checkpoint has passed the witness/checkpoint gate.

This gate models the C2SP `add-checkpoint` client side before a future network adapter is allowed to publish a checkpoint, accept witness cosignatures, or expose monitor/recovery state.

## Core API

Implemented in `core/rust/mercury-core/src/lib.rs`:

```text
SealedAuditWitnessClientReason
SealedAuditWitnessClientInput
SealedAuditWitnessClientDecision
evaluate_sealed_audit_witness_client(...)
```

Accepted witness client flow requires:

- accepted `SealedAuditWitnessCheckpointDecision`
- 32-byte witness policy digest
- non-expired policy epoch
- policy binding to log origin and witness operators
- pinned log public key and pinned witness keys
- witness operator count satisfying the checkpoint threshold
- enough submission endpoints and at least one monitor endpoint
- HTTPS or bastion-backed endpoints with TLS pins
- `old` size no larger than checkpoint size
- request checkpoint size matching the checkpoint decision
- consistency proof hash count in the C2SP limit of 63
- request body binding the policy epoch
- zero plaintext selectors in the witness request
- success status with enough known cosignatures from enough operators
- timestamped cosignatures bound to the checkpoint
- atomic latest-checkpoint persistence before the response is trusted
- split-view alert delivery configured
- private monitor retrieval with VRF or blinded selectors
- digest-only monitor results
- authenticated user-verified recovery when local checkpoint recovery is required

## Rejection Reasons

Stable reason labels:

```text
ACCEPTED
CHECKPOINT_GATE_REJECTED
POLICY_REJECTED
ENDPOINT_REJECTED
REQUEST_SHAPE_REJECTED
WITNESS_CONFLICT
WITNESS_UNAVAILABLE
WITNESS_RESPONSE_REJECTED
SPLIT_VIEW_ALERT
MONITOR_PRIVACY_REJECTED
RECOVERY_REJECTED
```

## C2SP Response Mapping

- `200` accepts only when the response has a known quorum of timestamped, checkpoint-bound cosignatures and the local latest checkpoint is persisted atomically.
- `409` maps to `WITNESS_CONFLICT` and enables `can_retry_witness_conflict`; the operator must reconcile the witness's latest checkpoint size.
- `408`, `425`, `429`, and `5xx` map to `WITNESS_UNAVAILABLE`.
- `400`, `403`, `404`, and `422` map to `SPLIT_VIEW_ALERT` because they may indicate malformed requests, unknown origins, untrusted signatures, or invalid consistency proofs.

## Fixture And Command Surface

Prototype fixtures:

```text
sealed_audit_witness_client_ready
sealed_audit_witness_client_conflict
sealed_audit_witness_client_unavailable
sealed_audit_witness_client_policy_rejected
sealed_audit_witness_client_monitor_privacy_rejected
```

Backend commands:

```text
run_sealed_audit_witness_client_ready
run_sealed_audit_witness_client_conflict
run_sealed_audit_witness_client_unavailable
run_sealed_audit_witness_client_policy_rejected
run_sealed_audit_witness_client_monitor_privacy_rejected
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_witness_client_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_witness_client_ready
```

## Remaining Production Work

The gate is not a network client yet. Remaining work:

- C2SP witness protocol HTTP adapter
- durable witness policy file/database loader
- durable latest-checkpoint store
- real consistency proof serialization
- real note/cosignature parsing
- private monitor retrieval implementation
- split-view alert routing
- proof bundle persistence for offline verification
