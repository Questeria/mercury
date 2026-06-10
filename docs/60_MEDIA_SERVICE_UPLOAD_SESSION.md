# Media Service Upload Session

Generated: 2026-05-28

## Status

Mercury now has a prototype media service upload session in `mercury-core`, checked fixtures in `fixtures/prototypes`, and backend command surfaces in `mercury-bindings`.

The session composes:

```text
PrototypeMediaUploadSession
MediaServiceAdapterInput
MediaServiceAdapterDecision
```

It records one operation transcript for the path that seals media locally, checks the media object-store gate, writes only sealed ciphertext to the local store, and then verifies that a future media service adapter is allowed to upload the ciphertext object.

## Session Flow

The happy path is:

1. start service upload session
2. run local media upload session
3. require accepted media object-store decision
4. evaluate media service adapter readiness
5. record one service upload call
6. finish session

The session does not model production networking yet. The service upload call count is an accepted-only placeholder for the future object-storage adapter call.

## Stop Points

The session stops before remote upload when:

- local media upload rejects the object
- the media object-store gate rejects plaintext, oversized, auto-download, or malformed media
- the media service adapter rejects missing authentication
- upload authorization is missing
- the object namespace is not bound
- the content digest is not verified
- a plaintext/debug adapter is selected
- a development adapter is selected without explicit development approval

Every stop point preserves `plaintext_exposed = false`.

## Event Transcript

Stable event kinds:

```text
service_upload_started
media_upload_session_evaluated
media_service_adapter_evaluated
service_upload_finished
```

Stable terminal reasons:

```text
completed
media_upload_rejected
media_service_adapter_rejected
```

UI and binding code should use labels and capability booleans rather than reimplementing policy decisions.

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_media_service_upload_session --test media_service_adapter --test prototype_media_upload_session
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover the plaintext-free happy path, media-upload rejection, media-service adapter rejection, stable reason/event codes, checked fixture drift, backend command envelopes, platform bridge JSON, and UI simulator exposure.

## Binding Fixtures

Prototype fixtures:

```text
media_service_upload_session_happy_path
media_service_upload_session_media_rejected
media_service_upload_session_auth_rejected
media_service_upload_session_digest_unverified
```

Backend commands:

```text
run_media_service_upload_session_happy_path
run_media_service_upload_session_media_rejected
run_media_service_upload_session_auth_rejected
run_media_service_upload_session_digest_unverified
```

## Next Backend Step

Expose checked media retention fixtures and backend commands through `mercury-bindings`, the platform bridge, and the UI simulator.
