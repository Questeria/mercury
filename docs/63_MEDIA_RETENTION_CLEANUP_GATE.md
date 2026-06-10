# Media Retention Cleanup Gate

Generated: 2026-05-28

## Status

Mercury now has a core decision boundary for attachment lifecycle cleanup, checked fixtures in `fixtures/prototypes`, and backend command surfaces in `mercury-bindings`:

```text
MediaRetentionOperation
MediaRetentionInput
MediaRetentionDecision
MediaRetentionReason
MediaRetentionAdapter
apply_media_retention_with_adapter(...)
```

This is not a production media service or cache implementation. It is the backend contract that decides whether encrypted media should be retained, locally evicted, remotely deleted, or both before real object storage and cache deletion code is connected.

## Operations

Stable operations:

```text
retain
evict_local_cache
delete_remote_object
delete_remote_and_evict_local_cache
```

`retain` is an accepted no-op that keeps the audit hash and does not call the cleanup adapter. Destructive cleanup operations call the adapter only after the decision accepts.

## Security Rules

- Plaintext media deletion paths are rejected before adapter calls.
- Cleanup is limited to `MediaCiphertext` records.
- Object IDs and content digests must be opaque fixed-length values.
- Retention/legal holds block destructive cleanup.
- Remote object deletion requires an explicit user delete request.
- Local cache eviction can run offline, but requires a user delete request or cache-eviction trigger.
- Remote deletion rejects plaintext/debug adapters and unapproved development adapters.
- Remote deletion requires service authentication, delete authorization, namespace binding, and verified content digest.
- Every outcome keeps `plaintext_bytes_exposed = false`.
- Every outcome keeps a hash-only audit trail through `keeps_audit_hash = true`.

## Stable Reasons

```text
ACCEPTED
PLAINTEXT_DELETION_FORBIDDEN
MEDIA_RECORD_KIND_MISMATCH
BAD_OBJECT_ID_LENGTH
BAD_CONTENT_DIGEST_LENGTH
RETENTION_HOLD_ACTIVE
USER_DELETE_REQUIRED
CACHE_EVICTION_NOT_REQUESTED
PLAINTEXT_ADAPTER_FORBIDDEN
DEVELOPMENT_ADAPTER_FORBIDDEN
SERVICE_AUTHENTICATION_MISSING
DELETE_AUTHORIZATION_MISSING
OBJECT_NAMESPACE_UNBOUND
CONTENT_DIGEST_UNVERIFIED
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test media_retention
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover delete-and-evict acceptance, adapter gating, retain no-op behavior, offline cache eviction, plaintext deletion rejection, wrong record-kind rejection, malformed metadata rejection, retention holds, missing user delete intent, remote adapter class checks, service auth/authorization checks, namespace/digest checks, stable operation/reason labels, fixture drift, backend command envelopes, platform bridge JSON, and UI simulator exposure.

## Binding Fixtures

Prototype fixtures:

```text
media_retention_delete_and_evict_ready
media_retention_retain_ready
media_retention_hold_rejected
media_retention_auth_missing
```

Backend commands:

```text
run_media_retention_delete_and_evict_ready
run_media_retention_retain_ready
run_media_retention_hold_rejected
run_media_retention_auth_missing
```

## Next Backend Step

Expose checked media cleanup session fixtures and backend commands through `mercury-bindings`, the platform bridge, and the UI simulator.
