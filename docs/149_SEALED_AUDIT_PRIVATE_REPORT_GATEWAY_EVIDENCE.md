# Sealed Audit Private Report Gateway Evidence

Generated: 2026-05-28

## Purpose

Mercury now has a checked accepted-only store for private sealed-audit report gateway-unavailability and operator-accountability evidence.

The gateway-evidence store sits behind private report reconciliation. It rejects unavailable-gateway incident evidence unless reconciliation already accepted, retry attempts are exhausted, gateway/relay observations are authenticated, the client is not self-asserting unavailability, operator escalation is routed through blinded accountability buckets, and all stored/UI-visible state is digest-only.

This prevents a retry worker, gateway adapter, relay, or UI from turning an ambiguous timeout into a forged security incident or selector-bearing operator report.

## Core Surface

Implemented in `core/rust/mercury-core/src/lib.rs`:

- `SealedAuditPrivateReportGatewayEvidenceReason`
- `SealedAuditPrivateReportGatewayEvidenceWrite`
- `SealedAuditPrivateReportGatewayEvidenceRecord`
- `SealedAuditPrivateReportGatewayEvidenceDecision`
- `AcceptedSealedAuditPrivateReportGatewayEvidenceWrite`
- `SealedAuditPrivateReportGatewayEvidenceStore`
- `PrototypeSealedAuditPrivateReportGatewayEvidenceStore`
- `evaluate_sealed_audit_private_report_gateway_evidence(...)`
- `put_sealed_audit_private_report_gateway_evidence_record(...)`

## Accepted Requirements

Accepted gateway-evidence records require:

- accepted, persisted, digest-only private report reconciliation decision
- digest-only evidence id, reconciliation id, report id, receipt id, unavailable-evidence proof, relay observation, gateway error, target absence proof, retry exhaustion, rate-limit state, gateway key state, accountability route, blinded failure bucket, monitor submission, and audit checkpoint
- monotonic evidence sequence and prior evidence binding
- reconciliation binding
- gateway-authenticated unavailable evidence
- signed relay observation
- target absence proof binding
- timeout or gateway/server-unavailable classification
- no client-asserted unavailability
- retry exhaustion and rate-limit continuity binding
- gateway key state binding
- operator accountability route and escalation binding
- blinded failure bucket only
- private monitor route
- user-visible incident status only after policy approval
- encrypted evidence records and append-only guard
- digest-only UI unavailable status

Rejected records do not mutate the store.

## Prototype Fixtures

Checked-in fixtures:

```text
sealed_audit_private_report_gateway_evidence_ready
sealed_audit_private_report_gateway_evidence_reconciliation_rejected
sealed_audit_private_report_gateway_evidence_unavailable_rejected
sealed_audit_private_report_gateway_evidence_accountability_rejected
sealed_audit_private_report_gateway_evidence_plaintext_rejected
```

Backend command envelopes:

```text
run_sealed_audit_private_report_gateway_evidence_ready
run_sealed_audit_private_report_gateway_evidence_reconciliation_rejected
run_sealed_audit_private_report_gateway_evidence_unavailable_rejected
run_sealed_audit_private_report_gateway_evidence_accountability_rejected
run_sealed_audit_private_report_gateway_evidence_plaintext_rejected
```

## Security Impact

Mercury now has fourteen sealed-audit layers:

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
14. accepted-only private report gateway-unavailability and operator-accountability evidence persistence

The fourteenth layer keeps timeout/unavailability reports from becoming forged incidents, retry-exhaustion bypasses, operator-accountability gaps, or metadata-leak paths.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_private_report_gateway_evidence
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
```

Simulator checks:

```powershell
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_private_report_gateway_evidence_ready
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_private_report_gateway_evidence_ready
```

Run the full preflight before pushing the increment:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
