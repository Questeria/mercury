# Media Cleanup Session

Generated: 2026-05-28

## Status

Mercury now has a core prototype cleanup session for attachment lifecycle side effects, checked fixtures in `fixtures/prototypes`, and backend command surfaces in `mercury-bindings`:

```text
PrototypeMediaCleanupSession
PrototypeMediaCleanupSessionInput
PrototypeMediaCleanupSessionOutcome
PrototypeMediaCleanupSessionReason
PrototypeMediaCleanupSessionEvent
PrototypeMediaCleanupSessionEventKind
```

The session composes:

```text
MediaRetentionInput
PrototypeEncryptedLocalStore
```

It seeds a sealed media-cache record for prototype testing, evaluates the retention cleanup gate, and performs only the side effects the accepted decision allows.

## Session Flow

The delete-and-evict happy path is:

1. seed a local sealed `MediaCiphertext` cache record
2. start cleanup session
3. evaluate media retention cleanup readiness
4. delete the remote encrypted object when `can_delete_remote_object = true`
5. delete the local sealed cache when `can_evict_local_cache = true`
6. finish cleanup session

The retain path is an accepted no-op: it keeps the sealed local cache, skips remote deletion, skips local deletion, and preserves hash-only audit.

## Stop Points

The session stops when the media retention gate rejects the cleanup request. In that case:

- no remote delete call is made
- no local cache deletion is attempted
- any seeded sealed cache remains present
- event metadata keeps `plaintext_bytes_exposed = false`

Local cache eviction is idempotent for absent cache records: the session can complete with `local_cache_delete_attempted = true` and `local_cache_deleted = false`.

## Event Transcript

Stable event kinds:

```text
cleanup_started
media_retention_evaluated
remote_delete_evaluated
local_cache_delete_evaluated
cleanup_finished
```

Stable terminal reasons:

```text
completed
media_retention_rejected
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_media_cleanup_session --test media_retention
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover remote delete plus local eviction, retain no-op behavior, retention rejection before side effects, idempotent local eviction for absent cache records, stable event labels, stable reason labels, plaintext-free event metadata, fixture drift, backend command envelopes, platform bridge JSON, and UI simulator exposure.

## Binding Fixtures

Prototype fixtures:

```text
media_cleanup_session_happy_path
media_cleanup_session_retain_ready
media_cleanup_session_retention_rejected
media_cleanup_session_cache_absent
```

Backend commands:

```text
run_media_cleanup_session_happy_path
run_media_cleanup_session_retain_ready
run_media_cleanup_session_retention_rejected
run_media_cleanup_session_cache_absent
```

## Next Backend Step

Thread the media object index store into upload, download, and cleanup session prototypes so each attachment operation can report shared manifest persistence before a real object database is connected.
