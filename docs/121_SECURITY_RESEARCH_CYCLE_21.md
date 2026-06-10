# Security Research Cycle 21: Secure Backup And Restore

Generated: 2026-05-28

## Sources Reviewed

- Signal Secure Backups: <https://signal.org/blog/introducing-secure-backups/>
- Signal Secure Value Recovery: <https://signal.org/blog/secure-value-recovery/>
- Apple Platform Security, iCloud Keychain escrow: <https://support.apple.com/en-euro/guide/security/sec3e341e75d/web>
- Android Auto Backup documentation: <https://developer.android.com/identity/data/autobackup>
- NIST SP 800-57 Part 1 Rev. 5: <https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final>
- RFC 9420, The Messaging Layer Security Protocol: <https://www.rfc-editor.org/info/rfc9420/>
- Kintsugi: Decentralized E2EE Key Recovery: <https://arxiv.org/abs/2507.21122>

## Finding

Backup and restore are one of the easiest ways to accidentally defeat an encrypted messenger. Even if message transport, MLS state, local storage, and media storage are correctly encrypted, a recovery archive can become a plaintext or weakly protected copy of the entire account.

The research points to a conservative Mercury contract:

- recovery must remain opt-in and impossible to unlock without user-held or threshold-held secret material
- high-entropy archive keys are safer than memorable PINs
- if lower-entropy material is ever supported, it needs online rate limiting, threshold participation, or hardware-backed attempt control
- server-backed restore needs authentication, rate limiting, and opaque identifiers
- OS-level automatic backup is a separate side channel and must be explicitly excluded for Mercury secrets
- backup manifests need authentication and replay protection
- restored MLS state must be sealed, epoch-bound, and followed by device and group rekeying so restore does not undo MLS forward secrecy or post-compromise recovery
- retention limits are security policy, not storage housekeeping

## Increment

Added `evaluate_secure_backup_restore(...)` with checked fixtures and backend command envelopes. The new gate rejects:

- rejected account recovery
- unknown scope or transport
- non-portable archive encryption suites
- backup keys below 192 bits for standard accounts or 256 bits for high-security accounts
- missing key digests or weak KDF policy
- missing device approval, threshold quorum, server authentication, server rate limiting, or opaque account identifiers
- unencrypted backup material
- plaintext export fields
- OS plaintext backup inclusion
- unsealed or epoch-unbound MLS state
- missing device/group rekey on restore
- unauthenticated manifests
- missing replay protection
- missing audit digest
- unbounded retention

## Security Impact

Mercury now has a hard backend boundary that prevents backup, restore, local export, cloud archive, or device transfer from becoming a plaintext bypass around local encrypted storage and MLS group-state security. This does not make Mercury "government-proof"; it makes the next implementation step auditable and deliberately hostile to the most common recovery-system failure modes.

## Verification

Focused checks:

```powershell
cargo test -p mercury-core --test secure_backup_restore
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype secure_backup_restore_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_secure_backup_restore_ready
```

## Next Research Target

Completed by `docs/123_SEALED_AUDIT_EVENT_CHAIN.md` and `docs/124_SECURITY_RESEARCH_CYCLE_22.md`. Follow-on research should focus on production sealed-audit storage, checkpoint signing keys, witness gossip, and privacy-preserving monitor queries.
