# Sealed Audit Verifier Policy Store

Generated: 2026-05-28

## Purpose

Mercury now has an accepted-only verifier policy snapshot store and private monitor freshness scheduler boundary behind the sealed audit proof-cache adapter.

This boundary models the local policy database a device needs before it can keep offline audit proofs current, rotate log and witness key pins, schedule private monitor refreshes, and route split-view evidence without exposing audit selectors.

## Core API

Implemented in `core/rust/mercury-core/src/lib.rs`:

```text
SealedAuditVerifierPolicyReason
SealedAuditVerifierPolicySnapshot
SealedAuditVerifierPolicyRecord
SealedAuditVerifierPolicyDecision
AcceptedSealedAuditVerifierPolicySnapshot
SealedAuditVerifierPolicyStore
PrototypeSealedAuditVerifierPolicyStore
evaluate_sealed_audit_verifier_policy_snapshot(...)
put_sealed_audit_verifier_policy_snapshot(...)
```

Accepted verifier policy snapshots require:

- accepted `SealedAuditProofCacheDecision`
- versioned policy snapshot shape
- 32-byte policy, log-key-pinset, witness-key-pinset, monitor-query-plan, and proof-cache digests
- policy epoch at least as new as the proof-cache decision
- current, non-expired policy snapshot time window
- verified policy signature and policy consistency proof
- offline re-verification of cached proof state
- enough pinned log and witness keys for the witness quorum threshold
- private monitor endpoints and fresh monitor state
- encrypted append-only scheduler state
- no split-view evidence
- no plaintext audit selectors or plaintext metadata fields
- digest-only UI status

## Adapter Behavior

`put_sealed_audit_verifier_policy_snapshot(...)` evaluates the snapshot before calling the adapter. Rejected snapshots do not mutate storage.

The prototype store rejects:

- rejected or plaintext-tainted proof-cache decisions
- malformed digest, epoch, signature, scheduler, or consistency-proof state
- expired policy snapshots
- unauthenticated required key rotation
- stale private monitor freshness
- split-view evidence that requires escalation
- plaintext selector or metadata exposure
- duplicate snapshot digests
- non-advancing policy epochs

## Fixture And Command Surface

Prototype fixtures:

```text
sealed_audit_verifier_policy_ready
sealed_audit_verifier_policy_expired
sealed_audit_verifier_policy_key_rotation_required
sealed_audit_verifier_policy_monitor_privacy_rejected
sealed_audit_verifier_policy_plaintext_rejected
```

Backend commands:

```text
run_sealed_audit_verifier_policy_ready
run_sealed_audit_verifier_policy_expired
run_sealed_audit_verifier_policy_key_rotation_required
run_sealed_audit_verifier_policy_monitor_privacy_rejected
run_sealed_audit_verifier_policy_plaintext_rejected
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_verifier_policy_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_verifier_policy_ready
```

## Remaining Production Work

The store is still a prototype boundary. Remaining production work:

- durable encrypted policy snapshot database
- signed policy import and rotation service
- real private monitor scheduler and retry backoff
- production implementation of the incident evidence store described in `docs/137_SEALED_AUDIT_INCIDENT_EVIDENCE.md`
- production privacy-preserving contradiction or missing-proof report transport
- UI routing for stale policy, monitor refresh, key rotation, and split-view escalation states
