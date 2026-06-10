# Security Research Cycle 37: Self-Describing Backup KDF Metadata

Generated: 2026-06-08

## Primary Source Refresh

- RFC 9106, Argon2 inputs and parameter requirements: <https://www.ietf.org/rfc/rfc9106.html>
- Libsodium `crypto_pwhash_*` guidance on saving algorithm and parameter metadata with the verifier: <https://libsodium.gitbook.io/doc/password_hashing/default_phf>
- Signal official secure-backups design note: <https://signal.org/blog/introducing-secure-backups/>

## Finding

Mercury's secure-backup engine already met the existing gate floor: 16-byte salt, Argon2id v0x13, `t=3`, and `64 MiB` memory. RFC 9106 treats the salt, memory, passes, parallelism, version, and type as explicit inputs to the KDF, not ambient build settings.

The concrete gap was backup-format agility. The original sealed-backup header stored only salt, nonce, and digests, so restore behavior depended on compile-time KDF constants that were not encoded into the blob itself. Libsodium's official guidance is explicit here: save the algorithm identifier and parameters alongside the stored verifier so later retuning does not require guessing old settings.

Signal's secure-backups rollout is the product-level cross-check: the recovery key is generated on-device and is the only way to unlock the archive. Long-term restore compatibility is therefore part of the security model, not just convenience.

## Increment

This cycle lands a narrow backend compatibility improvement in `mercury-backup`:

- new backups now emit a v2 sealed-backup header that records the KDF algorithm, memory cost, iterations, and parallelism
- restore still accepts legacy v1 backups whose header omitted that metadata
- the authenticated manifest now binds the version and, for v2, the serialized KDF metadata so header tampering fails closed
- regression tests prove that v1 backups still reopen and round-trip exactly while new v2 backups expose self-describing KDF metadata

## Security Effect

Mercury no longer relies on hidden compile-time Argon2id settings to interpret every future backup blob. The live backup format is now explicit enough to support future KDF retuning or migration work without silently breaking old archives or guessing legacy parameters during restore.

This does not yet change Mercury's actual Argon2id cost profile. It removes a format-level agility trap while preserving restore compatibility.

## Verification

Focused checks for this increment:

```powershell
cargo test -p mercury-backup
```

Full repo preflight remains the release gate before push:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Exact Next Steps

1. Add a `needs_rehash` style decision to backup restore planning so Mercury can distinguish "old but acceptable" KDF profiles from current defaults.
2. Decide whether Mercury wants a higher-memory profile for high-security accounts or stronger desktop-class devices after compatibility is proven in production.
3. If Mercury changes the default KDF profile again, add a restore-path metric or audit event that records which sealed-backup header version was consumed, without logging secret-derived data.
