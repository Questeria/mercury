# Local Store Database Security Gate

Generated: 2026-05-28

## Status

Mercury now has a production-facing local database security profile gate in `mercury-core`:

```text
LocalStoreDatabaseEngine
LocalStoreDatabaseCipher
LocalStoreDatabaseKdf
LocalStoreDatabaseSecurityInput
LocalStoreDatabaseSecurityDecision
LocalStoreDatabaseSecurityReason
evaluate_local_store_database_security(...)
```

This is not a production encrypted database implementation. It is the contract a future SQLCipher-style or custom page-encrypted SQLite adapter must satisfy after the platform adapter and production-open gates accept.

## Accepted Profile

The accepted profile requires:

- accepted `PlatformLocalStoreAdapterDecision`
- accepted `LocalStoreProductionOpenDecision`
- non-plaintext database engine
- accepted database cipher profile
- raw key wrapped by platform keystore, or a sufficiently strong KDF
- 4096-byte encrypted pages with per-page random nonces
- per-page authentication
- separate encryption and MAC keys
- unique database salt
- encrypted WAL and journal files
- memory-only temporary storage for non-transaction temp files
- zero plaintext header bytes
- cloud/OS backup exclusion plus consistent encrypted snapshot backups
- secure delete enabled
- memory locking and key zeroization
- tested crash recovery
- zero plaintext metadata fields
- SQLite extension loading disabled
- debug plaintext export disabled

Accepted output enables:

```text
can_open_database = true
can_load_records = true
can_load_message_keys = true
can_host_mls_transactions = true
```

## Why This Exists

The earlier production-open gate proves that a Mercury store manifest is shaped correctly. This gate proves that the underlying database profile is safe enough to host message records, media indexes, MLS replay stores, Welcome outbox rows, and membership transaction witnesses.

The gate follows current storage guidance:

- SQLCipher encrypts pages and WAL page data, authenticates page writes, uses per-database salts, and supports raw key material for vaulted/platform-key use.
- SQLite WAL and temporary-file behavior must be configured deliberately; temp files other than transaction journals can otherwise hit disk.
- Android and Apple platform guidance supports hardware/secure-enclave-backed key wrapping where available.
- NIST key-management guidance treats master/key-derivation keys as lifecycle-managed secrets with cryptoperiod and destruction requirements.

## Checked Fixtures

Prototype fixtures:

```text
local_store_database_security_ready
local_store_database_security_plaintext_rejected
local_store_database_security_wal_rejected
local_store_database_security_backup_rejected
local_store_database_security_secret_lifecycle_rejected
```

Backend commands:

```text
run_local_store_database_security_ready
run_local_store_database_security_plaintext_rejected
run_local_store_database_security_wal_rejected
run_local_store_database_security_backup_rejected
run_local_store_database_security_secret_lifecycle_rejected
```

## Rejection Classes

Stable rejection labels:

```text
PLATFORM_ADAPTER_REJECTED
PRODUCTION_OPEN_REJECTED
PLAINTEXT_DATABASE_FORBIDDEN
WEAK_CIPHER_SUITE
KDF_TOO_WEAK
PAGE_SHAPE_REJECTED
MISSING_PER_PAGE_AUTHENTICATION
MAC_KEY_REUSE_FORBIDDEN
DATABASE_SALT_MISSING
KEY_NOT_KEYSTORE_WRAPPED
WAL_OR_JOURNAL_PLAINTEXT
FILE_TEMP_STORE_FORBIDDEN
PLAINTEXT_HEADER_FORBIDDEN
BACKUP_POLICY_REJECTED
SECURE_DELETE_MISSING
SECRET_LIFECYCLE_REJECTED
CRASH_RECOVERY_UNTESTED
PLAINTEXT_METADATA_FORBIDDEN
EXTENSION_LOADING_FORBIDDEN
DEBUG_EXPORT_FORBIDDEN
```

## UI And Platform Rules

UI and platform code must not open production local storage for message keys, MLS state, media indexes, or transaction records unless this gate accepts.

Treat these as hard stops:

- `PLAINTEXT_DATABASE_FORBIDDEN`: replace adapter or destructive repair.
- `WAL_OR_JOURNAL_PLAINTEXT`: do not load records; repair database configuration and crash-recovery posture.
- `BACKUP_POLICY_REJECTED`: disable unsafe OS/cloud backup or use a consistent encrypted snapshot path.
- `SECRET_LIFECYCLE_REJECTED`: do not ship; fix memory/key lifecycle before production use.

## Verification

Run:

```powershell
cargo test -p mercury-core --test local_store_database_security
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype local_store_database_security_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_local_store_database_security_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Production Adapter Notes

The future adapter should prefer a reviewed encrypted SQLite stack such as SQLCipher where licensing and platform packaging permit. If Mercury uses a custom page-encryption layer, it must still satisfy equivalent page authentication, nonce, key separation, WAL/journal, temporary-file, backup, and secret-lifecycle evidence before this gate can accept.

The follow-on adapter-selection gate is documented in `docs/116_LOCAL_STORE_DATABASE_ADAPTER_SELECTION.md`. It blocks unsafe SQLCipher/custom builds, unsupported platform packages, trial or unknown licenses, missing FIPS runtime evidence, unsafe SQLite runtime settings, missing migration/crash drills, and weak supply-chain evidence before a database adapter can be treated as shippable.
