# Media Service Adapter Gate

Generated: 2026-05-28

## Status

Mercury now has a production media service adapter contract in `mercury-core`:

```text
MediaServiceAdapterKind
MediaServiceAdapterInput
MediaServiceAdapterDecision
MediaServiceAdapter
upload_media_object_with_adapter(...)
```

This sits after the media object-store gate and prototype media upload session. It is the boundary a future object-storage implementation must pass before it can upload attachment ciphertext to a real service.

## What It Blocks

The gate rejects:

- media objects rejected by `MediaObjectStoreDecision`
- plaintext/debug media adapters
- development memory adapters unless explicitly allowed
- unauthenticated media services
- missing upload authorization
- unbound object namespaces
- unverified content digests

Accepted decisions expose:

```text
can_upload_object = true
can_persist_remote_ciphertext = true
forbids_plaintext_upload = true
plaintext_bytes_exposed = false
```

Rejected decisions keep upload capability false and preserve `plaintext_bytes_exposed = false`.

## Adapter Boundary

`upload_media_object_with_adapter(...)` evaluates `MediaServiceAdapterInput` first. It calls `MediaServiceAdapter::upload_accepted_media(...)` only after acceptance.

This gives future media implementations a narrow place to bind:

- authenticated small-scale object storage
- self-hosted media blob service
- private S3-compatible object storage
- CDN-backed ciphertext delivery
- approved development-only media stores

Plaintext media upload is never an accepted adapter kind.

## Verification

Run:

```powershell
cargo test -p mercury-core --test media_service_adapter
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover production acceptance, accepted-only adapter calls, media-gate rejection, plaintext/development adapter rejection, auth/namespace/digest requirements, stable kind/reason codes, checked fixture drift, backend command envelopes, platform bridge JSON, and UI simulator exposure.

## Binding Fixtures

Prototype fixtures:

```text
media_service_adapter_ready
media_service_adapter_auth_missing
media_service_adapter_plaintext_forbidden
media_service_adapter_digest_unverified
```

Backend commands:

```text
run_media_service_adapter_ready
run_media_service_adapter_auth_missing
run_media_service_adapter_plaintext_forbidden
run_media_service_adapter_digest_unverified
```

## Next Backend Step

Expose checked media retention fixtures and backend commands through `mercury-bindings`, the platform bridge, and the UI simulator.
