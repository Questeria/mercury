# Indexed Media Download Session

Generated: 2026-05-28

## Status

Mercury now has a core composed download prototype that checks the media object index store before running the media download session:

```text
PrototypeIndexedMediaDownloadSession
PrototypeIndexedMediaDownloadSessionInput
PrototypeIndexedMediaDownloadSessionOutcome
PrototypeIndexedMediaDownloadSessionReason
PrototypeIndexedMediaDownloadSessionEvent
PrototypeIndexedMediaDownloadSessionEventKind
```

It is exposed through `mercury-bindings` as checked prototype fixtures, backend command envelopes, platform bridge bodies, and simulator CLI commands.

## Session Flow

The accepted path is:

1. start indexed download
2. write/evaluate the manifest snapshot through `PrototypeMediaObjectIndexStore`
3. require the accepted manifest to expose `can_download = true`
4. run `PrototypeMediaDownloadSession`
5. finish indexed download

The download session is attempted only after the manifest store accepts and the lifecycle state is downloadable.

## Stop Points

The session stops when:

- media object index store rejects the manifest snapshot
- the accepted manifest is not downloadable
- media download session rejects

In all cases:

- no plaintext media bytes are exposed in events
- `plaintext_exposed = false`
- the download session is skipped unless the manifest can download

## Event Transcript

Stable event kinds:

```text
indexed_download_started
media_object_index_store_evaluated
media_download_session_evaluated
indexed_download_finished
```

Stable terminal reasons:

```text
completed
media_object_index_store_rejected
media_object_not_downloadable
media_download_rejected
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_indexed_media_download_session --test prototype_media_download_session --test media_object_index_store
cargo test -p mercury-bindings --tests
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover accepted manifest plus download, manifest-store rejection, non-downloadable manifest gating, download-session rejection, stable reason labels, stable event labels, and plaintext-free event metadata.

## Binding Contracts

Checked fixtures:

```text
indexed_media_download_session_happy_path
indexed_media_download_session_manifest_rejected
indexed_media_download_session_not_downloadable
indexed_media_download_session_download_rejected
```

Backend commands:

```text
run_indexed_media_download_session_happy_path
run_indexed_media_download_session_manifest_rejected
run_indexed_media_download_session_not_downloadable
run_indexed_media_download_session_download_rejected
```

The command and bridge results use surface `prototype_indexed_media_download_session`, include the manifest-store decision, optional media-download outcome, `index_store_records`, and a plaintext-free `events` transcript.

## Next Backend Step

The production media object index open gate is documented in `docs/72_MEDIA_OBJECT_INDEX_PRODUCTION_OPEN_GATE.md`. The remaining backend step is the real encrypted media object index database behind `ProductionMediaObjectIndexStoreAdapter`.
