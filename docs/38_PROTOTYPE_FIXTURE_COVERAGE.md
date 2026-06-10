# Prototype Fixture Coverage

Generated: 2026-05-28

## Status

Mercury now has checked JSON fixtures for non-visual prototype surfaces:

```text
fixtures/prototypes/local_store_sealed_message.json
fixtures/prototypes/local_store_unlock_ready.json
fixtures/prototypes/local_store_unlock_app_lock_required.json
fixtures/prototypes/local_store_unlock_recovery_required.json
fixtures/prototypes/local_store_unlock_plaintext_cache_forbidden.json
fixtures/prototypes/local_store_production_open_ready.json
fixtures/prototypes/local_store_production_open_wal_replay_required.json
fixtures/prototypes/local_store_production_open_plaintext_key_slot_forbidden.json
fixtures/prototypes/local_store_production_open_app_lock_required.json
fixtures/prototypes/local_store_keychain_android_ready.json
fixtures/prototypes/local_store_keychain_user_auth_required.json
fixtures/prototypes/local_store_keychain_exportable_secret_forbidden.json
fixtures/prototypes/local_store_keychain_development_backend_forbidden.json
fixtures/prototypes/production_store_session_happy_path.json
fixtures/prototypes/production_store_session_keychain_rejected.json
fixtures/prototypes/production_store_session_wal_replay_required.json
fixtures/prototypes/production_store_session_write_rejected.json
fixtures/prototypes/platform_local_store_adapter_desktop_ready.json
fixtures/prototypes/platform_local_store_adapter_mobile_hardware_required.json
fixtures/prototypes/platform_local_store_adapter_plaintext_forbidden.json
fixtures/prototypes/platform_local_store_adapter_app_lock_required.json
fixtures/prototypes/receive_session_happy_path.json
fixtures/prototypes/receive_session_ack_rejected.json
fixtures/prototypes/receive_session_ordering_gap.json
fixtures/prototypes/receive_session_store_write_rejected.json
fixtures/prototypes/inbound_sync_delivery_ready.json
fixtures/prototypes/inbound_sync_idle.json
fixtures/prototypes/inbound_sync_bootstrap_blocked.json
fixtures/prototypes/inbound_sync_transport_offline.json
fixtures/prototypes/inbound_sync_plaintext_preview_forbidden.json
fixtures/prototypes/authenticated_relay_source_delivery_ready.json
fixtures/prototypes/authenticated_relay_source_idle.json
fixtures/prototypes/authenticated_relay_source_auth_rejected.json
fixtures/prototypes/authenticated_relay_source_plaintext_forbidden.json
fixtures/prototypes/inbound_sync_session_happy_path.json
fixtures/prototypes/inbound_sync_session_idle.json
fixtures/prototypes/inbound_sync_session_sync_rejected.json
fixtures/prototypes/inbound_sync_session_receive_rejected.json
fixtures/prototypes/media_object_store_upload_ready.json
fixtures/prototypes/media_object_store_plaintext_rejected.json
fixtures/prototypes/media_object_store_auto_download_rejected.json
fixtures/prototypes/media_object_store_oversize_rejected.json
fixtures/prototypes/media_upload_session_happy_path.json
fixtures/prototypes/media_upload_session_plaintext_rejected.json
fixtures/prototypes/media_upload_session_seal_rejected.json
fixtures/prototypes/media_upload_session_store_write_rejected.json
fixtures/prototypes/media_service_adapter_ready.json
fixtures/prototypes/media_service_adapter_auth_missing.json
fixtures/prototypes/media_service_adapter_plaintext_forbidden.json
fixtures/prototypes/media_service_adapter_digest_unverified.json
fixtures/prototypes/media_service_upload_session_happy_path.json
fixtures/prototypes/media_service_upload_session_media_rejected.json
fixtures/prototypes/media_service_upload_session_auth_rejected.json
fixtures/prototypes/media_service_upload_session_digest_unverified.json
fixtures/prototypes/media_service_download_ready.json
fixtures/prototypes/media_service_download_plaintext_preview_rejected.json
fixtures/prototypes/media_service_download_auth_missing.json
fixtures/prototypes/media_service_download_digest_unverified.json
fixtures/prototypes/media_download_session_happy_path.json
fixtures/prototypes/media_download_session_download_rejected.json
fixtures/prototypes/media_download_session_store_write_rejected.json
fixtures/prototypes/media_download_session_open_rejected.json
fixtures/prototypes/media_retention_delete_and_evict_ready.json
fixtures/prototypes/media_retention_retain_ready.json
fixtures/prototypes/media_retention_hold_rejected.json
fixtures/prototypes/media_retention_auth_missing.json
fixtures/prototypes/media_cleanup_session_happy_path.json
fixtures/prototypes/media_cleanup_session_retain_ready.json
fixtures/prototypes/media_cleanup_session_retention_rejected.json
fixtures/prototypes/media_cleanup_session_cache_absent.json
fixtures/prototypes/media_object_index_remote_and_local_ready.json
fixtures/prototypes/media_object_index_absent_upload_ready.json
fixtures/prototypes/media_object_index_delete_pending_ready.json
fixtures/prototypes/media_object_index_deleted_terminal.json
fixtures/prototypes/media_object_index_plaintext_metadata_rejected.json
fixtures/prototypes/media_object_index_bad_lifecycle_rejected.json
fixtures/prototypes/media_object_index_store_write_ready.json
fixtures/prototypes/media_object_index_store_index_rejected.json
fixtures/prototypes/media_object_index_store_bad_object_rejected.json
fixtures/prototypes/media_object_index_store_deleted_snapshot.json
fixtures/prototypes/indexed_media_upload_session_happy_path.json
fixtures/prototypes/indexed_media_upload_session_service_rejected.json
fixtures/prototypes/indexed_media_upload_session_index_store_rejected.json
fixtures/prototypes/indexed_media_download_session_happy_path.json
fixtures/prototypes/indexed_media_download_session_manifest_rejected.json
fixtures/prototypes/indexed_media_download_session_not_downloadable.json
fixtures/prototypes/indexed_media_download_session_download_rejected.json
fixtures/prototypes/indexed_media_cleanup_session_happy_path.json
fixtures/prototypes/indexed_media_cleanup_session_manifest_rejected.json
fixtures/prototypes/indexed_media_cleanup_session_not_cleanable.json
fixtures/prototypes/indexed_media_cleanup_session_cleanup_rejected.json
fixtures/prototypes/crypto_seal_open_roundtrip.json
fixtures/prototypes/relay_delivery_once.json
fixtures/prototypes/ai_participant_draft_accepted.json
fixtures/prototypes/backend_session_happy_path.json
fixtures/prototypes/backend_session_bootstrap_blocked.json
fixtures/prototypes/backend_session_relay_rejected.json
fixtures/prototypes/backend_session_ai_rejected.json
```

