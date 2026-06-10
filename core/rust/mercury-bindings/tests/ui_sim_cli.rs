use std::process::Command;

#[test]
fn ui_sim_lists_fixture_names() {
    let output = run(["--list"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");

    assert!(output.status.success());
    assert!(stdout.contains("bootstrap_accepted"));
    assert!(stdout.contains("client_receive_ordering_gap"));
}

#[test]
fn ui_sim_lists_prototype_fixture_names() {
    let output = run(["--list-prototypes"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");

    assert!(output.status.success());
    assert!(stdout.contains("local_store_sealed_message"));
    assert!(stdout.contains("local_store_unlock_ready"));
    assert!(stdout.contains("local_store_production_open_ready"));
    assert!(stdout.contains("local_store_keychain_android_ready"));
    assert!(stdout.contains("production_store_session_happy_path"));
    assert!(stdout.contains("platform_local_store_adapter_desktop_ready"));
    assert!(stdout.contains("local_store_database_security_ready"));
    assert!(stdout.contains("local_store_database_adapter_selection_ready"));
    assert!(stdout.contains("receive_session_happy_path"));
    assert!(stdout.contains("inbound_sync_delivery_ready"));
    assert!(stdout.contains("authenticated_relay_source_delivery_ready"));
    assert!(stdout.contains("inbound_sync_session_happy_path"));
    assert!(stdout.contains("media_object_store_upload_ready"));
    assert!(stdout.contains("media_upload_session_happy_path"));
    assert!(stdout.contains("media_service_adapter_ready"));
    assert!(stdout.contains("media_service_upload_session_happy_path"));
    assert!(stdout.contains("media_service_download_ready"));
    assert!(stdout.contains("media_download_session_happy_path"));
    assert!(stdout.contains("media_retention_delete_and_evict_ready"));
    assert!(stdout.contains("media_cleanup_session_happy_path"));
    assert!(stdout.contains("media_object_index_remote_and_local_ready"));
    assert!(stdout.contains("media_object_index_store_write_ready"));
    assert!(stdout.contains("indexed_media_upload_session_happy_path"));
    assert!(stdout.contains("indexed_media_download_session_happy_path"));
    assert!(stdout.contains("indexed_media_cleanup_session_happy_path"));
    assert!(stdout.contains("anonymous_nullifier_store_ready"));
    assert!(stdout.contains("mls_provider_evidence_store_ready"));
    assert!(stdout.contains("mls_provider_evidence_use_ready"));
    assert!(stdout.contains("mls_provider_adapter_selection_ready"));
    assert!(stdout.contains("secure_backup_restore_ready"));
    assert!(stdout.contains("sealed_audit_event_chain_ready"));
    assert!(stdout.contains("sealed_audit_event_store_ready"));
    assert!(stdout.contains("sealed_audit_witness_checkpoint_ready"));
    assert!(stdout.contains("sealed_audit_witness_client_ready"));
    assert!(stdout.contains("sealed_audit_proof_bundle_ready"));
    assert!(stdout.contains("sealed_audit_proof_cache_ready"));
    assert!(stdout.contains("sealed_audit_verifier_policy_ready"));
    assert!(stdout.contains("sealed_audit_incident_evidence_ready"));
    assert!(stdout.contains("sealed_audit_recovery_export_ready"));
    assert!(stdout.contains("sealed_audit_database_adapter_ready"));
    assert!(stdout.contains("sealed_audit_private_report_transport_ready"));
    assert!(stdout.contains("sealed_audit_private_report_outbox_ready"));
    assert!(stdout.contains("sealed_audit_private_report_receipt_ready"));
    assert!(stdout.contains("sealed_audit_private_report_reconciliation_ready"));
    assert!(stdout.contains("sealed_audit_private_report_gateway_evidence_ready"));
    assert!(stdout.contains("mls_key_package_admission_ready"));
    assert!(stdout.contains("mls_key_package_consume_store_ready"));
    assert!(stdout.contains("mls_welcome_send_outbox_ready"));
    assert!(stdout.contains("mls_membership_transaction_ready"));
    assert!(stdout.contains("mls_welcome_admission_ready"));
    assert!(stdout.contains("mls_welcome_replay_store_ready"));
    assert!(stdout.contains("mls_commit_admission_ready"));
    assert!(stdout.contains("mls_commit_replay_store_ready"));
    assert!(stdout.contains("ai_participant_draft_accepted"));
}

#[test]
fn ui_sim_lists_backend_command_names() {
    let output = run(["--list-commands"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");

    assert!(output.status.success());
    assert!(stdout.contains("run_session_happy_path"));
    assert!(stdout.contains("run_session_ai_rejected"));
    assert!(stdout.contains("local_ai_draft_assist"));
    assert!(stdout.contains("run_production_store_session_wal_replay_required"));
    assert!(stdout.contains("run_platform_local_store_adapter_desktop_ready"));
    assert!(stdout.contains("run_local_store_database_security_ready"));
    assert!(stdout.contains("run_local_store_database_adapter_selection_ready"));
    assert!(stdout.contains("run_receive_session_happy_path"));
    assert!(stdout.contains("run_inbound_sync_delivery_ready"));
    assert!(stdout.contains("run_authenticated_relay_source_delivery_ready"));
    assert!(stdout.contains("run_inbound_sync_session_happy_path"));
    assert!(stdout.contains("run_media_object_store_upload_ready"));
    assert!(stdout.contains("run_media_upload_session_happy_path"));
    assert!(stdout.contains("run_media_service_adapter_ready"));
    assert!(stdout.contains("run_media_service_upload_session_happy_path"));
    assert!(stdout.contains("run_media_service_download_ready"));
    assert!(stdout.contains("run_media_download_session_happy_path"));
    assert!(stdout.contains("run_media_retention_delete_and_evict_ready"));
    assert!(stdout.contains("run_media_cleanup_session_happy_path"));
    assert!(stdout.contains("run_media_object_index_remote_and_local_ready"));
    assert!(stdout.contains("run_media_object_index_store_write_ready"));
    assert!(stdout.contains("run_indexed_media_upload_session_happy_path"));
    assert!(stdout.contains("run_indexed_media_download_session_happy_path"));
    assert!(stdout.contains("run_indexed_media_cleanup_session_happy_path"));
    assert!(stdout.contains("run_anonymous_credential_issuer_trust_ready"));
    assert!(stdout.contains("run_anonymous_credential_issuer_trust_witness_audit_rejected"));
    assert!(stdout.contains("run_anonymous_group_membership_proof_ready"));
    assert!(stdout.contains("run_anonymous_rate_limit_nullifier_ready"));
    assert!(stdout.contains("run_anonymous_nullifier_store_ready"));
    assert!(stdout.contains("run_group_chat_mls_ready"));
    assert!(stdout.contains("run_group_chat_mls_provider_security_required"));
    assert!(stdout.contains("run_mls_provider_evidence_store_ready"));
    assert!(stdout.contains("run_mls_provider_evidence_use_ready"));
    assert!(stdout.contains("run_mls_provider_adapter_selection_ready"));
    assert!(stdout.contains("run_secure_backup_restore_ready"));
    assert!(stdout.contains("run_sealed_audit_event_chain_ready"));
    assert!(stdout.contains("run_sealed_audit_event_store_ready"));
    assert!(stdout.contains("run_sealed_audit_witness_checkpoint_ready"));
    assert!(stdout.contains("run_sealed_audit_witness_client_ready"));
    assert!(stdout.contains("run_sealed_audit_proof_bundle_ready"));
    assert!(stdout.contains("run_sealed_audit_proof_cache_ready"));
    assert!(stdout.contains("run_sealed_audit_verifier_policy_ready"));
    assert!(stdout.contains("run_sealed_audit_incident_evidence_ready"));
    assert!(stdout.contains("run_sealed_audit_recovery_export_ready"));
    assert!(stdout.contains("run_sealed_audit_database_adapter_ready"));
    assert!(stdout.contains("run_sealed_audit_private_report_transport_ready"));
    assert!(stdout.contains("run_sealed_audit_private_report_outbox_ready"));
    assert!(stdout.contains("run_sealed_audit_private_report_receipt_ready"));
    assert!(stdout.contains("run_sealed_audit_private_report_reconciliation_ready"));
    assert!(stdout.contains("run_sealed_audit_private_report_gateway_evidence_ready"));
    assert!(stdout.contains("run_mls_key_package_admission_ready"));
    assert!(stdout.contains("run_mls_key_package_consume_store_ready"));
    assert!(stdout.contains("run_mls_welcome_send_outbox_ready"));
    assert!(stdout.contains("run_mls_membership_transaction_ready"));
    assert!(stdout.contains("run_mls_welcome_admission_ready"));
    assert!(stdout.contains("run_mls_welcome_replay_store_ready"));
    assert!(stdout.contains("run_mls_commit_admission_ready"));
    assert!(stdout.contains("run_mls_commit_replay_store_ready"));
    assert!(stdout.contains("run_group_relay_envelope_ready"));
}

#[test]
fn ui_sim_prints_single_scenario_json() {
    let output = run(["--scenario", "bootstrap_sync_incomplete"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(json["source"], "client_bootstrap");
    assert_eq!(json["reason_label"], "SYNC_INCOMPLETE");
    assert_eq!(json["requires_sync"], true);
    assert_eq!(json["can_open_message_ui"], false);
}

#[test]
fn ui_sim_prints_single_prototype_json() {
    let output = run(["--prototype", "relay_delivery_once"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(json["surface"], "prototype_relay_server");
    assert_eq!(json["submission"]["accepted"], true);
    assert_eq!(json["delivery"]["ciphertext_delivered_len"], 128);
    assert_eq!(json["post_delivery"]["ciphertext_retained"], false);
}

#[test]
fn ui_sim_prints_backend_command_json() {
    let output = run(["--command", "run_session_happy_path"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(json["command"]["accepted"], true);
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_session_happy_path"
    );
    assert_eq!(json["command"]["plaintext_payload_len"], 0);
    assert_eq!(json["result"]["surface"], "prototype_backend_session");
    assert_eq!(json["result"]["events"][0]["kind_label"], "session_started");
}

#[test]
fn ui_sim_prints_local_ai_command_json() {
    let output = run(["--command", "local_ai_draft_assist"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(json["command"]["accepted"], true);
    assert_eq!(json["command"]["actor_kind_label"], "local_ai");
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_local_ai_draft_assist"
    );
    assert_eq!(json["command"]["can_request_ai_draft"], true);
    assert_eq!(json["command"]["can_run_session"], false);
    assert_eq!(json["result"]["surface"], "prototype_ai_participant");
}

#[test]
fn ui_sim_prints_production_store_session_command_json() {
    let output = run(["--command", "run_production_store_session_happy_path"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_production_store_session_happy_path"
    );
    assert_eq!(json["command"]["can_run_session"], true);
    assert_eq!(
        json["result"]["surface"],
        "prototype_production_store_session"
    );
    assert_eq!(json["result"]["outcome"]["accepted"], true);
}

#[test]
fn ui_sim_prints_platform_adapter_command_json() {
    let output = run([
        "--command",
        "run_platform_local_store_adapter_mobile_hardware_required",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_platform_local_store_adapter_mobile_hardware_required"
    );
    assert_eq!(json["result"]["surface"], "platform_local_store_adapter");
    assert_eq!(json["result"]["decision"]["accepted"], false);
    assert_eq!(
        json["result"]["decision"]["reason_label"],
        "HARDWARE_BACKING_REQUIRED"
    );
}

#[test]
fn ui_sim_prints_local_store_database_security_command_json() {
    let output = run(["--command", "run_local_store_database_security_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_local_store_database_security_ready"
    );
    assert_eq!(json["result"]["surface"], "local_store_database_security");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(
        json["result"]["decision"]["can_host_mls_transactions"],
        true
    );
}

#[test]
fn ui_sim_prints_local_store_database_adapter_selection_command_json() {
    let output = run([
        "--command",
        "run_local_store_database_adapter_selection_ready",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_local_store_database_adapter_selection_ready"
    );
    assert_eq!(
        json["result"]["surface"],
        "local_store_database_adapter_selection"
    );
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["can_ship_release"], true);
}

#[test]
fn ui_sim_prints_mls_provider_adapter_selection_command_json() {
    let output = run(["--command", "run_mls_provider_adapter_selection_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_provider_adapter_selection_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_provider_adapter_selection");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["can_open_mls_group"], true);
    assert_eq!(
        json["result"]["decision"]["forbids_plaintext_key_export"],
        true
    );
}

#[test]
fn ui_sim_prints_secure_backup_restore_command_json() {
    let output = run(["--command", "run_secure_backup_restore_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_secure_backup_restore_ready"
    );
    assert_eq!(json["result"]["surface"], "secure_backup_restore");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["can_restore_mls_state"], true);
    assert_eq!(
        json["result"]["decision"]["forbids_os_plaintext_backup"],
        true
    );
}

#[test]
fn ui_sim_prints_sealed_audit_event_chain_command_json() {
    let output = run(["--command", "run_sealed_audit_event_chain_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_event_chain_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_event_chain");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["tamper_evident"], true);
    assert_eq!(json["result"]["decision"]["append_only"], true);
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
}

#[test]
fn ui_sim_prints_sealed_audit_event_store_command_json() {
    let output = run(["--command", "run_sealed_audit_event_store_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_event_store_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_event_store");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["can_detect_replay"], true);
    assert_eq!(
        json["result"]["decision"]["keeps_plaintext_metadata"],
        false
    );
    assert_eq!(json["result"]["store"]["has_sequence"], true);
}

#[test]
fn ui_sim_prints_sealed_audit_witness_checkpoint_command_json() {
    let output = run(["--command", "run_sealed_audit_witness_checkpoint_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_witness_checkpoint_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_witness_checkpoint");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(
        json["result"]["decision"]["signature_algorithm_label"],
        "hybrid_ed25519_ml_dsa_44"
    );
    assert_eq!(json["result"]["decision"]["can_detect_split_view"], true);
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
}

#[test]
fn ui_sim_prints_sealed_audit_witness_client_command_json() {
    let output = run(["--command", "run_sealed_audit_witness_client_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_witness_client_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_witness_client");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(
        json["result"]["decision"]["can_publish_witnessed_checkpoint"],
        true
    );
    assert_eq!(json["result"]["decision"]["can_monitor_privately"], true);
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
}

#[test]
fn ui_sim_prints_sealed_audit_proof_bundle_command_json() {
    let output = run(["--command", "run_sealed_audit_proof_bundle_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_proof_bundle_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_proof_bundle");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["can_verify_offline"], true);
    assert_eq!(json["result"]["decision"]["can_persist_proof_bundle"], true);
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
}

#[test]
fn ui_sim_prints_sealed_audit_proof_cache_command_json() {
    let output = run(["--command", "run_sealed_audit_proof_cache_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_proof_cache_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_proof_cache");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["can_verify_offline"], true);
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
    assert_eq!(json["result"]["cache"]["has_record"], true);
}

#[test]
fn ui_sim_prints_sealed_audit_verifier_policy_command_json() {
    let output = run(["--command", "run_sealed_audit_verifier_policy_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_verifier_policy_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_verifier_policy");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_snapshot"], true);
    assert_eq!(
        json["result"]["decision"]["can_schedule_private_monitor"],
        true
    );
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
    assert_eq!(json["result"]["store"]["has_snapshot"], true);
}

#[test]
fn ui_sim_prints_sealed_audit_incident_evidence_command_json() {
    let output = run(["--command", "run_sealed_audit_incident_evidence_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_incident_evidence_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_incident_evidence");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["can_report_privately"], true);
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_sealed_audit_recovery_export_command_json() {
    let output = run(["--command", "run_sealed_audit_recovery_export_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_recovery_export_ready"
    );
    assert_eq!(json["result"]["surface"], "sealed_audit_recovery_export");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["can_restore_state"], true);
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_sealed_audit_database_and_report_transport_command_json() {
    let database_output = run(["--command", "run_sealed_audit_database_adapter_ready"]);
    let database_stdout =
        String::from_utf8(database_output.stdout).expect("sim output should be utf8");
    let database_json: serde_json::Value =
        serde_json::from_str(&database_stdout).expect("sim should print json");

    assert!(database_output.status.success());
    assert_eq!(
        database_json["command"]["command_kind_label"],
        "run_sealed_audit_database_adapter_ready"
    );
    assert_eq!(
        database_json["result"]["surface"],
        "sealed_audit_database_adapter"
    );
    assert_eq!(database_json["result"]["decision"]["accepted"], true);
    assert_eq!(
        database_json["result"]["decision"]["can_persist_sealed_audit"],
        true
    );

    let transport_output = run([
        "--command",
        "run_sealed_audit_private_report_transport_ready",
    ]);
    let transport_stdout =
        String::from_utf8(transport_output.stdout).expect("sim output should be utf8");
    let transport_json: serde_json::Value =
        serde_json::from_str(&transport_stdout).expect("sim should print json");

    assert!(transport_output.status.success());
    assert_eq!(
        transport_json["command"]["command_kind_label"],
        "run_sealed_audit_private_report_transport_ready"
    );
    assert_eq!(
        transport_json["result"]["surface"],
        "sealed_audit_private_report_transport"
    );
    assert_eq!(transport_json["result"]["decision"]["accepted"], true);
    assert_eq!(
        transport_json["result"]["decision"]["can_submit_private_report"],
        true
    );
}

#[test]
fn ui_sim_prints_sealed_audit_private_report_outbox_command_json() {
    let output = run(["--command", "run_sealed_audit_private_report_outbox_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_private_report_outbox_ready"
    );
    assert_eq!(
        json["result"]["surface"],
        "sealed_audit_private_report_outbox"
    );
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["can_submit_now"], true);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_sealed_audit_private_report_receipt_command_json() {
    let output = run(["--command", "run_sealed_audit_private_report_receipt_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_private_report_receipt_ready"
    );
    assert_eq!(
        json["result"]["surface"],
        "sealed_audit_private_report_receipt"
    );
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["can_mark_delivered"], true);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_sealed_audit_private_report_reconciliation_command_json() {
    let output = run([
        "--command",
        "run_sealed_audit_private_report_reconciliation_ready",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_private_report_reconciliation_ready"
    );
    assert_eq!(
        json["result"]["surface"],
        "sealed_audit_private_report_reconciliation"
    );
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["can_reconcile_delivery"], true);
    assert_eq!(json["result"]["decision"]["can_schedule_retry"], false);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_sealed_audit_private_report_gateway_evidence_command_json() {
    let output = run([
        "--command",
        "run_sealed_audit_private_report_gateway_evidence_ready",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_sealed_audit_private_report_gateway_evidence_ready"
    );
    assert_eq!(
        json["result"]["surface"],
        "sealed_audit_private_report_gateway_evidence"
    );
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(
        json["result"]["decision"]["can_raise_gateway_incident"],
        true
    );
    assert_eq!(json["result"]["decision"]["can_notify_operator"], true);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_receive_session_command_json() {
    let output = run(["--command", "run_receive_session_ack_rejected"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_receive_session_ack_rejected"
    );
    assert_eq!(json["result"]["surface"], "prototype_receive_session");
    assert_eq!(json["result"]["outcome"]["completed"], false);
    assert_eq!(
        json["result"]["outcome"]["reason_label"],
        "delivery_ack_rejected"
    );
    assert_eq!(json["result"]["events"][0]["kind_label"], "receive_started");
    assert_eq!(
        json["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "delivery_ack_evaluated"
    );
    assert_eq!(
        json["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn ui_sim_prints_inbound_sync_command_json() {
    let output = run(["--command", "run_inbound_sync_transport_offline"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_inbound_sync_transport_offline"
    );
    assert_eq!(json["result"]["surface"], "inbound_sync_gate");
    assert_eq!(json["result"]["decision"]["accepted"], false);
    assert_eq!(
        json["result"]["decision"]["reason_label"],
        "TRANSPORT_OFFLINE"
    );
    assert_eq!(json["result"]["decision"]["requires_network_retry"], true);
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
}

#[test]
fn ui_sim_prints_authenticated_relay_source_command_json() {
    let output = run([
        "--command",
        "run_authenticated_relay_source_plaintext_forbidden",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_authenticated_relay_source_plaintext_forbidden"
    );
    assert_eq!(json["result"]["surface"], "authenticated_relay_source");
    assert_eq!(json["result"]["decision"]["accepted"], false);
    assert_eq!(
        json["result"]["decision"]["reason_label"],
        "PLAINTEXT_IDENTITY_FORBIDDEN"
    );
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
    assert_eq!(
        json["result"]["inbound_sync"]["reason_label"],
        "TRANSPORT_AUTH_REJECTED"
    );
}

#[test]
fn ui_sim_prints_media_object_store_command_json() {
    let output = run(["--command", "run_media_object_store_auto_download_rejected"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_object_store_auto_download_rejected"
    );
    assert_eq!(json["result"]["surface"], "media_object_store");
    assert_eq!(json["result"]["decision"]["accepted"], false);
    assert_eq!(
        json["result"]["decision"]["reason_label"],
        "AUTOMATIC_DOWNLOAD_FORBIDDEN"
    );
    assert_eq!(json["result"]["decision"]["plaintext_bytes_exposed"], false);
}

#[test]
fn ui_sim_prints_media_upload_session_command_json() {
    let output = run(["--command", "run_media_upload_session_seal_rejected"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_upload_session_seal_rejected"
    );
    assert_eq!(json["result"]["surface"], "prototype_media_upload_session");
    assert_eq!(json["result"]["outcome"]["completed"], false);
    assert_eq!(
        json["result"]["outcome"]["reason_label"],
        "local_store_seal_rejected"
    );
    assert_eq!(json["result"]["outcome"]["crypto_seal_calls"], 0);
    assert_eq!(
        json["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "local_store_seal_evaluated"
    );
}

#[test]
fn ui_sim_prints_media_service_adapter_command_json() {
    let output = run(["--command", "run_media_service_adapter_digest_unverified"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_service_adapter_digest_unverified"
    );
    assert_eq!(json["result"]["surface"], "media_service_adapter");
    assert_eq!(json["result"]["decision"]["accepted"], false);
    assert_eq!(
        json["result"]["decision"]["reason_label"],
        "CONTENT_DIGEST_UNVERIFIED"
    );
    assert_eq!(json["result"]["decision"]["forbids_plaintext_upload"], true);
}

#[test]
fn ui_sim_prints_media_service_upload_session_command_json() {
    let output = run([
        "--command",
        "run_media_service_upload_session_media_rejected",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_service_upload_session_media_rejected"
    );
    assert_eq!(
        json["result"]["surface"],
        "prototype_media_service_upload_session"
    );
    assert_eq!(json["result"]["outcome"]["completed"], false);
    assert_eq!(
        json["result"]["outcome"]["reason_label"],
        "media_upload_rejected"
    );
    assert_eq!(json["result"]["outcome"]["service_upload_calls"], 0);
    assert_eq!(
        json["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "media_upload_session_evaluated"
    );
}

#[test]
fn ui_sim_prints_media_service_download_command_json() {
    let output = run(["--command", "run_media_service_download_digest_unverified"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_service_download_digest_unverified"
    );
    assert_eq!(json["result"]["surface"], "media_service_download");
    assert_eq!(json["result"]["decision"]["accepted"], false);
    assert_eq!(
        json["result"]["decision"]["reason_label"],
        "CONTENT_DIGEST_UNVERIFIED"
    );
    assert_eq!(
        json["result"]["decision"]["forbids_plaintext_preview"],
        true
    );
}

#[test]
fn ui_sim_prints_media_download_session_command_json() {
    let output = run(["--command", "run_media_download_session_download_rejected"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_download_session_download_rejected"
    );
    assert_eq!(
        json["result"]["surface"],
        "prototype_media_download_session"
    );
    assert_eq!(json["result"]["outcome"]["completed"], false);
    assert_eq!(
        json["result"]["outcome"]["reason_label"],
        "media_service_download_rejected"
    );
    assert_eq!(json["result"]["outcome"]["service_download_calls"], 0);
    assert_eq!(
        json["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "media_service_download_evaluated"
    );
}

#[test]
fn ui_sim_prints_media_retention_command_json() {
    let output = run(["--command", "run_media_retention_auth_missing"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_retention_auth_missing"
    );
    assert_eq!(json["result"]["surface"], "media_retention");
    assert_eq!(json["result"]["decision"]["accepted"], false);
    assert_eq!(
        json["result"]["decision"]["reason_label"],
        "SERVICE_AUTHENTICATION_MISSING"
    );
    assert_eq!(
        json["result"]["decision"]["forbids_plaintext_deletion"],
        true
    );
}

#[test]
fn ui_sim_prints_media_cleanup_session_command_json() {
    let output = run(["--command", "run_media_cleanup_session_cache_absent"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_cleanup_session_cache_absent"
    );
    assert_eq!(json["result"]["surface"], "prototype_media_cleanup_session");
    assert_eq!(json["result"]["outcome"]["completed"], true);
    assert_eq!(
        json["result"]["outcome"]["local_cache_delete_attempted"],
        true
    );
    assert_eq!(json["result"]["outcome"]["local_cache_deleted"], false);
}

#[test]
fn ui_sim_prints_media_object_index_command_json() {
    let output = run(["--command", "run_media_object_index_deleted_terminal"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_object_index_deleted_terminal"
    );
    assert_eq!(json["result"]["surface"], "media_object_index");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(
        json["result"]["decision"]["lifecycle_state_label"],
        "deleted"
    );
    assert_eq!(json["result"]["decision"]["can_upload"], false);
    assert_eq!(json["result"]["decision"]["can_download"], false);
    assert_eq!(json["result"]["decision"]["can_cleanup"], false);
}

#[test]
fn ui_sim_prints_media_object_index_store_command_json() {
    let output = run(["--command", "run_media_object_index_store_deleted_snapshot"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_media_object_index_store_deleted_snapshot"
    );
    assert_eq!(json["result"]["surface"], "media_object_index_store");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["record"]["lifecycle_state_label"], "deleted");
    assert_eq!(json["result"]["record"]["plaintext_bytes_exposed"], false);
}

#[test]
fn ui_sim_prints_indexed_media_upload_session_command_json() {
    let output = run(["--command", "run_indexed_media_upload_session_happy_path"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_indexed_media_upload_session_happy_path"
    );
    assert_eq!(
        json["result"]["surface"],
        "prototype_indexed_media_upload_session"
    );
    assert_eq!(json["result"]["outcome"]["completed"], true);
    assert_eq!(json["result"]["outcome"]["reason_label"], "completed");
    assert_eq!(json["result"]["outcome"]["index_store_records"], 1);
    assert_eq!(json["result"]["outcome"]["plaintext_exposed"], false);
}

#[test]
fn ui_sim_prints_indexed_media_download_session_command_json() {
    let output = run(["--command", "run_indexed_media_download_session_happy_path"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_indexed_media_download_session_happy_path"
    );
    assert_eq!(
        json["result"]["surface"],
        "prototype_indexed_media_download_session"
    );
    assert_eq!(json["result"]["outcome"]["completed"], true);
    assert_eq!(json["result"]["outcome"]["index_store_records"], 1);
    assert_eq!(
        json["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "indexed_download_finished"
    );
}

#[test]
fn ui_sim_prints_indexed_media_cleanup_session_command_json() {
    let output = run(["--command", "run_indexed_media_cleanup_session_happy_path"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_indexed_media_cleanup_session_happy_path"
    );
    assert_eq!(
        json["result"]["surface"],
        "prototype_indexed_media_cleanup_session"
    );
    assert_eq!(json["result"]["outcome"]["completed"], true);
    assert_eq!(json["result"]["outcome"]["index_store_records"], 1);
    assert_eq!(
        json["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "indexed_cleanup_finished"
    );
}

#[test]
fn ui_sim_prints_group_relay_envelope_command_json() {
    let output = run(["--command", "run_group_relay_envelope_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_group_relay_envelope_ready"
    );
    assert_eq!(json["result"]["surface"], "group_relay_envelope");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["can_enqueue_relay"], true);
    assert_eq!(
        json["result"]["input"]["anonymous_rate_limit_accepted"],
        true
    );
}

#[test]
fn ui_sim_prints_anonymous_nullifier_store_command_json() {
    let output = run(["--command", "run_anonymous_nullifier_store_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_anonymous_nullifier_store_ready"
    );
    assert_eq!(json["result"]["surface"], "anonymous_nullifier_store");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_mls_provider_evidence_store_command_json() {
    let output = run(["--command", "run_mls_provider_evidence_store_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_provider_evidence_store_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_provider_evidence_store");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(
        json["result"]["decision"]["can_use_as_provider_evidence"],
        true
    );
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_mls_provider_evidence_use_command_json() {
    let output = run(["--command", "run_mls_provider_evidence_use_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_provider_evidence_use_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_provider_evidence_use");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(
        json["result"]["decision"]["can_use_provider_evidence"],
        true
    );
}

#[test]
fn ui_sim_prints_mls_key_package_admission_command_json() {
    let output = run(["--command", "run_mls_key_package_admission_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_key_package_admission_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_key_package_admission");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["can_add_member"], true);
    assert_eq!(json["result"]["decision"]["can_send_welcome"], true);
    assert_eq!(json["result"]["decision"]["prevents_key_reuse"], true);
}

#[test]
fn ui_sim_prints_mls_key_package_consume_store_command_json() {
    let output = run(["--command", "run_mls_key_package_consume_store_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_key_package_consume_store_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_key_package_consume_store");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(
        json["result"]["decision"]["binds_welcome_send_transaction"],
        true
    );
    assert_eq!(json["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_mls_welcome_send_outbox_command_json() {
    let output = run(["--command", "run_mls_welcome_send_outbox_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_welcome_send_outbox_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_welcome_send_outbox");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(
        json["result"]["decision"]["can_send_welcome_after_commit"],
        true
    );
    assert_eq!(json["result"]["decision"]["binds_commit"], true);
    assert_eq!(json["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(json["result"]["outbox"]["has_record"], true);
}

#[test]
fn ui_sim_prints_mls_membership_transaction_command_json() {
    let output = run(["--command", "run_mls_membership_transaction_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_membership_transaction_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_membership_transaction");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(
        json["result"]["decision"]["can_commit_membership_change_once"],
        true
    );
    assert_eq!(
        json["result"]["decision"]["uses_single_storage_transaction"],
        true
    );
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_mls_welcome_admission_command_json() {
    let output = run(["--command", "run_mls_welcome_admission_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_welcome_admission_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_welcome_admission");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["can_join_group"], true);
    assert_eq!(json["result"]["decision"]["can_open_group"], true);
    assert_eq!(
        json["result"]["decision"]["forbids_plaintext_group_metadata"],
        true
    );
}

#[test]
fn ui_sim_prints_mls_commit_admission_command_json() {
    let output = run(["--command", "run_mls_commit_admission_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_commit_admission_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_commit_admission");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["can_apply_commit"], true);
    assert_eq!(json["result"]["decision"]["can_continue_group"], true);
    assert_eq!(
        json["result"]["decision"]["forbids_plaintext_commit_metadata"],
        true
    );
}

#[test]
fn ui_sim_prints_mls_commit_replay_store_command_json() {
    let output = run(["--command", "run_mls_commit_replay_store_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_commit_replay_store_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_commit_replay_store");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_mls_welcome_replay_store_command_json() {
    let output = run(["--command", "run_mls_welcome_replay_store_ready"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_mls_welcome_replay_store_ready"
    );
    assert_eq!(json["result"]["surface"], "mls_welcome_replay_store");
    assert_eq!(json["result"]["decision"]["accepted"], true);
    assert_eq!(json["result"]["decision"]["persisted_record"], true);
    assert_eq!(json["result"]["decision"]["consumes_key_package"], true);
    assert_eq!(json["result"]["decision"]["deletes_init_key"], true);
    assert_eq!(json["result"]["decision"]["keeps_digest_only"], true);
    assert_eq!(json["result"]["store"]["has_record"], true);
}

#[test]
fn ui_sim_prints_inbound_sync_session_command_json() {
    let output = run(["--command", "run_inbound_sync_session_receive_rejected"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(
        json["command"]["command_kind_label"],
        "run_inbound_sync_session_receive_rejected"
    );
    assert_eq!(json["result"]["surface"], "prototype_inbound_sync_session");
    assert_eq!(json["result"]["outcome"]["completed"], false);
    assert_eq!(
        json["result"]["outcome"]["reason_label"],
        "receive_rejected"
    );
    assert_eq!(
        json["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "delivery_ack_evaluated"
    );
}

#[test]
fn ui_sim_prints_bridge_json() {
    let output = run(["--bridge", "backend_command", "run_session_happy_path"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(json["bridge"]["accepted"], true);
    assert_eq!(json["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        json["body"]["command"]["command_kind_label"],
        "run_session_happy_path"
    );
}

#[test]
fn ui_sim_prints_bridge_json_request() {
    let output = run([
        "--bridge-json",
        r#"{"request_id":"0123456789abcdef0123456789abcdef","operation":"platform_fixture","target":"bootstrap_accepted"}"#,
    ]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");

    assert!(output.status.success());
    assert_eq!(json["bridge"]["accepted"], true);
    assert_eq!(json["body"]["source"], "client_bootstrap");
}

#[test]
fn ui_sim_prints_sequence_json() {
    let output = run(["--sequence", "receive_gap_retry"]);
    let stdout = String::from_utf8(output.stdout).expect("sim output should be utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("sim should print json");
    let states = json.as_array().expect("sequence output should be an array");

    assert!(output.status.success());
    assert_eq!(states.len(), 3);
    assert_eq!(states[1]["scenario"], "client_receive_ordering_gap");
    assert_eq!(states[1]["view"]["requires_client_retry"], true);
    assert_eq!(states[2]["view"]["accepted"], true);
}

#[test]
fn ui_sim_rejects_unknown_scenario() {
    let output = run(["--scenario", "unknown"]);
    let stderr = String::from_utf8(output.stderr).expect("sim stderr should be utf8");

    assert!(!output.status.success());
    assert!(stderr.contains("unknown scenario"));
}

#[test]
fn ui_sim_rejects_unknown_prototype() {
    let output = run(["--prototype", "unknown"]);
    let stderr = String::from_utf8(output.stderr).expect("sim stderr should be utf8");

    assert!(!output.status.success());
    assert!(stderr.contains("unknown prototype"));
}

#[test]
fn ui_sim_rejects_unknown_command() {
    let output = run(["--command", "unknown"]);
    let stderr = String::from_utf8(output.stderr).expect("sim stderr should be utf8");

    assert!(!output.status.success());
    assert!(stderr.contains("unknown command"));
}

fn run<const N: usize>(args: [&str; N]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mercury-ui-sim"))
        .args(args)
        .output()
        .expect("failed to run mercury-ui-sim")
}
