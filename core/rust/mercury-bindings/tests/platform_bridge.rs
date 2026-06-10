use mercury_bindings::{
    PlatformBridgeOperation, PlatformBridgeRequest, platform_bridge_handle_json,
    platform_bridge_response_value,
};

#[test]
fn platform_bridge_returns_platform_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PlatformFixture,
        "bootstrap_accepted",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "platform_fixture");
    assert_eq!(value["bridge"]["reason_label"], "accepted");
    assert_eq!(value["body"]["source"], "client_bootstrap");
    assert_eq!(value["body"]["can_open_message_ui"], true);
}

#[test]
fn platform_bridge_json_returns_backend_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "local_ai_draft_assist"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_code"], 3);
    assert_eq!(value["body"]["command"]["actor_kind_label"], "local_ai");
    assert_eq!(value["body"]["command"]["can_request_ai_draft"], true);
    assert_eq!(
        value["body"]["result"]["surface"],
        "prototype_ai_participant"
    );
}

#[test]
fn platform_bridge_json_returns_production_store_session_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_production_store_session_write_rejected"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_production_store_session_write_rejected"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "prototype_production_store_session"
    );
    assert_eq!(value["body"]["result"]["outcome"]["accepted"], false);
    assert_eq!(
        value["body"]["result"]["outcome"]["reason_label"],
        "STORE_WRITE_REJECTED"
    );
}

#[test]
fn platform_bridge_json_returns_platform_adapter_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_platform_local_store_adapter_app_lock_required"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_platform_local_store_adapter_app_lock_required"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "platform_local_store_adapter"
    );
    assert_eq!(value["body"]["result"]["decision"]["accepted"], false);
    assert_eq!(
        value["body"]["result"]["decision"]["reason_label"],
        "APP_LOCK_REQUIRED"
    );
}

#[test]
fn platform_bridge_json_returns_receive_session_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_receive_session_store_write_rejected"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_receive_session_store_write_rejected"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "prototype_receive_session"
    );
    assert_eq!(
        value["body"]["result"]["outcome"]["reason_label"],
        "local_store_write_rejected"
    );
}

#[test]
fn platform_bridge_json_returns_inbound_sync_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_inbound_sync_plaintext_preview_forbidden"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_inbound_sync_plaintext_preview_forbidden"
    );
    assert_eq!(value["body"]["result"]["surface"], "inbound_sync_gate");
    assert_eq!(
        value["body"]["result"]["decision"]["reason_label"],
        "PLAINTEXT_NOTIFICATION_PREVIEW_FORBIDDEN"
    );
    assert_eq!(
        value["body"]["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn platform_bridge_json_returns_authenticated_relay_source_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_authenticated_relay_source_delivery_ready"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_authenticated_relay_source_delivery_ready"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "authenticated_relay_source"
    );
    assert_eq!(
        value["body"]["result"]["decision"]["reason_label"],
        "DELIVERY_READY"
    );
}

#[test]
fn platform_bridge_json_returns_media_object_store_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_object_store_upload_ready"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_object_store_upload_ready"
    );
    assert_eq!(value["body"]["result"]["surface"], "media_object_store");
    assert_eq!(value["body"]["result"]["decision"]["accepted"], true);
    assert_eq!(
        value["body"]["result"]["decision"]["reason_label"],
        "ACCEPTED"
    );
}

#[test]
fn platform_bridge_json_returns_media_upload_session_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_upload_session_happy_path"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_upload_session_happy_path"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "prototype_media_upload_session"
    );
    assert_eq!(value["body"]["result"]["outcome"]["completed"], true);
    assert_eq!(
        value["body"]["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "upload_finished"
    );
}

#[test]
fn platform_bridge_json_returns_media_service_adapter_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_service_adapter_ready"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_service_adapter_ready"
    );
    assert_eq!(value["body"]["result"]["surface"], "media_service_adapter");
    assert_eq!(value["body"]["result"]["decision"]["accepted"], true);
    assert_eq!(
        value["body"]["result"]["decision"]["reason_label"],
        "ACCEPTED"
    );
}

#[test]
fn platform_bridge_json_returns_media_service_upload_session_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_service_upload_session_happy_path"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_service_upload_session_happy_path"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "prototype_media_service_upload_session"
    );
    assert_eq!(value["body"]["result"]["outcome"]["completed"], true);
    assert_eq!(
        value["body"]["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "service_upload_finished"
    );
}

