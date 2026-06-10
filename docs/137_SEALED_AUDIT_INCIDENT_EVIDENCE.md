# Sealed Audit Incident Evidence Store

Generated: 2026-05-28

## Purpose

Mercury now has a checked boundary for split-view, missing-proof, and private-monitor incident evidence after verifier policy has already passed.

The boundary exists so UI, platform shells, and future production services cannot turn raw audit failures into user-visible incident state unless the evidence is digest-bound, encrypted at rest, append-only, policy-bound, and selector-free.

## Core Surface

Implemented in `core/rust/mercury-core/src/lib.rs`:

- `SealedAuditIncidentEvidenceReason`
- `SealedAuditIncidentEvidenceWrite`
- `SealedAuditIncidentEvidenceRecord`
- `SealedAuditIncidentEvidenceDecision`
- `AcceptedSealedAuditIncidentEvidenceWrite`
- `SealedAuditIncidentEvidenceStore`
- `PrototypeSealedAuditIncidentEvidenceStore`
- `evaluate_sealed_audit_incident_evidence(...)`
- `put_sealed_audit_incident_evidence_record(...)`

## Accepted Evidence Requirements

Accepted incident evidence must have:

- accepted verifier policy state with offline verification, private monitor scheduling, UI-safe status, and no plaintext exposure
- 32-byte incident, verifier-policy, proof-cache, checkpoint, witness/operator, contradiction, missing-proof report, monitor-report, and accountability-route digests
- policy epoch and proof-cache indices matching the verifier-policy decision
- verified incident signature
- at least one split-view, missing-proof, or monitor-failure evidence count
- blinded missing-proof reports when missing proofs are present
- private monitor reports when monitor failures are present
- verified contradiction proof when split-view evidence is present
- witness/operator signatures meeting the configured quorum
- configured accountability route
- encrypted store records and append-only guard
- zero plaintext selectors and zero plaintext metadata fields
- digest-only UI status

## Rejection Reasons

Stable reasons:

```text
ACCEPTED
VERIFIER_POLICY_REJECTED
NO_INCIDENT_EVIDENCE
MISSING_PROOF_REPORT_REQUIRED
SPLIT_VIEW_EVIDENCE_REQUIRED
OPERATOR_ACCOUNTABILITY_REQUIRED
PLAINTEXT_METADATA_FORBIDDEN
BAD_RECORD_SHAPE
```

Rejected evidence is not persisted.

## Prototype Fixtures

Checked-in fixtures:

```text
sealed_audit_incident_evidence_ready
sealed_audit_incident_evidence_policy_rejected
sealed_audit_incident_evidence_missing_proof_report
sealed_audit_incident_evidence_split_view
sealed_audit_incident_evidence_plaintext_rejected
```

Backend command envelopes:

```text
run_sealed_audit_incident_evidence_ready
run_sealed_audit_incident_evidence_policy_rejected
run_sealed_audit_incident_evidence_missing_proof_report
run_sealed_audit_incident_evidence_split_view
run_sealed_audit_incident_evidence_plaintext_rejected
```

## Security Impact

Mercury can now represent sealed-audit incidents without exposing the audit subject, raw monitor selector, account identifier, group identifier, or plaintext event metadata to UI code.

This turns split-view and missing-proof handling into an accepted-only store boundary rather than an ambient diagnostics path. Future production reporting can wire to this trait and preserve the same policy: report and escalate digest-bound evidence, never raw selectors.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_incident_evidence
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_incident_evidence_ready
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_incident_evidence_ready
```

Run the full preflight before pushing the increment:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
