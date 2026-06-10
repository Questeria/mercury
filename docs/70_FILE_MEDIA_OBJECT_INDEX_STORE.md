# File Media Object Index Store

Generated: 2026-05-28

## Status

Mercury now has a file-backed prototype for accepted media object index manifests:

```text
PrototypeFileMediaObjectIndexStore
```

It uses the same `MediaObjectIndexStoreWrite` contract as `PrototypeMediaObjectIndexStore`, so every durable manifest still passes through the media object index gate first.

## What It Does

- Persists only accepted opaque media object manifest records.
- Uses hex-encoded object IDs for durable record paths.
- Writes object ID, content digest, media key commitment, lifecycle state, ciphertext length, cache/remote flags, digest verification state, and retention-hold state.
- Reopens records from disk and re-evaluates the stored manifest shape against the media object index gate.
- Rejects plaintext metadata and unaccepted manifests before creating a file.
- Preserves an existing durable manifest when a replacement write is rejected.
- Deletes durable manifest records idempotently.

## What It Does Not Do Yet

This is not the production media object database.

Still pending:

- Production database page encryption and migration format.
- Crash-safe transactional batching across manifest plus object-service state.
- Real object-service or CDN integration.
- Adapter injection into indexed upload, download, and cleanup sessions.
- Secure media cache separation and platform storage lifecycle.

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_file_media_object_index_store --test media_object_index_store
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover accepted manifest persistence across reopen, plaintext metadata rejection before file creation, rejected replacement preservation, idempotent delete, and compatibility with the in-memory media object index store tests.

## Next Backend Step

The media object index adapter boundary and indexed-session injection are documented in `docs/71_MEDIA_OBJECT_INDEX_ADAPTER.md`, and the production open gate is documented in `docs/72_MEDIA_OBJECT_INDEX_PRODUCTION_OPEN_GATE.md`. The remaining backend step is a real encrypted media object index database behind `ProductionMediaObjectIndexStoreAdapter`.