#[test]
fn platform_bridge_json_returns_media_service_download_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_service_download_ready"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_service_download_ready"
    );
    assert_eq!(value["body"]["result"]["surface"], "media_service_download");
    assert_eq!(value["body"]["result"]["decision"]["accepted"], true);
    assert_eq!(
        value["body"]["result"]["decision"]["reason_label"],
        "ACCEPTED"
    );
}

#[test]
fn platform_bridge_json_returns_media_download_session_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_download_session_happy_path"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_download_session_happy_path"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "prototype_media_download_session"
    );
    assert_eq!(value["body"]["result"]["outcome"]["completed"], true);
    assert_eq!(
        value["body"]["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "download_finished"
    );
}

#[test]
fn platform_bridge_json_returns_media_retention_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_retention_delete_and_evict_ready"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_retention_delete_and_evict_ready"
    );
    assert_eq!(value["body"]["result"]["surface"], "media_retention");
    assert_eq!(value["body"]["result"]["decision"]["accepted"], true);
    assert_eq!(
        value["body"]["result"]["decision"]["can_delete_remote_object"],
        true
    );
    assert_eq!(
        value["body"]["result"]["decision"]["can_evict_local_cache"],
        true
    );
}

#[test]
fn platform_bridge_json_returns_media_cleanup_session_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_cleanup_session_happy_path"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_cleanup_session_happy_path"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "prototype_media_cleanup_session"
    );
    assert_eq!(value["body"]["result"]["outcome"]["completed"], true);
    assert_eq!(value["body"]["result"]["outcome"]["remote_delete_calls"], 1);
    assert_eq!(
        value["body"]["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "cleanup_finished"
    );
}

#[test]
fn platform_bridge_json_returns_media_object_index_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_object_index_absent_upload_ready"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_object_index_absent_upload_ready"
    );
    assert_eq!(value["body"]["result"]["surface"], "media_object_index");
    assert_eq!(value["body"]["result"]["decision"]["can_upload"], true);
    assert_eq!(value["body"]["result"]["decision"]["can_download"], false);
    assert_eq!(value["body"]["result"]["decision"]["can_cleanup"], false);
}

#[test]
fn platform_bridge_json_returns_media_object_index_store_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_media_object_index_store_write_ready"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_media_object_index_store_write_ready"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "media_object_index_store"
    );
    assert_eq!(value["body"]["result"]["decision"]["accepted"], true);
    assert_eq!(
        value["body"]["result"]["decision"]["persisted_record"],
        true
    );
    assert_eq!(value["body"]["result"]["record"]["object_id_len"], 32);
}

#[test]
fn platform_bridge_json_returns_inbound_sync_session_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_inbound_sync_session_happy_path"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_inbound_sync_session_happy_path"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "prototype_inbound_sync_session"
    );
    assert_eq!(value["body"]["result"]["outcome"]["completed"], true);
    assert_eq!(
        value["body"]["result"]["events"]
            .as_array()
            .expect("events should be an array")
            .last()
            .expect("terminal event")["kind_label"],
        "sync_finished"
    );
}

#[test]
fn platform_bridge_json_returns_anonymous_group_relay_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_group_relay_envelope_plaintext_metadata_rejected"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_group_relay_envelope_plaintext_metadata_rejected"
    );
    assert_eq!(value["body"]["result"]["surface"], "group_relay_envelope");
    assert_eq!(
        value["body"]["result"]["decision"]["reason_label"],
        "PLAINTEXT_SENDER_METADATA"
    );
    assert_eq!(
        value["body"]["result"]["decision"]["plaintext_bytes_exposed"],
        false
    );
}

#[test]
fn platform_bridge_json_returns_anonymous_nullifier_store_command_body() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "backend_command",
          "target": "run_anonymous_nullifier_store_replay_rejected"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "backend_command");
    assert_eq!(
        value["body"]["command"]["command_kind_label"],
        "run_anonymous_nullifier_store_replay_rejected"
    );
    assert_eq!(
        value["body"]["result"]["surface"],
        "anonymous_nullifier_store"
    );
    assert_eq!(
        value["body"]["result"]["decision"]["reason_label"],
        "NULLIFIER_ALREADY_RECORDED"
    );
}

