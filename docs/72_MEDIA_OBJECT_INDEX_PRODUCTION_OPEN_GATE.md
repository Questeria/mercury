# Media Object Index Production Open Gate

Generated: 2026-05-28

## Status

Mercury now has a production-facing open gate for a future durable media object index:

```text
MediaObjectIndexProductionOpenInput
MediaObjectIndexProductionOpenDecision
MediaObjectIndexProductionOpenReason
ProductionMediaObjectIndexStoreAdapter
open_production_media_object_index(...)
replay_production_media_object_index_wal(...)
```

This is still not the final production media index database. It is the contract a production encrypted manifest/index implementation must satisfy before upload, download, cleanup, or object-service integration can load or write attachment manifests.

## Accepted Manifest

The gate accepts only a clean media index manifest with:

- supported `MERCURY_MEDIA_OBJECT_INDEX_VERSION`
- matching media index header magic
- `mercury_local_store_v1` sealing suite code
- expected nonce and authentication-tag lengths
- zero plaintext metadata rows
- zero plaintext cache paths
- object ID, content digest, and lifecycle indexes present
- bound media object namespace
- authenticated media service
- clean crash-recovery state

Accepted output enables:

```text
can_open_index = true
can_load_manifests = true
can_write_manifests = true
can_use_remote_objects = true
```

## Rejection Classes

Stable rejection labels:

```text
UNSUPPORTED_INDEX_VERSION
HEADER_MAGIC_MISMATCH
HEADER_SUITE_MISMATCH
BAD_HEADER_NONCE_LENGTH
BAD_HEADER_TAG_LENGTH
PLAINTEXT_METADATA_FORBIDDEN
PLAINTEXT_CACHE_PATH_FORBIDDEN
OBJECT_ID_INDEX_MISSING
CONTENT_DIGEST_INDEX_MISSING
LIFECYCLE_INDEX_MISSING
OBJECT_NAMESPACE_UNBOUND
MEDIA_SERVICE_UNAUTHENTICATED
WAL_REPLAY_REQUIRED
DIRTY_SHUTDOWN_WITHOUT_WAL
WAL_REPLAY_FAILED
```

The decision separates:

- `requires_network_setup`
- `requires_migration`
- `requires_crash_recovery`
- `requires_destructive_repair`

## Adapter Boundary

`ProductionMediaObjectIndexStoreAdapter` extends `MediaObjectIndexStoreAdapter` with explicit production open and WAL replay operations. The helper functions call adapter methods only after the gate accepts or explicitly allows WAL replay.

Prototype in-memory and file-backed media object index stores implement the production boundary as no-op opens/replays so session tests can exercise the same trait shape before a real encrypted index database exists.

## Checked Fixtures

Prototype fixtures:

```text
media_object_index_production_open_ready
media_object_index_production_open_wal_replay_required
media_object_index_production_open_plaintext_metadata_forbidden
media_object_index_production_open_namespace_unbound
```

These fixtures expose ready, crash-recovery, plaintext metadata repair, and network namespace setup states through the simulator.

## Verification

Run:

```powershell
cargo test -p mercury-core --test media_object_index_production_open
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype media_object_index_production_open_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused core test covers clean accepted manifests, header/schema rejection, plaintext metadata/cache-path rejection, missing indexes, network setup gates, crash-recovery routing, accepted-only adapter open, explicit WAL replay, and stable codes/labels.

## Next Backend Step

Implement the real encrypted media object index database behind `ProductionMediaObjectIndexStoreAdapter`, then connect it to the media object service and indexed upload/download/cleanup sessions.
