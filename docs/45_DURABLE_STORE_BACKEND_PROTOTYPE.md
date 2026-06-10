# Durable Store Backend Prototype

Generated: 2026-05-28

## Status

Mercury now has a file-backed encrypted-store prototype in `mercury-core`:

```text
PrototypeFileEncryptedLocalStore
```

It implements the same `EncryptedLocalStoreAdapter` boundary as the memory-only `PrototypeEncryptedLocalStore`, so every durable write still passes through `LocalStoreWriteRequest` and the local-store policy gate first.

## What It Does

- Persists accepted local-store records to disk.
- Uses hex-encoded namespace and record IDs for filesystem paths.
- Writes only typed payload classes already accepted by policy:
  - sealed bytes
  - hash digest bytes
  - public metadata bytes
- Reopens records from disk and re-evaluates the stored shape against local-store policy.
- Rejects plaintext-forbidden records before any file is created.
- Leaves existing durable records untouched when a replacement write is rejected.
- Deletes durable records by locator.

## What It Does Not Do Yet

This is not the production encrypted database.

Still pending:

- OS keychain or keystore integration.
- Database page encryption.
- Migration format.
- Crash-safe transactional batching.
- Secret zeroization guarantees.
- Media cache separation.
- App-lock and unlock lifecycle.
- Production cryptographic storage envelope.

## Stable Record Codes

`LocalStoreRecordKind` now has stable `code`, `label`, and `from_code` helpers. `LocalStorePayloadKind` has the same helper shape. The file-store prototype uses those stable codes in its on-disk record header.

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_file_encrypted_store
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers:

- accepted sealed record persistence across reopen
- rejected plaintext record creates no file
- rejected policy replacement preserves the existing durable record
- durable delete removes records and remains idempotent

## Next Backend Step

The platform bridge contract is documented in `docs/46_PLATFORM_BRIDGE_CONTRACT.md`.
