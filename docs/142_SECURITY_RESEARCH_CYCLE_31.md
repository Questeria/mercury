# Security Research Cycle 31: Production Sealed-Audit Storage And Private Reports

Generated: 2026-05-28

## Sources Reviewed

- Oblivious HTTP, RFC 9458: <https://www.ietf.org/rfc/rfc9458.html>
- Privacy Pass Architecture, RFC 9576: <https://www.ietf.org/rfc/rfc9576.html>
- SQLCipher Design: <https://www.zetetic.net/sqlcipher/design/>
- SQLCipher API: <https://www.zetetic.net/sqlcipher/sqlcipher-api/>
- SQLite Write-Ahead Logging: <https://www.sqlite.org/wal.html>
- SQLite Atomic Commit: <https://www2.sqlite.org/atomiccommit.html>
- NIST SP 800-111: <https://csrc.nist.gov/pubs/sp/800/111/final>

## Finding

Recovery/export state is not enough by itself. A production sealed-audit deployment also needs storage and report-submission constraints that preserve the same privacy and rollback guarantees after the restored state is operational.

RFC 9458 is useful for Mercury's incident-report path because it separates request relay from gateway processing and is explicitly suited to sensitive telemetry-style submissions when the application carries no linking state between requests. RFC 9576 gives the anonymous-token architecture needed for rate limiting without falling back to IP identity.

SQLCipher's design reinforces the database requirements: page-level encryption, per-page authentication, encrypted journal/WAL data, memory wiping, and disabling file-based temporary stores matter for a local sealed-audit database. SQLite's WAL documentation adds a deployment rule: WAL files are part of persistent database state and checkpoint policy is a correctness/security property, not just performance tuning. NIST SP 800-111 keeps the threat model anchored on device loss and unauthorized storage access.

For Mercury, the right backend increment is a checked adapter boundary rather than direct database code: require accepted recovery/export state, encrypted and authenticated database pages, encrypted WAL/temp behavior, append-only schema constraints, migration/crash drills, and private OHTTP/Privacy-Pass-style incident report submission before UI or platform code can treat production sealed-audit operations as available.

## Increment

Added the sealed-audit database adapter and private report transport gates:

- `SealedAuditDatabaseAdapterReason`
- `SealedAuditDatabaseAdapterInput`
- `SealedAuditDatabaseAdapterDecision`
- `evaluate_sealed_audit_database_adapter(...)`
- `SealedAuditPrivateReportTransportReason`
- `SealedAuditPrivateReportTransportInput`
- `SealedAuditPrivateReportTransportDecision`
- `evaluate_sealed_audit_private_report_transport(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The database gate rejects recovery-export failures, unencrypted database profiles, missing append-only guards, missing migration/crash drills, plaintext headers/selectors/metadata/schema, and malformed digest shapes.

The report transport gate rejects database-adapter failures, missing OHTTP/HPKE/state-free transport properties, missing anonymous rate-limit token controls, missing replay/retry guards, plaintext selector exposure, and malformed digest shapes.

## Security Impact

Mercury now has ten sealed-audit layers:

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

The tenth layer prevents production storage or incident-report submission from bypassing encryption-at-rest, append-only, migration, checkpoint, replay, rate-limit, and selector-redaction requirements.

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

Run full preflight before commit.

This target is implemented by `docs/143_SEALED_AUDIT_PRIVATE_REPORT_OUTBOX.md` and researched in `docs/144_SECURITY_RESEARCH_CYCLE_32.md`.

## Next Research Target

Design the private report delivery receipt and gateway transparency boundary:

- gateway receipt verification without linking reports to users
- relay/gateway key transparency and rotation evidence
- retry completion state that cannot falsely mark reports delivered
- blinded failure classification for unavailable gateways or spent tokens
- monitor-side proof that submitted reports reached the intended transparency/audit workflow