#[test]
fn platform_bridge_returns_unlock_prototype_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "local_store_unlock_app_lock_required",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "local_store_unlock");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(value["body"]["decision"]["requires_user_auth"], true);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "APP_LOCK_REQUIRED"
    );
}

#[test]
fn platform_bridge_returns_production_open_prototype_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "local_store_production_open_wal_replay_required",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "local_store_production_open");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(value["body"]["decision"]["can_replay_wal"], true);
    assert_eq!(value["body"]["decision"]["requires_crash_recovery"], true);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "WAL_REPLAY_REQUIRED"
    );
}

#[test]
fn platform_bridge_returns_keychain_prototype_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "local_store_keychain_exportable_secret_forbidden",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "local_store_keychain_unlock");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "EXPORTABLE_SECRET_FORBIDDEN"
    );
    assert_eq!(
        value["body"]["decision"]["requires_destructive_repair"],
        true
    );
}

#[test]
fn platform_bridge_returns_production_store_session_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "production_store_session_wal_replay_required",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(
        value["body"]["surface"],
        "prototype_production_store_session"
    );
    assert_eq!(value["body"]["outcome"]["accepted"], false);
    assert_eq!(value["body"]["outcome"]["wal_replayed"], true);
    assert_eq!(
        value["body"]["outcome"]["reason_label"],
        "WAL_REPLAY_REQUIRED"
    );
}

#[test]
fn platform_bridge_returns_authenticated_relay_source_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "authenticated_relay_source_delivery_ready",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "authenticated_relay_source");
    assert_eq!(value["body"]["decision"]["accepted"], true);
    assert_eq!(value["body"]["decision"]["reason_label"], "DELIVERY_READY");
    assert_eq!(
        value["body"]["inbound_sync"]["reason_label"],
        "DELIVERY_READY"
    );
}

#[test]
fn platform_bridge_returns_media_object_store_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_object_store_oversize_rejected",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "media_object_store");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "CIPHERTEXT_TOO_LARGE"
    );
}

#[test]
fn platform_bridge_returns_media_upload_session_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_upload_session_store_write_rejected",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "prototype_media_upload_session");
    assert_eq!(value["body"]["outcome"]["completed"], false);
    assert_eq!(
        value["body"]["outcome"]["reason_label"],
        "local_store_write_rejected"
    );
}

#[test]
fn platform_bridge_returns_media_service_adapter_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_service_adapter_digest_unverified",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "media_service_adapter");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "CONTENT_DIGEST_UNVERIFIED"
    );
}

#[test]
fn platform_bridge_returns_media_service_upload_session_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_service_upload_session_digest_unverified",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(
        value["body"]["surface"],
        "prototype_media_service_upload_session"
    );
    assert_eq!(value["body"]["outcome"]["completed"], false);
    assert_eq!(
        value["body"]["outcome"]["media_service"]["reason_label"],
        "CONTENT_DIGEST_UNVERIFIED"
    );
}

#[test]
fn platform_bridge_returns_media_service_download_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_service_download_plaintext_preview_rejected",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "media_service_download");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "PLAINTEXT_PREVIEW_FORBIDDEN"
    );
}

#[test]
fn platform_bridge_returns_media_download_session_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_download_session_store_write_rejected",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "prototype_media_download_session");
    assert_eq!(value["body"]["outcome"]["completed"], false);
    assert_eq!(
        value["body"]["outcome"]["reason_label"],
        "local_store_write_rejected"
    );
}

#[test]
fn platform_bridge_returns_media_retention_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_retention_auth_missing",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "media_retention");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "SERVICE_AUTHENTICATION_MISSING"
    );
    assert_eq!(value["body"]["decision"]["requires_network_setup"], true);
}

#[test]
fn platform_bridge_returns_media_cleanup_session_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_cleanup_session_cache_absent",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "prototype_media_cleanup_session");
    assert_eq!(value["body"]["outcome"]["completed"], true);
    assert_eq!(
        value["body"]["outcome"]["local_cache_delete_attempted"],
        true
    );
    assert_eq!(value["body"]["outcome"]["local_cache_deleted"], false);
}

