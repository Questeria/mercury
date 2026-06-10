# Indexed Media Upload Session

Generated: 2026-05-28

## Status

Mercury now has a core composed upload prototype that runs media service upload and then writes the media object index store:

```text
PrototypeIndexedMediaUploadSession
PrototypeIndexedMediaUploadSessionInput
PrototypeIndexedMediaUploadSessionOutcome
PrototypeIndexedMediaUploadSessionReason
PrototypeIndexedMediaUploadSessionEvent
PrototypeIndexedMediaUploadSessionEventKind
```

It is exposed through `mercury-bindings` as checked prototype fixtures, backend command envelopes, platform bridge bodies, and simulator CLI commands.

## Session Flow

The accepted path is:

1. start indexed upload
2. run `PrototypeMediaServiceUploadSession`
3. evaluate authenticated media-service upload readiness
4. write the accepted `MediaObjectIndexStoreWrite`
5. finish indexed upload

The index-store write is attempted only after the service upload completes. If media upload or media-service authorization fails, the manifest store is not mutated.

## Stop Points

The session stops when:

- media service upload rejects before object-service side effects complete
- media object index store rejects the manifest snapshot

In both cases:

- no plaintext media bytes are exposed in events
- `plaintext_exposed = false`
- the index store remains empty unless the store write itself accepted

## Event Transcript

Stable event kinds:

```text
indexed_upload_started
media_service_upload_evaluated
media_object_index_store_evaluated
indexed_upload_finished
```

Stable terminal reasons:

```text
completed
media_service_upload_rejected
media_object_index_store_rejected
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_indexed_media_upload_session --test prototype_media_service_upload_session --test media_object_index_store
cargo test -p mercury-bindings --tests
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover accepted service upload plus manifest persistence, service rejection before store mutation, index-store rejection after service upload, stable reason labels, stable event labels, and plaintext-free event metadata.

## Binding Contracts

Checked fixtures:

```text
indexed_media_upload_session_happy_path
indexed_media_upload_session_service_rejected
indexed_media_upload_session_index_store_rejected
```

Backend commands:

```text
run_indexed_media_upload_session_happy_path
run_indexed_media_upload_session_service_rejected
run_indexed_media_upload_session_index_store_rejected
```

The command and bridge results use surface `prototype_indexed_media_upload_session`, include the nested service-upload outcome, optional index-store decision, `index_store_records`, and a plaintext-free `events` transcript.

## Next Backend Step

The production media object index open gate is documented in `docs/72_MEDIA_OBJECT_INDEX_PRODUCTION_OPEN_GATE.md`. The remaining backend step is the real encrypted media object index database behind `ProductionMediaObjectIndexStoreAdapter`.
