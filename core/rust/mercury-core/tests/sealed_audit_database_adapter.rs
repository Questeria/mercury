use mercury_core::{
    LocalStoreDatabaseAdapterSelectionDecision, LocalStoreDatabaseAdapterSelectionReason,
    LocalStoreDatabaseSecurityReason, SealedAuditDatabaseAdapterDecision,
    SealedAuditDatabaseAdapterInput, SealedAuditDatabaseAdapterReason,
    SealedAuditPrivateReportTransportDecision, SealedAuditPrivateReportTransportInput,
    SealedAuditPrivateReportTransportReason, SealedAuditRecoveryExportDecision,
    SealedAuditRecoveryExportReason,
};

const DIGEST: [u8; 32] = [0xA1; 32];
const OTHER_DIGEST: [u8; 32] = [0xA2; 32];

#[test]
fn database_adapter_accepts_encrypted_append_only_profile() {
    let decision = valid_database_adapter_input().evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason, SealedAuditDatabaseAdapterReason::Accepted);
    assert!(decision.can_open_database);
    assert!(decision.can_persist_sealed_audit);
    assert!(decision.can_run_migration);
    assert!(!decision.requires_database_encryption);
    assert!(!decision.requires_append_only_guard);
    assert!(!decision.requires_migration_drill);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.latest_export_sequence, 5);
    assert_eq!(decision.policy_epoch, 7);
}

#[test]
fn database_adapter_rejects_recovery_export_database_append_only_migration_plaintext_and_shape() {
    let recovery_rejected = SealedAuditDatabaseAdapterInput {
        recovery_export_decision: SealedAuditRecoveryExportDecision {
            accepted: false,
            reason: SealedAuditRecoveryExportReason::RollbackExportRejected,
            ..accepted_recovery_export_decision()
        },
        ..valid_database_adapter_input()
    };
    assert_database_rejected(
        recovery_rejected.evaluate(),
        SealedAuditDatabaseAdapterReason::RecoveryExportRejected,
    );

    let database_rejected = SealedAuditDatabaseAdapterInput {
        all_tables_encrypted: false,
        ..valid_database_adapter_input()
    };
    let database_decision = database_rejected.evaluate();
    assert_database_rejected(
        database_decision,
        SealedAuditDatabaseAdapterReason::DatabaseEncryptionRequired,
    );
    assert!(database_decision.requires_database_encryption);

    let append_only_rejected = SealedAuditDatabaseAdapterInput {
        append_only_incident_table: false,
        ..valid_database_adapter_input()
    };
    let append_only_decision = append_only_rejected.evaluate();
    assert_database_rejected(
        append_only_decision,
        SealedAuditDatabaseAdapterReason::AppendOnlyGuardRequired,
    );
    assert!(append_only_decision.requires_append_only_guard);

    let migration_rejected = SealedAuditDatabaseAdapterInput {
        crash_recovery_drill_passed: false,
        ..valid_database_adapter_input()
    };
    let migration_decision = migration_rejected.evaluate();
    assert_database_rejected(
        migration_decision,
        SealedAuditDatabaseAdapterReason::MigrationDrillRequired,
    );
    assert!(migration_decision.requires_migration_drill);

    let plaintext_rejected = SealedAuditDatabaseAdapterInput {
        plaintext_metadata_fields: 1,
        ..valid_database_adapter_input()
    };
    let plaintext_decision = plaintext_rejected.evaluate();
    assert_database_rejected(
        plaintext_decision,
        SealedAuditDatabaseAdapterReason::PlaintextMetadataForbidden,
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);

    let bad_shape = SealedAuditDatabaseAdapterInput {
        schema_digest: &OTHER_DIGEST[..16],
        ..valid_database_adapter_input()
    };
    assert_database_rejected(
        bad_shape.evaluate(),
        SealedAuditDatabaseAdapterReason::BadRecordShape,
    );
}

