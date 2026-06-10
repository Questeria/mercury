# Backend Command Envelope

Generated: 2026-05-28

## Status

Mercury now has a stable prototype command envelope:

```text
PrototypeBackendCommandKind
PrototypeBackendCommandReason
PrototypeBackendCommand
PrototypeBackendCommandDecision
PrototypeBackendCommandView
```

The command envelope rejects:

- command IDs that are not 32 bytes
- plaintext command payloads
- remote AI actors attempting to run backend or AI commands
- local AI actors attempting to run human-owned backend session commands

Accepted commands emit stable command kind/reason codes and labels, and declare whether they can run a session, request an AI draft, and emit an event stream.

## Simulator Support

The UI simulator can now wrap deterministic backend session fixtures in command envelopes:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --list-commands
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command local_ai_draft_assist
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_production_store_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_platform_local_store_adapter_desktop_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_local_store_database_security_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_local_store_database_adapter_selection_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_receive_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_inbound_sync_delivery_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_authenticated_relay_source_delivery_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_inbound_sync_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_media_object_store_upload_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_media_upload_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_media_service_adapter_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_media_object_index_remote_and_local_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_media_object_index_store_write_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_credential_issuer_trust_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_group_membership_proof_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_rate_limit_nullifier_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_nullifier_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_group_chat_mls_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_use_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_adapter_selection_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_secure_backup_restore_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_event_chain_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_event_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_witness_checkpoint_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_witness_client_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_proof_bundle_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_proof_cache_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_verifier_policy_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_incident_evidence_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_private_report_outbox_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_private_report_receipt_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_key_package_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_key_package_consume_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_send_outbox_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_membership_transaction_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_replay_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_commit_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_commit_replay_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_group_relay_envelope_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --all-commands
```

Command output has:

```text
command
result
```

`command` is the stable command view. `result` is the existing prototype session or AI participant fixture.

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test platform_bridge
cargo test -p mercury-bindings --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Command Set

- `run_session_happy_path`
- `run_session_bootstrap_blocked`
- `run_session_relay_rejected`
- `run_session_ai_rejected`
- `local_ai_draft_assist`
- `run_production_store_session_happy_path`
- `run_production_store_session_keychain_rejected`
- `run_production_store_session_wal_replay_required`
- `run_production_store_session_write_rejected`
- `run_platform_local_store_adapter_desktop_ready`
- `run_platform_local_store_adapter_mobile_hardware_required`
- `run_platform_local_store_adapter_plaintext_forbidden`
- `run_platform_local_store_adapter_app_lock_required`
- `run_local_store_database_security_ready`
- `run_local_store_database_security_plaintext_rejected`
- `run_local_store_database_security_wal_rejected`
- `run_local_store_database_security_backup_rejected`
- `run_local_store_database_security_secret_lifecycle_rejected`
- `run_local_store_database_adapter_selection_ready`
- `run_local_store_database_adapter_selection_license_rejected`
- `run_local_store_database_adapter_selection_fips_rejected`
- `run_local_store_database_adapter_selection_migration_rejected`
- `run_local_store_database_adapter_selection_supply_chain_rejected`
- `run_receive_session_happy_path`
- `run_receive_session_ack_rejected`
- `run_receive_session_ordering_gap`
- `run_receive_session_store_write_rejected`
- `run_inbound_sync_delivery_ready`
- `run_inbound_sync_idle`
- `run_inbound_sync_bootstrap_blocked`
- `run_inbound_sync_transport_offline`
- `run_inbound_sync_plaintext_preview_forbidden`
- `run_authenticated_relay_source_delivery_ready`
- `run_authenticated_relay_source_idle`
- `run_authenticated_relay_source_auth_rejected`
- `run_authenticated_relay_source_plaintext_forbidden`
- `run_inbound_sync_session_happy_path`
- `run_inbound_sync_session_idle`
- `run_inbound_sync_session_sync_rejected`
- `run_inbound_sync_session_receive_rejected`
- `run_media_object_store_upload_ready`
- `run_media_object_store_plaintext_rejected`
- `run_media_object_store_auto_download_rejected`
- `run_media_object_store_oversize_rejected`
- `run_media_upload_session_happy_path`
- `run_media_upload_session_plaintext_rejected`
- `run_media_upload_session_seal_rejected`
- `run_media_upload_session_store_write_rejected`
- `run_media_service_adapter_ready`
- `run_media_service_adapter_auth_missing`
- `run_media_service_adapter_plaintext_forbidden`
- `run_media_service_adapter_digest_unverified`
- `run_media_service_upload_session_happy_path`
- `run_media_service_upload_session_media_rejected`
- `run_media_service_upload_session_auth_rejected`
- `run_media_service_upload_session_digest_unverified`
- `run_media_service_download_ready`
- `run_media_service_download_plaintext_preview_rejected`
- `run_media_service_download_auth_missing`
- `run_media_service_download_digest_unverified`
- `run_media_download_session_happy_path`
- `run_media_download_session_download_rejected`
- `run_media_download_session_store_write_rejected`
- `run_media_download_session_open_rejected`
- `run_media_retention_delete_and_evict_ready`
- `run_media_retention_retain_ready`
- `run_media_retention_hold_rejected`
- `run_media_retention_auth_missing`
- `run_media_cleanup_session_happy_path`
- `run_media_cleanup_session_retain_ready`
- `run_media_cleanup_session_retention_rejected`
- `run_media_cleanup_session_cache_absent`
- `run_media_object_index_remote_and_local_ready`
- `run_media_object_index_absent_upload_ready`
- `run_media_object_index_delete_pending_ready`
- `run_media_object_index_deleted_terminal`
- `run_media_object_index_plaintext_metadata_rejected`
- `run_media_object_index_bad_lifecycle_rejected`
- `run_media_object_index_store_write_ready`
- `run_media_object_index_store_index_rejected`
- `run_media_object_index_store_bad_object_rejected`
- `run_media_object_index_store_deleted_snapshot`
- `run_indexed_media_upload_session_happy_path`
- `run_indexed_media_upload_session_service_rejected`
- `run_indexed_media_upload_session_index_store_rejected`
- `run_indexed_media_download_session_happy_path`
- `run_indexed_media_download_session_manifest_rejected`
- `run_indexed_media_download_session_not_downloadable`
- `run_indexed_media_download_session_download_rejected`
- `run_indexed_media_cleanup_session_happy_path`
- `run_indexed_media_cleanup_session_manifest_rejected`
- `run_indexed_media_cleanup_session_not_cleanable`
- `run_indexed_media_cleanup_session_cleanup_rejected`
- `run_anonymous_credential_issuer_trust_ready`
- `run_anonymous_credential_issuer_trust_transparency_required`
- `run_anonymous_credential_issuer_trust_revoked`
- `run_anonymous_credential_issuer_trust_partitioning_metadata_rejected`
- `run_anonymous_credential_issuer_trust_witness_audit_rejected`
- `run_anonymous_group_membership_proof_ready`
- `run_anonymous_group_membership_proof_high_security_pq_required`
- `run_anonymous_group_membership_proof_replay_rejected`
- `run_anonymous_group_membership_proof_route_binding_required`
- `run_anonymous_group_membership_proof_plaintext_identity_rejected`
- `run_anonymous_rate_limit_nullifier_ready`
- `run_anonymous_rate_limit_nullifier_replay_rejected`
- `run_anonymous_rate_limit_nullifier_limit_exceeded`
- `run_anonymous_rate_limit_nullifier_opaque_store_required`
- `run_anonymous_nullifier_store_ready`
- `run_anonymous_nullifier_store_replay_rejected`
- `run_anonymous_nullifier_store_plaintext_metadata_rejected`
- `run_group_chat_mls_ready`
- `run_group_chat_mls_setup_required`
- `run_group_chat_membership_sync_required`
- `run_group_chat_plaintext_metadata_forbidden`
- `run_group_chat_high_security_mls_required`
- `run_group_chat_high_security_pq_required`
- `run_group_chat_mls_provider_security_required`
- `run_mls_provider_evidence_store_ready`
- `run_mls_provider_evidence_store_gate_rejected`
- `run_mls_provider_evidence_store_duplicate_rejected`
- `run_mls_provider_evidence_store_plaintext_rejected`
- `run_mls_provider_evidence_use_ready`
- `run_mls_provider_evidence_use_missing`
- `run_mls_provider_evidence_use_expired`
- `run_mls_provider_evidence_use_suite_mismatch`
- `run_mls_provider_evidence_use_plaintext_rejected`
- `run_mls_provider_adapter_selection_ready`
- `run_mls_provider_adapter_selection_provider_rejected`
- `run_mls_provider_adapter_selection_pq_draft_rejected`
- `run_mls_provider_adapter_selection_storage_rejected`
- `run_mls_provider_adapter_selection_supply_chain_rejected`
- `run_secure_backup_restore_ready`
- `run_secure_backup_restore_recovery_rejected`
- `run_secure_backup_restore_plaintext_rejected`
- `run_secure_backup_restore_mls_rekey_rejected`
- `run_secure_backup_restore_cloud_policy_rejected`
- `run_sealed_audit_event_chain_ready`
- `run_sealed_audit_event_chain_plaintext_rejected`
- `run_sealed_audit_event_chain_rollback_rejected`
- `run_sealed_audit_event_chain_witness_rejected`
- `run_sealed_audit_event_chain_binding_rejected`
- `run_sealed_audit_event_store_ready`
- `run_sealed_audit_event_store_chain_rejected`
- `run_sealed_audit_event_store_duplicate_rejected`
- `run_sealed_audit_event_store_rollback_rejected`
- `run_sealed_audit_event_store_plaintext_rejected`
- `run_sealed_audit_witness_checkpoint_ready`
- `run_sealed_audit_witness_checkpoint_store_rejected`
- `run_sealed_audit_witness_checkpoint_quorum_rejected`
- `run_sealed_audit_witness_checkpoint_split_view_rejected`
- `run_sealed_audit_witness_checkpoint_privacy_rejected`
- `run_sealed_audit_witness_client_ready`
- `run_sealed_audit_witness_client_conflict`
- `run_sealed_audit_witness_client_unavailable`
- `run_sealed_audit_witness_client_policy_rejected`
- `run_sealed_audit_witness_client_monitor_privacy_rejected`
- `run_sealed_audit_proof_bundle_ready`
- `run_sealed_audit_proof_bundle_client_rejected`
- `run_sealed_audit_proof_bundle_stale_witness`
- `run_sealed_audit_proof_bundle_policy_rejected`
- `run_sealed_audit_proof_bundle_privacy_rejected`
- `run_sealed_audit_proof_cache_ready`
- `run_sealed_audit_proof_cache_bundle_rejected`
- `run_sealed_audit_proof_cache_duplicate_rejected`
- `run_sealed_audit_proof_cache_policy_stale`
- `run_sealed_audit_proof_cache_plaintext_rejected`
- `run_sealed_audit_verifier_policy_ready`
- `run_sealed_audit_verifier_policy_expired`
- `run_sealed_audit_verifier_policy_key_rotation_required`
- `run_sealed_audit_verifier_policy_monitor_privacy_rejected`
- `run_sealed_audit_verifier_policy_plaintext_rejected`
- `run_sealed_audit_incident_evidence_ready`
- `run_sealed_audit_incident_evidence_policy_rejected`
- `run_sealed_audit_incident_evidence_missing_proof_report`
- `run_sealed_audit_incident_evidence_split_view`
- `run_sealed_audit_incident_evidence_plaintext_rejected`
- `run_sealed_audit_recovery_export_ready`
- `run_sealed_audit_recovery_export_incident_rejected`
- `run_sealed_audit_recovery_export_quorum_required`
- `run_sealed_audit_recovery_export_rollback_rejected`
- `run_sealed_audit_recovery_export_plaintext_rejected`
- `run_sealed_audit_database_adapter_ready`
- `run_sealed_audit_database_adapter_encryption_rejected`
- `run_sealed_audit_database_adapter_append_only_rejected`
- `run_sealed_audit_private_report_transport_ready`
- `run_sealed_audit_private_report_transport_plaintext_rejected`
- `run_sealed_audit_private_report_outbox_ready`
- `run_sealed_audit_private_report_outbox_transport_rejected`
- `run_sealed_audit_private_report_outbox_replay_rejected`
- `run_sealed_audit_private_report_outbox_rate_limit_rejected`
- `run_sealed_audit_private_report_outbox_plaintext_rejected`
- `run_sealed_audit_private_report_receipt_ready`
- `run_sealed_audit_private_report_receipt_outbox_rejected`
- `run_sealed_audit_private_report_receipt_missing`
- `run_sealed_audit_private_report_receipt_transparency_rejected`
- `run_sealed_audit_private_report_receipt_plaintext_rejected`
- `run_sealed_audit_private_report_reconciliation_ready`
- `run_sealed_audit_private_report_reconciliation_receipt_rejected`
- `run_sealed_audit_private_report_reconciliation_retry_rejected`
- `run_sealed_audit_private_report_reconciliation_false_delivery_rejected`
- `run_sealed_audit_private_report_reconciliation_plaintext_rejected`
- `run_sealed_audit_private_report_gateway_evidence_ready`
- `run_sealed_audit_private_report_gateway_evidence_reconciliation_rejected`
- `run_sealed_audit_private_report_gateway_evidence_unavailable_rejected`
- `run_sealed_audit_private_report_gateway_evidence_accountability_rejected`
- `run_sealed_audit_private_report_gateway_evidence_plaintext_rejected`
- `run_mls_key_package_admission_ready`
- `run_mls_key_package_admission_group_rejected`
- `run_mls_key_package_admission_lifetime_rejected`
- `run_mls_key_package_admission_suite_mismatch`
- `run_mls_key_package_admission_credential_rejected`
- `run_mls_key_package_admission_replay_rejected`
- `run_mls_key_package_admission_plaintext_rejected`
- `run_mls_key_package_consume_store_ready`
- `run_mls_key_package_consume_store_admission_rejected`
- `run_mls_key_package_consume_store_duplicate_rejected`
- `run_mls_key_package_consume_store_bad_shape`
- `run_mls_key_package_consume_store_plaintext_rejected`
- `run_mls_welcome_send_outbox_ready`
- `run_mls_welcome_send_outbox_consume_rejected`
- `run_mls_welcome_send_outbox_duplicate_transaction_rejected`
- `run_mls_welcome_send_outbox_key_package_queued`
- `run_mls_welcome_send_outbox_bad_shape`
- `run_mls_welcome_send_outbox_plaintext_rejected`
- `run_mls_membership_transaction_ready`
- `run_mls_membership_transaction_binding_rejected`
- `run_mls_membership_transaction_storage_rejected`
- `run_mls_membership_transaction_duplicate_rejected`
- `run_mls_membership_transaction_plaintext_rejected`
- `run_mls_welcome_admission_ready`
- `run_mls_welcome_admission_secrets_missing`
- `run_mls_welcome_admission_tree_rejected`
- `run_mls_welcome_admission_confirmation_rejected`
- `run_mls_welcome_admission_tie_break_rejected`
- `run_mls_welcome_admission_replay_rejected`
- `run_mls_welcome_admission_plaintext_rejected`
- `run_mls_welcome_replay_store_ready`
- `run_mls_welcome_replay_store_admission_rejected`
- `run_mls_welcome_replay_store_duplicate_rejected`
- `run_mls_welcome_replay_store_key_package_reused`
- `run_mls_welcome_replay_store_bad_shape`
- `run_mls_welcome_replay_store_plaintext_rejected`
- `run_mls_commit_admission_ready`
- `run_mls_commit_admission_bad_epoch`
- `run_mls_commit_admission_auth_rejected`
- `run_mls_commit_admission_path_rejected`
- `run_mls_commit_admission_tie_break_rejected`
- `run_mls_commit_admission_replay_rejected`
- `run_mls_commit_admission_plaintext_rejected`
- `run_mls_commit_replay_store_ready`
- `run_mls_commit_replay_store_admission_rejected`
- `run_mls_commit_replay_store_duplicate_rejected`
- `run_mls_commit_replay_store_local_member_removed`
- `run_mls_commit_replay_store_plaintext_rejected`
- `run_group_relay_envelope_ready`
- `run_group_relay_envelope_transcript_sync_required`
- `run_group_relay_envelope_transcript_rekey_required`
- `run_group_relay_envelope_missing_delivery_token`
- `run_group_relay_envelope_plaintext_metadata_rejected`

The `local_ai_draft_assist` command is the only accepted command owned by `local_ai`. It sets `can_request_ai_draft = true`, `can_run_session = false`, and `emits_event_stream = false`.

The `run_production_store_session_*` commands are human-owned readiness checks for the production-store session prototype. They reuse the same plaintext-free command gate and return `prototype_production_store_session` results for happy path, keychain rejection, WAL replay required, and write rejection branches.

The `run_platform_local_store_adapter_*` commands are human-owned platform storage readiness checks. They return `platform_local_store_adapter` results for desktop ready, mobile hardware-key required, plaintext storage forbidden, and app-lock required branches.

The `run_local_store_database_security_*` commands are human-owned production database profile checks. They return `local_store_database_security` results for accepted SQLCipher-style storage, plaintext database rejection, plaintext WAL/journal rejection, unsafe backup policy rejection, and secret lifecycle rejection.

The `run_local_store_database_adapter_selection_*` commands are human-owned production database adapter checks. They return `local_store_database_adapter_selection` results for accepted SQLCipher adapter selection, license rejection, FIPS evidence rejection, missing migration drills, and missing supply-chain evidence.

The `run_receive_session_*` commands are human-owned inbound readiness checks. They return `prototype_receive_session` results for accepted receive, acknowledgement rejection, ordering-gap retry, and local-store write rejection branches. Each result includes a plaintext-free `events` transcript with stable kind/reason labels and terminal state.

The `run_inbound_sync_*` commands are human-owned background sync readiness checks. They return `inbound_sync_gate` results for delivery-ready, idle, bootstrap-blocked, transport-offline, and plaintext-preview-forbidden branches.

The `run_authenticated_relay_source_*` commands are human-owned transport-source readiness checks. They return `authenticated_relay_source` results for accepted delivery, accepted idle, server-auth rejection, and plaintext metadata rejection branches, plus the inbound-sync decision produced from each source decision.

The `run_inbound_sync_session_*` commands are human-owned composed sync operation checks. They return `prototype_inbound_sync_session` results with a single plaintext-free transcript over sync and receive work.

The `run_media_object_store_*` commands are human-owned media readiness checks. They return `media_object_store` results for encrypted upload-ready, plaintext rejection, automatic-download rejection, and oversized ciphertext branches.

The `run_media_upload_session_*` commands are human-owned attachment operation checks. They return `prototype_media_upload_session` results with plaintext-free event transcripts for accepted upload, plaintext upload rejection, local seal rejection, and local-store write rejection branches.

The `run_media_service_adapter_*` commands are human-owned media service readiness checks. They return `media_service_adapter` results for production object-store readiness, missing service authentication, plaintext adapter rejection, and unverified content digest branches.

The `run_media_service_upload_session_*`, `run_media_service_download_*`, and `run_media_download_session_*` commands cover plaintext-free attachment service upload/download readiness and composed received-media persistence/open transcripts.

The `run_media_retention_*` and `run_media_cleanup_session_*` commands cover attachment cleanup readiness and cleanup side effects while preserving hash-only audit.

The `run_media_object_index_*` commands expose shared attachment lifecycle capabilities for upload, download, cleanup, delete-pending, terminal-deleted, plaintext-metadata rejection, and lifecycle-mismatch states.

The `run_media_object_index_store_*` commands expose prototype manifest-store writes for accepted snapshots, index rejection, malformed object IDs, and terminal deleted audit snapshots.

The `run_indexed_media_upload_session_*` commands expose the composed upload-and-index operation for accepted upload, service rejection before index mutation, and index-store rejection after service upload.

The `run_indexed_media_download_session_*` commands expose the composed index-and-download operation for accepted download, manifest rejection, non-downloadable lifecycle state, and download-session rejection.

The `run_indexed_media_cleanup_session_*` commands expose the composed index-and-cleanup operation for accepted cleanup, manifest rejection, non-cleanable lifecycle state, and cleanup-session rejection.

The `run_anonymous_credential_issuer_trust_*` commands expose anonymous credential issuer trust checks for accepted issuer use, missing transparency, revoked issuer keys, witness/auditor rejection, and partitioning-metadata rejection.

The `run_anonymous_group_membership_proof_*` commands expose anonymous proof checks for accepted membership, high-security PQ proof requirement, replay/nullifier rejection, route-binding rejection, and plaintext identity rejection.

The `run_anonymous_rate_limit_nullifier_*` commands expose anonymous abuse-control checks for accepted ARC-style windows, spent-nullifier replay rejection, exhausted presentation windows, and non-opaque nullifier storage.

The `run_anonymous_nullifier_store_*` commands expose accepted-only nullifier persistence checks for stored opaque nullifiers, duplicate/replay rejection, and plaintext metadata rejection.

The `run_group_chat_*` commands expose checked group-room readiness for accepted MLS setup, missing MLS provider setup, membership sync, plaintext metadata rejection, high-security MLS/PQ requirements, and MLS provider-security rejection.

The `run_mls_provider_evidence_store_*` commands expose accepted-only provider evidence persistence for digest-only evidence records, failed provider-security gates, duplicate evidence ids, and plaintext evidence rejection.

The `run_mls_provider_evidence_use_*` commands expose current-readiness checks for persisted provider evidence, including missing records, expiry, suite mismatch, and plaintext-taint rejection.

The `run_sealed_audit_event_chain_*` commands expose tamper-evident audit readiness for digest-only sealed event chains, including plaintext event rejection, local rollback rejection, witness quorum rejection, and event-binding rejection.

The `run_sealed_audit_event_store_*` commands expose accepted-only sealed audit persistence for digest-only records, chain-gate rejection, duplicate sequence rejection, rollback rejection, and plaintext metadata rejection.

The `run_sealed_audit_witness_checkpoint_*` commands expose witness-backed checkpoint publication readiness for store-accepted records, PQ/hybrid checkpoint signatures, consistency proof checks, witness quorum and key pinning, split-view evidence rejection, and privacy-preserving monitor-query enforcement.

The `run_sealed_audit_witness_client_*` commands expose production witness-client readiness for C2SP add-checkpoint submission, witness conflict handling, witness availability, policy/key pinning, atomic latest-checkpoint persistence, alert routing, and private monitor queries.

The `run_sealed_audit_proof_bundle_*` commands expose offline proof-bundle readiness for witness-client-approved checkpoints, persisted digest-only proof cache entries, verifier policy snapshots, inclusion and consistency proof evidence, witness freshness, cache recovery, and UI-safe status without audit selector leakage.

The `run_sealed_audit_proof_cache_*` commands expose accepted-only proof-cache persistence for proof-bundle-approved, digest-only, encrypted, append-only records with offline verifier success, monitor freshness, policy snapshot binding, duplicate and rollback rejection, authenticated recovery, and no selector-bearing plaintext metadata.

The `run_sealed_audit_verifier_policy_*` commands expose accepted-only verifier policy snapshots and private monitor freshness state for proof-cache-approved records, signed policy import, key-rotation authentication, offline re-verification, stale monitor refresh, split-view escalation, encrypted append-only scheduler state, and selector-free status.

The `run_sealed_audit_incident_evidence_*` commands expose accepted-only split-view, missing-proof, and private-monitor incident evidence for verifier-policy-approved state, digest-bound contradiction reports, blinded missing-proof reports, private monitor reports, witness/operator accountability, retry/backoff metadata, encrypted append-only records, and selector-free UI status.

The `run_sealed_audit_recovery_export_*` commands expose accepted-only sealed-audit recovery/export and cross-device incident sync readiness for incident-evidence-approved state, encrypted/authenticated export manifests, restore quorum, device binding, rollback protection, audit-checkpoint binding, private cross-device sync, redacted incident selectors, encrypted append-only storage, and digest-only UI status.

The `run_sealed_audit_database_adapter_*` commands expose production sealed-audit database readiness for recovery-export-approved state, accepted encrypted database adapter selection, encrypted tables/WAL, memory-only temp stores, page authentication, platform key wrapping, append-only schema constraints, transactional writes, WAL checkpoint policy, deterministic migration, crash recovery, and selector-free UI status.

The `run_sealed_audit_private_report_transport_*` commands expose private incident-report transport readiness for database-adapter-approved state, OHTTP-style relay/gateway separation, pinned HPKE gateway keys, state-free target behavior, Privacy Pass-style anonymous rate limits, encrypted report outbox state, replay/retry guards, constant-size padding, private monitor routing, and selector-free payloads.

The `run_sealed_audit_private_report_outbox_*` commands expose accepted-only private report outbox persistence for transport-approved state, digest-only OHTTP request/response transcript records, encrypted payload and outbox state, replay-window binding, duplicate rejection, retry/backoff persistence, anonymous rate-limit token spend-once state, route privacy, and selector-free UI status.

The `run_sealed_audit_private_report_receipt_*` commands expose accepted-only private report delivery receipt persistence for outbox-approved state, verified gateway receipts, report/response/key binding, gateway-key transparency and consistency evidence, authenticated key rotation, private monitor proof, blinded failure classification, retry completion persistence, duplicate and replay rejection, encrypted receipt records, and selector-free UI status.

The `run_sealed_audit_private_report_reconciliation_*` commands expose accepted-only private report retry reconciliation for receipt-approved state, retry schedule binding, retry idempotency, duplicate retry rejection, no retry after delivered receipt, anonymous rate-limit continuity, crash recovery cursor binding, false delivery rejection, operator accountability routing, blinded failure buckets, encrypted append-only records, and selector-free UI retry status.

The `run_sealed_audit_private_report_gateway_evidence_*` commands expose accepted-only private report gateway-unavailability and operator-accountability evidence for reconciliation-approved state, authenticated unavailable evidence, signed relay observation, target absence proof, no client-asserted unavailability, retry exhaustion, anonymous rate-limit continuity, gateway key binding, blinded failure buckets, private monitor routing, policy-gated incident visibility, encrypted append-only records, and selector-free UI unavailable status.

The `run_mls_key_package_admission_*` commands expose group membership-add readiness for accepted KeyPackages, rejected group state, stale lifetime, suite mismatch, invalid credential, replayed KeyPackage hash, and plaintext identity rejection.

The `run_mls_key_package_consume_store_*` commands expose accepted-only sender-side KeyPackage consumption for digest-only KeyPackage records, durable Welcome-send transaction binding, duplicate KeyPackage rejection across groups, malformed digest rejection, and plaintext metadata rejection.

The `run_mls_welcome_send_outbox_*` commands expose accepted-only sender-side Welcome outbox persistence for digest-only queued Welcome records, accepted Commit binding, delivery-route binding, duplicate transaction rejection, duplicate KeyPackage queued rejection, malformed digest rejection, and plaintext metadata rejection.

The `run_mls_membership_transaction_*` commands expose the one-transaction MLS membership-change witness for accepted Commit replay, KeyPackage consumption, Welcome outbox insertion, cross-record binding, durable serializable transaction requirements, duplicate transaction rejection, and plaintext metadata rejection.

The `run_mls_welcome_admission_*` commands expose receiving-side group join readiness for accepted Welcome processing, missing encrypted group secrets, rejected ratchet trees, invalid confirmation tags, losing Commit tie-break state, replayed Welcome hashes, and plaintext group metadata rejection.

The `run_mls_welcome_replay_store_*` commands expose accepted-only Welcome replay persistence for digest-only Welcome records, consumed KeyPackage/init-key binding, tree and transcript binding, transactional group-state commit evidence, duplicate Welcome hash rejection, consumed KeyPackage rejection, malformed digest rejection, and plaintext metadata rejection.

The `run_mls_commit_admission_*` commands expose current-epoch MLS Commit readiness for accepted epoch advancement, bad epoch, authentication failure, update-path/tree failure, losing tie-break, replayed Commit hash, and plaintext Commit metadata rejection.

The `run_mls_commit_replay_store_*` commands expose accepted-only Commit replay persistence for digest-only Commit records, admission rejection, duplicate Commit hash rejection, terminal local-member removal, and plaintext metadata rejection.

The `run_group_relay_envelope_*` commands expose checked group relay enqueue state for accepted metadata-hidden relay, transcript sync required, transcript rekey required, missing sealed-sender delivery token, and plaintext metadata rejection.

## Next Step

Next backend security work should replace more prototype-only local database, anonymous credential, nullifier storage, sealed audit incident-evidence/recovery-export databases, private report outbox/submission transcript, private report delivery receipt, private report retry reconciliation, private report unavailable-gateway accountability evidence, MLS provider-evidence, MLS KeyPackage consume-store, MLS Welcome send outbox, MLS membership transaction, MLS Welcome replay-store, MLS Commit admission, and MLS Commit replay-store boundaries with production adapters while preserving this command envelope as the UI/platform boundary.
