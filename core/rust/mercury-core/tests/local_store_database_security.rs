use mercury_core::{
    LocalStoreCrashRecoveryState, LocalStoreDatabaseCipher, LocalStoreDatabaseEngine,
    LocalStoreDatabaseKdf, LocalStoreDatabaseSecurityInput, LocalStoreDatabaseSecurityReason,
    LocalStoreKeyScope, LocalStoreProductionOpenInput, LocalStoreProductionOpenReason,
    LocalStoreSealingSuite, LocalStoreUnlockDatabaseHeaderState, LocalStoreUnlockInput,
    LocalStoreUnlockSecretState, MERCURY_LOCAL_STORE_MIN_KDF_ITERATIONS,
    MERCURY_LOCAL_STORE_PAGE_SIZE, MERCURY_LOCAL_STORE_VERSION, PlatformLocalStoreAdapterInput,
    PlatformLocalStoreAdapterKind, PlatformLocalStoreAdapterReason, PlatformLocalStoreRuntime,
};

#[test]
fn database_security_accepts_sqlcipher_style_profile() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_open_database);
    assert!(decision.can_load_records);
    assert!(decision.can_load_message_keys);
    assert!(decision.can_host_mls_transactions);
    assert!(!decision.requires_user_auth);
    assert!(!decision.requires_install_setup);
    assert!(!decision.requires_hardware_backing);
    assert!(!decision.requires_recovery);
    assert!(!decision.requires_migration);
    assert!(!decision.requires_crash_recovery);
    assert!(!decision.requires_destructive_repair);
    assert!(!decision.requires_backup_reconfiguration);
    assert!(decision.forbids_plaintext_storage);
    assert_eq!(decision.reason, LocalStoreDatabaseSecurityReason::Accepted);
    assert_eq!(decision.engine_label, "sqlcipher_v4");
    assert_eq!(decision.cipher_label, "aes_256_cbc_hmac_sha512");
    assert_eq!(decision.kdf_label, "raw_key_from_platform_keystore");
    assert_eq!(
        decision.platform_adapter_reason,
        PlatformLocalStoreAdapterReason::Accepted
    );
    assert_eq!(
        decision.production_open_reason,
        LocalStoreProductionOpenReason::Accepted
    );
}

#[test]
fn database_security_propagates_platform_and_open_rejection() {
    let mut no_adapter = valid_input();
    no_adapter.platform_adapter = PlatformLocalStoreAdapterInput {
        runtime: PlatformLocalStoreRuntime::Mobile,
        adapter_kind: PlatformLocalStoreAdapterKind::ProductionEncryptedDatabase,
        database_root_present: true,
        os_keychain_available: true,
        hardware_backed_key_store: false,
        app_lock_satisfied: true,
        allow_development_adapters: false,
    }
    .evaluate();
    let no_adapter_decision = no_adapter.evaluate();

    assert_rejected(
        no_adapter_decision,
        LocalStoreDatabaseSecurityReason::PlatformAdapterRejected,
    );
    assert!(no_adapter_decision.requires_hardware_backing);
    assert_eq!(
        no_adapter_decision.platform_adapter_reason,
        PlatformLocalStoreAdapterReason::HardwareBackingRequired
    );

    let mut replay_required = valid_input();
    let mut open_input = valid_production_open_input();
    open_input.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;
    replay_required.production_open = open_input.evaluate();
    let replay_required_decision = replay_required.evaluate();

    assert_rejected(
        replay_required_decision,
        LocalStoreDatabaseSecurityReason::ProductionOpenRejected,
    );
    assert!(replay_required_decision.requires_crash_recovery);
    assert_eq!(
        replay_required_decision.production_open_reason,
        LocalStoreProductionOpenReason::WalReplayRequired
    );
}

#[test]
fn database_security_rejects_plaintext_and_weak_crypto_profiles() {
    let mut plain = valid_input();
    plain.engine = LocalStoreDatabaseEngine::PlainSqlite;
    let plain_decision = plain.evaluate();
    assert_rejected(
        plain_decision,
        LocalStoreDatabaseSecurityReason::PlaintextDatabaseForbidden,
    );
    assert!(plain_decision.requires_install_setup);
    assert!(plain_decision.requires_destructive_repair);

    let mut no_cipher = valid_input();
    no_cipher.cipher = LocalStoreDatabaseCipher::None;
    let no_cipher_decision = no_cipher.evaluate();
    assert_rejected(
        no_cipher_decision,
        LocalStoreDatabaseSecurityReason::WeakCipherSuite,
    );
    assert!(no_cipher_decision.requires_migration);

    let mut weak_kdf = valid_input();
    weak_kdf.kdf = LocalStoreDatabaseKdf::Pbkdf2HmacSha512;
    weak_kdf.kdf_iterations = MERCURY_LOCAL_STORE_MIN_KDF_ITERATIONS - 1;
    let weak_kdf_decision = weak_kdf.evaluate();
    assert_rejected(
        weak_kdf_decision,
        LocalStoreDatabaseSecurityReason::KdfTooWeak,
    );
    assert!(weak_kdf_decision.requires_migration);
}