#[test]
fn private_report_transport_accepts_ohttp_privacy_pass_route() {
    let decision = valid_private_report_transport_input().evaluate();

    assert!(decision.accepted);
    assert_eq!(
        decision.reason,
        SealedAuditPrivateReportTransportReason::Accepted
    );
    assert!(decision.can_submit_private_report);
    assert!(decision.can_retry_safely);
    assert!(!decision.requires_private_transport);
    assert!(!decision.requires_replay_guard);
    assert!(!decision.requires_rate_limit_token);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn private_report_transport_rejects_database_transport_replay_plaintext_and_shape() {
    let database_rejected = SealedAuditPrivateReportTransportInput {
        database_adapter_decision: SealedAuditDatabaseAdapterDecision {
            accepted: false,
            reason: SealedAuditDatabaseAdapterReason::DatabaseEncryptionRequired,
            ..accepted_database_adapter_decision()
        },
        ..valid_private_report_transport_input()
    };
    assert_transport_rejected(
        database_rejected.evaluate(),
        SealedAuditPrivateReportTransportReason::DatabaseAdapterRejected,
    );

    let transport_rejected = SealedAuditPrivateReportTransportInput {
        ohttp_target_state_free: false,
        ..valid_private_report_transport_input()
    };
    let transport_decision = transport_rejected.evaluate();
    assert_transport_rejected(
        transport_decision,
        SealedAuditPrivateReportTransportReason::PrivateReportTransportRequired,
    );
    assert!(transport_decision.requires_private_transport);

    let rate_limit_rejected = SealedAuditPrivateReportTransportInput {
        privacy_pass_tokens_required: false,
        ..valid_private_report_transport_input()
    };
    let rate_limit_decision = rate_limit_rejected.evaluate();
    assert_transport_rejected(
        rate_limit_decision,
        SealedAuditPrivateReportTransportReason::PrivateReportTransportRequired,
    );
    assert!(rate_limit_decision.requires_rate_limit_token);

    let replay_rejected = SealedAuditPrivateReportTransportInput {
        replay_guard_enabled: false,
        ..valid_private_report_transport_input()
    };
    let replay_decision = replay_rejected.evaluate();
    assert_transport_rejected(
        replay_decision,
        SealedAuditPrivateReportTransportReason::ReplayGuardRequired,
    );
    assert!(replay_decision.requires_replay_guard);

    let plaintext_rejected = SealedAuditPrivateReportTransportInput {
        plaintext_selector_count: 1,
        ..valid_private_report_transport_input()
    };
    let plaintext_decision = plaintext_rejected.evaluate();
    assert_transport_rejected(
        plaintext_decision,
        SealedAuditPrivateReportTransportReason::PlaintextMetadataForbidden,
    );
    assert!(plaintext_decision.plaintext_bytes_exposed);

    let bad_shape = SealedAuditPrivateReportTransportInput {
        report_transport_config_digest: &OTHER_DIGEST[..16],
        ..valid_private_report_transport_input()
    };
    assert_transport_rejected(
        bad_shape.evaluate(),
        SealedAuditPrivateReportTransportReason::BadRecordShape,
    );
}

#[test]
fn database_and_transport_reasons_have_stable_codes_and_labels() {
    let database_reasons = [
        (SealedAuditDatabaseAdapterReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditDatabaseAdapterReason::RecoveryExportRejected,
            1,
            "RECOVERY_EXPORT_REJECTED",
        ),
        (
            SealedAuditDatabaseAdapterReason::DatabaseEncryptionRequired,
            2,
            "DATABASE_ENCRYPTION_REQUIRED",
        ),
        (
            SealedAuditDatabaseAdapterReason::AppendOnlyGuardRequired,
            3,
            "APPEND_ONLY_GUARD_REQUIRED",
        ),
        (
            SealedAuditDatabaseAdapterReason::MigrationDrillRequired,
            4,
            "MIGRATION_DRILL_REQUIRED",
        ),
        (
            SealedAuditDatabaseAdapterReason::PlaintextMetadataForbidden,
            5,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditDatabaseAdapterReason::BadRecordShape,
            6,
            "BAD_RECORD_SHAPE",
        ),
    ];
    for (reason, code, label) in database_reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let transport_reasons = [
        (
            SealedAuditPrivateReportTransportReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            SealedAuditPrivateReportTransportReason::DatabaseAdapterRejected,
            1,
            "DATABASE_ADAPTER_REJECTED",
        ),
        (
            SealedAuditPrivateReportTransportReason::PrivateReportTransportRequired,
            2,
            "PRIVATE_REPORT_TRANSPORT_REQUIRED",
        ),
        (
            SealedAuditPrivateReportTransportReason::ReplayGuardRequired,
            3,
            "REPLAY_GUARD_REQUIRED",
        ),
        (
            SealedAuditPrivateReportTransportReason::PlaintextMetadataForbidden,
            4,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditPrivateReportTransportReason::BadRecordShape,
            5,
            "BAD_RECORD_SHAPE",
        ),
    ];
    for (reason, code, label) in transport_reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_database_adapter_input() -> SealedAuditDatabaseAdapterInput<'static> {
    SealedAuditDatabaseAdapterInput {
        recovery_export_decision: accepted_recovery_export_decision(),
        database_selection_decision: accepted_database_selection_decision(),
        adapter_format_version: 1,
        database_profile_digest: &DIGEST,
        schema_digest: &DIGEST,
        event_table_digest: &DIGEST,
        proof_cache_table_digest: &DIGEST,
        verifier_policy_table_digest: &DIGEST,
        incident_evidence_table_digest: &DIGEST,
        recovery_export_table_digest: &DIGEST,
        checkpoint_table_digest: &DIGEST,
        migration_plan_digest: &DIGEST,
        crash_recovery_plan_digest: &DIGEST,
        latest_export_sequence: 5,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        plaintext_header_bytes: 0,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        all_tables_encrypted: true,
        wal_encrypted: true,
        temp_store_memory_only: true,
        page_authentication_enabled: true,
        platform_key_wrapping_enabled: true,
        key_rotation_supported: true,
        cipher_integrity_check_passed: true,
        append_only_event_table: true,
        append_only_proof_cache_table: true,
        append_only_policy_table: true,
        append_only_incident_table: true,
        append_only_recovery_export_table: true,
        monotonic_sequence_constraints: true,
        duplicate_digest_constraints: true,
        transactional_batch_writes: true,
        wal_checkpoint_policy_verified: true,
        deterministic_migration_tested: true,
        crash_recovery_drill_passed: true,
        plaintext_free_schema: true,
        ui_status_digest_only: true,
    }
}

fn valid_private_report_transport_input() -> SealedAuditPrivateReportTransportInput<'static> {
    SealedAuditPrivateReportTransportInput {
        database_adapter_decision: accepted_database_adapter_decision(),
        report_format_version: 1,
        report_transport_config_digest: &DIGEST,
        ohttp_gateway_key_digest: &DIGEST,
        ohttp_relay_policy_digest: &DIGEST,
        privacy_pass_issuer_key_digest: &DIGEST,
        report_outbox_digest: &DIGEST,
        replay_window_digest: &DIGEST,
        rate_limit_bucket_digest: &DIGEST,
        retry_backoff_digest: &DIGEST,
        incident_report_schema_digest: &DIGEST,
        audit_checkpoint_digest: &DIGEST,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        report_window_s: 3600,
        max_reports_per_window: 3,
        ohttp_relay_configured: true,
        ohttp_gateway_key_pinned: true,
        ohttp_target_state_free: true,
        hpke_request_encryption: true,
        gateway_response_authenticated: true,
        privacy_pass_tokens_required: true,
        privacy_pass_issuer_key_pinned: true,
        anonymous_rate_limit_enforced: true,
        report_payload_encrypted: true,
        report_payload_digest_only: true,
        selector_blinding_enabled: true,
        report_outbox_encrypted: true,
        retry_backoff_enabled: true,
        replay_guard_enabled: true,
        duplicate_report_rejected: true,
        constant_size_padding_enabled: true,
        no_cookie_or_auth_state: true,
        private_monitor_route_used: true,
        ui_status_digest_only: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
    }
}