#[test]
fn platform_bridge_returns_media_object_index_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_object_index_plaintext_metadata_rejected",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "media_object_index");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "PLAINTEXT_METADATA_FORBIDDEN"
    );
    assert_eq!(value["body"]["decision"]["plaintext_bytes_exposed"], false);
}

#[test]
fn platform_bridge_returns_media_object_index_store_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "media_object_index_store_bad_object_rejected",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(value["body"]["surface"], "media_object_index_store");
    assert_eq!(value["body"]["decision"]["accepted"], false);
    assert_eq!(
        value["body"]["decision"]["reason_label"],
        "BAD_OBJECT_ID_LENGTH"
    );
    assert_eq!(value["body"]["record"], serde_json::Value::Null);
}

#[test]
fn platform_bridge_returns_indexed_media_upload_session_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "indexed_media_upload_session_service_rejected",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(
        value["body"]["surface"],
        "prototype_indexed_media_upload_session"
    );
    assert_eq!(value["body"]["outcome"]["completed"], false);
    assert_eq!(
        value["body"]["outcome"]["reason_label"],
        "media_service_upload_rejected"
    );
    assert_eq!(
        value["body"]["outcome"]["index_store"],
        serde_json::Value::Null
    );
}

#[test]
fn platform_bridge_returns_indexed_media_download_session_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "indexed_media_download_session_download_rejected",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(
        value["body"]["surface"],
        "prototype_indexed_media_download_session"
    );
    assert_eq!(value["body"]["outcome"]["completed"], false);
    assert_eq!(
        value["body"]["outcome"]["reason_label"],
        "media_download_rejected"
    );
    assert_eq!(
        value["body"]["outcome"]["download"]["reason_label"],
        "media_service_download_rejected"
    );
}

#[test]
fn platform_bridge_returns_indexed_media_cleanup_session_fixture_body() {
    let value = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::PrototypeFixture,
        "indexed_media_cleanup_session_not_cleanable",
        0,
    ));

    assert_eq!(value["bridge"]["accepted"], true);
    assert_eq!(value["bridge"]["operation_label"], "prototype_fixture");
    assert_eq!(
        value["body"]["surface"],
        "prototype_indexed_media_cleanup_session"
    );
    assert_eq!(value["body"]["outcome"]["completed"], false);
    assert_eq!(
        value["body"]["outcome"]["reason_label"],
        "media_object_not_cleanable"
    );
    assert_eq!(value["body"]["outcome"]["cleanup"], serde_json::Value::Null);
}

#[test]
fn platform_bridge_rejects_plaintext_payloads_before_target_lookup() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "prototype_fixture",
          "target": "does_not_matter",
          "payload": {
            "plaintext": "never bridge this"
          }
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], false);
    assert_eq!(
        value["bridge"]["reason_label"],
        "plaintext_payload_forbidden"
    );
    assert_eq!(value["bridge"]["plaintext_payload_len"], 1);
    assert_eq!(value["body"], serde_json::Value::Null);
}

#[test]
fn platform_bridge_rejects_bad_request_ids_and_unknown_targets() {
    let bad_id = platform_bridge_response_value(PlatformBridgeRequest::new(
        "short",
        PlatformBridgeOperation::BackendCommand,
        "run_session_happy_path",
        0,
    ));
    assert_eq!(bad_id["bridge"]["accepted"], false);
    assert_eq!(bad_id["bridge"]["reason_label"], "bad_request_id_length");

    let unknown = platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        PlatformBridgeOperation::BackendCommand,
        "missing",
        0,
    ));
    assert_eq!(unknown["bridge"]["accepted"], false);
    assert_eq!(unknown["bridge"]["reason_label"], "unknown_target");
    assert_eq!(unknown["body"], serde_json::Value::Null);
}

#[test]
fn platform_bridge_json_rejects_unknown_operations() {
    let json = platform_bridge_handle_json(
        r#"{
          "request_id": "0123456789abcdef0123456789abcdef",
          "operation": "raw_sql",
          "target": "anything"
        }"#,
    )
    .expect("bridge json should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["bridge"]["accepted"], false);
    assert_eq!(value["bridge"]["operation_code"], 0);
    assert_eq!(value["bridge"]["reason_label"], "unknown_operation");
}
