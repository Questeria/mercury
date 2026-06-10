# Media Object Index Adapter

Generated: 2026-05-28

## Status

Mercury now has a production-shaped media object index adapter boundary in `mercury-core`:

```text
MediaObjectIndexStoreAdapter
AcceptedMediaObjectIndexStoreWrite
put_media_object_index_record(...)
read_media_object_index_record(...)
delete_media_object_index_record(...)
```

Both `PrototypeMediaObjectIndexStore` and `PrototypeFileMediaObjectIndexStore` implement the boundary.

Indexed upload, download, and cleanup sessions expose `run_with_index_store(...)` helpers so tests and future production callers can inject any adapter while the existing `run(...)` methods keep the default prototype command contracts stable.

## Write Flow

`put_media_object_index_record(...)` evaluates `MediaObjectIndexStoreWrite` before calling the adapter. Rejected writes return a `MediaObjectIndexStoreDecision` without mutating the adapter.

Accepted writes are wrapped in `AcceptedMediaObjectIndexStoreWrite`, so adapter implementations receive only manifest writes that already passed:

- object ID length validation
- content digest length validation
- media key commitment length validation
- media object index policy evaluation

## Read And Delete Flow

Reads and deletes use opaque object ID bytes only. The adapter surface does not accept plaintext media metadata, captions, filenames, preview text, or UI labels.

The file-backed prototype also revalidates records after reopen and rejects records whose object ID does not match their durable path.

## Verification

Run:

```powershell
cargo test -p mercury-core --test media_object_index_store_adapter --test media_object_index_store --test prototype_file_media_object_index_store
cargo test -p mercury-core --test prototype_indexed_media_upload_session --test prototype_indexed_media_download_session --test prototype_indexed_media_cleanup_session
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover accepted-only adapter writes, no adapter mutation on rejected metadata, opaque read/delete, in-memory conformance, file-backed conformance, and indexed upload/download/cleanup session injection.

## Next Backend Step

The production media object index open gate is documented in `docs/72_MEDIA_OBJECT_INDEX_PRODUCTION_OPEN_GATE.md`. The remaining backend step is the real encrypted media object index database behind `ProductionMediaObjectIndexStoreAdapter`.
