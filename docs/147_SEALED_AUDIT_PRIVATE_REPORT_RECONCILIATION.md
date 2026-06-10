# Sealed Audit Private Report Reconciliation

Generated: 2026-05-28

## Purpose

Mercury now has a checked accepted-only reconciliation boundary for private sealed-audit incident report retries and delivery completion.

The reconciliation store sits behind the private report receipt store. It rejects retry and delivery state unless the private report receipt has already accepted, the pending outbox and delivered-state digests are bound, the retry schedule is monotonic and idempotency-bound, anonymous rate-limit token state is preserved, crash recovery resumes only pending work, and operator-accountability routing is digest-only.

This prevents a retry worker, UI, or gateway adapter from duplicating reports, bypassing anonymous rate limits, or showing a delivered state without a receipt-backed transition.

## Core Surface

Implemented in `core/rust/mercury-core/src/lib.rs`:

- `SealedAuditPrivateReportReconciliationReason`
- `SealedAuditPrivateReportReconciliationWrite`
- `SealedAuditPrivateReportReconciliationRecord`
- `SealedAuditPrivateReportReconciliationDecision`
- `AcceptedSealedAuditPrivateReportReconciliationWrite`
- `SealedAuditPrivateReportReconciliationStore`
- `PrototypeSealedAuditPrivateReportReconciliationStore`
- `evaluate_sealed_audit_private_report_reconciliation(...)`
- `put_sealed_audit_private_report_reconciliation_record(...)`

## Accepted Requirements

Accepted private report reconciliation decisions require:

- accepted, persisted, digest-only private report receipt decision
- digest-only reconciliation id, report id, receipt id, pending outbox, retry schedule, rate-limit state, delivered state, blinded failure bucket, operator accountability route, crash recovery cursor, and audit checkpoint
- monotonic reconciliation sequence and prior reconciliation binding
- retry schedule binding
- monotonic retry-after state
- duplicate retry rejection
- idempotency-key binding for retry attempts
- no retry after a delivered receipt
- anonymous rate-limit window binding
- preservation of spend-once rate-limit token state
- proof that retry does not mint a new report
- pending outbox binding and delivered state that requires the receipt
- crash recovery cursor binding and resume-pending-only behavior
- operator accountability route binding
- escalation when a receipt never appears
- blinded failure buckets only
- encrypted reconciliation records and append-only guard
- digest-only UI retry and delivery status

Rejected records do not mutate the store.

## Prototype Fixtures

Checked-in fixtures:

```text
sealed_audit_private_report_reconciliation_ready
sealed_audit_private_report_reconciliation_receipt_rejected
sealed_audit_private_report_reconciliation_retry_rejected
sealed_audit_private_report_reconciliation_false_delivery_rejected
sealed_audit_private_report_reconciliation_plaintext_rejected
```

Backend command envelopes:

```text
run_sealed_audit_private_report_reconciliation_ready
run_sealed_audit_private_report_reconciliation_receipt_rejected
run_sealed_audit_private_report_reconciliation_retry_rejected
run_sealed_audit_private_report_reconciliation_false_delivery_rejected
run_sealed_audit_private_report_reconciliation_plaintext_rejected
```

## Security Impact

Mercury now has thirteen sealed-audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness
4. witness client and private monitor operation readiness
5. proof bundle persistence and offline verification readiness
6. accepted-only proof-cache persistence
7. verifier policy snapshot and private monitor freshness readiness
8. accepted-only incident evidence and privacy-preserving report readiness
9. accepted-only recovery/export and cross-device incident sync readiness
10. production sealed-audit database and private report transport readiness
11. accepted-only private report outbox and submission transcript persistence
12. accepted-only private report delivery receipt and gateway transparency persistence
13. accepted-only private report retry reconciliation and delivery-state transition persistence

The thirteenth layer keeps retry scheduling and delivery completion from bypassing receipt verification, retry idempotency, rate-limit continuity, crash recovery safety, operator accountability, or selector redaction.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_private_report_reconciliation
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
```

Simulator checks:

```powershell
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_private_report_reconciliation_ready
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_private_report_reconciliation_ready
```

Run the full preflight before pushing the increment:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
