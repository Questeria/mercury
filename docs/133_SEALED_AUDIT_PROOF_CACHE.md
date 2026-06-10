# Sealed Audit Proof Cache Adapter

Generated: 2026-05-28

## Purpose

Mercury now has an accepted-only proof-cache adapter behind the sealed audit proof-bundle gate.

This adapter models the boundary a production encrypted proof cache must satisfy before local UI or platform code can treat an audit proof as durable, replayable offline, policy-bound, and safe to show without exposing audit selectors.

## Core API

Implemented in `core/rust/mercury-core/src/lib.rs`:

```text
SealedAuditProofCacheReason
SealedAuditProofCacheWrite
SealedAuditProofCacheRecord
SealedAuditProofCacheDecision
AcceptedSealedAuditProofCacheWrite
SealedAuditProofCacheAdapter
PrototypeSealedAuditProofCache
evaluate_sealed_audit_proof_cache_write(...)
put_sealed_audit_proof_cache_record(...)
```

Accepted proof-cache writes require:

- accepted `SealedAuditProofBundleDecision`
- proof-bundle permission to verify offline and persist
- no plaintext exposure from prior proof-bundle state
- versioned cache record format
- 32-byte proof bundle digest
- 32-byte audit event hash
- 32-byte checkpoint digest
- 32-byte verifier policy snapshot digest
- event sequence, log index, checkpoint size, and policy epoch matching the proof-bundle decision
- positive witness timestamp and non-regressing local verification time
- offline verification pass
- monitor freshness evidence
- zero plaintext metadata fields
- authenticated user-verified cache recovery when the proof-bundle decision requires recovery
- encrypted cache record
- append-only guard

## Adapter Behavior

`put_sealed_audit_proof_cache_record(...)` evaluates the write before calling the adapter. Rejected writes do not mutate storage.

The prototype adapter rejects:

- rejected proof bundles
- malformed digest or version shape
- duplicate proof bundle digests
- duplicate event hashes
- rollback log indexes
- stale verifier policy snapshots
- failed offline verification
- missing monitor freshness evidence
- plaintext metadata
- unauthenticated proof-cache recovery
- unencrypted or non-append-only cache writes

## Fixture And Command Surface

Prototype fixtures:

```text
sealed_audit_proof_cache_ready
sealed_audit_proof_cache_bundle_rejected
sealed_audit_proof_cache_duplicate_rejected
sealed_audit_proof_cache_policy_stale
sealed_audit_proof_cache_plaintext_rejected
```

Backend commands:

```text
run_sealed_audit_proof_cache_ready
run_sealed_audit_proof_cache_bundle_rejected
run_sealed_audit_proof_cache_duplicate_rejected
run_sealed_audit_proof_cache_policy_stale
run_sealed_audit_proof_cache_plaintext_rejected
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_proof_cache_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_proof_cache_ready
```

## Remaining Production Work

The adapter is not the final encrypted database implementation. Remaining work:

- durable encrypted proof-cache database implementation
- serialized C2SP proof parser and migration tests
- verifier policy snapshot database and rotation workflow
- scheduled offline verifier runner
- private monitor freshness scheduler
- proof-cache export/import ceremony for device recovery
