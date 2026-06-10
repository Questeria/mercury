# Media Object Index Gate

Generated: 2026-05-28

## Status

Mercury now has a core media object index/manifest gate for shared attachment lifecycle metadata, checked fixtures in `fixtures/prototypes`, and backend command surfaces in `mercury-bindings`:

```text
MediaObjectLifecycleState
MediaObjectIndexInput
MediaObjectIndexDecision
MediaObjectIndexReason
```

## Purpose

Upload, download, and cleanup sessions now have one core model for the object metadata they will eventually share through a real object database:

- opaque 32-byte object IDs
- opaque 32-byte content digests
- opaque 32-byte media-key commitments
- bounded ciphertext lengths
- verified digest state
- sealed local-cache presence
- remote object/service-record presence
- lifecycle state
- retention-hold cleanup blocking

The gate never accepts plaintext metadata bytes and never exposes plaintext bytes in its decision.

## Lifecycle States

Stable states:

```text
absent
local_cached
remote_stored
remote_and_local_cached
delete_pending
deleted
```

The state and presence flags must agree. For example, `remote_stored` requires a remote object and no local cache, while `remote_and_local_cached` requires both. `delete_pending` must still have at least one local or remote object to clean up. `deleted` is terminal and cannot be reused for upload or download.

## Capability Booleans

Accepted decisions expose:

```text
can_upload
can_download
can_cleanup
has_local_cache
has_remote_object
keeps_audit_hash
requires_user_action
plaintext_bytes_exposed
```

Rules:

- `can_upload` is true only for `absent` manifests with valid opaque metadata.
- `can_download` is true only when a remote object exists and the state is not terminal or pending deletion.
- `can_cleanup` is true only when a local cache or remote object exists and no retention hold is active.
- `keeps_audit_hash` remains true for accepted and rejected decisions.
- `plaintext_bytes_exposed` is always false.

## Rejections

Stable rejection labels:

```text
PLAINTEXT_METADATA_FORBIDDEN
MEDIA_RECORD_KIND_MISMATCH
BAD_OBJECT_ID_LENGTH
BAD_CONTENT_DIGEST_LENGTH
BAD_MEDIA_KEY_COMMITMENT_LENGTH
BAD_CIPHERTEXT_LENGTH
CIPHERTEXT_TOO_LARGE
CONTENT_DIGEST_UNVERIFIED
LOCAL_CACHE_WITHOUT_CIPHERTEXT
REMOTE_WITHOUT_SERVICE_RECORD
BAD_LIFECYCLE_STATE
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test media_object_index --test media_object_store --test media_retention
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover upload-only absent manifests, download/cleanup-ready remote-plus-local manifests, terminal deleted manifests, delete-pending cleanup, retention-hold cleanup blocking, plaintext metadata rejection, wrong record kinds, malformed opaque metadata, ciphertext bounds, unverified digests, inconsistent lifecycle state, missing remote service records, stable state/reason labels, fixture drift, backend command envelopes, platform bridge JSON, and UI simulator exposure.

## Binding Fixtures

Prototype fixtures:

```text
media_object_index_remote_and_local_ready
media_object_index_absent_upload_ready
media_object_index_delete_pending_ready
media_object_index_deleted_terminal
media_object_index_plaintext_metadata_rejected
media_object_index_bad_lifecycle_rejected
```

Backend commands:

```text
run_media_object_index_remote_and_local_ready
run_media_object_index_absent_upload_ready
run_media_object_index_delete_pending_ready
run_media_object_index_deleted_terminal
run_media_object_index_plaintext_metadata_rejected
run_media_object_index_bad_lifecycle_rejected
```

## Next Backend Step

The media object index store and file-backed media object index store prototypes are documented in `docs/66_MEDIA_OBJECT_INDEX_STORE.md` and `docs/70_FILE_MEDIA_OBJECT_INDEX_STORE.md`.
