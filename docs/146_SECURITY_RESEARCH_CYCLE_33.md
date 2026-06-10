# Security Research Cycle 33: Private Report Delivery Receipts And Gateway Transparency

Generated: 2026-05-28

## Sources Reviewed

- Oblivious HTTP, RFC 9458: <https://www.rfc-editor.org/rfc/rfc9458.html>
- Privacy Pass Architecture, RFC 9576: <https://www.rfc-editor.org/rfc/rfc9576.html>
- Certificate Transparency Version 2.0, RFC 9162: <https://datatracker.ietf.org/doc/html/rfc9162>
- C2SP Transparency Log Checkpoints: <https://c2sp.org/tlog-checkpoint>
- C2SP Transparency Log Cosignatures: <https://c2sp.org/tlog-cosignature>
- C2SP Signed Notes: <https://c2sp.org/signed-note>

## Finding

The private report outbox makes submission attempts durable, but it does not prove delivery. The next privacy and integrity risk is false completion: a client, retry worker, gateway, or UI can accidentally or maliciously mark a report delivered before the gateway receipt, gateway-key transparency state, and monitor submission evidence are verified.

RFC 9458 matters because response errors and key-configuration failures can be visible outside the encapsulated response path. Mercury should not treat any unencapsulated error, stale key response, or relay-observed status as delivery proof. The receipt must be bound to the encapsulated response transcript and gateway key.

RFC 9576 keeps abuse controls from becoming identity controls: receipt state should prove token redemption/spend behavior indirectly, not store plaintext identity or rate-limit metadata.

RFC 9162 and the C2SP transparency specs reinforce that log trust requires signed tree heads or checkpoints plus inclusion/consistency evidence. For Mercury, gateway key and receipt transparency need digest-bound checkpoints, consistency proof evidence, key-rotation authentication, and monitor-side proof before delivery status becomes actionable.

## Increment

Added the sealed audit private report receipt store:

- `SealedAuditPrivateReportReceiptReason`
- `SealedAuditPrivateReportReceiptWrite`
- `SealedAuditPrivateReportReceiptRecord`
- `SealedAuditPrivateReportReceiptDecision`
- `AcceptedSealedAuditPrivateReportReceiptWrite`
- `SealedAuditPrivateReportReceiptStore`
- `PrototypeSealedAuditPrivateReportReceiptStore`
- `evaluate_sealed_audit_private_report_receipt(...)`
- `put_sealed_audit_private_report_receipt_record(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The store accepts only outbox-approved, digest-only, encrypted, append-only receipt records with gateway receipt signature verification, report/response/key binding, gateway-key transparency and consistency evidence, authenticated key rotation, private monitor proof, blinded failure classification, retry completion persistence, duplicate receipt rejection, delivery replay rejection, and selector-free UI status.

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

The twelfth layer keeps delivery completion from becoming a privacy leak or an unaudited trust shortcut.

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

Run full preflight before commit.

## Next Research Target

Implemented by:

- `docs/147_SEALED_AUDIT_PRIVATE_REPORT_RECONCILIATION.md`
- `docs/148_SECURITY_RESEARCH_CYCLE_34.md`

Next research target: private report operator-accountability and unavailable-gateway evidence:

- receipt-missing escalation without selector leakage
- gateway unavailability evidence that cannot be forged by clients
- relay/gateway/operator accountability routing with blinded buckets
- retry exhaustion evidence that remains unlinkable to the reporting user
- policy for when a missing receipt becomes a user-visible security incident
