# Local Store Database Adapter Selection Gate

Generated: 2026-05-28

## Status

Mercury now has a production database adapter selection gate in `mercury-core`:

```text
LocalStoreDatabaseAdapterKind
LocalStoreDatabaseBindingKind
LocalStoreDatabaseTargetPlatform
LocalStoreDatabaseLicenseKind
LocalStoreDatabaseAdapterSelectionInput
LocalStoreDatabaseAdapterSelectionDecision
LocalStoreDatabaseAdapterSelectionReason
evaluate_local_store_database_adapter_selection(...)
```

This is not the production SQLCipher or custom database implementation. It is the gate that prevents an unsafe build, package, license, FIPS posture, or runtime configuration from being treated as the production encrypted local store.

## Accepted Profile

The accepted profile requires:

- accepted `LocalStoreDatabaseSecurityDecision`
- non-plaintext adapter kind
- known Rust/platform binding path
- supported target platform package
- redistribution-safe license posture
- SQLCipher major version 4 or later when SQLCipher is used
- verified SQLite and SQLCipher source provenance
- documented crypto provider
- FIPS validation, runtime self-tests, and runtime FIPS-mode check when FIPS is requested
- SQLCipher codec and extra init/shutdown compile configuration when SQLCipher is used
- memory-only temp-store configuration
- SQLite extension loading disabled
- SQLite trusted schema disabled
- secure delete configured
- SQLCipher memory-security posture enabled
- SQLCipher integrity check on open
- current-major SQLCipher compatibility mode
- deterministic migration drill
- crash-recovery drill
- signed release artifacts
- SBOM plus CVE monitoring
- debug SQLCipher/plaintext logging disabled

Accepted output enables:

```text
can_link_adapter = true
can_open_database = true
can_ship_release = true
can_host_mls_transactions = true
```

## Checked Fixtures

Prototype fixtures:

```text
local_store_database_adapter_selection_ready
local_store_database_adapter_selection_license_rejected
local_store_database_adapter_selection_fips_rejected
local_store_database_adapter_selection_migration_rejected
local_store_database_adapter_selection_supply_chain_rejected
```

Backend commands:

```text
run_local_store_database_adapter_selection_ready
run_local_store_database_adapter_selection_license_rejected
run_local_store_database_adapter_selection_fips_rejected
run_local_store_database_adapter_selection_migration_rejected
run_local_store_database_adapter_selection_supply_chain_rejected
```

## Rejection Classes

Stable rejection labels:

```text
DATABASE_PROFILE_REJECTED
ADAPTER_KIND_REJECTED
BINDING_KIND_REJECTED
PLATFORM_UNSUPPORTED
LICENSE_REJECTED
SQLCIPHER_VERSION_TOO_OLD
SOURCE_AUTHENTICITY_MISSING
CRYPTO_PROVIDER_UNVERIFIED
FIPS_VALIDATION_MISSING
FIPS_RUNTIME_CHECK_MISSING
SQLCIPHER_CODEC_NOT_ENABLED
UNSAFE_SQLITE_CONFIGURATION
EXTENSION_LOADING_ENABLED
TRUSTED_SCHEMA_ENABLED
SECURE_DELETE_MISSING
MEMORY_SECURITY_MISSING
INTEGRITY_CHECK_MISSING
COMPATIBILITY_MODE_REJECTED
MIGRATION_DRILL_MISSING
CRASH_RECOVERY_DRILL_MISSING
UNSIGNED_RELEASE_ARTIFACT
SBOM_OR_CVE_MONITORING_MISSING
DEBUG_SQLCIPHER_LOGGING_ENABLED
```

## UI And Platform Rules

UI and platform code must not treat the production local store as shippable unless this gate accepts.

Treat these as hard stops:

- `LICENSE_REJECTED`: do not ship with a trial, unknown, or non-redistributable adapter package.
- `FIPS_VALIDATION_MISSING` or `FIPS_RUNTIME_CHECK_MISSING`: do not present the deployment as FIPS-capable.
- `SOURCE_AUTHENTICITY_MISSING`, `UNSIGNED_RELEASE_ARTIFACT`, or `SBOM_OR_CVE_MONITORING_MISSING`: route to supply-chain remediation.
- `MIGRATION_DRILL_MISSING` or `CRASH_RECOVERY_DRILL_MISSING`: do not migrate user stores or host MLS transaction records.
- `EXTENSION_LOADING_ENABLED` or `TRUSTED_SCHEMA_ENABLED`: do not open attacker-influenced databases.

## Verification

Run:

```powershell
cargo test -p mercury-core --test local_store_database_adapter_selection
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype local_store_database_adapter_selection_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_local_store_database_adapter_selection_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Production Adapter Notes

The preferred first path remains SQLCipher, because it has cross-platform history, Rust binding paths through `rusqlite`/`libsqlite3-sys`, and a FIPS Enterprise option. A custom page-encrypted SQLite path is allowed only if it can satisfy the same database security, source provenance, build hardening, migration, crash-recovery, and supply-chain evidence.
