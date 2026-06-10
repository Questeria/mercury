# Media Object Store Gate

Generated: 2026-05-28

## Status

Mercury now has a core decision boundary for encrypted attachment/media upload readiness:

```text
MediaObjectStoreInput
MediaObjectStoreDecision
MediaObjectStoreReason
evaluate_media_object_store(...)
```

This is not a media server implementation. It is the backend contract a future media service and client upload adapter must satisfy before any attachment object can be uploaded or cached locally.

## Security Rules

- Plaintext media bytes are rejected before upload.
- Automatic media download requests are rejected by default.
- The existing outbound send gate must accept before media upload.
- The media must already be sealed under a `MediaCiphertext` record policy.
- Object IDs are opaque 32-byte identifiers.
- Ciphertext must be nonempty and bounded by `MERCURY_MAX_MEDIA_OBJECT_BYTES`.
- Sealed headers, content digests, and media-key commitments must have stable fixed bounds.
- Every outcome has `plaintext_bytes_exposed = false`.

## Stable Reasons

```text
ACCEPTED
PLAINTEXT_UPLOAD_FORBIDDEN
AUTOMATIC_DOWNLOAD_FORBIDDEN
OUTBOUND_SEND_REJECTED
MEDIA_SEALING_REJECTED
MEDIA_RECORD_KIND_MISMATCH
BAD_OBJECT_ID_LENGTH
BAD_CIPHERTEXT_LENGTH
CIPHERTEXT_TOO_LARGE
BAD_SEALED_HEADER_LENGTH
BAD_CONTENT_DIGEST_LENGTH
BAD_MEDIA_KEY_COMMITMENT_LENGTH
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test media_object_store
cargo test -p mercury-core --test prototype_media_upload_session
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused core tests cover accepted encrypted media upload, plaintext upload rejection, automatic download rejection, send/seal rejection, media record-kind mismatch, malformed metadata, size bounds, stable reason labels, and the prototype upload-session transcript. The binding tests cover fixture drift, backend command envelopes, platform bridge JSON, and UI simulator exposure.

## Prototype Upload Session

Mercury now also has a core media upload orchestration:

```text
PrototypeMediaUploadSession
PrototypeMediaUploadSessionInput
PrototypeMediaUploadSessionOutcome
PrototypeMediaUploadSessionReason
PrototypeMediaUploadSessionEvent
PrototypeMediaUploadSessionEventKind
```

The prototype session seals local media bytes, evaluates the media object-store gate against the sealed ciphertext metadata, then writes only a sealed `MediaCiphertext` record to the encrypted local-store prototype. Every event keeps `plaintext_bytes_exposed = false`.

## Binding Fixtures

Prototype fixtures:

```text
media_object_store_upload_ready
media_object_store_plaintext_rejected
media_object_store_auto_download_rejected
media_object_store_oversize_rejected
```

Backend commands:

```text
run_media_object_store_upload_ready
run_media_object_store_plaintext_rejected
run_media_object_store_auto_download_rejected
run_media_object_store_oversize_rejected
```

## Next Backend Step

Expose checked media retention fixtures and backend commands through `mercury-bindings`, the platform bridge, and the UI simulator.
