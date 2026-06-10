# Indexed Media Cleanup Session

Generated: 2026-05-28

## Status

Mercury now has a core composed cleanup prototype that checks the media object index store before running the media cleanup session:

```text
PrototypeIndexedMediaCleanupSession
PrototypeIndexedMediaCleanupSessionInput
PrototypeIndexedMediaCleanupSessionOutcome
PrototypeIndexedMediaCleanupSessionReason
PrototypeIndexedMediaCleanupSessionEvent
PrototypeIndexedMediaCleanupSessionEventKind
```

It is exposed through `mercury-bindings` as checked prototype fixtures, backend command envelopes, platform bridge bodies, and simulator CLI commands.

## Session Flow

The accepted path is:

1. start indexed cleanup
2. write/evaluate the manifest snapshot through `PrototypeMediaObjectIndexStore`
3. require the accepted manifest to expose `can_cleanup = true`
4. run `PrototypeMediaCleanupSession`
5. finish indexed cleanup

The cleanup session is attempted only after the manifest store accepts and the lifecycle state is cleanable.

## Stop Points

The session stops when:

- media object index store rejects the manifest snapshot
- the accepted manifest is not cleanable
- media cleanup session rejects

In all cases:

- no plaintext media bytes are exposed in events
- `plaintext_exposed = false`
- remote delete and local sealed-cache eviction are skipped unless the manifest can cleanup

## Event Transcript

Stable event kinds:

```text
indexed_cleanup_started
media_object_index_store_evaluated
media_cleanup_session_evaluated
indexed_cleanup_finished
```

Stable terminal reasons:

```text
completed
media_object_index_store_rejected
media_object_not_cleanable
media_cleanup_rejected
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_indexed_media_cleanup_session --test prototype_media_cleanup_session --test media_object_index_store
cargo test -p mercury-bindings --tests
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover accepted manifest plus cleanup, manifest-store rejection, non-cleanable manifest gating, cleanup-session rejection, stable reason labels, stable event labels, and plaintext-free event metadata.

## Binding Contracts

Checked fixtures:

```text
indexed_media_cleanup_session_happy_path
indexed_media_cleanup_session_manifest_rejected
indexed_media_cleanup_session_not_cleanable
indexed_media_cleanup_session_cleanup_rejected
```

Backend commands:

```text
run_indexed_media_cleanup_session_happy_path
run_indexed_media_cleanup_session_manifest_rejected
run_indexed_media_cleanup_session_not_cleanable
run_indexed_media_cleanup_session_cleanup_rejected
```

The command and bridge results use surface `prototype_indexed_media_cleanup_session`, include the manifest-store decision, optional media-cleanup outcome, `index_store_records`, and a plaintext-free `events` transcript.

## Next Backend Step

The production media object index open gate is documented in `docs/72_MEDIA_OBJECT_INDEX_PRODUCTION_OPEN_GATE.md`. The remaining backend step is the real encrypted media object index database behind `ProductionMediaObjectIndexStoreAdapter`.
