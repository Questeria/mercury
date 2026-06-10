# Security Research Cycle 32: Private Report Outbox And Submission Transcript

Generated: 2026-05-28

## Sources Reviewed

- Oblivious HTTP, RFC 9458: <https://www.rfc-editor.org/rfc/rfc9458.html>
- Privacy Pass Architecture, RFC 9576: <https://www.rfc-editor.org/rfc/rfc9576.html>
- The Privacy Pass HTTP Authentication Scheme, RFC 9577: <https://www.rfc-editor.org/rfc/rfc9577.html>

## Finding

The private report transport gate proves the shape of a safe report path, but production submission still needs a durable outbox boundary. Without it, a crash or retry loop could replay stale incidents, reuse unlinkability-sensitive state, spend anonymous rate-limit tokens incorrectly, or expose plaintext report selectors in UI status.

RFC 9458 reinforces that Mercury should treat request unlinkability as an end-to-end property of the relay, gateway, encapsulated request, encapsulated response, and client state. The report outbox therefore records only digests of the OHTTP request and response transcripts and rejects records that depend on cookies, account authentication, reusable client state, or non-private route selection.

RFC 9576 and RFC 9577 reinforce that abuse control should be token-based and unlinkable. The outbox therefore requires bound anonymous rate-limit token evidence and spend-once state without storing plaintext identity or selector metadata.

## Increment

Added the sealed audit private report outbox:

- `SealedAuditPrivateReportOutboxReason`
- `SealedAuditPrivateReportOutboxWrite`
- `SealedAuditPrivateReportOutboxRecord`
- `SealedAuditPrivateReportOutboxDecision`
- `AcceptedSealedAuditPrivateReportOutboxWrite`
- `SealedAuditPrivateReportOutboxStore`
- `PrototypeSealedAuditPrivateReportOutbox`
- `evaluate_sealed_audit_private_report_outbox(...)`
- `put_sealed_audit_private_report_outbox_record(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The store accepts only transport-approved, digest-only, encrypted, append-only report records with transcript digests, route privacy, replay-window binding, retry/backoff persistence, duplicate report rejection, anonymous rate-limit token spend-once state, and selector-free UI status.

## Security Impact

Mercury now has eleven sealed-audit layers:

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

The eleventh layer keeps report submission durable without turning retry state, rate-limit state, route selection, or UI status into a privacy leak.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_private_report_outbox
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
```

Simulator checks:

```powershell
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_private_report_outbox_ready
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_private_report_outbox_ready
```

Run full preflight before commit.

This target is implemented by `docs/145_SEALED_AUDIT_PRIVATE_REPORT_RECEIPT.md` and researched in `docs/146_SECURITY_RESEARCH_CYCLE_33.md`.

## Next Research Target

Design the private report retry worker and receipt reconciliation boundary:

- accepted-only transition from pending outbox to delivered receipt state
- retry scheduling that cannot create duplicate reports or bypass rate limits
- blinded failure buckets for user-visible status without selector leakage
- crash recovery that resumes pending reports without falsely completing them
- operator accountability when a gateway receipt never appears
