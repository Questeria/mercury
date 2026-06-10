use mercury_core::{
    LocalStoreCrashRecoveryState, LocalStoreDatabaseAdapterKind,
    LocalStoreDatabaseAdapterSelectionDecision, LocalStoreDatabaseAdapterSelectionInput,
    LocalStoreDatabaseAdapterSelectionReason, LocalStoreDatabaseBindingKind,
    LocalStoreDatabaseCipher, LocalStoreDatabaseEngine, LocalStoreDatabaseKdf,
    LocalStoreDatabaseLicenseKind, LocalStoreDatabaseSecurityInput,
    LocalStoreDatabaseSecurityReason, LocalStoreDatabaseTargetPlatform, LocalStoreKeyScope,
    LocalStoreProductionOpenInput, LocalStoreSealingSuite, LocalStoreUnlockDatabaseHeaderState,
    LocalStoreUnlockInput, LocalStoreUnlockSecretState, MERCURY_LOCAL_STORE_MIN_KDF_ITERATIONS,
    MERCURY_LOCAL_STORE_PAGE_SIZE, MERCURY_LOCAL_STORE_VERSION, PlatformLocalStoreAdapterInput,
    PlatformLocalStoreAdapterKind, PlatformLocalStoreRuntime,
};

#[test]
fn adapter_selection_accepts_verified_sqlcipher_build() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_link_adapter);
    assert!(decision.can_open_database);
    assert!(decision.can_ship_release);
    assert!(decision.can_host_mls_transactions);
    assert!(!decision.requires_license_review);
    assert!(!decision.requires_fips_attestation);
    assert!(!decision.requires_migration_drill);
    assert!(!decision.requires_supply_chain_review);
    assert!(!decision.requires_platform_packaging);
    assert!(decision.forbids_plaintext_storage);
    assert_eq!(
        decision.reason,
        LocalStoreDatabaseAdapterSelectionReason::Accepted
    );
    assert_eq!(decision.adapter_kind_label, "sqlcipher_community");
    assert_eq!(decision.binding_kind_label, "rusqlite_bundled_sqlcipher");
    assert_eq!(decision.target_platform_label, "windows");
    assert_eq!(decision.license_kind_label, "community_bsd");
    assert_eq!(
        decision.database_security_reason,
        LocalStoreDatabaseSecurityReason::Accepted
    );
}

#[test]
fn adapter_selection_rejects_database_profile_adapter_binding_and_platform_gaps() {
    let mut bad_profile = valid_input();
    let mut db_security = valid_database_security_input();
    db_security.engine = LocalStoreDatabaseEngine::PlainSqlite;
    bad_profile.database_security = db_security.evaluate();
    let bad_profile_decision = bad_profile.evaluate();
    assert_rejected(
        bad_profile_decision,
        LocalStoreDatabaseAdapterSelectionReason::DatabaseProfileRejected,
    );
    assert_eq!(
        bad_profile_decision.database_security_reason,
        LocalStoreDatabaseSecurityReason::PlaintextDatabaseForbidden
    );
    assert!(bad_profile_decision.requires_migration_drill);

    let mut plain_adapter = valid_input();
    plain_adapter.adapter_kind = LocalStoreDatabaseAdapterKind::PlainSqlite;
    assert_rejected(
        plain_adapter.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::AdapterKindRejected,
    );

    let mut unknown_binding = valid_input();
    unknown_binding.binding_kind = LocalStoreDatabaseBindingKind::Unknown;
    let unknown_binding_decision = unknown_binding.evaluate();
    assert_rejected(
        unknown_binding_decision,
        LocalStoreDatabaseAdapterSelectionReason::BindingKindRejected,
    );
    assert!(unknown_binding_decision.requires_supply_chain_review);

    let mut unsupported_platform = valid_input();
    unsupported_platform.target_platform = LocalStoreDatabaseTargetPlatform::Unknown;
    let unsupported_platform_decision = unsupported_platform.evaluate();
    assert_rejected(
        unsupported_platform_decision,
        LocalStoreDatabaseAdapterSelectionReason::PlatformUnsupported,
    );
    assert!(unsupported_platform_decision.requires_platform_packaging);
}

