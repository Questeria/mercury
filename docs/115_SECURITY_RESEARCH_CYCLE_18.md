# Security Research Cycle 18: Local Database Security Profile

Generated: 2026-05-28

## Sources Reviewed

- SQLCipher design: <https://www.zetetic.net/sqlcipher/design/>
- SQLCipher API and keying notes: <https://www.zetetic.net/sqlcipher/sqlcipher-api/>
- SQLite write-ahead logging: <https://www.sqlite.org/wal.html>
- SQLite temporary files: <https://www.sqlite.org/tempfiles.html>
- Android Keystore system: <https://developer.android.com/privacy-and-security/keystore>
- Apple Keychain data protection: <https://support.apple.com/guide/security/keychain-data-protection-secb0694df1a/web>
- NIST SP 800-57 Part 1 Rev. 6 initial public draft: <https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-57pt1r6.ipd.pdf>
- KeyDroid paper: <https://arxiv.org/abs/2507.07927>

## Finding

Mercury had strong local-store policy gates, keychain gates, production-open gates, and MLS transaction invariants. The missing storage boundary was a single profile decision that says whether the actual database engine is safe enough to hold those records.

The important lesson from the research pass is that "SQLite database exists" and "store manifest opens" are too weak for Mercury. A secure messenger needs evidence that the database profile encrypts page data, authenticates page writes, protects WAL/journal and temporary files, excludes unsafe backups, wraps raw database keys with platform keystores, and handles memory/key lifecycle.

## Increment

Added a local database security profile gate that:

- rejects plaintext SQLite engines
- rejects weak or absent database ciphers
- rejects weak KDF settings when a KDF is used
- requires per-page random nonces and page authentication
- requires encryption and MAC key separation
- requires unique database salt
- requires platform-keystore wrapping for raw database keys
- requires encrypted WAL and journal files
- requires memory-only temp store behavior for non-transaction temporary files
- rejects plaintext database headers
- rejects unsafe backup policy
- requires secure delete, memory locking, and key zeroization
- requires tested crash recovery
- rejects plaintext metadata, SQLite extension loading, and debug plaintext exports
- exposes checked fixtures and backend commands for accepted, plaintext database, WAL/journal, backup, and secret-lifecycle states

## Security Impact

This turns the first real local database adapter into an evidence-backed profile instead of a trust assumption. UI and platform shells now have a backend command that can block production database use before message keys, MLS state, Welcome outbox rows, or membership transaction records are loaded.

## Verification

Focused checks:

```powershell
cargo test -p mercury-core --test local_store_database_security
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype local_store_database_security_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_local_store_database_security_ready
```

## Next Research Target

Completed in `docs/116_LOCAL_STORE_DATABASE_ADAPTER_SELECTION.md` and `docs/117_SECURITY_RESEARCH_CYCLE_19.md`.

The MLS provider adapter-selection/provenance gate was completed in `docs/118_MLS_PROVIDER_ADAPTER_SELECTION.md` and `docs/119_SECURITY_RESEARCH_CYCLE_20.md`.