const fn accepted_recovery_export_decision() -> SealedAuditRecoveryExportDecision {
    SealedAuditRecoveryExportDecision {
        accepted: true,
        reason: SealedAuditRecoveryExportReason::Accepted,
        persisted_record: true,
        record_count: 1,
        export_sequence: 5,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_export_state: true,
        can_restore_state: true,
        can_sync_cross_device: true,
        requires_restore_quorum: false,
        requires_policy_refresh: false,
        rejects_rollback: false,
        requires_device_binding: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

const fn accepted_database_selection_decision() -> LocalStoreDatabaseAdapterSelectionDecision {
    LocalStoreDatabaseAdapterSelectionDecision {
        accepted: true,
        can_link_adapter: true,
        can_open_database: true,
        can_ship_release: true,
        can_host_mls_transactions: true,
        requires_license_review: false,
        requires_fips_attestation: false,
        requires_migration_drill: false,
        requires_supply_chain_review: false,
        requires_platform_packaging: false,
        forbids_plaintext_storage: true,
        adapter_kind_code: 1,
        adapter_kind_label: "sqlcipher_community",
        binding_kind_code: 1,
        binding_kind_label: "rusqlite_bundled_sqlcipher",
        target_platform_code: 1,
        target_platform_label: "windows",
        license_kind_code: 1,
        license_kind_label: "community_bsd",
        reason: LocalStoreDatabaseAdapterSelectionReason::Accepted,
        database_security_reason: LocalStoreDatabaseSecurityReason::Accepted,
    }
}

const fn accepted_database_adapter_decision() -> SealedAuditDatabaseAdapterDecision {
    SealedAuditDatabaseAdapterDecision {
        accepted: true,
        reason: SealedAuditDatabaseAdapterReason::Accepted,
        can_open_database: true,
        can_persist_sealed_audit: true,
        can_run_migration: true,
        requires_database_encryption: false,
        requires_append_only_guard: false,
        requires_migration_drill: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
        latest_export_sequence: 5,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
    }
}

fn assert_database_rejected(
    decision: SealedAuditDatabaseAdapterDecision,
    reason: SealedAuditDatabaseAdapterReason,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.can_open_database);
    assert!(!decision.can_persist_sealed_audit);
    assert!(decision.keeps_digest_only);
}

fn assert_transport_rejected(
    decision: SealedAuditPrivateReportTransportDecision,
    reason: SealedAuditPrivateReportTransportReason,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.can_submit_private_report);
    assert!(!decision.can_retry_safely);
    assert!(decision.keeps_digest_only);
}
