# Media Download Session

Generated: 2026-05-28

## Status

Mercury now has a prototype media download session in `mercury-core`, checked fixtures in `fixtures/prototypes`, and backend command surfaces in `mercury-bindings`.

The session composes:

```text
MediaServiceDownloadInput
PrototypeEncryptedLocalStore
LocalStoreOpenRequest
PrototypeLocalStoreCryptoProvider
```

It records one received-attachment transcript for the path that verifies media-service download readiness, writes only sealed media ciphertext to local storage, and runs local open checks without exposing plaintext bytes in events or outcomes.

## Session Flow

The happy path is:

1. start media download session
2. evaluate media-service download gate
3. cache downloaded sealed ciphertext as `MediaCiphertext`
4. evaluate local open metadata and open through the local crypto provider
5. finish session

The session uses the actual downloaded ciphertext length for the download gate so callers cannot pass stale or misleading ciphertext metadata.

## Stop Points

The session stops when:

- the media-service download gate rejects the object
- local-store policy rejects the sealed media write
- local open metadata rejects before decrypt
- local open returns a plaintext length mismatch

Every stop point preserves `plaintext_exposed = false`.

## Event Transcript

Stable event kinds:

```text
download_started
media_service_download_evaluated
local_store_write_evaluated
local_store_open_evaluated
download_finished
```

Stable terminal reasons:

```text
completed
media_service_download_rejected
local_store_write_rejected
local_store_open_rejected
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_media_download_session --test media_service_download --test local_store_crypto_provider
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover accepted download/cache/open, download-gate rejection, local-store write rejection, local-open rejection, actual downloaded ciphertext length use, and stable reason/event labels.

## Binding Fixtures

Prototype fixtures:

```text
media_download_session_happy_path
media_download_session_download_rejected
media_download_session_store_write_rejected
media_download_session_open_rejected
```

Backend commands:

```text
run_media_download_session_happy_path
run_media_download_session_download_rejected
run_media_download_session_store_write_rejected
run_media_download_session_open_rejected
```

## Next Backend Step

Expose checked media retention fixtures and backend commands through `mercury-bindings`, the platform bridge, and the UI simulator.