#[test]
fn adapter_selection_rejects_license_version_and_source_gaps() {
    let mut trial = valid_input();
    trial.license_kind = LocalStoreDatabaseLicenseKind::TrialEvaluation;
    let trial_decision = trial.evaluate();
    assert_rejected(
        trial_decision,
        LocalStoreDatabaseAdapterSelectionReason::LicenseRejected,
    );
    assert!(trial_decision.requires_license_review);

    let mut old_sqlcipher = valid_input();
    old_sqlcipher.sqlcipher_major_version = 3;
    let old_sqlcipher_decision = old_sqlcipher.evaluate();
    assert_rejected(
        old_sqlcipher_decision,
        LocalStoreDatabaseAdapterSelectionReason::SqlcipherVersionTooOld,
    );
    assert!(old_sqlcipher_decision.requires_migration_drill);

    let mut unverified_source = valid_input();
    unverified_source.sqlcipher_source_verified = false;
    let unverified_source_decision = unverified_source.evaluate();
    assert_rejected(
        unverified_source_decision,
        LocalStoreDatabaseAdapterSelectionReason::SourceAuthenticityMissing,
    );
    assert!(unverified_source_decision.requires_supply_chain_review);
}

#[test]
fn adapter_selection_requires_fips_attestation_when_requested() {
    let mut fips_missing = valid_input();
    fips_missing.adapter_kind = LocalStoreDatabaseAdapterKind::SqlCipherEnterpriseFips;
    fips_missing.license_kind = LocalStoreDatabaseLicenseKind::EnterpriseFips;
    fips_missing.fips_required = true;
    fips_missing.fips_module_validated = false;
    let fips_missing_decision = fips_missing.evaluate();
    assert_rejected(
        fips_missing_decision,
        LocalStoreDatabaseAdapterSelectionReason::FipsValidationMissing,
    );
    assert!(fips_missing_decision.requires_fips_attestation);

    let mut runtime_missing = valid_input();
    runtime_missing.adapter_kind = LocalStoreDatabaseAdapterKind::SqlCipherEnterpriseFips;
    runtime_missing.license_kind = LocalStoreDatabaseLicenseKind::EnterpriseFips;
    runtime_missing.fips_required = true;
    runtime_missing.fips_module_validated = true;
    runtime_missing.fips_mode_checked_at_runtime = false;
    let runtime_missing_decision = runtime_missing.evaluate();
    assert_rejected(
        runtime_missing_decision,
        LocalStoreDatabaseAdapterSelectionReason::FipsRuntimeCheckMissing,
    );
    assert!(runtime_missing_decision.requires_fips_attestation);
}

#[test]
fn adapter_selection_rejects_unsafe_sqlite_and_sqlcipher_runtime_settings() {
    let mut no_codec = valid_input();
    no_codec.compile_has_codec = false;
    assert_rejected(
        no_codec.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::SqlcipherCodecNotEnabled,
    );

    let mut file_temp = valid_input();
    file_temp.temp_store_memory_configured = false;
    let file_temp_decision = file_temp.evaluate();
    assert_rejected(
        file_temp_decision,
        LocalStoreDatabaseAdapterSelectionReason::UnsafeSqliteConfiguration,
    );
    assert!(file_temp_decision.requires_migration_drill);

    let mut extensions = valid_input();
    extensions.extension_loading_disabled = false;
    assert_rejected(
        extensions.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::ExtensionLoadingEnabled,
    );

    let mut trusted_schema = valid_input();
    trusted_schema.trusted_schema_disabled = false;
    assert_rejected(
        trusted_schema.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::TrustedSchemaEnabled,
    );

    let mut secure_delete = valid_input();
    secure_delete.secure_delete_configured = false;
    assert_rejected(
        secure_delete.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::SecureDeleteMissing,
    );

    let mut memory_security = valid_input();
    memory_security.cipher_memory_security_enabled = false;
    assert_rejected(
        memory_security.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::MemorySecurityMissing,
    );

    let mut no_integrity = valid_input();
    no_integrity.cipher_integrity_check_on_open = false;
    assert_rejected(
        no_integrity.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::IntegrityCheckMissing,
    );

    let mut compatibility = valid_input();
    compatibility.sqlcipher_compatibility_current_major = false;
    let compatibility_decision = compatibility.evaluate();
    assert_rejected(
        compatibility_decision,
        LocalStoreDatabaseAdapterSelectionReason::CompatibilityModeRejected,
    );
    assert!(compatibility_decision.requires_migration_drill);
}

