# Media Upload Session Prototype

Generated: 2026-05-28

## Status

Mercury now has a deterministic attachment upload orchestration in `mercury-core`:

```text
PrototypeMediaUploadSession
PrototypeMediaUploadSessionInput
PrototypeMediaUploadSessionOutcome
PrototypeMediaUploadSessionReason
PrototypeMediaUploadSessionEvent
PrototypeMediaUploadSessionEventKind
```

This is not a production media server. It proves the backend-side order of operations for attachment upload before UI, mobile FFI, and real object storage are connected.

## Flow

```text
local media bytes
  -> local-store crypto seal
  -> media object-store gate
  -> encrypted local-store write
```

The session only writes a local media record after:

- the local-store seal request accepts for `MediaCiphertext`
- the media object-store gate accepts sealed ciphertext metadata
- plaintext upload bytes are zero
- automatic download is not requested
- outbound send and media-sealing decisions are accepted
- local encrypted-store policy accepts the final sealed record

## Event Transcript

Media upload sessions emit plaintext-free progress events:

```text
upload_started
local_store_seal_evaluated
media_object_store_evaluated
local_store_write_evaluated
upload_finished
```

Each event has stable kind/reason codes, labels, terminal state, acceptance state, and `plaintext_bytes_exposed = false`.

## Stop Points

Stable media upload session reason labels:

```text
completed
local_store_seal_rejected
media_object_store_rejected
local_store_write_rejected
```

Rejected branches stop before later side effects:

- seal rejection stops before media gate evaluation or local persistence
- media object-store rejection stops before local persistence
- store-write rejection leaves the encrypted local store unchanged

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_media_upload_session
cargo test -p mercury-core --test media_object_store --test local_store_crypto_provider
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover the accepted upload path, plaintext upload rejection, local seal rejection, local-store policy rejection, stable reason codes/labels, plaintext-free event transcripts, checked fixture drift, backend command envelopes, platform bridge JSON, and UI simulator exposure.

## Binding Fixtures

Prototype fixtures:

```text
media_upload_session_happy_path
media_upload_session_plaintext_rejected
media_upload_session_seal_rejected
media_upload_session_store_write_rejected
```

Backend commands:

```text
run_media_upload_session_happy_path
run_media_upload_session_plaintext_rejected
run_media_upload_session_seal_rejected
run_media_upload_session_store_write_rejected
```

## Next Backend Step

Expose checked media retention fixtures and backend commands through `mercury-bindings`, the platform bridge, and the UI simulator.
