# Security Research Cycle 35: Gateway Unavailability Evidence

Generated: 2026-05-28

## Sources Reviewed

- HTTP Semantics, RFC 9110: <https://www.rfc-editor.org/info/rfc9110/>
- HTTP Early Data, RFC 8470: <https://www.rfc-editor.org/rfc/rfc8470.html>
- Oblivious HTTP, RFC 9458: <https://www.ietf.org/rfc/rfc9458.html>
- Privacy Pass Architecture, RFC 9576: <https://www.ietf.org/rfc/rfc9576.html>
- Idempotency-Key HTTP Header Field draft: <https://datatracker.ietf.org/doc/html/draft-ietf-httpapi-idempotency-key-header>

## Finding

Private report reconciliation proves retry safety, but a missing receipt still needs a separate evidence boundary before Mercury can raise a user-visible gateway incident or notify an operator. A timeout, 425 replay risk, 429 rate limit, or 5xx gateway failure is not enough by itself: the client must not be able to forge unavailability, and the relay/gateway/operator evidence must not leak selectors.

RFC 9110 distinguishes transient service/gateway failures and provides `Retry-After` guidance for unavailable states. Mercury should therefore treat gateway-unavailable status as evidence only when it is bound to authenticated gateway/relay observations, target absence proof, retry exhaustion, and rate-limit continuity.

RFC 8470 reinforces that replay-prone requests require conservative handling. Mercury rejects gateway evidence unless the retry path has exhausted a reconciliation-approved schedule and the evidence cannot mark delivery or incident state from a replayable exchange.

RFC 9458 keeps client identity and plaintext request content separated across relay and gateway. Mercury therefore stores only digest-bound relay observation, gateway error, target absence, and accountability-route state; UI status is digest-only.

RFC 9576 and the Idempotency-Key draft preserve the same invariant as the prior reconciliation layer: unavailability evidence must not mint a fresh report, bypass rate limits, or correlate retries through selector-bearing metadata.

## Increment

Added the sealed audit private report gateway evidence store:

- `SealedAuditPrivateReportGatewayEvidenceReason`
- `SealedAuditPrivateReportGatewayEvidenceWrite`
- `SealedAuditPrivateReportGatewayEvidenceRecord`
- `SealedAuditPrivateReportGatewayEvidenceDecision`
- `AcceptedSealedAuditPrivateReportGatewayEvidenceWrite`
- `SealedAuditPrivateReportGatewayEvidenceStore`
- `PrototypeSealedAuditPrivateReportGatewayEvidenceStore`
- `evaluate_sealed_audit_private_report_gateway_evidence(...)`
- `put_sealed_audit_private_report_gateway_evidence_record(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The store accepts only reconciliation-approved, digest-only, encrypted, append-only gateway evidence records with authenticated unavailable evidence, signed relay observations, target absence proof binding, no client-asserted unavailability, retry exhaustion, rate-limit continuity, gateway key state binding, operator escalation routing, blinded failure buckets, private monitor routing, policy-gated user-visible incidents, and selector-free UI status.

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

The fourteenth layer keeps missing receipts from becoming forged, replay-prone, rate-limit-bypassing, or metadata-leaking gateway incidents.

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

Run full preflight before commit.

## Pause Resume Target

(Internal checkpoint note omitted from the public mirror.)