#[test]
fn adapter_selection_rejects_migration_crash_supply_chain_and_debug_gaps() {
    let mut migration = valid_input();
    migration.deterministic_migration_tested = false;
    let migration_decision = migration.evaluate();
    assert_rejected(
        migration_decision,
        LocalStoreDatabaseAdapterSelectionReason::MigrationDrillMissing,
    );
    assert!(migration_decision.requires_migration_drill);

    let mut crash = valid_input();
    crash.crash_recovery_drill_passed = false;
    assert_rejected(
        crash.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::CrashRecoveryDrillMissing,
    );

    let mut unsigned = valid_input();
    unsigned.release_artifacts_signed = false;
    let unsigned_decision = unsigned.evaluate();
    assert_rejected(
        unsigned_decision,
        LocalStoreDatabaseAdapterSelectionReason::UnsignedReleaseArtifact,
    );
    assert!(unsigned_decision.requires_supply_chain_review);

    let mut no_sbom = valid_input();
    no_sbom.sbom_present = false;
    assert_rejected(
        no_sbom.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::SbomOrCveMonitoringMissing,
    );

    let mut debug = valid_input();
    debug.debug_sqlcipher_logging_enabled = true;
    assert_rejected(
        debug.evaluate(),
        LocalStoreDatabaseAdapterSelectionReason::DebugSqlcipherLoggingEnabled,
    );
}

