# Media Service Download Gate

Generated: 2026-05-28

## Status

Mercury now has a core media service download gate in `mercury-core`, checked fixtures in `fixtures/prototypes`, and backend command surfaces in `mercury-bindings`:

```text
MediaServiceDownloadInput
MediaServiceDownloadDecision
MediaServiceDownloadReason
MediaServiceDownloadAdapter
download_media_object_with_adapter(...)
```

This is the receive-side complement to the media upload service gate. It is not a production media server implementation. It defines the checks a future object-storage download path must pass before received attachment ciphertext can be downloaded, cached locally, or handed to a local open path.

## What It Blocks

The gate rejects:

- plaintext preview bytes
- automatic/background download requests
- plaintext/debug media adapters
- unapproved development media adapters
- unauthenticated media services
- missing download authorization
- unbound object namespaces
- unverified content digests
- malformed object IDs
- malformed, empty, or oversized ciphertext metadata
- malformed sealed-header, content-digest, or media-key commitment lengths

Accepted decisions expose:

```text
can_download_object = true
can_persist_local_ciphertext = true
forbids_plaintext_preview = true
plaintext_bytes_exposed = false
```

Rejected decisions keep download and local-persistence capability false and preserve `plaintext_bytes_exposed = false`.

## Adapter Boundary

`download_media_object_with_adapter(...)` evaluates `MediaServiceDownloadInput` first. It calls `MediaServiceDownloadAdapter::download_accepted_media(...)` only after acceptance.

This gives future media implementations a narrow place to bind:

- authenticated small-scale object storage
- self-hosted media blob service
- private S3-compatible object storage
- CDN-backed ciphertext fetches
- approved development-only media stores

Plaintext previews and automatic downloads are never accepted by this gate.

## Verification

Run:

```powershell
cargo test -p mercury-core --test media_service_download --test media_service_adapter --test media_object_store
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover production acceptance, accepted-only adapter calls, plaintext-preview rejection, automatic-download rejection, plaintext/development adapter rejection, auth/namespace/digest requirements, metadata bounds, and stable reason labels.

## Binding Fixtures

Prototype fixtures:

```text
media_service_download_ready
media_service_download_plaintext_preview_rejected
media_service_download_auth_missing
media_service_download_digest_unverified
```

Backend commands:

```text
run_media_service_download_ready
run_media_service_download_plaintext_preview_rejected
run_media_service_download_auth_missing
run_media_service_download_digest_unverified
```

## Next Backend Step

Expose checked media retention fixtures and backend commands through `mercury-bindings`, the platform bridge, and the UI simulator.
