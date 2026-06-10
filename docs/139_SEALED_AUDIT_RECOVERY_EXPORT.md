# Sealed Audit Recovery Export Store

Generated: 2026-05-28

## Purpose

Mercury now has a checked boundary for exporting, restoring, and syncing sealed-audit state across devices after incident evidence has already passed.

The boundary prevents recovered audit state from becoming trusted unless the export manifest is encrypted, authenticated, device-bound, restore-quorum-approved, rollback-checked, audit-checkpoint-bound, append-only, and selector-free.

## Core Surface

Implemented in `core/rust/mercury-core/src/lib.rs`:

- `SealedAuditRecoveryExportReason`
- `SealedAuditRecoveryExportWrite`
- `SealedAuditRecoveryExportRecord`
- `SealedAuditRecoveryExportDecision`
- `AcceptedSealedAuditRecoveryExportWrite`
- `SealedAuditRecoveryExportStore`
- `PrototypeSealedAuditRecoveryExportStore`
- `evaluate_sealed_audit_recovery_export(...)`
- `put_sealed_audit_recovery_export_record(...)`

## Accepted Export Requirements

Accepted recovery/export records must have:

- accepted incident evidence state with private reporting, UI-safe status, digest-only state, and no plaintext exposure
- 32-byte export manifest, device-set, recovery-policy, verifier-policy, proof-cache, incident, incident-evidence, ciphertext, restore-authorization, sync-state, and audit-checkpoint digests
- export sequence strictly newer than the previous sequence
- policy epoch and proof-cache indices matching the incident evidence decision
- valid creation, expiry, and restore times
- manifest signature verification
- device binding verification
- recovery policy verification
- encrypted and authenticated export ciphertext
- verified restore authorization
- device quorum and recovery-share quorum
- rollback guard verification
- private cross-device sync
- redacted incident selectors
- verified audit-log checkpoint
- encrypted store records and append-only guard
- zero plaintext selectors and metadata fields
- digest-only UI status

## Rejection Reasons

Stable reasons:

```text
ACCEPTED
INCIDENT_EVIDENCE_REJECTED
RESTORE_QUORUM_REQUIRED
STALE_POLICY_SNAPSHOT
ROLLBACK_EXPORT_REJECTED
DEVICE_BINDING_REQUIRED
PLAINTEXT_METADATA_FORBIDDEN
BAD_RECORD_SHAPE
```

Rejected export records are not persisted.

## Prototype Fixtures

Checked-in fixtures:

```text
sealed_audit_recovery_export_ready
sealed_audit_recovery_export_incident_rejected
sealed_audit_recovery_export_quorum_required
sealed_audit_recovery_export_rollback_rejected
sealed_audit_recovery_export_plaintext_rejected
```

Backend command envelopes:

```text
run_sealed_audit_recovery_export_ready
run_sealed_audit_recovery_export_incident_rejected
run_sealed_audit_recovery_export_quorum_required
run_sealed_audit_recovery_export_rollback_rejected
run_sealed_audit_recovery_export_plaintext_rejected
```

## Security Impact

Mercury can now model cross-device sealed-audit recovery without trusting a restored cache just because it decrypts.

This adds an explicit gate for recovery authorization, rollback defense, device quorum, incident-selector redaction, and audit-checkpoint binding. Future desktop/mobile recovery UX can consume these command outputs and keep restored incident evidence read-only until the backend says it is safe.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_recovery_export
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_recovery_export_ready
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_recovery_export_ready
```

Run the full preflight before pushing the increment:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