#[test]
fn adapter_selection_reasons_and_profile_labels_have_stable_codes() {
    let reasons = [
        (
            LocalStoreDatabaseAdapterSelectionReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::DatabaseProfileRejected,
            1,
            "DATABASE_PROFILE_REJECTED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::AdapterKindRejected,
            2,
            "ADAPTER_KIND_REJECTED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::BindingKindRejected,
            3,
            "BINDING_KIND_REJECTED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::PlatformUnsupported,
            4,
            "PLATFORM_UNSUPPORTED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::LicenseRejected,
            5,
            "LICENSE_REJECTED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::SqlcipherVersionTooOld,
            6,
            "SQLCIPHER_VERSION_TOO_OLD",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::SourceAuthenticityMissing,
            7,
            "SOURCE_AUTHENTICITY_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::CryptoProviderUnverified,
            8,
            "CRYPTO_PROVIDER_UNVERIFIED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::FipsValidationMissing,
            9,
            "FIPS_VALIDATION_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::FipsRuntimeCheckMissing,
            10,
            "FIPS_RUNTIME_CHECK_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::SqlcipherCodecNotEnabled,
            11,
            "SQLCIPHER_CODEC_NOT_ENABLED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::UnsafeSqliteConfiguration,
            12,
            "UNSAFE_SQLITE_CONFIGURATION",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::ExtensionLoadingEnabled,
            13,
            "EXTENSION_LOADING_ENABLED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::TrustedSchemaEnabled,
            14,
            "TRUSTED_SCHEMA_ENABLED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::SecureDeleteMissing,
            15,
            "SECURE_DELETE_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::MemorySecurityMissing,
            16,
            "MEMORY_SECURITY_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::IntegrityCheckMissing,
            17,
            "INTEGRITY_CHECK_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::CompatibilityModeRejected,
            18,
            "COMPATIBILITY_MODE_REJECTED",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::MigrationDrillMissing,
            19,
            "MIGRATION_DRILL_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::CrashRecoveryDrillMissing,
            20,
            "CRASH_RECOVERY_DRILL_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::UnsignedReleaseArtifact,
            21,
            "UNSIGNED_RELEASE_ARTIFACT",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::SbomOrCveMonitoringMissing,
            22,
            "SBOM_OR_CVE_MONITORING_MISSING",
        ),
        (
            LocalStoreDatabaseAdapterSelectionReason::DebugSqlcipherLoggingEnabled,
            23,
            "DEBUG_SQLCIPHER_LOGGING_ENABLED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    assert_eq!(LocalStoreDatabaseAdapterKind::SqlCipherCommunity.code(), 1);
    assert_eq!(
        LocalStoreDatabaseAdapterKind::SqlCipherEnterpriseFips.label(),
        "sqlcipher_enterprise_fips"
    );
    assert_eq!(
        LocalStoreDatabaseBindingKind::RusqliteBundledSqlcipher.label(),
        "rusqlite_bundled_sqlcipher"
    );
    assert_eq!(LocalStoreDatabaseTargetPlatform::Android.code(), 5);
    assert_eq!(
        LocalStoreDatabaseLicenseKind::CommunityBsd.label(),
        "community_bsd"
    );
}

fn valid_input() -> LocalStoreDatabaseAdapterSelectionInput {
    LocalStoreDatabaseAdapterSelectionInput {
        database_security: valid_database_security_input().evaluate(),
        adapter_kind: LocalStoreDatabaseAdapterKind::SqlCipherCommunity,
        binding_kind: LocalStoreDatabaseBindingKind::RusqliteBundledSqlcipher,
        target_platform: LocalStoreDatabaseTargetPlatform::Windows,
        license_kind: LocalStoreDatabaseLicenseKind::CommunityBsd,
        sqlcipher_major_version: 4,
        sqlite_source_verified: true,
        sqlcipher_source_verified: true,
        platform_package_supported: true,
        license_allows_redistribution: true,
        crypto_provider_documented: true,
        fips_required: false,
        fips_module_validated: false,
        fips_runtime_self_tests_available: false,
        fips_mode_checked_at_runtime: false,
        compile_has_codec: true,
        compile_has_sqlcipher_extra_init_shutdown: true,
        temp_store_memory_configured: true,
        extension_loading_disabled: true,
        trusted_schema_disabled: true,
        secure_delete_configured: true,
        cipher_memory_security_enabled: true,
        cipher_integrity_check_on_open: true,
        sqlcipher_compatibility_current_major: true,
        deterministic_migration_tested: true,
        crash_recovery_drill_passed: true,
        release_artifacts_signed: true,
        sbom_present: true,
        cve_monitoring_enabled: true,
        debug_sqlcipher_logging_enabled: false,
    }
}

fn valid_database_security_input() -> LocalStoreDatabaseSecurityInput {
    LocalStoreDatabaseSecurityInput {
        platform_adapter: PlatformLocalStoreAdapterInput {
            runtime: PlatformLocalStoreRuntime::Desktop,
            adapter_kind: PlatformLocalStoreAdapterKind::ProductionEncryptedDatabase,
            database_root_present: true,
            os_keychain_available: true,
            hardware_backed_key_store: true,
            app_lock_satisfied: true,
            allow_development_adapters: false,
        }
        .evaluate(),
        production_open: valid_production_open_input().evaluate(),
        engine: LocalStoreDatabaseEngine::SqlCipherV4,
        cipher: LocalStoreDatabaseCipher::Aes256CbcHmacSha512,
        kdf: LocalStoreDatabaseKdf::RawKeyFromPlatformKeystore,
        kdf_iterations: MERCURY_LOCAL_STORE_MIN_KDF_ITERATIONS,
        page_size: MERCURY_LOCAL_STORE_PAGE_SIZE,
        per_page_random_nonce: true,
        per_page_authentication: true,
        encryption_key_separate_from_mac_key: true,
        unique_database_salt: true,
        raw_key_wrapped_by_platform_keystore: true,
        encrypted_wal: true,
        encrypted_journal: true,
        temp_store_memory_only: true,
        plaintext_header_bytes: 0,
        os_cloud_backup_excluded: true,
        backup_uses_consistent_encrypted_snapshot: true,
        secure_delete_enabled: true,
        memory_locking_enabled: true,
        zeroizes_key_material: true,
        crash_recovery_tested: true,
        plaintext_metadata_fields: 0,
        sqlite_extension_loading_enabled: false,
        debug_plaintext_export_enabled: false,
    }
}

fn valid_production_open_input() -> LocalStoreProductionOpenInput {
    LocalStoreProductionOpenInput {
        unlock: LocalStoreUnlockInput {
            store_version: MERCURY_LOCAL_STORE_VERSION,
            keychain_available: true,
            device_secret: LocalStoreUnlockSecretState::PresentSealed,
            database_header: LocalStoreUnlockDatabaseHeaderState::Authenticated,
            app_lock_satisfied: true,
            recovery_required: false,
            plaintext_cache_records: 0,
        },
        header_magic_matches: true,
        header_suite_code: LocalStoreSealingSuite::MercuryLocalStoreV1.code(),
        header_nonce_len: LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        header_tag_len: LocalStoreSealingSuite::MercuryLocalStoreV1.authentication_tag_len(),
        required_key_slots: 1,
        sealed_key_slots: 1,
        plaintext_key_slots: 0,
        root_key_scope: LocalStoreKeyScope::DeviceLocal,
        root_key_generation: 1,
        crash_recovery: LocalStoreCrashRecoveryState::Clean,
    }
}

fn assert_rejected(
    decision: LocalStoreDatabaseAdapterSelectionDecision,
    reason: LocalStoreDatabaseAdapterSelectionReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_link_adapter);
    assert!(!decision.can_open_database);
    assert!(!decision.can_ship_release);
    assert!(!decision.can_host_mls_transactions);
    assert!(decision.forbids_plaintext_storage);
    assert_eq!(decision.reason, reason);
}
