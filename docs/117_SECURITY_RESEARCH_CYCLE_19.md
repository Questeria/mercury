# Security Research Cycle 19: Database Adapter Selection And Provenance

Generated: 2026-05-28

## Sources Reviewed

- SQLCipher license information: <https://www.zetetic.net/sqlcipher/license/>
- SQLCipher FIPS 140-3 package notes: <https://www.zetetic.net/sqlcipher/fips/>
- SQLCipher FIPS 140-3 non-proprietary security policy: <https://csrc.nist.gov/CSRC/media/projects/cryptographic-module-validation-program/documents/security-policies/140sp5185.pdf>
- NIST FIPS 140-3: <https://csrc.nist.gov/pubs/fips/140-3/final>
- SQLCipher README and source-authenticity notes: <https://github.com/sqlcipher/sqlcipher>
- SQLCipher API documentation: <https://www.zetetic.net/sqlcipher/sqlcipher-api/>
- SQLite extension-loading API documentation: <https://www.sqlite.org/c3ref/enable_load_extension.html>
- SQLite PRAGMA documentation for `trusted_schema`, `secure_delete`, temp-store behavior, and debug-only pragmas: <https://sqlite.org/pragma.html>
- `rusqlite` README: <https://github.com/rusqlite/rusqlite>
- `rusqlite` feature list: <https://docs.rs/crate/rusqlite/latest/features>
- SQLCipher for Android repository: <https://github.com/sqlcipher/sqlcipher-android>

## Finding

After the local database security profile gate, the next practical risk was adapter selection. A future platform integration could satisfy the abstract database profile but still ship a weak production boundary by linking the wrong package, trial license, unsupported platform build, unverified source, non-FIPS module in a FIPS-labeled deployment, SQLCipher without codec hooks, unsafe SQLite extension/trusted-schema settings, missing migration/crash drills, or unsigned artifacts without SBOM/CVE monitoring.

The research pass supports a stricter stance:

- SQLCipher Community has BSD-style redistribution duties, while trial packages are not production redistribution packages.
- SQLCipher Enterprise FIPS has explicit runtime and attestation requirements; Mercury must not claim FIPS posture from package naming alone.
- SQLCipher v4+ compatibility and migration settings matter because major-version defaults can change.
- Rust can plausibly reach SQLCipher through `rusqlite`/`libsqlite3-sys` bundled or external SQLCipher paths, but that does not prove platform package support by itself.
- SQLite extension loading and trusted schema must be disabled for a messenger that may handle attacker-influenced local database state.
- Signed artifacts, source authenticity, SBOM, and CVE monitoring are part of the cryptographic trust boundary once the encrypted database is a native dependency.

## Increment

Added a local database adapter selection gate that:

- requires an accepted local database security decision
- rejects plaintext or unknown adapter kinds
- rejects unknown binding paths
- rejects unsupported target platforms
- rejects trial, unknown, or non-redistributable licenses
- rejects SQLCipher major versions below 4
- requires verified SQLite and SQLCipher source provenance
- requires documented crypto provider identity
- requires FIPS validation, runtime self-tests, and runtime FIPS-mode checks when FIPS is requested
- requires SQLCipher codec and extra init/shutdown build configuration when SQLCipher is used
- requires memory temp-store, extension-loading disabled, trusted schema disabled, secure delete, memory security, integrity check on open, and current-major compatibility
- requires deterministic migration and crash-recovery drills
- requires signed artifacts, SBOM, and CVE monitoring
- rejects debug SQLCipher/plaintext logging
- exposes checked fixtures and backend commands for accepted, license, FIPS, migration, and supply-chain states

## Security Impact

Mercury now has an explicit deployment gate between "the database profile would be safe" and "this native encrypted database adapter may ship." That prevents future UI/platform code from accidentally treating an unsafe SQLCipher/custom build as production-ready.

## Verification

Focused checks:

```powershell
cargo test -p mercury-core --test local_store_database_adapter_selection
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype local_store_database_adapter_selection_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_local_store_database_adapter_selection_ready
```

## Next Research Target

Completed in `docs/118_MLS_PROVIDER_ADAPTER_SELECTION.md` and `docs/119_SECURITY_RESEARCH_CYCLE_20.md`.

Next, study production sealed local backup/export and multi-device restore designs. The next backend increment should make recovery/export safe for group MLS state and account secrets without creating plaintext cloud-backup, local-cache, or device-transfer bypasses.
