# Media Object Index Store

Generated: 2026-05-28

## Status

Mercury now has a core prototype store for accepted media object index snapshots, checked fixtures in `fixtures/prototypes`, and backend command surfaces in `mercury-bindings`:

```text
MediaObjectIndexStoreWrite
MediaObjectIndexStoreDecision
MediaObjectIndexStoreReason
MediaObjectIndexRecord
PrototypeMediaObjectIndexStore
PrototypeFileMediaObjectIndexStore
MediaObjectIndexStoreAdapter
```

This is not a production database yet. It is a plaintext-free contract surface and file-backed conformance harness for the future media object database.

## Purpose

The store gives upload, download, cleanup, and future media database work one narrow backend contract for persisting attachment lifecycle metadata after the media object index gate accepts it.

It stores only opaque media metadata:

- object ID bytes
- content digest bytes
- media key commitment bytes
- lifecycle state
- ciphertext length
- local cache presence
- remote object presence
- digest verification state
- retention-hold state

It does not store plaintext media, plaintext captions, plaintext filenames, or plaintext preview metadata.

## Write Rules

A write is accepted only when:

- the object ID is 32 bytes
- the content digest is 32 bytes
- the media key commitment is 32 bytes
- the embedded `MediaObjectIndexInput` accepts

Rejected writes do not mutate the store and still report `keeps_audit_hash = true` and `plaintext_bytes_exposed = false`.

Accepted writes are upserts keyed by object ID. The same object can move from `local_cached` to `remote_and_local_cached` without growing duplicate records. A terminal `deleted` snapshot can be persisted for audit while keeping upload, download, and cleanup capabilities closed.

## Verification

Run:

```powershell
cargo test -p mercury-core --test media_object_index_store --test media_object_index
cargo test -p mercury-core --test media_object_index_store_adapter --test prototype_file_media_object_index_store --test media_object_index_store
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover accepted persistence, plaintext-free records, index rejection without mutation, opaque byte length validation, lifecycle upserts, terminal deleted audit snapshots, stable store reason labels, adapter gating, file-backed reopen, rejected replacement preservation, fixture drift, backend command envelopes, platform bridge JSON, and UI simulator exposure.

## Binding Fixtures

Prototype fixtures:

```text
media_object_index_store_write_ready
media_object_index_store_index_rejected
media_object_index_store_bad_object_rejected
media_object_index_store_deleted_snapshot
```

Backend commands:

```text
run_media_object_index_store_write_ready
run_media_object_index_store_index_rejected
run_media_object_index_store_bad_object_rejected
run_media_object_index_store_deleted_snapshot
```

## Next Backend Step

The file-backed media object index store is documented in `docs/70_FILE_MEDIA_OBJECT_INDEX_STORE.md`, the adapter boundary plus indexed-session injection are documented in `docs/71_MEDIA_OBJECT_INDEX_ADAPTER.md`, and the production open gate is documented in `docs/72_MEDIA_OBJECT_INDEX_PRODUCTION_OPEN_GATE.md`. The remaining backend step is a real encrypted media object index database behind `ProductionMediaObjectIndexStoreAdapter`.
