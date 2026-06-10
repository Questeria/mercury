use mercury_bindings::{BACKEND_COMMANDS, backend_command_by_name, backend_command_json};

#[test]
fn backend_command_names_are_resolvable() {
    for descriptor in BACKEND_COMMANDS {
        assert_eq!(backend_command_by_name(descriptor.name), Some(descriptor));
    }

    assert_eq!(backend_command_by_name("does_not_exist"), None);
}

#[test]
fn backend_command_json_wraps_command_and_result() {
    let descriptor =
        backend_command_by_name("run_session_happy_path").expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_session_happy_path"
    );
    assert_eq!(value["result"]["surface"], "prototype_backend_session");
    assert_eq!(value["result"]["completed"], true);
}

#[test]
fn backend_command_json_wraps_local_ai_draft_assist() {
    let descriptor =
        backend_command_by_name("local_ai_draft_assist").expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(value["command"]["actor_kind_label"], "local_ai");
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_local_ai_draft_assist"
    );
    assert_eq!(value["command"]["can_run_session"], false);
    assert_eq!(value["command"]["can_request_ai_draft"], true);
    assert_eq!(value["command"]["emits_event_stream"], false);
    assert_eq!(value["result"]["surface"], "prototype_ai_participant");
    assert_eq!(value["result"]["decision"]["accepted"], true);
}

#[test]
fn backend_command_json_wraps_production_store_session_readiness() {
    let descriptor = backend_command_by_name("run_production_store_session_wal_replay_required")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(value["command"]["actor_kind_label"], "human");
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_production_store_session_wal_replay_required"
    );
    assert_eq!(value["command"]["can_run_session"], true);
    assert_eq!(value["command"]["can_request_ai_draft"], false);
    assert_eq!(
        value["result"]["surface"],
        "prototype_production_store_session"
    );
    assert_eq!(value["result"]["outcome"]["accepted"], false);
    assert_eq!(value["result"]["outcome"]["wal_replayed"], true);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "WAL_REPLAY_REQUIRED"
    );
}

#[test]
fn backend_command_json_wraps_platform_adapter_readiness() {
    let descriptor = backend_command_by_name("run_platform_local_store_adapter_desktop_ready")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(value["command"]["actor_kind_label"], "human");
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_platform_local_store_adapter_desktop_ready"
    );
    assert_eq!(value["result"]["surface"], "platform_local_store_adapter");
    assert_eq!(value["result"]["decision"]["accepted"], true);
    assert_eq!(value["result"]["decision"]["can_open_adapter"], true);
    assert_eq!(
        value["result"]["decision"]["forbids_plaintext_storage"],
        true
    );
}

#[test]
fn backend_command_json_wraps_local_store_database_security_commands() {
    let ready = command_value("run_local_store_database_security_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_local_store_database_security_ready"
    );
    assert_eq!(ready["result"]["surface"], "local_store_database_security");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(
        ready["result"]["decision"]["can_host_mls_transactions"],
        true
    );
    assert_eq!(
        ready["result"]["decision"]["forbids_plaintext_storage"],
        true
    );

    let plaintext = command_value("run_local_store_database_security_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_DATABASE_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["requires_destructive_repair"],
        true
    );

    let wal = command_value("run_local_store_database_security_wal_rejected");
    assert_eq!(wal["result"]["decision"]["accepted"], false);
    assert_eq!(
        wal["result"]["decision"]["reason_label"],
        "WAL_OR_JOURNAL_PLAINTEXT"
    );
    assert_eq!(wal["result"]["decision"]["requires_crash_recovery"], true);

    let backup = command_value("run_local_store_database_security_backup_rejected");
    assert_eq!(backup["result"]["decision"]["accepted"], false);
    assert_eq!(
        backup["result"]["decision"]["reason_label"],
        "BACKUP_POLICY_REJECTED"
    );
    assert_eq!(
        backup["result"]["decision"]["requires_backup_reconfiguration"],
        true
    );

    let lifecycle = command_value("run_local_store_database_security_secret_lifecycle_rejected");
    assert_eq!(lifecycle["result"]["decision"]["accepted"], false);
    assert_eq!(
        lifecycle["result"]["decision"]["reason_label"],
        "SECRET_LIFECYCLE_REJECTED"
    );
    assert_eq!(lifecycle["result"]["decision"]["requires_migration"], true);
}

#[test]
fn backend_command_json_wraps_local_store_database_adapter_selection_commands() {
    let ready = command_value("run_local_store_database_adapter_selection_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_local_store_database_adapter_selection_ready"
    );
    assert_eq!(
        ready["result"]["surface"],
        "local_store_database_adapter_selection"
    );
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_ship_release"], true);
    assert_eq!(
        ready["result"]["decision"]["can_host_mls_transactions"],
        true
    );

    let license = command_value("run_local_store_database_adapter_selection_license_rejected");
    assert_eq!(license["result"]["decision"]["accepted"], false);
    assert_eq!(
        license["result"]["decision"]["reason_label"],
        "LICENSE_REJECTED"
    );
    assert_eq!(
        license["result"]["decision"]["requires_license_review"],
        true
    );

    let fips = command_value("run_local_store_database_adapter_selection_fips_rejected");
    assert_eq!(fips["result"]["decision"]["accepted"], false);
    assert_eq!(
        fips["result"]["decision"]["reason_label"],
        "FIPS_VALIDATION_MISSING"
    );
    assert_eq!(
        fips["result"]["decision"]["requires_fips_attestation"],
        true
    );

    let migration = command_value("run_local_store_database_adapter_selection_migration_rejected");
    assert_eq!(migration["result"]["decision"]["accepted"], false);
    assert_eq!(
        migration["result"]["decision"]["reason_label"],
        "MIGRATION_DRILL_MISSING"
    );
    assert_eq!(
        migration["result"]["decision"]["requires_migration_drill"],
        true
    );

    let supply_chain =
        command_value("run_local_store_database_adapter_selection_supply_chain_rejected");
    assert_eq!(supply_chain["result"]["decision"]["accepted"], false);
    assert_eq!(
        supply_chain["result"]["decision"]["reason_label"],
        "SBOM_OR_CVE_MONITORING_MISSING"
    );
    assert_eq!(
        supply_chain["result"]["decision"]["requires_supply_chain_review"],
        true
    );
}

#[test]
fn backend_command_json_wraps_mls_provider_adapter_selection_commands() {
    let ready = command_value("run_mls_provider_adapter_selection_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_provider_adapter_selection_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_provider_adapter_selection");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_ship_release"], true);
    assert_eq!(ready["result"]["decision"]["can_change_membership"], true);
    assert_eq!(
        ready["result"]["decision"]["crypto_backend_label"],
        "libcrux_hybrid_pq"
    );

    let provider = command_value("run_mls_provider_adapter_selection_provider_rejected");
    assert_eq!(provider["result"]["decision"]["accepted"], false);
    assert_eq!(
        provider["result"]["decision"]["reason_label"],
        "PROVIDER_SECURITY_REJECTED"
    );
    assert_eq!(
        provider["result"]["decision"]["provider_security_reason"],
        "KNOWN_ANSWER_TESTS_MISSING"
    );

    let pq_draft = command_value("run_mls_provider_adapter_selection_pq_draft_rejected");
    assert_eq!(pq_draft["result"]["decision"]["accepted"], false);
    assert_eq!(
        pq_draft["result"]["decision"]["reason_label"],
        "PQ_DRAFT_PIN_MISSING"
    );
    assert_eq!(pq_draft["result"]["decision"]["requires_pq_upgrade"], true);

    let storage = command_value("run_mls_provider_adapter_selection_storage_rejected");
    assert_eq!(storage["result"]["decision"]["accepted"], false);
    assert_eq!(
        storage["result"]["decision"]["reason_label"],
        "STORAGE_PROVIDER_UNSAFE"
    );
    assert_eq!(
        storage["result"]["decision"]["requires_storage_review"],
        true
    );

    let supply_chain = command_value("run_mls_provider_adapter_selection_supply_chain_rejected");
    assert_eq!(supply_chain["result"]["decision"]["accepted"], false);
    assert_eq!(
        supply_chain["result"]["decision"]["reason_label"],
        "SBOM_OR_CVE_MONITORING_MISSING"
    );
    assert_eq!(
        supply_chain["result"]["decision"]["requires_supply_chain_review"],
        true
    );
}

