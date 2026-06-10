# Production Local Store Open Gate

Generated: 2026-05-28

## Status

Mercury now has a production-facing local-store open gate in `mercury-core`:

```text
LocalStoreProductionOpenInput
LocalStoreProductionOpenDecision
LocalStoreProductionOpenReason
LocalStoreCrashRecoveryState
evaluate_local_store_production_open(...)
```

This is still not the production encrypted database implementation. It is the backend contract a future SQLite/page-encryption or custom encrypted database adapter must satisfy before loading records or message keys.

## Database Format Checks

The production open gate accepts only a clean manifest with:

- a supported unlock decision from `LocalStoreUnlockInput`
- matching Mercury store header magic
- `mercury_local_store_v1` sealing suite code
- expected header nonce length
- expected header authentication tag length
- at least one required sealed key slot
- zero plaintext key slots
- a device-local root key scope
- a positive root key generation
- a clean crash-recovery state

Accepted output enables:

```text
can_open_database = true
can_load_records = true
can_load_message_keys = true
```

## Key Hierarchy Contract

The open manifest treats the database root as device-local material. The root key must be sealed, device scoped, and generation tracked. Future production adapters should derive or unwrap record keys only after this gate accepts.

Plaintext key slots are fatal and require destructive repair. Missing sealed key slots require recovery rather than silent database open.

## Migration And Repair Classes

Stable rejection labels:

```text
UNLOCK_REJECTED
HEADER_MAGIC_MISMATCH
HEADER_SUITE_MISMATCH
BAD_HEADER_NONCE_LENGTH
BAD_HEADER_TAG_LENGTH
KEY_SLOT_MISSING
PLAINTEXT_KEY_SLOT_FORBIDDEN
ROOT_KEY_SCOPE_MISMATCH
ROOT_KEY_GENERATION_INVALID
WAL_REPLAY_REQUIRED
DIRTY_SHUTDOWN_WITHOUT_WAL
WAL_REPLAY_FAILED
```

The gate separates:

- `requires_user_auth`
- `requires_recovery`
- `requires_migration`
- `requires_crash_recovery`
- `requires_destructive_repair`

This lets mobile and desktop shells route to authentication, recovery, migration, write-ahead-log replay, or destructive repair without parsing database internals.

## Crash Safety Lifecycle

Crash-recovery states are explicit:

```text
CLEAN
WAL_REPLAY_REQUIRED
DIRTY_WITHOUT_WAL
REPLAY_FAILED
```

`WAL_REPLAY_REQUIRED` does not allow normal record loading. It only enables `can_replay_wal`, so the backend can replay or discard pending transactional data before loading message keys. Dirty shutdown without a valid write-ahead log and failed replay both require destructive repair.

## App Lock Lifecycle

App-lock state remains owned by the unlock gate. `LocalStoreProductionOpenDecision` embeds the unlock decision and propagates user-auth, recovery, migration, and destructive-repair flags when unlock fails. A future OS keychain/keystore adapter should call the unlock gate first, then call this production open gate with the authenticated header manifest.

## Verification

Run:

```powershell
cargo test -p mercury-core --test local_store_production_open
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype local_store_production_open_wal_replay_required
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers clean accepted manifests, unlock rejection propagation, bad header shape, bad key-slot manifests, crash-recovery routing, and stable codes/labels for production-open reasons and crash states.

Checked prototype fixtures cover ready, write-ahead-log replay required, plaintext-key-slot forbidden, and app-lock propagation states.

## Next Backend Step

The keychain/keystore adapter contract is documented in `docs/49_KEYCHAIN_KEYSTORE_ADAPTER_CONTRACT.md`. The follow-on database profile gate is documented in `docs/114_LOCAL_STORE_DATABASE_SECURITY.md`, and the production adapter selection/provenance gate is documented in `docs/116_LOCAL_STORE_DATABASE_ADAPTER_SELECTION.md`.
