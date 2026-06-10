# Sealed Audit Proof Bundle Gate

Generated: 2026-05-28

## Purpose

Mercury now has a production-shaped gate for sealed audit proof bundles after the sealed audit event chain, event store, witness checkpoint, and witness client gates have accepted.

This gate models the local proof material a device needs before it can verify a security-critical audit event offline, persist proof cache state, recover after cache loss, or show proof status to UI without exposing audit selectors.

## Core API

Implemented in `core/rust/mercury-core/src/lib.rs`:

```text
SealedAuditProofBundleReason
SealedAuditProofBundleInput
SealedAuditProofBundleDecision
evaluate_sealed_audit_proof_bundle(...)
```

Accepted proof bundles require:

- accepted `SealedAuditWitnessClientDecision`
- witness-client permission to publish witnessed checkpoints and monitor privately
- no plaintext exposure from prior witness-client state
- versioned persisted proof bundle
- encrypted, append-only proof cache state with a 32-byte cache digest
- authenticated user-verified proof-cache recovery if the local proof cache is missing
- 32-byte verifier policy snapshot digest
- verifier policy epoch matching the witness-client policy epoch
- verifier log and witness key pins
- verifier witness threshold matching the witness-client threshold
- enough verified witness cosignatures to satisfy the threshold
- non-negative event sequence and log index
- 32-byte event hash and Merkle leaf hash
- checkpoint size matching the accepted witness-client checkpoint
- log index inside the checkpoint size
- bounded inclusion proof hash count
- verified inclusion proof with a root matching the checkpoint
- bounded verified consistency proof evidence
- witness timestamp freshness within local policy
- monitor freshness check evidence
- authenticated or opaque proof extra data
- 32-byte audit subject digest
- zero plaintext selectors
- digest-only UI status

## Rejection Reasons

Stable reason labels:

```text
ACCEPTED
WITNESS_CLIENT_REJECTED
POLICY_REJECTED
PROOF_SHAPE_REJECTED
INCLUSION_PROOF_REJECTED
WITNESS_FRESHNESS_REJECTED
PRIVACY_REJECTED
CACHE_RECOVERY_REJECTED
```

## Fixture And Command Surface

Prototype fixtures:

```text
sealed_audit_proof_bundle_ready
sealed_audit_proof_bundle_client_rejected
sealed_audit_proof_bundle_stale_witness
sealed_audit_proof_bundle_policy_rejected
sealed_audit_proof_bundle_privacy_rejected
```

Backend commands:

```text
run_sealed_audit_proof_bundle_ready
run_sealed_audit_proof_bundle_client_rejected
run_sealed_audit_proof_bundle_stale_witness
run_sealed_audit_proof_bundle_policy_rejected
run_sealed_audit_proof_bundle_privacy_rejected
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_proof_bundle_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_proof_bundle_ready
```

## Remaining Production Work

The gate is not a production proof cache yet. Remaining work:

- durable proof-bundle cache adapter
- serialized C2SP inclusion proof parser/verifier
- serialized checkpoint and cosignature parser
- durable verifier policy snapshot loader
- cache recovery protocol and UI ceremony
- background offline-verifier runner
- private monitor freshness scheduler
