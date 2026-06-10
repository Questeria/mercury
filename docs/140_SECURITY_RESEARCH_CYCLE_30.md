# Security Research Cycle 30: Sealed-Audit Recovery Export And Cross-Device Sync

Generated: 2026-05-28

## Sources Reviewed

- SEEMless: Secure End-to-End Encrypted Messaging with less Trust: <https://www.microsoft.com/en-us/research/publication/seemless-secure-end-to-end-encrypted-messaging-with-less-trust/>
- SafetyPin: Encrypted Backups with Human-Memorable Secrets: <https://arxiv.org/abs/2010.06712>
- Forward Integrity and Crash Recovery for Secure Logs: <https://dblp.org/rec/journals/iacr/BlassN19>
- Rebound: Secure and Auditable State Rollback for Confidential Cloud Applications: <https://arxiv.org/abs/2511.13641>
- NIST SP 800-57 Part 1 Revision 6 Initial Public Draft: <https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-57pt1r6.ipd.pdf>

## Finding

The incident evidence store prevents split-view and missing-proof reporting from exposing audit selectors. The next risk is recovery: a cross-device export can reintroduce rollback, stale-policy, unauthorized restore, or selector leakage if restored audit state is trusted only because it decrypts.

SEEMless emphasizes that secure messaging recovery must account for key replacement and provider-managed key directories. SafetyPin shows the value of splitting recovery trust instead of relying on one server. Secure-log crash recovery research focuses on preserving forward integrity when recovery is necessary. Rebound frames rollback as something that needs explicit policy authorization and tamper-evident accounting. NIST key-management guidance highlights that recovery procedures need authorization, notification, accounting, and purpose limits.

For Mercury, sealed-audit recovery/export therefore needs a checked local state machine: accepted incident evidence first, encrypted export manifest second, restore quorum third, rollback guard fourth, and selector-free cross-device sync last.

## Increment

Added the sealed audit recovery export store:

- `SealedAuditRecoveryExportReason`
- `SealedAuditRecoveryExportWrite`
- `SealedAuditRecoveryExportRecord`
- `SealedAuditRecoveryExportDecision`
- `AcceptedSealedAuditRecoveryExportWrite`
- `SealedAuditRecoveryExportStore`
- `PrototypeSealedAuditRecoveryExportStore`
- `evaluate_sealed_audit_recovery_export(...)`
- `put_sealed_audit_recovery_export_record(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The store accepts only incident-evidence-approved, digest-only, encrypted, authenticated, device-bound export manifests with restore quorum, recovery-share quorum, rollback protection, audit-checkpoint verification, private cross-device sync, redacted incident selectors, append-only storage, and zero plaintext metadata.

## Security Impact

Mercury now has nine sealed-audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness
4. witness client and private monitor operation readiness
5. proof bundle persistence and offline verification readiness
6. accepted-only proof-cache persistence
7. verifier policy snapshot and private monitor freshness readiness
8. accepted-only incident evidence and privacy-preserving report readiness
9. accepted-only recovery/export and cross-device incident sync readiness

The ninth layer keeps recovered audit status from bypassing policy freshness, device authorization, rollback defenses, or selector redaction.

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

This target is implemented by `docs/141_SEALED_AUDIT_DATABASE_ADAPTER.md` and researched in `docs/142_SECURITY_RESEARCH_CYCLE_31.md`.

## Next Research Target

Design the private sealed-audit report outbox and submission transcript boundary:

- accepted-only report outbox persistence behind the private report transport gate
- OHTTP request/response transcript digests without request-linking state
- anonymous rate-limit token spend tracking without plaintext identity
- retry/backoff persistence with duplicate report rejection
- crash recovery for report submission without replaying stale incidents