The fixtures are generated from the same Rust prototype APIs used by tests. `mercury-bindings` exposes:

```text
prototype_fixture_value(...)
prototype_fixture_json(...)
prototype_fixture_by_name(...)
PROTOTYPE_FIXTURES
```

## Simulator Support

The UI simulator can now emit prototype state fixtures:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --list-prototypes
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype relay_delivery_once
cargo run -p mercury-bindings --bin mercury-ui-sim -- --all-prototypes
```

Use these fixtures when the UI needs backend-shaped state that is not a final visual decision view yet.

## Fixture Intent

- `local_store_sealed_message` shows a stored encrypted-only message record without exposing plaintext bytes.
- `local_store_unlock_ready` shows a fully accepted database unlock path.
- `local_store_unlock_app_lock_required` shows a user-auth path before opening the local database.
- `local_store_unlock_recovery_required` shows a recovery path before opening the local database.
- `local_store_unlock_plaintext_cache_forbidden` shows a destructive-repair path for unsafe plaintext cache records.
- `local_store_production_open_ready` shows a production manifest that can load encrypted records and message keys.
- `local_store_production_open_wal_replay_required` shows a crash-recovery path that can replay the write-ahead log but cannot load records yet.
- `local_store_production_open_plaintext_key_slot_forbidden` shows a destructive-repair path for unsafe plaintext key slots.
- `local_store_production_open_app_lock_required` shows unlock-gate rejection propagation through the production open gate.
- `local_store_keychain_android_ready` shows a hardware-backed platform keystore path that can build an unlock input.
- `local_store_keychain_user_auth_required` shows a user-authentication path before unlock input construction.
- `local_store_keychain_exportable_secret_forbidden` shows a destructive-repair path for unsafe exportable device secrets.
- `local_store_keychain_development_backend_forbidden` shows production rejection of a development-only secret backend.
- `production_store_session_happy_path` shows keychain, unlock, production-open, adapter open, sealed write, and sealed read succeeding without plaintext exposure.
- `production_store_session_keychain_rejected` shows exportable-secret rejection before adapter open or write.
- `production_store_session_wal_replay_required` shows WAL replay without opening records or writing.
- `production_store_session_write_rejected` shows store-open success followed by storage-policy rejection of a plaintext write.
- `platform_local_store_adapter_desktop_ready` shows a desktop production encrypted database adapter ready to open.
- `platform_local_store_adapter_mobile_hardware_required` shows a mobile adapter blocked until hardware-backed key storage is available.
- `platform_local_store_adapter_plaintext_forbidden` shows plaintext durable storage rejected before adapter open.
- `platform_local_store_adapter_app_lock_required` shows user-auth/app-lock blocking adapter open.
- `receive_session_happy_path` shows relay delivery, acknowledgement, receive gate, encrypted local persistence, and a plaintext-free inbound event transcript accepting.
- `receive_session_ack_rejected` shows a bad acknowledgement token stopping before client receive and persistence, with the terminal event pinned.
- `receive_session_ordering_gap` shows client receive requesting retry before local persistence, with the terminal event pinned.
- `receive_session_store_write_rejected` shows receive acceptance followed by local-store plaintext write rejection, with the terminal event pinned.
- `inbound_sync_delivery_ready` shows background sync permitted to poll relay and hand a pending delivery to the receive session.
- `inbound_sync_idle` shows safe background polling with no receive session handoff.
- `inbound_sync_bootstrap_blocked` shows bootstrap denying sync before relay polling.
- `inbound_sync_transport_offline` shows a network retry state without receive-session handoff.
- `inbound_sync_plaintext_preview_forbidden` shows plaintext notification previews rejected before sync.
- `authenticated_relay_source_delivery_ready` shows transport/auth validation feeding a delivery-ready inbound sync handoff.
- `authenticated_relay_source_idle` shows authenticated relay polling without a pending receive handoff.
- `authenticated_relay_source_auth_rejected` shows server authentication rejection before inbound sync can poll.
- `authenticated_relay_source_plaintext_forbidden` shows plaintext identity metadata rejected before relay polling.
- `inbound_sync_session_happy_path` shows authenticated source, sync, and receive composed into one accepted plaintext-free transcript.
- `inbound_sync_session_idle` shows accepted authenticated idle sync without receive side effects.
- `inbound_sync_session_sync_rejected` shows sync rejection before receive work after source acceptance.
- `inbound_sync_session_receive_rejected` shows receive rejection as the terminal source-backed sync-session event.
- `media_object_store_upload_ready` shows encrypted media upload readiness without plaintext exposure.
- `media_object_store_plaintext_rejected` shows plaintext media bytes rejected before upload.
- `media_object_store_auto_download_rejected` shows automatic attachment download rejected by default.
- `media_object_store_oversize_rejected` shows oversized media ciphertext rejected before upload.
- `media_upload_session_happy_path` shows media sealing, upload readiness, encrypted local persistence, and a plaintext-free upload transcript accepting.
- `media_upload_session_plaintext_rejected` shows plaintext upload bytes rejected by the media object-store gate before local persistence.
- `media_upload_session_seal_rejected` shows local media sealing rejection before media gate evaluation or local persistence.
- `media_upload_session_store_write_rejected` shows media upload readiness followed by local-store plaintext write rejection.
- `media_service_adapter_ready` shows an authenticated production object store ready to accept sealed media.
- `media_service_adapter_auth_missing` shows media service authentication blocking upload before adapter side effects.
- `media_service_adapter_plaintext_forbidden` shows plaintext/debug media adapters rejected.
- `media_service_adapter_digest_unverified` shows unverified content digests blocking upload.
- `media_service_upload_session_happy_path` shows local media sealing, local ciphertext persistence, media-service readiness, and an accepted service upload call in one transcript.
- `media_service_upload_session_media_rejected` shows media upload rejection stopping before media-service adapter evaluation.
- `media_service_upload_session_auth_rejected` shows local media upload completion followed by media-service authentication rejection.
- `media_service_upload_session_digest_unverified` shows digest verification blocking the service upload after local ciphertext persistence.
- `media_service_download_ready` shows an authenticated production object store ready to download received sealed media.
- `media_service_download_plaintext_preview_rejected` shows plaintext previews rejected before download.
- `media_service_download_auth_missing` shows media service authentication blocking received attachment fetches.
- `media_service_download_digest_unverified` shows digest verification blocking received attachment fetches.
- `media_download_session_happy_path` shows received media download readiness, sealed local persistence, and local open checks in one plaintext-free transcript.
- `media_download_session_download_rejected` shows media-service download rejection before local persistence.
- `media_download_session_store_write_rejected` shows download acceptance followed by local-store plaintext write rejection.
- `media_download_session_open_rejected` shows sealed local persistence followed by local-open metadata rejection.
- `media_retention_delete_and_evict_ready` shows remote encrypted-object deletion and local sealed-cache eviction readiness.
- `media_retention_retain_ready` shows a safe retain no-op that keeps a hash-only audit trail and does not need network auth.
- `media_retention_hold_rejected` shows retention/legal hold blocking destructive cleanup.
- `media_retention_auth_missing` shows media-service authentication blocking remote delete before adapter side effects.
- `media_cleanup_session_happy_path` shows accepted remote encrypted-object deletion and local sealed-cache eviction in one plaintext-free transcript.
- `media_cleanup_session_retain_ready` shows retain no-op cleanup preserving the seeded local sealed cache.
- `media_cleanup_session_retention_rejected` shows retention rejection stopping before remote delete or local cache eviction.
- `media_cleanup_session_cache_absent` shows idempotent local sealed-cache eviction when the cache record is already absent.
- `media_object_index_remote_and_local_ready` shows a shared attachment manifest ready for received-object download and cleanup.
- `media_object_index_absent_upload_ready` shows an absent manifest that can start upload but cannot download or cleanup.
- `media_object_index_delete_pending_ready` shows a remote object pending deletion that can cleanup but cannot download.
- `media_object_index_deleted_terminal` shows a terminal deleted manifest that cannot be reused for upload or download.
- `media_object_index_plaintext_metadata_rejected` shows plaintext metadata rejected before lifecycle capabilities are exposed.
- `media_object_index_bad_lifecycle_rejected` shows inconsistent lifecycle state and cache/remote presence rejected.
- `media_object_index_store_write_ready` shows an accepted manifest snapshot persisted without plaintext exposure.
- `media_object_index_store_index_rejected` shows an unaccepted index decision blocking store mutation.
- `media_object_index_store_bad_object_rejected` shows malformed object ID bytes rejected before store mutation.
- `media_object_index_store_deleted_snapshot` shows a terminal deleted manifest snapshot persisted for audit without reopening capabilities.
- `indexed_media_upload_session_happy_path` shows accepted service upload followed by manifest-store persistence in one plaintext-free transcript.
- `indexed_media_upload_session_service_rejected` shows service upload rejection stopping before manifest-store mutation.
- `indexed_media_upload_session_index_store_rejected` shows service upload completion followed by manifest-store rejection without persisting a manifest.
- `indexed_media_download_session_happy_path` shows accepted manifest downloadability followed by received-media download/cache/open in one plaintext-free transcript.
- `indexed_media_download_session_manifest_rejected` shows manifest-store rejection stopping before download work.
- `indexed_media_download_session_not_downloadable` shows an accepted but non-downloadable manifest stopping before download work.
- `indexed_media_download_session_download_rejected` shows manifest acceptance followed by media-download rejection.
- `indexed_media_cleanup_session_happy_path` shows accepted manifest cleanup plus remote delete/local sealed-cache eviction in one plaintext-free transcript.
- `indexed_media_cleanup_session_manifest_rejected` shows manifest-store rejection stopping before cleanup work.
- `indexed_media_cleanup_session_not_cleanable` shows an accepted but non-cleanable manifest stopping before cleanup work.
- `indexed_media_cleanup_session_cleanup_rejected` shows manifest acceptance followed by cleanup rejection.
- `crypto_seal_open_roundtrip` shows seal/open metadata and provider call counts without exposing plaintext bytes.
- `relay_delivery_once` shows accepted submission, pending queue state, one delivery, and post-delivery ciphertext clearing.
- `ai_participant_draft_accepted` shows visible AI grant state, draft-only capabilities, and hash-only audit metadata.
- `backend_session_happy_path` shows the composed startup, seal, store, relay, open, and AI draft flow without plaintext exposure.
- `backend_session_bootstrap_blocked` shows a full-flow stop before crypto, storage, relay, or AI side effects.
- `backend_session_relay_rejected` shows encrypted local persistence with no relay queue delivery.
- `backend_session_ai_rejected` shows secure delivery and open succeeding before AI grant visibility blocks AI output.

The backend session, receive session, inbound sync session, media upload session, media service upload session, media download session, media cleanup session, indexed media upload session, indexed media download session, and indexed media cleanup session fixtures include plaintext-free operation `events` arrays.

## Verification

The binding tests compare generated prototype fixture values with checked-in JSON:

```powershell
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Step

The local-store unlock gate is documented in `docs/47_LOCAL_STORE_UNLOCK_GATE.md`; the production open gate is documented in `docs/48_PRODUCTION_LOCAL_STORE_OPEN_GATE.md`.
