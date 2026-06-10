# Keychain Keystore Adapter Contract

Generated: 2026-05-28

## Status

Mercury now has a platform keychain/keystore adapter contract in `mercury-core`:

```text
LocalStoreKeychainUnlockInput
LocalStoreKeychainUnlockDecision
LocalStoreKeychainBackend
LocalStoreKeychainProtection
LocalStoreKeychainReason
evaluate_local_store_keychain_unlock(...)
```

This is not an OS integration yet. It is the contract future iOS, Android, macOS, Windows, and Linux adapters must satisfy before feeding `LocalStoreUnlockInput` into the local-store unlock gate.

## Backend Classes

Stable backend labels:

```text
ios_keychain
android_keystore
macos_keychain
windows_credential_vault
linux_secret_service
development_memory
```

Stable protection labels:

```text
hardware_backed
os_protected
development_only
```

Development-only backends are rejected unless `allow_development_backend` is explicitly set. That keeps local test scaffolding from becoming a silent production storage backend.

## Rejection Classes

Stable reason labels:

```text
BACKEND_UNAVAILABLE
DEVELOPMENT_BACKEND_FORBIDDEN
USER_AUTH_REQUIRED
DEVICE_SECRET_MISSING
DEVICE_SECRET_CORRUPT
PLAINTEXT_SECRET_FORBIDDEN
EXPORTABLE_SECRET_FORBIDDEN
```

The adapter contract rejects:

- unavailable keychain/keystore backends
- development-only storage in production mode
- exportable device secrets
- plaintext device secrets
- missing or corrupt sealed device secrets
- unsatisfied user authentication

Accepted output sets `can_build_unlock_input = true` and carries the exact `LocalStoreUnlockInput` the next gate should evaluate.

## Security Rules

- Platform code must not pass raw key bytes through this contract.
- Device secrets must be present as sealed material and non-exportable.
- User-auth-required stores must block until the app lock or OS authentication is satisfied.
- Header authentication failures are left to the local-store unlock gate so the keychain adapter does not parse encrypted database internals.
- Development memory storage must be opt-in and test-only.

## Verification

Run:

```powershell
cargo test -p mercury-core --test local_store_keychain_unlock
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype local_store_keychain_exportable_secret_forbidden
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers accepted hardware-backed unlock input, unavailable backends, dev-only backend blocking, missing/corrupt/plaintext/exportable secrets, user-auth separation, database-header rejection handoff, and stable codes/labels.

Checked prototype fixtures cover hardware-backed ready, user-auth required, exportable-secret forbidden, and development-backend forbidden states.

## Next Backend Step

The production local-store adapter trait shape is documented in `docs/50_PRODUCTION_LOCAL_STORE_ADAPTER_TRAIT.md`.
