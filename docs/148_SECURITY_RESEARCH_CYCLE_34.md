# Security Research Cycle 34: Private Report Retry Reconciliation

Generated: 2026-05-28

## Sources Reviewed

- HTTP Semantics, RFC 9110: <https://www.rfc-editor.org/info/rfc9110/>
- HTTP Early Data, RFC 8470: <https://www.rfc-editor.org/rfc/rfc8470.html>
- Oblivious HTTP, RFC 9458: <https://www.rfc-editor.org/rfc/rfc9458.html>
- Privacy Pass Architecture, RFC 9576: <https://www.ietf.org/rfc/rfc9576.html>
- Idempotency-Key HTTP Header Field draft: <https://datatracker.ietf.org/doc/html/draft-ietf-httpapi-idempotency-key-header>

## Finding

The private report receipt gate proves delivery, but retry workers still need a separate safety boundary. A retry after an ambiguous network result is a state-changing operation: if it is not idempotency-bound, replay-aware, and rate-limit-continuity-preserving, it can duplicate reports, consume new anonymous rate-limit capacity, or let a UI mark delivery before receipt-backed reconciliation.

RFC 9110's retry model only treats idempotent requests as inherently safe to retry. Mercury private reports are not safe by default, so the retry worker must carry a durable idempotency binding and reject duplicate retry acceptance.

RFC 8470's replay guidance reinforces that ambiguous or replay-prone exchanges must not become completion evidence. Mercury therefore requires delivered state to be receipt-bound and rejects false delivery state even when retry metadata exists.

RFC 9458 keeps the OHTTP privacy split intact: retry status cannot expose relay-observed or gateway-observed selectors. Reconciliation records must stay encrypted, digest-only, and UI-safe.

RFC 9576 and the Idempotency-Key draft both push the same operational invariant for Mercury: retries must preserve the original authorization/rate-limit continuity and must not mint a new report under a fresh token or unrelated request identity.

## Increment

Added the sealed audit private report reconciliation store:

- `SealedAuditPrivateReportReconciliationReason`
- `SealedAuditPrivateReportReconciliationWrite`
- `SealedAuditPrivateReportReconciliationRecord`
- `SealedAuditPrivateReportReconciliationDecision`
- `AcceptedSealedAuditPrivateReportReconciliationWrite`
- `SealedAuditPrivateReportReconciliationStore`
- `PrototypeSealedAuditPrivateReportReconciliationStore`
- `evaluate_sealed_audit_private_report_reconciliation(...)`
- `put_sealed_audit_private_report_reconciliation_record(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The store accepts only receipt-approved, digest-only, encrypted, append-only reconciliation records with retry schedule binding, retry idempotency binding, duplicate retry rejection, no retry after delivered receipt, rate-limit token spend preservation, no fresh report minting, crash recovery cursor binding, pending-only resume, false delivery rejection, operator accountability routing, blinded failure buckets, and selector-free UI status.

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

The thirteenth layer keeps retry scheduling from becoming a duplicate-report, rate-limit-bypass, false-delivery, or metadata-leak path.

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

Run full preflight before commit.

## Next Research Target

Design the private report operator-accountability and unavailable-gateway evidence store:

- receipt-missing escalation without selector leakage
- gateway unavailability evidence that cannot be forged by clients
- relay/gateway/operator accountability routing with blinded buckets
- retry exhaustion evidence that remains unlinkable to the reporting user
- policy for when a missing receipt becomes a user-visible security incident

This target has now been implemented by `docs/149_SEALED_AUDIT_PRIVATE_REPORT_GATEWAY_EVIDENCE.md` and `docs/150_SECURITY_RESEARCH_CYCLE_35.md`.