#[test]
fn database_security_rejects_page_authentication_and_key_lifecycle_gaps() {
    let mut bad_page = valid_input();
    bad_page.page_size = MERCURY_LOCAL_STORE_PAGE_SIZE * 2;
    let bad_page_decision = bad_page.evaluate();
    assert_rejected(
        bad_page_decision,
        LocalStoreDatabaseSecurityReason::PageShapeRejected,
    );

    let mut no_page_auth = valid_input();
    no_page_auth.per_page_authentication = false;
    let no_page_auth_decision = no_page_auth.evaluate();
    assert_rejected(
        no_page_auth_decision,
        LocalStoreDatabaseSecurityReason::MissingPerPageAuthentication,
    );

    let mut reused_mac_key = valid_input();
    reused_mac_key.encryption_key_separate_from_mac_key = false;
    let reused_mac_key_decision = reused_mac_key.evaluate();
    assert_rejected(
        reused_mac_key_decision,
        LocalStoreDatabaseSecurityReason::MacKeyReuseForbidden,
    );

    let mut missing_salt = valid_input();
    missing_salt.unique_database_salt = false;
    let missing_salt_decision = missing_salt.evaluate();
    assert_rejected(
        missing_salt_decision,
        LocalStoreDatabaseSecurityReason::DatabaseSaltMissing,
    );

    let mut no_keystore_wrap = valid_input();
    no_keystore_wrap.raw_key_wrapped_by_platform_keystore = false;
    let no_keystore_wrap_decision = no_keystore_wrap.evaluate();
    assert_rejected(
        no_keystore_wrap_decision,
        LocalStoreDatabaseSecurityReason::KeyNotKeystoreWrapped,
    );
    assert!(no_keystore_wrap_decision.requires_hardware_backing);
}

#[test]
fn database_security_rejects_wal_temp_backup_and_secret_gaps() {
    let mut plaintext_wal = valid_input();
    plaintext_wal.encrypted_wal = false;
    let plaintext_wal_decision = plaintext_wal.evaluate();
    assert_rejected(
        plaintext_wal_decision,
        LocalStoreDatabaseSecurityReason::WalOrJournalPlaintext,
    );
    assert!(plaintext_wal_decision.requires_crash_recovery);

    let mut file_temp = valid_input();
    file_temp.temp_store_memory_only = false;
    let file_temp_decision = file_temp.evaluate();
    assert_rejected(
        file_temp_decision,
        LocalStoreDatabaseSecurityReason::FileTempStoreForbidden,
    );

    let mut plaintext_header = valid_input();
    plaintext_header.plaintext_header_bytes = 16;
    let plaintext_header_decision = plaintext_header.evaluate();
    assert_rejected(
        plaintext_header_decision,
        LocalStoreDatabaseSecurityReason::PlaintextHeaderForbidden,
    );

    let mut backup = valid_input();
    backup.os_cloud_backup_excluded = false;
    let backup_decision = backup.evaluate();
    assert_rejected(
        backup_decision,
        LocalStoreDatabaseSecurityReason::BackupPolicyRejected,
    );
    assert!(backup_decision.requires_backup_reconfiguration);

    let mut no_secure_delete = valid_input();
    no_secure_delete.secure_delete_enabled = false;
    let no_secure_delete_decision = no_secure_delete.evaluate();
    assert_rejected(
        no_secure_delete_decision,
        LocalStoreDatabaseSecurityReason::SecureDeleteMissing,
    );

    let mut no_zeroize = valid_input();
    no_zeroize.zeroizes_key_material = false;
    let no_zeroize_decision = no_zeroize.evaluate();
    assert_rejected(
        no_zeroize_decision,
        LocalStoreDatabaseSecurityReason::SecretLifecycleRejected,
    );

    let mut untested_recovery = valid_input();
    untested_recovery.crash_recovery_tested = false;
    let untested_recovery_decision = untested_recovery.evaluate();
    assert_rejected(
        untested_recovery_decision,
        LocalStoreDatabaseSecurityReason::CrashRecoveryUntested,
    );
    assert!(untested_recovery_decision.requires_crash_recovery);
}

#[test]
fn database_security_rejects_plaintext_metadata_extensions_and_debug_exports() {
    let mut metadata = valid_input();
    metadata.plaintext_metadata_fields = 1;
    let metadata_decision = metadata.evaluate();
    assert_rejected(
        metadata_decision,
        LocalStoreDatabaseSecurityReason::PlaintextMetadataForbidden,
    );

    let mut extensions = valid_input();
    extensions.sqlite_extension_loading_enabled = true;
    let extensions_decision = extensions.evaluate();
    assert_rejected(
        extensions_decision,
        LocalStoreDatabaseSecurityReason::ExtensionLoadingForbidden,
    );

    let mut debug_export = valid_input();
    debug_export.debug_plaintext_export_enabled = true;
    let debug_export_decision = debug_export.evaluate();
    assert_rejected(
        debug_export_decision,
        LocalStoreDatabaseSecurityReason::DebugExportForbidden,
    );
}

