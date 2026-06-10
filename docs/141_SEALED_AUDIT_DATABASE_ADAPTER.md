# Sealed Audit Database Adapter And Private Report Transport

Generated: 2026-05-28

## Purpose

Mercury now has a checked boundary for opening production sealed-audit storage and for submitting private incident reports.

The database gate sits behind accepted recovery/export state. It rejects storage profiles that are not encrypted, page-authenticated, append-only, transactionally checkpointed, migration-tested, crash-recovery-tested, and selector-free.

The report transport gate sits behind the accepted database adapter. It rejects incident-report submission unless the report path is OHTTP-style state-free, HPKE-protected, anonymous-rate-limited, replay-guarded, retry-safe, encrypted at rest, and digest-only.

## Core Surface

Implemented in `core/rust/mercury-core/src/lib.rs`:

- `SealedAuditDatabaseAdapterReason`
- `SealedAuditDatabaseAdapterInput`
- `SealedAuditDatabaseAdapterDecision`
- `evaluate_sealed_audit_database_adapter(...)`
- `SealedAuditPrivateReportTransportReason`
- `SealedAuditPrivateReportTransportInput`
- `SealedAuditPrivateReportTransportDecision`
- `evaluate_sealed_audit_private_report_transport(...)`

## Accepted Database Requirements

Accepted sealed-audit database adapter decisions require:

- accepted recovery/export decision
- accepted local encrypted database adapter selection
- encrypted tables for event, proof-cache, verifier-policy, incident-evidence, recovery-export, and checkpoint state
- encrypted WAL state
- memory-only temporary storage
- page authentication and open-time integrity checks
- platform key wrapping and key-rotation support
- append-only tables and monotonic sequence constraints
- duplicate digest constraints
- transactional batch writes and verified WAL checkpoint policy
- deterministic migration drill
- crash-recovery drill
- zero plaintext headers, selectors, metadata fields, and schema leaks
- digest-only UI status

## Accepted Report Transport Requirements

Accepted private report transport decisions require:

- accepted sealed-audit database adapter decision
- OHTTP relay configuration and pinned gateway key digest
- state-free target behavior without cookies or auth state
- HPKE request encryption and authenticated gateway responses
- Privacy Pass-style anonymous rate-limit tokens with pinned issuer key digest
- encrypted and digest-only report payloads
- selector blinding
- encrypted report outbox
- retry/backoff and replay guards
- duplicate report rejection
- constant-size padding
- private monitor routing
- digest-only UI status

## Prototype Fixtures

Checked-in fixtures:

```text
sealed_audit_database_adapter_ready
sealed_audit_database_adapter_encryption_rejected
sealed_audit_database_adapter_append_only_rejected
sealed_audit_private_report_transport_ready
sealed_audit_private_report_transport_plaintext_rejected
```

Backend command envelopes:

```text
run_sealed_audit_database_adapter_ready
run_sealed_audit_database_adapter_encryption_rejected
run_sealed_audit_database_adapter_append_only_rejected
run_sealed_audit_private_report_transport_ready
run_sealed_audit_private_report_transport_plaintext_rejected
```

## Security Impact

Mercury can now distinguish "the audit state decrypts" from "the audit database and report path are safe to use."

This closes a subtle recovery gap: restored incident evidence cannot become production-operational unless the storage adapter preserves append-only, rollback-resistant, encrypted semantics and the report transport avoids IP/request linkage, replay, rate-limit abuse, plaintext selector leakage, and retry amplification.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_database_adapter
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Run the full preflight before pushing the increment:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
