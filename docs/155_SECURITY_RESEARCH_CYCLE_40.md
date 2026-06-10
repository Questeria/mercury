# Security Research Cycle 40: Restore-Planning Backup Audit Summary

Generated: 2026-06-08

## Primary Source Refresh

- RFC 9106, Argon2 inputs and explicit parameter requirements: <https://www.ietf.org/rfc/rfc9106.html>
- Libsodium password hashing guidance on storing algorithm and cost metadata plus `*_needs_rehash`: <https://libsodium.gitbook.io/doc/password_hashing/default_phf>
- Signal official secure-backups design note: <https://signal.org/blog/introducing-secure-backups/>
- Signal Secure Backups support documentation: <https://support.signal.org/hc/en-us/articles/9708267671322-Signal-Secure-Backups>
- NIST SP 800-57 Part 1 Rev. 5 key-management guidance covering backup, archive, recovery, and audit/accountability concerns: <https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final>

## Finding

Cycle 39 made restore planning recompute Mercury's secure-backup gate from the sealed archive's actual KDF profile plus the current deployment attestation. That was the right policy boundary, but it still left a gap for future restore orchestration and audit plumbing: there was no stable non-secret summary of what restore planning decided.

The refreshed primary sources still support adding that surface:

- RFC 9106 treats the salt, version, algorithm, memory, passes, and parallelism as explicit KDF inputs, not ambient implementation detail.
- Libsodium's official API stores those parameters with the artifact and exposes a later "rehash needed" style decision instead of forcing callers to guess what happened.
- Signal treats restore as a first-class security workflow and makes the recovery key the only way to unlock the backup.
- NIST SP 800-57 frames backup and recovery as lifecycle key-management controls that belong inside audit and accountability planning, not outside it.

## Increment

This cycle lands a narrow backend audit-view addition in `mercury-backup`:

- `BackupKdfMigrationStatus` now exposes stable code and label projections.
- `SealedBackup::plan_restore(...)` now returns a `BackupRestoreAuditRecord` alongside the existing restore input, decision, and migration status.
- The audit record contains only non-secret metadata: sealed-backup version, encoded KDF algorithm/cost profile, whether the profile is explicit/current, migration status, and the secure-backup gate reason.

It deliberately does not include recovery-code bytes, derived backup-key material, or plaintext archive contents.

## Security Effect

Mercury now has a backend-safe restore-planning summary that a future restore/session layer can hand to metrics, sealed-audit routing, or operator-visible accountability code without re-parsing the archive and without risking secret leakage.

That reduces two failure modes:

- restore code inventing its own ad hoc logging from sensitive backup state
- migration/accountability code losing the distinction between "accepted on the current profile", "accepted but needs reseal", and "rejected for current policy reasons"

## Verification

Focused checks for this increment:

```powershell
cargo test -p mercury-backup
```

Full repo preflight remains the broader regression pass:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Exact Next Steps

1. Thread `BackupRestoreAuditRecord` into the first real restore/session orchestration boundary instead of reconstructing log fields downstream.
2. When Mercury wires secure-backup restore into the sealed-audit path, keep this record digest-only and bind it to the later append-only audit contract instead of adding plaintext restore metadata.
3. If Mercury raises the default backup KDF profile again, preserve accept-plus-migrate behavior and use this audit record to distinguish legacy restores from current-profile restores.
