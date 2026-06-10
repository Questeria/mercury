# Sealed Audit Private Report Receipt

Generated: 2026-05-28

## Purpose

Mercury now has a checked accepted-only receipt boundary for private sealed-audit incident report delivery.

The receipt store sits behind the private report outbox. It rejects delivery completion unless the outbox already accepted the submission and the receipt proves gateway acknowledgement, report binding, response transcript binding, gateway-key transparency, monitor submission evidence, retry completion safety, and selector-free status.

This prevents the UI or a retry worker from falsely marking a private report delivered just because an HTTP exchange happened or because retry state was updated.

## Core Surface

Implemented in `core/rust/mercury-core/src/lib.rs`:

- `SealedAuditPrivateReportReceiptReason`
- `SealedAuditPrivateReportReceiptWrite`
- `SealedAuditPrivateReportReceiptRecord`
- `SealedAuditPrivateReportReceiptDecision`
- `AcceptedSealedAuditPrivateReportReceiptWrite`
- `SealedAuditPrivateReportReceiptStore`
- `PrototypeSealedAuditPrivateReportReceiptStore`
- `evaluate_sealed_audit_private_report_receipt(...)`
- `put_sealed_audit_private_report_receipt_record(...)`

## Accepted Requirements

Accepted private report receipt decisions require:

- accepted and persisted private report outbox decision
- digest-only receipt id, report id, gateway receipt, gateway signing key, transparency checkpoint, consistency proof, key rotation, response transcript, monitor proof, blinded failure class, retry completion, and audit checkpoint
- gateway receipt signature verification
- receipt binding to report id, response transcript, and gateway key
- gateway key transparency verification
- gateway key consistency proof
- non-stale gateway key state
- authenticated gateway key rotation evidence
- relay policy binding
- private monitor route
- monitor-side submission proof
- blinded failure classification
- monotonic delivery completion state
- duplicate receipt and delivery replay rejection
- retry completion persistence
- delivered state only after receipt verification
- encrypted receipt record and append-only guard
- digest-only UI status

Rejected records do not mutate the store.

## Prototype Fixtures

Checked-in fixtures:

```text
sealed_audit_private_report_receipt_ready
sealed_audit_private_report_receipt_outbox_rejected
sealed_audit_private_report_receipt_missing
sealed_audit_private_report_receipt_transparency_rejected
sealed_audit_private_report_receipt_plaintext_rejected
```

Backend command envelopes:

```text
run_sealed_audit_private_report_receipt_ready
run_sealed_audit_private_report_receipt_outbox_rejected
run_sealed_audit_private_report_receipt_missing
run_sealed_audit_private_report_receipt_transparency_rejected
run_sealed_audit_private_report_receipt_plaintext_rejected
```

## Security Impact

Mercury now has twelve sealed-audit layers:

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

The twelfth layer keeps report delivery status from bypassing gateway receipt verification, transparency consistency, monitor evidence, replay protection, encrypted receipt storage, or selector redaction.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_private_report_receipt
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
```

Simulator checks:

```powershell
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_private_report_receipt_ready
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_private_report_receipt_ready
```

Run the full preflight before pushing the increment:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
