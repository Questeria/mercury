# Platform Local Store Adapter Gate

Generated: 2026-05-28

## Status

Mercury now has a pre-open platform adapter gate in `mercury-core`:

```text
PlatformLocalStoreRuntime
PlatformLocalStoreAdapterKind
PlatformLocalStoreAdapterInput
PlatformLocalStoreAdapterDecision
PlatformLocalStoreAdapterFactory
open_platform_local_store_adapter(...)
```

This sits before the production local-store adapter and models the desktop/mobile shell boundary that chooses an OS-backed storage implementation.

## What It Blocks

The gate rejects:

- unknown runtimes
- plaintext file-store adapters
- prototype or development adapters unless explicitly allowed
- missing database roots
- unavailable OS keychain or keystore services
- mobile adapters without hardware-backed key storage
- unsatisfied app-lock/user-auth state

Accepted decisions expose:

```text
can_open_adapter = true
forbids_plaintext_storage = true
```

Rejected decisions keep `can_open_adapter = false` and surface stable setup/auth/hardware booleans for platform shells.

## Factory Boundary

`open_platform_local_store_adapter(...)` evaluates `PlatformLocalStoreAdapterInput` first. It calls `PlatformLocalStoreAdapterFactory::open_platform_adapter(...)` only after acceptance.

This gives desktop and mobile implementations a narrow place to bind:

- Windows/macOS/Linux encrypted database adapters
- Android Keystore-backed local stores
- iOS Keychain/Secure Enclave-backed local stores
- approved development-only prototype stores

Plaintext durable storage is never an accepted adapter kind.

## Verification

Run:

```powershell
cargo test -p mercury-core --test platform_local_store_adapter
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers desktop acceptance, mobile hardware-key requirements, plaintext and development-adapter rejection, setup/auth flags, accepted-only factory opening, and stable runtime/adapter/reason codes.

## Simulator And Bridge Fixtures

Prototype fixture names:

```text
platform_local_store_adapter_desktop_ready
platform_local_store_adapter_mobile_hardware_required
platform_local_store_adapter_plaintext_forbidden
platform_local_store_adapter_app_lock_required
```

Backend command names:

```text
run_platform_local_store_adapter_desktop_ready
run_platform_local_store_adapter_mobile_hardware_required
run_platform_local_store_adapter_plaintext_forbidden
run_platform_local_store_adapter_app_lock_required
```

These fixtures and commands let UI, desktop, and mobile shells request local-store adapter readiness without duplicating security logic.

## Next Backend Step

Add platform adapter gate coverage to any future desktop/mobile FFI packaging once those package targets exist.
