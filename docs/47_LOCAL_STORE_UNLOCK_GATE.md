# Local Store Unlock Gate

Generated: 2026-05-28

## Status

Mercury now has a production-facing local-store unlock decision gate in `mercury-core`:

```text
LocalStoreUnlockInput
LocalStoreUnlockDecision
LocalStoreUnlockReason
evaluate_local_store_unlock(...)
```

This is the backend decision that future OS keychain, keystore, encrypted database, app-lock, and recovery adapters should call before opening local data or loading message keys.

## Accepted State

Accepted unlock requires:

- supported local-store version
- no plaintext cache records
- no pending recovery
- OS keychain or keystore available
- sealed device secret present
- authenticated database header
- app lock satisfied

When accepted, the decision allows:

```text
can_open_database = true
can_unseal_device_secret = true
can_load_message_keys = true
```

## Rejection Classes

Stable rejection labels:

```text
UNSUPPORTED_STORE_VERSION
PLAINTEXT_CACHE_FORBIDDEN
RECOVERY_REQUIRED
KEYCHAIN_UNAVAILABLE
DEVICE_SECRET_MISSING
DEVICE_SECRET_CORRUPT
PLAINTEXT_SECRET_FORBIDDEN
DATABASE_HEADER_MISSING
DATABASE_HEADER_CORRUPT
DATABASE_HEADER_AUTHENTICATION_FAILED
APP_LOCK_REQUIRED
```

The decision separates:

- `requires_user_auth`
- `requires_recovery`
- `requires_migration`
- `requires_destructive_repair`

That lets platform shells choose the right backend flow without interpreting raw store internals.

## Security Rules

- Unsupported store versions do not open the database and require migration.
- Plaintext cache records block unlock before keychain use.
- Plaintext device secrets require destructive repair, not silent import.
- Missing or corrupt device secrets require recovery.
- Corrupt database headers require destructive repair.
- Header authentication failure and app-lock failure require user authentication.

## Verification

Run:

```powershell
cargo test -p mercury-core --test local_store_unlock
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype local_store_unlock_app_lock_required
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused core test covers acceptance, version rejection, plaintext cache rejection, recovery paths, keychain/app-lock auth paths, bad database headers, and stable reason labels.

Checked prototype fixtures now cover ready, app-lock, recovery, and plaintext-cache-forbidden unlock states.

## Next Backend Step

The production local-store open gate is documented in `docs/48_PRODUCTION_LOCAL_STORE_OPEN_GATE.md`.
