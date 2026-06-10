# Security Research Cycle 39: Restore-Time Backup Policy Reconstruction

Generated: 2026-06-08

## Primary Source Refresh

- RFC 9106, Argon2 parameter selection and explicit cost inputs: <https://www.ietf.org/rfc/rfc9106.html>
- Libsodium password hashing guidance on self-describing parameter storage and `crypto_pwhash_str_needs_rehash`: <https://libsodium.gitbook.io/doc/password_hashing/default_phf>
- Signal Secure Backups product and design docs: <https://signal.org/blog/introducing-secure-backups/> and <https://support.signal.org/hc/en-us/articles/9708267671322-Signal-Secure-Backups>
- NIST SP 800-57 Part 1 Rev. 5 key-management guidance for backup, archive, and recovery controls: <https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final>

## Finding

Cycle 38 added a restore-side KDF migration signal, but Mercury still exposed the backup gate input only at backup creation time through `CreatedBackup::restore_input`. That was enough to prove the engine generated an acceptable archive, but it was not the right backend surface for a real restore flow.

Restore-time policy decisions need to be reconstructed from:

1. the sealed archive's encoded cryptographic profile
2. the current deployment attestation
3. the restore context Mercury does not yet encode in the blob, such as scope, transport, and retention policy

The primary-source refresh points in the same direction:

- RFC 9106 treats Argon2 memory and iteration costs as explicit security parameters.
- Libsodium's official API stores algorithm and cost metadata alongside the hash and expects applications to make a fresh `needs_rehash` decision later.
- Signal's secure-backup design treats restore as a first-class security event, not just decryption of old bytes.
- NIST SP 800-57 frames backup, archive, and recovery as lifecycle controls that need current policy enforcement, not one-time admission.

## Increment

This cycle lands the narrow backend restore-planning helper in `mercury-backup`:

- `SealedBackup::plan_restore(...)` now reconstructs `SecureBackupRestoreInput` from the archive's encoded KDF profile plus the caller's current restore context and deployment attestation.
- The helper evaluates the backup gate immediately and returns both the reconstructed input and decision, along with the existing KDF migration recommendation.
- Backup creation now uses the same shared restore-input builder, so create-time and restore-time policy inputs stay aligned.

## Security Effect

Mercury no longer needs to treat the creation-time `restore_input` as the only policy surface for backups. A future restore/session orchestration layer can now:

1. parse a sealed archive
2. reconstruct the gate input from the real archive header and current attestation
3. reject restores whose current deployment posture no longer satisfies Mercury's backup policy
4. separately decide whether a successfully opened archive should be re-sealed under the current KDF profile

That reduces the risk of restore code drifting into one of two bad states:

- trusting stale create-time policy facts during a later restore, or
- ignoring the archive's encoded KDF profile and evaluating restore against today's defaults instead of the actual stored parameters

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

1. Thread `SealedBackup::plan_restore(...)` into the first real restore/session orchestration boundary instead of passing around create-time `restore_input`.
2. When Mercury adds MLS-bearing backups, extend restore planning to supply true MLS restore facts rather than the current non-MLS placeholder fields.
3. Add a non-secret audit record for restore planning that captures the gate reason and KDF migration status without logging recovery-code-derived data.
