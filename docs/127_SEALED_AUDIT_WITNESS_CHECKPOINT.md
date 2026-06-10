# Sealed Audit Witness Checkpoint Gate

Generated: 2026-05-28

## Purpose

Mercury now has a backend gate for publishing witnessed sealed-audit checkpoints. It sits after the sealed audit event-chain gate and accepted-only event-store boundary.

This gate prevents a future checkpoint publisher from treating an audit record as public, witnessed, monitor-safe, or split-view resistant unless it was first persisted as a sealed, digest-only audit record and then passed stricter checkpoint operation checks.

## Core API

Implemented in `core/rust/mercury-core/src/lib.rs`:

```text
SealedAuditCheckpointSignatureAlgorithm
SealedAuditWitnessCheckpointReason
SealedAuditWitnessCheckpointInput
SealedAuditWitnessCheckpointDecision
evaluate_sealed_audit_witness_checkpoint(...)
```

The accepted path requires:

- accepted and persisted `SealedAuditEventStoreDecision`
- witness-backed transparency anchor
- non-empty log origin and 32-byte log identity digest
- monotonic checkpoint size at or beyond the persisted event sequence
- 32-byte checkpoint root hash
- PQ or hybrid checkpoint signature policy (`ml_dsa_44` or `hybrid_ed25519_ml_dsa_44`)
- pinned signing-key digest and valid key rotation window
- retained previous signing key for verification
- verified consistency proof with at most 63 proof hashes
- witness threshold of at least two
- enough witnesses and at least two operators
- pinned witness keys
- timestamped cosignatures bound to the checkpoint
- no split-view evidence
- private monitor retrieval, no plaintext selectors, and digest-only monitor results
- authenticated user-verified recovery if the local latest checkpoint is missing

## Rejection Reasons

Stable reason labels:

```text
ACCEPTED
STORE_REJECTED
ANCHOR_REJECTED
CHECKPOINT_SHAPE_REJECTED
LOG_ORIGIN_REJECTED
SIGNING_KEY_REJECTED
KEY_ROTATION_REJECTED
CONSISTENCY_PROOF_REJECTED
WITNESS_QUORUM_REJECTED
STALE_CHECKPOINT
SPLIT_VIEW_EVIDENCE
MONITOR_PRIVACY_REJECTED
RECOVERY_STATE_REJECTED
```

## Fixture And Command Surface

Prototype fixtures:

```text
sealed_audit_witness_checkpoint_ready
sealed_audit_witness_checkpoint_store_rejected
sealed_audit_witness_checkpoint_quorum_rejected
sealed_audit_witness_checkpoint_split_view_rejected
sealed_audit_witness_checkpoint_privacy_rejected
```

Backend commands:

```text
run_sealed_audit_witness_checkpoint_ready
run_sealed_audit_witness_checkpoint_store_rejected
run_sealed_audit_witness_checkpoint_quorum_rejected
run_sealed_audit_witness_checkpoint_split_view_rejected
run_sealed_audit_witness_checkpoint_privacy_rejected
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_witness_checkpoint_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_witness_checkpoint_ready
```

## Security Effect

This increment closes the operation gap between "an audit event was accepted and persisted" and "a checkpoint is safe to publish or treat as witnessed."

The gate blocks:

- unpersisted or rejected audit-store decisions
- local/private anchors that cannot satisfy witness quorum
- malformed or stale checkpoints
- classical-only checkpoint signatures under the high-security policy
- missing signing-key lifecycle evidence
- missing consistency proof
- unsatisfied witness quorum or missing key pins
- cosignatures that are not timestamped or not bound to the checkpoint
- split-view evidence
- plaintext monitor selectors and non-private monitor queries
- unauthenticated recovery after local checkpoint loss

## Remaining Production Work

The gate is not yet a networked witness service. Remaining work:

- durable checkpoint publisher adapter
- configured witness policy file or database
- real C2SP witness protocol client
- real consistency-proof verifier
- real ML-DSA/hybrid signature implementation and key rotation service
- privacy-preserving monitor query protocol
- operator/user alerting for split-view evidence
- production recovery workflow for local checkpoint loss