#[test]
fn database_security_reasons_and_profile_labels_have_stable_codes() {
    let reasons = [
        (LocalStoreDatabaseSecurityReason::Accepted, 0, "ACCEPTED"),
        (
            LocalStoreDatabaseSecurityReason::PlatformAdapterRejected,
            1,
            "PLATFORM_ADAPTER_REJECTED",
        ),
        (
            LocalStoreDatabaseSecurityReason::ProductionOpenRejected,
            2,
            "PRODUCTION_OPEN_REJECTED",
        ),
        (
            LocalStoreDatabaseSecurityReason::PlaintextDatabaseForbidden,
            3,
            "PLAINTEXT_DATABASE_FORBIDDEN",
        ),
        (
            LocalStoreDatabaseSecurityReason::WeakCipherSuite,
            4,
            "WEAK_CIPHER_SUITE",
        ),
        (
            LocalStoreDatabaseSecurityReason::KdfTooWeak,
            5,
            "KDF_TOO_WEAK",
        ),
        (
            LocalStoreDatabaseSecurityReason::PageShapeRejected,
            6,
            "PAGE_SHAPE_REJECTED",
        ),
        (
            LocalStoreDatabaseSecurityReason::MissingPerPageAuthentication,
            7,
            "MISSING_PER_PAGE_AUTHENTICATION",
        ),
        (
            LocalStoreDatabaseSecurityReason::MacKeyReuseForbidden,
            8,
            "MAC_KEY_REUSE_FORBIDDEN",
        ),
        (
            LocalStoreDatabaseSecurityReason::DatabaseSaltMissing,
            9,
            "DATABASE_SALT_MISSING",
        ),
        (
            LocalStoreDatabaseSecurityReason::KeyNotKeystoreWrapped,
            10,
            "KEY_NOT_KEYSTORE_WRAPPED",
        ),
        (
            LocalStoreDatabaseSecurityReason::WalOrJournalPlaintext,
            11,
            "WAL_OR_JOURNAL_PLAINTEXT",
        ),
        (
            LocalStoreDatabaseSecurityReason::FileTempStoreForbidden,
            12,
            "FILE_TEMP_STORE_FORBIDDEN",
        ),
        (
            LocalStoreDatabaseSecurityReason::PlaintextHeaderForbidden,
            13,
            "PLAINTEXT_HEADER_FORBIDDEN",
        ),
        (
            LocalStoreDatabaseSecurityReason::BackupPolicyRejected,
            14,
            "BACKUP_POLICY_REJECTED",
        ),
        (
            LocalStoreDatabaseSecurityReason::SecureDeleteMissing,
            15,
            "SECURE_DELETE_MISSING",
        ),
        (
            LocalStoreDatabaseSecurityReason::SecretLifecycleRejected,
            16,
            "SECRET_LIFECYCLE_REJECTED",
        ),
        (
            LocalStoreDatabaseSecurityReason::CrashRecoveryUntested,
            17,
            "CRASH_RECOVERY_UNTESTED",
        ),
        (
            LocalStoreDatabaseSecurityReason::PlaintextMetadataForbidden,
            18,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            LocalStoreDatabaseSecurityReason::ExtensionLoadingForbidden,
            19,
            "EXTENSION_LOADING_FORBIDDEN",
        ),
        (
            LocalStoreDatabaseSecurityReason::DebugExportForbidden,
            20,
            "DEBUG_EXPORT_FORBIDDEN",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    assert_eq!(LocalStoreDatabaseEngine::SqlCipherV4.code(), 1);
    assert_eq!(
        LocalStoreDatabaseEngine::SqlCipherV4.label(),
        "sqlcipher_v4"
    );
    assert_eq!(
        LocalStoreDatabaseCipher::Aes256CbcHmacSha512.label(),
        "aes_256_cbc_hmac_sha512"
    );
    assert_eq!(
        LocalStoreDatabaseKdf::RawKeyFromPlatformKeystore.label(),
        "raw_key_from_platform_keystore"
    );
}

fn valid_input() -> LocalStoreDatabaseSecurityInput {
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
    decision: mercury_core::LocalStoreDatabaseSecurityDecision,
    reason: LocalStoreDatabaseSecurityReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_open_database);
    assert!(!decision.can_load_records);
    assert!(!decision.can_load_message_keys);
    assert!(!decision.can_host_mls_transactions);
    assert!(decision.forbids_plaintext_storage);
    assert_eq!(decision.reason, reason);
}
