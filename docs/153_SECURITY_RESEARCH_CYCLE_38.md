# Security Research Cycle 38: Restore-Side Backup KDF Migration Signal

Generated: 2026-06-08

## Primary Source Refresh

- RFC 9106, Argon2 inputs and recommended parameter profiles: <https://www.ietf.org/rfc/rfc9106.html>
- Libsodium `crypto_pwhash_*` guidance on stored parameter metadata and `*_needs_rehash`: <https://libsodium.gitbook.io/doc/password_hashing/default_phf>
- Signal official secure-backups design note: <https://signal.org/blog/introducing-secure-backups/>

## Finding

Cycle 37 fixed the format-agility problem by making new Mercury backups self-describing: v2 archives now encode the KDF algorithm and parameters in the header. That closed the ambiguity on future restores, but Mercury still lacked a restore-side signal for what to do after opening an older archive.

The current primary-source guidance points in the same direction:

- RFC 9106 treats the KDF inputs as explicit security parameters, not ambient implementation detail.
- Libsodium's official password-hashing API includes both embedded parameter metadata and a `needs_rehash` decision so applications can accept older material while upgrading it to current defaults later.
- Signal's secure-backups design makes long-term restore continuity part of the security model, not a convenience feature, because users may depend on old archives after device loss.

That means Mercury should distinguish three states during restore planning:

1. reject archives whose profile fails the secure-backup gate
2. accept archives whose profile is still safe enough to restore
3. separately mark accepted-but-legacy archives for post-restore re-sealing under the current profile

## Increment

This cycle lands the narrow backend piece of step 3 in `mercury-backup`:

- `SealedBackup` now reports whether it stores explicit KDF metadata
- `SealedBackup` now reports whether it already matches Mercury's current self-describing KDF profile
- `SealedBackup::kdf_migration_status()` now exposes a restore-planning recommendation: keep current v2 archives as-is, but recommend re-sealing legacy v1 or non-default v2 archives after a successful restore

The existing accept/reject gate is unchanged. This increment does not weaken the policy floor or silently rewrite archives during open. It only adds a precise signal the future restore flow can consume.

## Security Effect

Mercury can now treat legacy backup acceptance and backup migration as separate decisions. That reduces the risk of either:

- rejecting a still-valid archive only because it is older than today's defaults, or
- silently leaving a successfully restored archive on a legacy/non-default profile with no structured upgrade signal

The immediate practical benefit is for v1 archives created before self-describing KDF metadata existed. They still restore, but Mercury can now detect that they should be re-sealed into the current v2 profile once the product wires this helper into a real restore flow.

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

1. Thread `SealedBackup::kdf_migration_status()` through the future restore/session orchestration layer so a successful restore can offer or schedule a re-seal.
2. When Mercury eventually raises its default KDF cost, keep legacy profiles restorable but route them to the same post-restore migration path instead of inventing a second compatibility branch.
3. Add a non-secret audit or metric surface that records whether a restore consumed `CurrentProfile` or `ResealRecommended`, without logging recovery-code-derived data.