#[test]
fn backend_command_json_wraps_secure_backup_restore_commands() {
    let ready = command_value("run_secure_backup_restore_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_secure_backup_restore_ready"
    );
    assert_eq!(ready["result"]["surface"], "secure_backup_restore");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_create_backup"], true);
    assert_eq!(ready["result"]["decision"]["can_restore_mls_state"], true);
    assert_eq!(
        ready["result"]["decision"]["forbids_plaintext_export"],
        true
    );

    let recovery = command_value("run_secure_backup_restore_recovery_rejected");
    assert_eq!(recovery["result"]["decision"]["accepted"], false);
    assert_eq!(
        recovery["result"]["decision"]["reason_label"],
        "ACCOUNT_RECOVERY_REJECTED"
    );
    assert_eq!(recovery["result"]["decision"]["requires_user_action"], true);

    let plaintext = command_value("run_secure_backup_restore_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_EXPORT_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["requires_backup_reconfiguration"],
        true
    );

    let mls_rekey = command_value("run_secure_backup_restore_mls_rekey_rejected");
    assert_eq!(mls_rekey["result"]["decision"]["accepted"], false);
    assert_eq!(
        mls_rekey["result"]["decision"]["reason_label"],
        "RESTORE_REKEY_MISSING"
    );
    assert_eq!(
        mls_rekey["result"]["decision"]["requires_group_rekey"],
        true
    );

    let cloud_policy = command_value("run_secure_backup_restore_cloud_policy_rejected");
    assert_eq!(cloud_policy["result"]["decision"]["accepted"], false);
    assert_eq!(
        cloud_policy["result"]["decision"]["reason_label"],
        "OS_BACKUP_POLICY_REJECTED"
    );
    assert_eq!(
        cloud_policy["result"]["decision"]["requires_backup_reconfiguration"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_event_chain_commands() {
    let ready = command_value("run_sealed_audit_event_chain_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_event_chain_ready"
    );
    assert_eq!(ready["result"]["surface"], "sealed_audit_event_chain");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_append_event"], true);
    assert_eq!(ready["result"]["decision"]["can_verify_inclusion"], true);
    assert_eq!(ready["result"]["decision"]["can_detect_rollback"], true);
    assert_eq!(
        ready["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );

    let plaintext = command_value("run_sealed_audit_event_chain_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_EVENT_FORBIDDEN"
    );
    assert_eq!(plaintext["result"]["decision"]["requires_redaction"], true);

    let rollback = command_value("run_sealed_audit_event_chain_rollback_rejected");
    assert_eq!(rollback["result"]["decision"]["accepted"], false);
    assert_eq!(
        rollback["result"]["decision"]["reason_label"],
        "ROLLBACK_PROTECTION_MISSING"
    );
    assert_eq!(
        rollback["result"]["decision"]["requires_storage_repair"],
        true
    );

    let witness = command_value("run_sealed_audit_event_chain_witness_rejected");
    assert_eq!(witness["result"]["decision"]["accepted"], false);
    assert_eq!(
        witness["result"]["decision"]["reason_label"],
        "WITNESS_QUORUM_MISSING"
    );
    assert_eq!(
        witness["result"]["decision"]["requires_transparency_setup"],
        true
    );

    let binding = command_value("run_sealed_audit_event_chain_binding_rejected");
    assert_eq!(binding["result"]["decision"]["accepted"], false);
    assert_eq!(
        binding["result"]["decision"]["reason_label"],
        "EVENT_BINDING_MISSING"
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_event_store_commands() {
    let ready = command_value("run_sealed_audit_event_store_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_event_store_ready"
    );
    assert_eq!(ready["result"]["surface"], "sealed_audit_event_store");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(ready["result"]["decision"]["can_publish_receipt"], true);
    assert_eq!(ready["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(
        ready["result"]["decision"]["keeps_plaintext_metadata"],
        false
    );
    assert_eq!(ready["result"]["store"]["has_sequence"], true);
    assert_eq!(ready["result"]["store"]["checkpoint_recorded"], true);

    let chain = command_value("run_sealed_audit_event_store_chain_rejected");
    assert_eq!(chain["result"]["decision"]["accepted"], false);
    assert_eq!(
        chain["result"]["decision"]["reason_label"],
        "CHAIN_REJECTED"
    );
    assert_eq!(chain["result"]["store"]["record_count"], 0);

    let duplicate = command_value("run_sealed_audit_event_store_duplicate_rejected");
    assert_eq!(duplicate["result"]["decision"]["accepted"], false);
    assert_eq!(
        duplicate["result"]["decision"]["reason_label"],
        "DUPLICATE_SEQUENCE"
    );
    assert_eq!(duplicate["result"]["decision"]["record_count"], 1);

    let rollback = command_value("run_sealed_audit_event_store_rollback_rejected");
    assert_eq!(rollback["result"]["decision"]["accepted"], false);
    assert_eq!(
        rollback["result"]["decision"]["reason_label"],
        "ROLLBACK_SEQUENCE"
    );
    assert_eq!(rollback["result"]["store"]["highest_event_sequence"], 42);

    let plaintext = command_value("run_sealed_audit_event_store_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_witness_checkpoint_commands() {
    let ready = command_value("run_sealed_audit_witness_checkpoint_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_witness_checkpoint_ready"
    );
    assert_eq!(
        ready["result"]["surface"],
        "sealed_audit_witness_checkpoint"
    );
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(
        ready["result"]["decision"]["signature_algorithm_label"],
        "hybrid_ed25519_ml_dsa_44"
    );
    assert_eq!(ready["result"]["decision"]["can_publish_checkpoint"], true);
    assert_eq!(
        ready["result"]["decision"]["can_request_witness_cosignature"],
        true
    );
    assert_eq!(ready["result"]["decision"]["can_monitor_privately"], true);

    let store = command_value("run_sealed_audit_witness_checkpoint_store_rejected");
    assert_eq!(store["result"]["decision"]["accepted"], false);
    assert_eq!(
        store["result"]["decision"]["reason_label"],
        "STORE_REJECTED"
    );

    let quorum = command_value("run_sealed_audit_witness_checkpoint_quorum_rejected");
    assert_eq!(quorum["result"]["decision"]["accepted"], false);
    assert_eq!(
        quorum["result"]["decision"]["reason_label"],
        "WITNESS_QUORUM_REJECTED"
    );
    assert_eq!(
        quorum["result"]["decision"]["requires_witness_repair"],
        true
    );

    let split_view = command_value("run_sealed_audit_witness_checkpoint_split_view_rejected");
    assert_eq!(split_view["result"]["decision"]["accepted"], false);
    assert_eq!(
        split_view["result"]["decision"]["reason_label"],
        "SPLIT_VIEW_EVIDENCE"
    );
    assert_eq!(
        split_view["result"]["decision"]["requires_user_warning"],
        true
    );

    let privacy = command_value("run_sealed_audit_witness_checkpoint_privacy_rejected");
    assert_eq!(privacy["result"]["decision"]["accepted"], false);
    assert_eq!(
        privacy["result"]["decision"]["reason_label"],
        "MONITOR_PRIVACY_REJECTED"
    );
    assert_eq!(
        privacy["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_witness_client_commands() {
    let ready = command_value("run_sealed_audit_witness_client_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_witness_client_ready"
    );
    assert_eq!(ready["result"]["surface"], "sealed_audit_witness_client");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(
        ready["result"]["decision"]["can_submit_add_checkpoint"],
        true
    );
    assert_eq!(
        ready["result"]["decision"]["can_publish_witnessed_checkpoint"],
        true
    );
    assert_eq!(ready["result"]["decision"]["can_monitor_privately"], true);

    let conflict = command_value("run_sealed_audit_witness_client_conflict");
    assert_eq!(conflict["result"]["decision"]["accepted"], false);
    assert_eq!(
        conflict["result"]["decision"]["reason_label"],
        "WITNESS_CONFLICT"
    );
    assert_eq!(
        conflict["result"]["decision"]["can_retry_witness_conflict"],
        true
    );
    assert_eq!(
        conflict["result"]["decision"]["requires_operator_alert"],
        true
    );

    let unavailable = command_value("run_sealed_audit_witness_client_unavailable");
    assert_eq!(unavailable["result"]["decision"]["accepted"], false);
    assert_eq!(
        unavailable["result"]["decision"]["reason_label"],
        "WITNESS_UNAVAILABLE"
    );
    assert_eq!(
        unavailable["result"]["decision"]["requires_witness_repair"],
        true
    );

    let policy = command_value("run_sealed_audit_witness_client_policy_rejected");
    assert_eq!(policy["result"]["decision"]["accepted"], false);
    assert_eq!(
        policy["result"]["decision"]["reason_label"],
        "POLICY_REJECTED"
    );
    assert_eq!(
        policy["result"]["decision"]["requires_policy_rotation"],
        true
    );

    let privacy = command_value("run_sealed_audit_witness_client_monitor_privacy_rejected");
    assert_eq!(privacy["result"]["decision"]["accepted"], false);
    assert_eq!(
        privacy["result"]["decision"]["reason_label"],
        "MONITOR_PRIVACY_REJECTED"
    );
    assert_eq!(
        privacy["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_proof_bundle_commands() {
    let ready = command_value("run_sealed_audit_proof_bundle_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_proof_bundle_ready"
    );
    assert_eq!(ready["result"]["surface"], "sealed_audit_proof_bundle");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_verify_offline"], true);
    assert_eq!(
        ready["result"]["decision"]["can_persist_proof_bundle"],
        true
    );
    assert_eq!(ready["result"]["decision"]["can_show_ui_status"], true);

    let client = command_value("run_sealed_audit_proof_bundle_client_rejected");
    assert_eq!(client["result"]["decision"]["accepted"], false);
    assert_eq!(
        client["result"]["decision"]["reason_label"],
        "WITNESS_CLIENT_REJECTED"
    );
    assert_eq!(client["result"]["decision"]["can_verify_offline"], false);

    let stale = command_value("run_sealed_audit_proof_bundle_stale_witness");
    assert_eq!(stale["result"]["decision"]["accepted"], false);
    assert_eq!(
        stale["result"]["decision"]["reason_label"],
        "WITNESS_FRESHNESS_REJECTED"
    );
    assert_eq!(
        stale["result"]["decision"]["requires_witness_refresh"],
        true
    );

    let policy = command_value("run_sealed_audit_proof_bundle_policy_rejected");
    assert_eq!(policy["result"]["decision"]["accepted"], false);
    assert_eq!(
        policy["result"]["decision"]["reason_label"],
        "POLICY_REJECTED"
    );
    assert_eq!(
        policy["result"]["decision"]["requires_policy_refresh"],
        true
    );

    let privacy = command_value("run_sealed_audit_proof_bundle_privacy_rejected");
    assert_eq!(privacy["result"]["decision"]["accepted"], false);
    assert_eq!(
        privacy["result"]["decision"]["reason_label"],
        "PRIVACY_REJECTED"
    );
    assert_eq!(privacy["result"]["decision"]["requires_redaction"], true);
    assert_eq!(
        privacy["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_proof_cache_commands() {
    let ready = command_value("run_sealed_audit_proof_cache_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_proof_cache_ready"
    );
    assert_eq!(ready["result"]["surface"], "sealed_audit_proof_cache");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(ready["result"]["decision"]["can_verify_offline"], true);
    assert_eq!(ready["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(ready["result"]["cache"]["has_record"], true);

    let bundle = command_value("run_sealed_audit_proof_cache_bundle_rejected");
    assert_eq!(bundle["result"]["decision"]["accepted"], false);
    assert_eq!(
        bundle["result"]["decision"]["reason_label"],
        "PROOF_BUNDLE_REJECTED"
    );
    assert_eq!(bundle["result"]["cache"]["record_count"], 0);

    let duplicate = command_value("run_sealed_audit_proof_cache_duplicate_rejected");
    assert_eq!(duplicate["result"]["decision"]["accepted"], false);
    assert_eq!(
        duplicate["result"]["decision"]["reason_label"],
        "DUPLICATE_PROOF"
    );
    assert_eq!(duplicate["result"]["decision"]["record_count"], 1);

    let policy = command_value("run_sealed_audit_proof_cache_policy_stale");
    assert_eq!(policy["result"]["decision"]["accepted"], false);
    assert_eq!(
        policy["result"]["decision"]["reason_label"],
        "POLICY_SNAPSHOT_STALE"
    );
    assert_eq!(
        policy["result"]["decision"]["requires_policy_refresh"],
        true
    );

    let plaintext = command_value("run_sealed_audit_proof_cache_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_verifier_policy_commands() {
    let ready = command_value("run_sealed_audit_verifier_policy_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_verifier_policy_ready"
    );
    assert_eq!(ready["result"]["surface"], "sealed_audit_verifier_policy");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_snapshot"], true);
    assert_eq!(
        ready["result"]["decision"]["can_schedule_private_monitor"],
        true
    );
    assert_eq!(ready["result"]["store"]["has_snapshot"], true);

    let expired = command_value("run_sealed_audit_verifier_policy_expired");
    assert_eq!(expired["result"]["decision"]["accepted"], false);
    assert_eq!(
        expired["result"]["decision"]["reason_label"],
        "POLICY_SNAPSHOT_EXPIRED"
    );
    assert_eq!(
        expired["result"]["decision"]["requires_policy_refresh"],
        true
    );

    let rotation = command_value("run_sealed_audit_verifier_policy_key_rotation_required");
    assert_eq!(rotation["result"]["decision"]["accepted"], false);
    assert_eq!(
        rotation["result"]["decision"]["reason_label"],
        "KEY_ROTATION_REQUIRED"
    );
    assert_eq!(
        rotation["result"]["decision"]["requires_key_rotation"],
        true
    );

    let monitor = command_value("run_sealed_audit_verifier_policy_monitor_privacy_rejected");
    assert_eq!(monitor["result"]["decision"]["accepted"], false);
    assert_eq!(
        monitor["result"]["decision"]["reason_label"],
        "MONITOR_FRESHNESS_STALE"
    );
    assert_eq!(
        monitor["result"]["decision"]["requires_monitor_refresh"],
        true
    );

    let plaintext = command_value("run_sealed_audit_verifier_policy_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_incident_evidence_commands() {
    let ready = command_value("run_sealed_audit_incident_evidence_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_incident_evidence_ready"
    );
    assert_eq!(ready["result"]["surface"], "sealed_audit_incident_evidence");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(ready["result"]["decision"]["can_escalate_incident"], true);
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let policy = command_value("run_sealed_audit_incident_evidence_policy_rejected");
    assert_eq!(policy["result"]["decision"]["accepted"], false);
    assert_eq!(
        policy["result"]["decision"]["reason_label"],
        "VERIFIER_POLICY_REJECTED"
    );

    let missing = command_value("run_sealed_audit_incident_evidence_missing_proof_report");
    assert_eq!(missing["result"]["decision"]["accepted"], false);
    assert_eq!(
        missing["result"]["decision"]["reason_label"],
        "MISSING_PROOF_REPORT_REQUIRED"
    );
    assert_eq!(
        missing["result"]["decision"]["requires_missing_proof_report"],
        true
    );

    let split = command_value("run_sealed_audit_incident_evidence_split_view");
    assert_eq!(split["result"]["decision"]["accepted"], false);
    assert_eq!(
        split["result"]["decision"]["reason_label"],
        "SPLIT_VIEW_EVIDENCE_REQUIRED"
    );
    assert_eq!(
        split["result"]["decision"]["requires_split_view_escalation"],
        true
    );

    let plaintext = command_value("run_sealed_audit_incident_evidence_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_recovery_export_commands() {
    let ready = command_value("run_sealed_audit_recovery_export_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_recovery_export_ready"
    );
    assert_eq!(ready["result"]["surface"], "sealed_audit_recovery_export");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(ready["result"]["decision"]["can_restore_state"], true);
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let incident = command_value("run_sealed_audit_recovery_export_incident_rejected");
    assert_eq!(incident["result"]["decision"]["accepted"], false);
    assert_eq!(
        incident["result"]["decision"]["reason_label"],
        "INCIDENT_EVIDENCE_REJECTED"
    );

    let quorum = command_value("run_sealed_audit_recovery_export_quorum_required");
    assert_eq!(quorum["result"]["decision"]["accepted"], false);
    assert_eq!(
        quorum["result"]["decision"]["reason_label"],
        "RESTORE_QUORUM_REQUIRED"
    );
    assert_eq!(
        quorum["result"]["decision"]["requires_restore_quorum"],
        true
    );

    let rollback = command_value("run_sealed_audit_recovery_export_rollback_rejected");
    assert_eq!(rollback["result"]["decision"]["accepted"], false);
    assert_eq!(
        rollback["result"]["decision"]["reason_label"],
        "ROLLBACK_EXPORT_REJECTED"
    );
    assert_eq!(rollback["result"]["decision"]["rejects_rollback"], true);

    let plaintext = command_value("run_sealed_audit_recovery_export_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_database_and_report_transport_commands() {
    let database = command_value("run_sealed_audit_database_adapter_ready");
    assert_eq!(
        database["command"]["command_kind_label"],
        "run_sealed_audit_database_adapter_ready"
    );
    assert_eq!(
        database["result"]["surface"],
        "sealed_audit_database_adapter"
    );
    assert_eq!(database["result"]["decision"]["accepted"], true);
    assert_eq!(
        database["result"]["decision"]["can_persist_sealed_audit"],
        true
    );

    let encryption = command_value("run_sealed_audit_database_adapter_encryption_rejected");
    assert_eq!(encryption["result"]["decision"]["accepted"], false);
    assert_eq!(
        encryption["result"]["decision"]["reason_label"],
        "DATABASE_ENCRYPTION_REQUIRED"
    );
    assert_eq!(
        encryption["result"]["decision"]["requires_database_encryption"],
        true
    );

    let append_only = command_value("run_sealed_audit_database_adapter_append_only_rejected");
    assert_eq!(append_only["result"]["decision"]["accepted"], false);
    assert_eq!(
        append_only["result"]["decision"]["reason_label"],
        "APPEND_ONLY_GUARD_REQUIRED"
    );
    assert_eq!(
        append_only["result"]["decision"]["requires_append_only_guard"],
        true
    );

    let transport = command_value("run_sealed_audit_private_report_transport_ready");
    assert_eq!(
        transport["command"]["command_kind_label"],
        "run_sealed_audit_private_report_transport_ready"
    );
    assert_eq!(
        transport["result"]["surface"],
        "sealed_audit_private_report_transport"
    );
    assert_eq!(transport["result"]["decision"]["accepted"], true);
    assert_eq!(
        transport["result"]["decision"]["can_submit_private_report"],
        true
    );

    let plaintext = command_value("run_sealed_audit_private_report_transport_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_private_report_outbox_commands() {
    let ready = command_value("run_sealed_audit_private_report_outbox_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_private_report_outbox_ready"
    );
    assert_eq!(
        ready["result"]["surface"],
        "sealed_audit_private_report_outbox"
    );
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(ready["result"]["decision"]["can_submit_now"], true);
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let transport = command_value("run_sealed_audit_private_report_outbox_transport_rejected");
    assert_eq!(transport["result"]["decision"]["accepted"], false);
    assert_eq!(
        transport["result"]["decision"]["reason_label"],
        "PRIVATE_REPORT_TRANSPORT_REQUIRED"
    );

    let replay = command_value("run_sealed_audit_private_report_outbox_replay_rejected");
    assert_eq!(replay["result"]["decision"]["accepted"], false);
    assert_eq!(
        replay["result"]["decision"]["reason_label"],
        "REPLAY_GUARD_REQUIRED"
    );
    assert_eq!(replay["result"]["decision"]["requires_replay_guard"], true);

    let rate_limit = command_value("run_sealed_audit_private_report_outbox_rate_limit_rejected");
    assert_eq!(rate_limit["result"]["decision"]["accepted"], false);
    assert_eq!(
        rate_limit["result"]["decision"]["reason_label"],
        "RATE_LIMIT_TOKEN_REQUIRED"
    );
    assert_eq!(
        rate_limit["result"]["decision"]["requires_rate_limit_token"],
        true
    );

    let plaintext = command_value("run_sealed_audit_private_report_outbox_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_private_report_receipt_commands() {
    let ready = command_value("run_sealed_audit_private_report_receipt_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_private_report_receipt_ready"
    );
    assert_eq!(
        ready["result"]["surface"],
        "sealed_audit_private_report_receipt"
    );
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(ready["result"]["decision"]["can_mark_delivered"], true);
    assert_eq!(ready["result"]["decision"]["can_stop_retrying"], true);
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let outbox = command_value("run_sealed_audit_private_report_receipt_outbox_rejected");
    assert_eq!(outbox["result"]["decision"]["accepted"], false);
    assert_eq!(
        outbox["result"]["decision"]["reason_label"],
        "PRIVATE_REPORT_OUTBOX_REQUIRED"
    );

    let missing = command_value("run_sealed_audit_private_report_receipt_missing");
    assert_eq!(missing["result"]["decision"]["accepted"], false);
    assert_eq!(
        missing["result"]["decision"]["reason_label"],
        "RECEIPT_REQUIRED"
    );
    assert_eq!(missing["result"]["decision"]["requires_receipt"], true);

    let transparency =
        command_value("run_sealed_audit_private_report_receipt_transparency_rejected");
    assert_eq!(transparency["result"]["decision"]["accepted"], false);
    assert_eq!(
        transparency["result"]["decision"]["reason_label"],
        "GATEWAY_TRANSPARENCY_REQUIRED"
    );
    assert_eq!(
        transparency["result"]["decision"]["requires_gateway_transparency"],
        true
    );

    let plaintext = command_value("run_sealed_audit_private_report_receipt_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_private_report_reconciliation_commands() {
    let ready = command_value("run_sealed_audit_private_report_reconciliation_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_private_report_reconciliation_ready"
    );
    assert_eq!(
        ready["result"]["surface"],
        "sealed_audit_private_report_reconciliation"
    );
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(ready["result"]["decision"]["can_reconcile_delivery"], true);
    assert_eq!(ready["result"]["decision"]["can_schedule_retry"], false);
    assert_eq!(ready["result"]["decision"]["can_show_retry_status"], true);
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let receipt = command_value("run_sealed_audit_private_report_reconciliation_receipt_rejected");
    assert_eq!(receipt["result"]["decision"]["accepted"], false);
    assert_eq!(
        receipt["result"]["decision"]["reason_label"],
        "PRIVATE_REPORT_RECEIPT_REQUIRED"
    );
    assert_eq!(
        receipt["result"]["decision"]["requires_private_report_receipt"],
        true
    );

    let retry = command_value("run_sealed_audit_private_report_reconciliation_retry_rejected");
    assert_eq!(retry["result"]["decision"]["accepted"], false);
    assert_eq!(
        retry["result"]["decision"]["reason_label"],
        "RETRY_SCHEDULE_REQUIRED"
    );
    assert_eq!(retry["result"]["decision"]["requires_retry_schedule"], true);

    let false_delivery =
        command_value("run_sealed_audit_private_report_reconciliation_false_delivery_rejected");
    assert_eq!(false_delivery["result"]["decision"]["accepted"], false);
    assert_eq!(
        false_delivery["result"]["decision"]["reason_label"],
        "FALSE_DELIVERY_REJECTED"
    );
    assert_eq!(
        false_delivery["result"]["decision"]["rejects_false_delivery"],
        true
    );

    let plaintext =
        command_value("run_sealed_audit_private_report_reconciliation_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_sealed_audit_private_report_gateway_evidence_commands() {
    let ready = command_value("run_sealed_audit_private_report_gateway_evidence_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_sealed_audit_private_report_gateway_evidence_ready"
    );
    assert_eq!(
        ready["result"]["surface"],
        "sealed_audit_private_report_gateway_evidence"
    );
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(
        ready["result"]["decision"]["can_raise_gateway_incident"],
        true
    );
    assert_eq!(ready["result"]["decision"]["can_notify_operator"], true);
    assert_eq!(
        ready["result"]["decision"]["can_show_unavailable_status"],
        true
    );
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let reconciliation =
        command_value("run_sealed_audit_private_report_gateway_evidence_reconciliation_rejected");
    assert_eq!(reconciliation["result"]["decision"]["accepted"], false);
    assert_eq!(
        reconciliation["result"]["decision"]["reason_label"],
        "PRIVATE_REPORT_RECONCILIATION_REQUIRED"
    );
    assert_eq!(
        reconciliation["result"]["decision"]["requires_private_report_reconciliation"],
        true
    );

    let unavailable =
        command_value("run_sealed_audit_private_report_gateway_evidence_unavailable_rejected");
    assert_eq!(unavailable["result"]["decision"]["accepted"], false);
    assert_eq!(
        unavailable["result"]["decision"]["reason_label"],
        "UNAVAILABLE_EVIDENCE_REQUIRED"
    );
    assert_eq!(
        unavailable["result"]["decision"]["requires_unavailable_evidence"],
        true
    );

    let accountability =
        command_value("run_sealed_audit_private_report_gateway_evidence_accountability_rejected");
    assert_eq!(accountability["result"]["decision"]["accepted"], false);
    assert_eq!(
        accountability["result"]["decision"]["reason_label"],
        "ACCOUNTABILITY_ROUTE_REQUIRED"
    );
    assert_eq!(
        accountability["result"]["decision"]["requires_accountability_route"],
        true
    );

    let plaintext =
        command_value("run_sealed_audit_private_report_gateway_evidence_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_receive_session_readiness() {
    let descriptor =
        backend_command_by_name("run_receive_session_ordering_gap").expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_receive_session_ordering_gap"
    );
    assert_eq!(value["result"]["surface"], "prototype_receive_session");
    assert_eq!(value["result"]["outcome"]["completed"], false);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "client_receive_rejected"
    );
    assert_eq!(
        value["result"]["outcome"]["client_receive"]["reason_label"],
        "ORDERING_GAP"
    );
    assert_eq!(
        value["result"]["events"][0]["kind_label"],
        "receive_started"
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "client_receive_evaluated"
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["terminal"],
        true
    );
}

#[test]
fn backend_command_json_wraps_inbound_sync_readiness() {
    let descriptor = backend_command_by_name("run_inbound_sync_bootstrap_blocked")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(value["command"]["actor_kind_label"], "human");
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_inbound_sync_bootstrap_blocked"
    );
    assert_eq!(value["command"]["can_run_session"], true);
    assert_eq!(value["result"]["surface"], "inbound_sync_gate");
    assert_eq!(value["result"]["decision"]["accepted"], false);
    assert_eq!(
        value["result"]["decision"]["reason_label"],
        "BOOTSTRAP_BLOCKED"
    );
    assert_eq!(
        value["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_inbound_sync_session_readiness() {
    let descriptor =
        backend_command_by_name("run_inbound_sync_session_idle").expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_inbound_sync_session_idle"
    );
    assert_eq!(value["result"]["surface"], "prototype_inbound_sync_session");
    assert_eq!(value["result"]["outcome"]["completed"], true);
    assert_eq!(value["result"]["outcome"]["reason_label"], "sync_idle");
    assert_eq!(value["result"]["outcome"]["receive_ran"], false);
    assert_eq!(
        value["result"]["events"][0]["kind_label"],
        "sync_gate_evaluated"
    );
}

#[test]
fn backend_command_json_wraps_authenticated_relay_source_readiness() {
    let descriptor = backend_command_by_name("run_authenticated_relay_source_auth_rejected")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_authenticated_relay_source_auth_rejected"
    );
    assert_eq!(value["result"]["surface"], "authenticated_relay_source");
    assert_eq!(value["result"]["decision"]["accepted"], false);
    assert_eq!(
        value["result"]["decision"]["reason_label"],
        "SERVER_AUTHENTICATION_REJECTED"
    );
    assert_eq!(
        value["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
    assert_eq!(
        value["result"]["inbound_sync"]["reason_label"],
        "TRANSPORT_AUTH_REJECTED"
    );
}

#[test]
fn backend_command_json_wraps_media_object_store_readiness() {
    let descriptor = backend_command_by_name("run_media_object_store_plaintext_rejected")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_object_store_plaintext_rejected"
    );
    assert_eq!(value["result"]["surface"], "media_object_store");
    assert_eq!(value["result"]["decision"]["accepted"], false);
    assert_eq!(
        value["result"]["decision"]["reason_label"],
        "PLAINTEXT_UPLOAD_FORBIDDEN"
    );
    assert_eq!(
        value["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_media_upload_session_readiness() {
    let descriptor = backend_command_by_name("run_media_upload_session_plaintext_rejected")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_upload_session_plaintext_rejected"
    );
    assert_eq!(value["result"]["surface"], "prototype_media_upload_session");
    assert_eq!(value["result"]["outcome"]["completed"], false);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "media_object_store_rejected"
    );
    assert_eq!(
        value["result"]["outcome"]["media"]["reason_label"],
        "PLAINTEXT_UPLOAD_FORBIDDEN"
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "media_object_store_evaluated"
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_media_service_adapter_readiness() {
    let descriptor = backend_command_by_name("run_media_service_adapter_auth_missing")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_service_adapter_auth_missing"
    );
    assert_eq!(value["result"]["surface"], "media_service_adapter");
    assert_eq!(value["result"]["decision"]["accepted"], false);
    assert_eq!(
        value["result"]["decision"]["reason_label"],
        "SERVICE_AUTHENTICATION_MISSING"
    );
    assert_eq!(value["result"]["decision"]["requires_network_setup"], true);
    assert_eq!(
        value["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_media_service_upload_session_readiness() {
    let descriptor = backend_command_by_name("run_media_service_upload_session_auth_rejected")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_service_upload_session_auth_rejected"
    );
    assert_eq!(
        value["result"]["surface"],
        "prototype_media_service_upload_session"
    );
    assert_eq!(value["result"]["outcome"]["completed"], false);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "media_service_adapter_rejected"
    );
    assert_eq!(
        value["result"]["outcome"]["media_service"]["reason_label"],
        "SERVICE_AUTHENTICATION_MISSING"
    );
    assert_eq!(value["result"]["outcome"]["service_upload_calls"], 0);
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "media_service_adapter_evaluated"
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_media_service_download_readiness() {
    let descriptor = backend_command_by_name("run_media_service_download_auth_missing")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_service_download_auth_missing"
    );
    assert_eq!(value["result"]["surface"], "media_service_download");
    assert_eq!(value["result"]["decision"]["accepted"], false);
    assert_eq!(
        value["result"]["decision"]["reason_label"],
        "SERVICE_AUTHENTICATION_MISSING"
    );
    assert_eq!(value["result"]["decision"]["requires_network_setup"], true);
    assert_eq!(
        value["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_media_download_session_readiness() {
    let descriptor = backend_command_by_name("run_media_download_session_open_rejected")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_download_session_open_rejected"
    );
    assert_eq!(
        value["result"]["surface"],
        "prototype_media_download_session"
    );
    assert_eq!(value["result"]["outcome"]["completed"], false);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "local_store_open_rejected"
    );
    assert_eq!(value["result"]["outcome"]["service_download_calls"], 1);
    assert_eq!(value["result"]["outcome"]["crypto_open_calls"], 0);
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "local_store_open_evaluated"
    );
}

#[test]
fn backend_command_json_wraps_media_retention_readiness() {
    let descriptor =
        backend_command_by_name("run_media_retention_hold_rejected").expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_retention_hold_rejected"
    );
    assert_eq!(value["result"]["surface"], "media_retention");
    assert_eq!(value["result"]["decision"]["accepted"], false);
    assert_eq!(
        value["result"]["decision"]["reason_label"],
        "RETENTION_HOLD_ACTIVE"
    );
    assert_eq!(
        value["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
    assert_eq!(value["result"]["decision"]["keeps_audit_hash"], true);
}

#[test]
fn backend_command_json_wraps_media_cleanup_session_readiness() {
    let descriptor = backend_command_by_name("run_media_cleanup_session_retention_rejected")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_cleanup_session_retention_rejected"
    );
    assert_eq!(
        value["result"]["surface"],
        "prototype_media_cleanup_session"
    );
    assert_eq!(value["result"]["outcome"]["completed"], false);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "media_retention_rejected"
    );
    assert_eq!(value["result"]["outcome"]["remote_delete_calls"], 0);
}

#[test]
fn backend_command_json_wraps_media_object_index_readiness() {
    let descriptor = backend_command_by_name("run_media_object_index_delete_pending_ready")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_object_index_delete_pending_ready"
    );
    assert_eq!(value["result"]["surface"], "media_object_index");
    assert_eq!(value["result"]["decision"]["accepted"], true);
    assert_eq!(
        value["result"]["decision"]["lifecycle_state_label"],
        "delete_pending"
    );
    assert_eq!(value["result"]["decision"]["can_download"], false);
    assert_eq!(value["result"]["decision"]["can_cleanup"], true);
    assert_eq!(
        value["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_media_object_index_store_readiness() {
    let descriptor = backend_command_by_name("run_media_object_index_store_index_rejected")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_media_object_index_store_index_rejected"
    );
    assert_eq!(value["result"]["surface"], "media_object_index_store");
    assert_eq!(value["result"]["decision"]["accepted"], false);
    assert_eq!(
        value["result"]["decision"]["reason_label"],
        "INDEX_REJECTED"
    );
    assert_eq!(
        value["result"]["decision"]["index"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(value["result"]["decision"]["persisted_record"], false);
}

#[test]
fn backend_command_json_wraps_indexed_media_upload_session_readiness() {
    let descriptor =
        backend_command_by_name("run_indexed_media_upload_session_index_store_rejected")
            .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_indexed_media_upload_session_index_store_rejected"
    );
    assert_eq!(
        value["result"]["surface"],
        "prototype_indexed_media_upload_session"
    );
    assert_eq!(value["result"]["outcome"]["completed"], false);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "media_object_index_store_rejected"
    );
    assert_eq!(
        value["result"]["outcome"]["index_store"]["reason_label"],
        "INDEX_REJECTED"
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "media_object_index_store_evaluated"
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_indexed_media_download_session_readiness() {
    let descriptor = backend_command_by_name("run_indexed_media_download_session_not_downloadable")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_indexed_media_download_session_not_downloadable"
    );
    assert_eq!(
        value["result"]["surface"],
        "prototype_indexed_media_download_session"
    );
    assert_eq!(value["result"]["outcome"]["completed"], false);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "media_object_not_downloadable"
    );
    assert_eq!(
        value["result"]["outcome"]["index_store"]["index"]["can_download"],
        false
    );
    assert_eq!(
        value["result"]["outcome"]["download"],
        serde_json::Value::Null
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "media_object_index_store_evaluated"
    );
}

#[test]
fn backend_command_json_wraps_indexed_media_cleanup_session_readiness() {
    let descriptor = backend_command_by_name("run_indexed_media_cleanup_session_cleanup_rejected")
        .expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["command"]["accepted"], true);
    assert_eq!(
        value["command"]["command_kind_label"],
        "run_indexed_media_cleanup_session_cleanup_rejected"
    );
    assert_eq!(
        value["result"]["surface"],
        "prototype_indexed_media_cleanup_session"
    );
    assert_eq!(value["result"]["outcome"]["completed"], false);
    assert_eq!(
        value["result"]["outcome"]["reason_label"],
        "media_cleanup_rejected"
    );
    assert_eq!(
        value["result"]["outcome"]["cleanup"]["reason_label"],
        "media_retention_rejected"
    );
    assert_eq!(
        value["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "media_cleanup_session_evaluated"
    );
}

#[test]
fn backend_command_json_wraps_anonymous_security_readiness() {
    let issuer = command_value("run_anonymous_credential_issuer_trust_ready");
    assert_eq!(
        issuer["command"]["command_kind_label"],
        "run_anonymous_credential_issuer_trust_ready"
    );
    assert_eq!(
        issuer["result"]["surface"],
        "anonymous_credential_issuer_trust"
    );
    assert_eq!(issuer["result"]["decision"]["accepted"], true);
    assert_eq!(
        issuer["result"]["decision"]["can_use_for_anonymous_membership_proof"],
        true
    );

    let proof = command_value("run_anonymous_group_membership_proof_high_security_pq_required");
    assert_eq!(
        proof["command"]["command_kind_label"],
        "run_anonymous_group_membership_proof_high_security_pq_required"
    );
    assert_eq!(
        proof["result"]["surface"],
        "anonymous_group_membership_proof"
    );
    assert_eq!(proof["result"]["decision"]["accepted"], false);
    assert_eq!(
        proof["result"]["decision"]["reason_label"],
        "HIGH_SECURITY_REQUIRES_PQ_PROOF"
    );
    assert_eq!(proof["result"]["decision"]["requires_rekey"], true);

    let nullifier = command_value("run_anonymous_rate_limit_nullifier_ready");
    assert_eq!(
        nullifier["command"]["command_kind_label"],
        "run_anonymous_rate_limit_nullifier_ready"
    );
    assert_eq!(
        nullifier["result"]["surface"],
        "anonymous_rate_limit_nullifier"
    );
    assert_eq!(nullifier["result"]["decision"]["accepted"], true);
    assert_eq!(
        nullifier["result"]["decision"]["can_rate_limit_without_identity"],
        true
    );
    assert_eq!(
        nullifier["result"]["decision"]["forbids_plaintext_rate_limit_metadata"],
        true
    );
}

#[test]
fn backend_command_json_wraps_anonymous_security_rejections() {
    let issuer =
        command_value("run_anonymous_credential_issuer_trust_partitioning_metadata_rejected");
    assert_eq!(issuer["result"]["decision"]["accepted"], false);
    assert_eq!(
        issuer["result"]["decision"]["reason_label"],
        "PARTITIONING_METADATA_PRESENT"
    );
    assert_eq!(
        issuer["result"]["decision"]["protects_anonymity_set"],
        false
    );

    let witness = command_value("run_anonymous_credential_issuer_trust_witness_audit_rejected");
    assert_eq!(witness["result"]["decision"]["accepted"], false);
    assert_eq!(
        witness["result"]["decision"]["reason_label"],
        "ISSUER_WITNESS_AUDIT_REJECTED"
    );
    assert_eq!(
        witness["result"]["input"]["issuer_witness_audit_reason_label"],
        "SPLIT_VIEW_REPORTED"
    );

    let proof = command_value("run_anonymous_group_membership_proof_plaintext_identity_rejected");
    assert_eq!(proof["result"]["decision"]["accepted"], false);
    assert_eq!(
        proof["result"]["decision"]["reason_label"],
        "PLAINTEXT_MEMBER_IDENTITY"
    );
    assert_eq!(
        proof["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );

    let nullifier = command_value("run_anonymous_rate_limit_nullifier_opaque_store_required");
    assert_eq!(nullifier["result"]["decision"]["accepted"], false);
    assert_eq!(
        nullifier["result"]["decision"]["reason_label"],
        "NULLIFIER_STORE_NOT_OPAQUE"
    );
    assert_eq!(nullifier["result"]["decision"]["requires_rekey"], true);
    assert_eq!(
        nullifier["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_group_chat_commands() {
    let ready = command_value("run_group_chat_mls_ready");
    assert_eq!(ready["command"]["accepted"], true);
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_group_chat_mls_ready"
    );
    assert_eq!(ready["result"]["surface"], "group_chat");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_open_group"], true);

    let provider = command_value("run_group_chat_mls_provider_security_required");
    assert_eq!(provider["result"]["decision"]["accepted"], false);
    assert_eq!(
        provider["result"]["decision"]["reason_label"],
        "MLS_PROVIDER_SECURITY_REJECTED"
    );
    assert_eq!(
        provider["result"]["input"]["mls_provider_security_reason_label"],
        "KNOWN_ANSWER_TESTS_MISSING"
    );

    let plaintext = command_value("run_group_chat_plaintext_metadata_forbidden");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_group_relay_envelope_commands() {
    let ready = command_value("run_group_relay_envelope_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_group_relay_envelope_ready"
    );
    assert_eq!(ready["result"]["surface"], "group_relay_envelope");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_enqueue_relay"], true);
    assert_eq!(
        ready["result"]["input"]["anonymous_rate_limit_accepted"],
        true
    );

    let missing_token = command_value("run_group_relay_envelope_missing_delivery_token");
    assert_eq!(
        missing_token["result"]["decision"]["reason_label"],
        "MISSING_DELIVERY_TOKEN"
    );
    assert_eq!(
        missing_token["result"]["decision"]["requires_user_action"],
        true
    );

    let plaintext = command_value("run_group_relay_envelope_plaintext_metadata_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_SENDER_METADATA"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_anonymous_nullifier_store_commands() {
    let ready = command_value("run_anonymous_nullifier_store_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_anonymous_nullifier_store_ready"
    );
    assert_eq!(ready["result"]["surface"], "anonymous_nullifier_store");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(
        ready["result"]["decision"]["keeps_context_digest_only"],
        true
    );
    assert_eq!(ready["result"]["store"]["record_count"], 1);

    let replay = command_value("run_anonymous_nullifier_store_replay_rejected");
    assert_eq!(replay["result"]["decision"]["accepted"], false);
    assert_eq!(
        replay["result"]["decision"]["reason_label"],
        "NULLIFIER_ALREADY_RECORDED"
    );
    assert_eq!(replay["result"]["store"]["record_count"], 1);

    let plaintext = command_value("run_anonymous_nullifier_store_plaintext_metadata_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_mls_provider_evidence_store_commands() {
    let ready = command_value("run_mls_provider_evidence_store_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_provider_evidence_store_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_provider_evidence_store");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(
        ready["result"]["decision"]["can_use_as_provider_evidence"],
        true
    );
    assert_eq!(ready["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(ready["result"]["store"]["record_count"], 1);

    let gate = command_value("run_mls_provider_evidence_store_gate_rejected");
    assert_eq!(gate["result"]["decision"]["accepted"], false);
    assert_eq!(
        gate["result"]["decision"]["reason_label"],
        "PROVIDER_SECURITY_REJECTED"
    );
    assert_eq!(
        gate["result"]["input"]["provider_security_reason_label"],
        "KNOWN_ANSWER_TESTS_MISSING"
    );

    let duplicate = command_value("run_mls_provider_evidence_store_duplicate_rejected");
    assert_eq!(duplicate["result"]["decision"]["accepted"], false);
    assert_eq!(
        duplicate["result"]["decision"]["reason_label"],
        "EVIDENCE_ALREADY_RECORDED"
    );
    assert_eq!(duplicate["result"]["store"]["record_count"], 1);

    let plaintext = command_value("run_mls_provider_evidence_store_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_EVIDENCE_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_mls_provider_evidence_use_commands() {
    let ready = command_value("run_mls_provider_evidence_use_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_provider_evidence_use_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_provider_evidence_use");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(
        ready["result"]["decision"]["can_use_provider_evidence"],
        true
    );

    let missing = command_value("run_mls_provider_evidence_use_missing");
    assert_eq!(missing["result"]["decision"]["accepted"], false);
    assert_eq!(
        missing["result"]["decision"]["reason_label"],
        "RECORD_MISSING"
    );
    assert_eq!(
        missing["result"]["decision"]["requires_provider_validation"],
        true
    );

    let expired = command_value("run_mls_provider_evidence_use_expired");
    assert_eq!(expired["result"]["decision"]["accepted"], false);
    assert_eq!(
        expired["result"]["decision"]["reason_label"],
        "EVIDENCE_EXPIRED"
    );

    let mismatch = command_value("run_mls_provider_evidence_use_suite_mismatch");
    assert_eq!(mismatch["result"]["decision"]["accepted"], false);
    assert_eq!(
        mismatch["result"]["decision"]["reason_label"],
        "SUITE_MISMATCH"
    );
    assert_eq!(mismatch["result"]["decision"]["requires_pq_upgrade"], true);

    let plaintext = command_value("run_mls_provider_evidence_use_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_EVIDENCE_DETECTED"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_mls_key_package_admission_commands() {
    let ready = command_value("run_mls_key_package_admission_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_key_package_admission_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_key_package_admission");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_add_member"], true);
    assert_eq!(ready["result"]["decision"]["can_send_welcome"], true);
    assert_eq!(ready["result"]["decision"]["prevents_key_reuse"], true);

    let group = command_value("run_mls_key_package_admission_group_rejected");
    assert_eq!(group["result"]["decision"]["accepted"], false);
    assert_eq!(
        group["result"]["decision"]["reason_label"],
        "GROUP_CHAT_REJECTED"
    );
    assert_eq!(group["result"]["decision"]["requires_sync"], true);

    let lifetime = command_value("run_mls_key_package_admission_lifetime_rejected");
    assert_eq!(lifetime["result"]["decision"]["accepted"], false);
    assert_eq!(
        lifetime["result"]["decision"]["reason_label"],
        "LIFETIME_NOT_CURRENT"
    );

    let suite = command_value("run_mls_key_package_admission_suite_mismatch");
    assert_eq!(suite["result"]["decision"]["accepted"], false);
    assert_eq!(
        suite["result"]["decision"]["reason_label"],
        "CIPHER_SUITE_MISMATCH"
    );
    assert_eq!(suite["result"]["decision"]["requires_pq_upgrade"], true);

    let credential = command_value("run_mls_key_package_admission_credential_rejected");
    assert_eq!(credential["result"]["decision"]["accepted"], false);
    assert_eq!(
        credential["result"]["decision"]["reason_label"],
        "CREDENTIAL_INVALID"
    );

    let replay = command_value("run_mls_key_package_admission_replay_rejected");
    assert_eq!(replay["result"]["decision"]["accepted"], false);
    assert_eq!(
        replay["result"]["decision"]["reason_label"],
        "KEY_PACKAGE_ALREADY_USED"
    );
    assert_eq!(replay["result"]["decision"]["requires_sync"], true);

    let plaintext = command_value("run_mls_key_package_admission_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_IDENTITY_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn backend_command_json_wraps_mls_key_package_consume_store_commands() {
    let ready = command_value("run_mls_key_package_consume_store_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_key_package_consume_store_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_key_package_consume_store");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(
        ready["result"]["decision"]["can_consume_key_package_once"],
        true
    );
    assert_eq!(ready["result"]["decision"]["can_send_welcome_once"], true);
    assert_eq!(
        ready["result"]["decision"]["binds_welcome_send_transaction"],
        true
    );

    let admission = command_value("run_mls_key_package_consume_store_admission_rejected");
    assert_eq!(admission["result"]["decision"]["accepted"], false);
    assert_eq!(
        admission["result"]["decision"]["reason_label"],
        "KEY_PACKAGE_ADMISSION_REJECTED"
    );

    let duplicate = command_value("run_mls_key_package_consume_store_duplicate_rejected");
    assert_eq!(duplicate["result"]["decision"]["accepted"], false);
    assert_eq!(
        duplicate["result"]["decision"]["reason_label"],
        "KEY_PACKAGE_ALREADY_CONSUMED"
    );
    assert_eq!(duplicate["result"]["decision"]["record_count"], 1);
    assert_eq!(
        duplicate["result"]["store"]["global_duplicate_check"]["keyed_by_key_package_hash"],
        true
    );

    let bad_shape = command_value("run_mls_key_package_consume_store_bad_shape");
    assert_eq!(bad_shape["result"]["decision"]["accepted"], false);
    assert_eq!(
        bad_shape["result"]["decision"]["reason_label"],
        "BAD_WELCOME_SEND_TRANSACTION_DIGEST"
    );

    let plaintext = command_value("run_mls_key_package_consume_store_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_mls_welcome_send_outbox_commands() {
    let ready = command_value("run_mls_welcome_send_outbox_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_welcome_send_outbox_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_welcome_send_outbox");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(
        ready["result"]["decision"]["can_enqueue_welcome_once"],
        true
    );
    assert_eq!(
        ready["result"]["decision"]["can_send_welcome_after_commit"],
        true
    );
    assert_eq!(ready["result"]["decision"]["binds_commit"], true);
    assert_eq!(ready["result"]["decision"]["binds_delivery_route"], true);

    let consume = command_value("run_mls_welcome_send_outbox_consume_rejected");
    assert_eq!(consume["result"]["decision"]["accepted"], false);
    assert_eq!(
        consume["result"]["decision"]["reason_label"],
        "KEY_PACKAGE_CONSUME_STORE_REJECTED"
    );

    let duplicate = command_value("run_mls_welcome_send_outbox_duplicate_transaction_rejected");
    assert_eq!(duplicate["result"]["decision"]["accepted"], false);
    assert_eq!(
        duplicate["result"]["decision"]["reason_label"],
        "WELCOME_SEND_TRANSACTION_ALREADY_QUEUED"
    );
    assert_eq!(duplicate["result"]["decision"]["record_count"], 1);

    let key_package = command_value("run_mls_welcome_send_outbox_key_package_queued");
    assert_eq!(key_package["result"]["decision"]["accepted"], false);
    assert_eq!(
        key_package["result"]["decision"]["reason_label"],
        "KEY_PACKAGE_ALREADY_QUEUED"
    );

    let bad_shape = command_value("run_mls_welcome_send_outbox_bad_shape");
    assert_eq!(bad_shape["result"]["decision"]["accepted"], false);
    assert_eq!(
        bad_shape["result"]["decision"]["reason_label"],
        "BAD_DELIVERY_ROUTE_ID"
    );

    let plaintext = command_value("run_mls_welcome_send_outbox_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_mls_membership_transaction_commands() {
    let ready = command_value("run_mls_membership_transaction_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_membership_transaction_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_membership_transaction");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(
        ready["result"]["decision"]["can_commit_membership_change_once"],
        true
    );
    assert_eq!(ready["result"]["decision"]["can_advance_epoch"], true);
    assert_eq!(
        ready["result"]["decision"]["can_send_welcome_from_outbox"],
        true
    );
    assert_eq!(
        ready["result"]["decision"]["uses_single_storage_transaction"],
        true
    );
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let binding = command_value("run_mls_membership_transaction_binding_rejected");
    assert_eq!(binding["result"]["decision"]["accepted"], false);
    assert_eq!(
        binding["result"]["decision"]["reason_label"],
        "BINDING_MISMATCH"
    );
    assert_eq!(binding["result"]["input"]["binding_group_ids_match"], false);

    let storage = command_value("run_mls_membership_transaction_storage_rejected");
    assert_eq!(storage["result"]["decision"]["accepted"], false);
    assert_eq!(
        storage["result"]["decision"]["reason_label"],
        "ATOMIC_TRANSACTION_MISSING"
    );
    assert_eq!(
        storage["result"]["input"]["single_storage_transaction"],
        false
    );

    let duplicate = command_value("run_mls_membership_transaction_duplicate_rejected");
    assert_eq!(duplicate["result"]["decision"]["accepted"], false);
    assert_eq!(
        duplicate["result"]["decision"]["reason_label"],
        "TRANSACTION_ALREADY_RECORDED"
    );
    assert_eq!(duplicate["result"]["decision"]["record_count"], 1);

    let plaintext = command_value("run_mls_membership_transaction_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_mls_welcome_admission_commands() {
    let ready = command_value("run_mls_welcome_admission_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_welcome_admission_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_welcome_admission");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_join_group"], true);
    assert_eq!(ready["result"]["decision"]["can_open_group"], true);
    assert_eq!(ready["result"]["decision"]["prevents_welcome_replay"], true);

    let secrets = command_value("run_mls_welcome_admission_secrets_missing");
    assert_eq!(secrets["result"]["decision"]["accepted"], false);
    assert_eq!(
        secrets["result"]["decision"]["reason_label"],
        "NO_MATCHING_ENCRYPTED_GROUP_SECRETS"
    );

    let tree = command_value("run_mls_welcome_admission_tree_rejected");
    assert_eq!(tree["result"]["decision"]["accepted"], false);
    assert_eq!(
        tree["result"]["decision"]["reason_label"],
        "RATCHET_TREE_HASH_MISMATCH"
    );
    assert_eq!(tree["result"]["decision"]["requires_sync"], true);

    let confirmation = command_value("run_mls_welcome_admission_confirmation_rejected");
    assert_eq!(confirmation["result"]["decision"]["accepted"], false);
    assert_eq!(
        confirmation["result"]["decision"]["reason_label"],
        "CONFIRMATION_TAG_INVALID"
    );

    let tie_break = command_value("run_mls_welcome_admission_tie_break_rejected");
    assert_eq!(tie_break["result"]["decision"]["accepted"], false);
    assert_eq!(
        tie_break["result"]["decision"]["reason_label"],
        "COMMIT_TIE_BREAK_REJECTED"
    );

    let replay = command_value("run_mls_welcome_admission_replay_rejected");
    assert_eq!(replay["result"]["decision"]["accepted"], false);
    assert_eq!(
        replay["result"]["decision"]["reason_label"],
        "WELCOME_ALREADY_PROCESSED"
    );

    let plaintext = command_value("run_mls_welcome_admission_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_GROUP_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_mls_commit_admission_commands() {
    let ready = command_value("run_mls_commit_admission_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_commit_admission_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_commit_admission");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["can_apply_commit"], true);
    assert_eq!(ready["result"]["decision"]["can_continue_group"], true);
    assert_eq!(ready["result"]["decision"]["prevents_commit_replay"], true);

    let bad_epoch = command_value("run_mls_commit_admission_bad_epoch");
    assert_eq!(bad_epoch["result"]["decision"]["accepted"], false);
    assert_eq!(bad_epoch["result"]["decision"]["reason_label"], "BAD_EPOCH");
    assert_eq!(bad_epoch["result"]["decision"]["requires_sync"], true);

    let auth = command_value("run_mls_commit_admission_auth_rejected");
    assert_eq!(auth["result"]["decision"]["accepted"], false);
    assert_eq!(
        auth["result"]["decision"]["reason_label"],
        "COMMIT_AUTHENTICATION_INVALID"
    );
    assert_eq!(auth["result"]["decision"]["requires_mls_setup"], true);

    let path = command_value("run_mls_commit_admission_path_rejected");
    assert_eq!(path["result"]["decision"]["accepted"], false);
    assert_eq!(
        path["result"]["decision"]["reason_label"],
        "UPDATE_PATH_SECRET_DECRYPT_FAILED"
    );
    assert_eq!(path["result"]["decision"]["requires_tree_repair"], true);

    let tie_break = command_value("run_mls_commit_admission_tie_break_rejected");
    assert_eq!(tie_break["result"]["decision"]["accepted"], false);
    assert_eq!(
        tie_break["result"]["decision"]["reason_label"],
        "COMMIT_TIE_BREAK_REJECTED"
    );

    let replay = command_value("run_mls_commit_admission_replay_rejected");
    assert_eq!(replay["result"]["decision"]["accepted"], false);
    assert_eq!(
        replay["result"]["decision"]["reason_label"],
        "COMMIT_ALREADY_PROCESSED"
    );

    let plaintext = command_value("run_mls_commit_admission_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_COMMIT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_mls_commit_replay_store_commands() {
    let ready = command_value("run_mls_commit_replay_store_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_commit_replay_store_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_commit_replay_store");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(ready["result"]["decision"]["can_apply_commit_once"], true);
    assert_eq!(ready["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let admission = command_value("run_mls_commit_replay_store_admission_rejected");
    assert_eq!(admission["result"]["decision"]["accepted"], false);
    assert_eq!(
        admission["result"]["decision"]["reason_label"],
        "COMMIT_ADMISSION_REJECTED"
    );

    let duplicate = command_value("run_mls_commit_replay_store_duplicate_rejected");
    assert_eq!(duplicate["result"]["decision"]["accepted"], false);
    assert_eq!(
        duplicate["result"]["decision"]["reason_label"],
        "COMMIT_ALREADY_RECORDED"
    );
    assert_eq!(duplicate["result"]["decision"]["record_count"], 1);

    let removed = command_value("run_mls_commit_replay_store_local_member_removed");
    assert_eq!(removed["result"]["decision"]["accepted"], true);
    assert_eq!(removed["result"]["decision"]["local_member_removed"], true);
    assert_eq!(removed["result"]["decision"]["can_continue_group"], false);

    let plaintext = command_value("run_mls_commit_replay_store_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

#[test]
fn backend_command_json_wraps_mls_welcome_replay_store_commands() {
    let ready = command_value("run_mls_welcome_replay_store_ready");
    assert_eq!(
        ready["command"]["command_kind_label"],
        "run_mls_welcome_replay_store_ready"
    );
    assert_eq!(ready["result"]["surface"], "mls_welcome_replay_store");
    assert_eq!(ready["result"]["decision"]["accepted"], true);
    assert_eq!(ready["result"]["decision"]["persisted_record"], true);
    assert_eq!(
        ready["result"]["decision"]["can_initialize_group_once"],
        true
    );
    assert_eq!(ready["result"]["decision"]["consumes_key_package"], true);
    assert_eq!(ready["result"]["decision"]["deletes_init_key"], true);
    assert_eq!(
        ready["result"]["decision"]["commits_group_state_transactionally"],
        true
    );
    assert_eq!(ready["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(ready["result"]["store"]["has_record"], true);

    let admission = command_value("run_mls_welcome_replay_store_admission_rejected");
    assert_eq!(admission["result"]["decision"]["accepted"], false);
    assert_eq!(
        admission["result"]["decision"]["reason_label"],
        "WELCOME_ADMISSION_REJECTED"
    );

    let duplicate = command_value("run_mls_welcome_replay_store_duplicate_rejected");
    assert_eq!(duplicate["result"]["decision"]["accepted"], false);
    assert_eq!(
        duplicate["result"]["decision"]["reason_label"],
        "WELCOME_ALREADY_RECORDED"
    );
    assert_eq!(duplicate["result"]["decision"]["record_count"], 1);

    let reused_key_package = command_value("run_mls_welcome_replay_store_key_package_reused");
    assert_eq!(reused_key_package["result"]["decision"]["accepted"], false);
    assert_eq!(
        reused_key_package["result"]["decision"]["reason_label"],
        "KEY_PACKAGE_ALREADY_CONSUMED"
    );
    assert_eq!(reused_key_package["result"]["decision"]["record_count"], 1);

    let bad_shape = command_value("run_mls_welcome_replay_store_bad_shape");
    assert_eq!(bad_shape["result"]["decision"]["accepted"], false);
    assert_eq!(
        bad_shape["result"]["decision"]["reason_label"],
        "BAD_CONSUMED_KEY_PACKAGE_REF"
    );

    let plaintext = command_value("run_mls_welcome_replay_store_plaintext_rejected");
    assert_eq!(plaintext["result"]["decision"]["accepted"], false);
    assert_eq!(
        plaintext["result"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(
        plaintext["result"]["decision"]["plaintext_bytes_exposed"],
        true
    );
}

fn command_value(name: &str) -> serde_json::Value {
    let descriptor = backend_command_by_name(name).expect("command should exist");
    let json = backend_command_json(descriptor).expect("command json should serialize");
    serde_json::from_str(&json).expect("json should parse")
}
