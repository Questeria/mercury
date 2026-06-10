use mercury_core::{
    LocalStoreUnlockDatabaseHeaderState, LocalStoreUnlockInput, LocalStoreUnlockReason,
    LocalStoreUnlockSecretState, MERCURY_LOCAL_STORE_VERSION,
};

#[test]
fn local_store_unlock_accepts_ready_store() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_open_database);
    assert!(decision.can_unseal_device_secret);
    assert!(decision.can_load_message_keys);
    assert!(!decision.requires_user_auth);
    assert!(!decision.requires_recovery);
    assert!(!decision.requires_migration);
    assert!(!decision.requires_destructive_repair);
    assert_eq!(decision.reason, LocalStoreUnlockReason::Accepted);
    assert_eq!(decision.reason.code(), 0);
    assert_eq!(decision.reason.label(), "ACCEPTED");
}

#[test]
fn local_store_unlock_rejects_unsupported_versions_before_open() {
    let mut input = valid_input();
    input.store_version = MERCURY_LOCAL_STORE_VERSION + 1;

    let decision = input.evaluate();

    assert_rejected(decision, LocalStoreUnlockReason::UnsupportedStoreVersion);
    assert!(decision.requires_migration);
    assert!(!decision.requires_destructive_repair);
}

#[test]
fn local_store_unlock_rejects_plaintext_cache_before_keychain_use() {
    let mut input = valid_input();
    input.keychain_available = false;
    input.plaintext_cache_records = 1;

    let decision = input.evaluate();

    assert_rejected(decision, LocalStoreUnlockReason::PlaintextCacheForbidden);
    assert!(decision.requires_destructive_repair);
    assert!(!decision.requires_user_auth);
}

#[test]
fn local_store_unlock_requires_recovery_for_recovery_or_missing_secret() {
    let mut recovery = valid_input();
    recovery.recovery_required = true;
    let recovery_decision = recovery.evaluate();
    assert_rejected(recovery_decision, LocalStoreUnlockReason::RecoveryRequired);
    assert!(recovery_decision.requires_recovery);

    let mut missing = valid_input();
    missing.device_secret = LocalStoreUnlockSecretState::Missing;
    let missing_decision = missing.evaluate();
    assert_rejected(
        missing_decision,
        LocalStoreUnlockReason::DeviceSecretMissing,
    );
    assert!(missing_decision.requires_recovery);

    let mut corrupt = valid_input();
    corrupt.device_secret = LocalStoreUnlockSecretState::Corrupt;
    let corrupt_decision = corrupt.evaluate();
    assert_rejected(
        corrupt_decision,
        LocalStoreUnlockReason::DeviceSecretCorrupt,
    );
    assert!(corrupt_decision.requires_recovery);
}

#[test]
fn local_store_unlock_rejects_plaintext_device_secret_as_destructive_repair() {
    let mut input = valid_input();
    input.device_secret = LocalStoreUnlockSecretState::PlaintextPresent;

    let decision = input.evaluate();

    assert_rejected(decision, LocalStoreUnlockReason::PlaintextSecretForbidden);
    assert!(decision.requires_destructive_repair);
}

#[test]
fn local_store_unlock_separates_keychain_and_app_lock_auth() {
    let mut keychain = valid_input();
    keychain.keychain_available = false;
    let keychain_decision = keychain.evaluate();
    assert_rejected(
        keychain_decision,
        LocalStoreUnlockReason::KeychainUnavailable,
    );
    assert!(keychain_decision.requires_user_auth);

    let mut app_lock = valid_input();
    app_lock.app_lock_satisfied = false;
    let app_lock_decision = app_lock.evaluate();
    assert_rejected(app_lock_decision, LocalStoreUnlockReason::AppLockRequired);
    assert!(app_lock_decision.requires_user_auth);
}

#[test]
fn local_store_unlock_rejects_bad_database_headers() {
    let mut missing = valid_input();
    missing.database_header = LocalStoreUnlockDatabaseHeaderState::Missing;
    let missing_decision = missing.evaluate();
    assert_rejected(
        missing_decision,
        LocalStoreUnlockReason::DatabaseHeaderMissing,
    );
    assert!(missing_decision.requires_recovery);

    let mut corrupt = valid_input();
    corrupt.database_header = LocalStoreUnlockDatabaseHeaderState::Corrupt;
    let corrupt_decision = corrupt.evaluate();
    assert_rejected(
        corrupt_decision,
        LocalStoreUnlockReason::DatabaseHeaderCorrupt,
    );
    assert!(corrupt_decision.requires_destructive_repair);

    let mut auth_failed = valid_input();
    auth_failed.database_header = LocalStoreUnlockDatabaseHeaderState::AuthenticationFailed;
    let auth_failed_decision = auth_failed.evaluate();
    assert_rejected(
        auth_failed_decision,
        LocalStoreUnlockReason::DatabaseHeaderAuthenticationFailed,
    );
    assert!(auth_failed_decision.requires_user_auth);
}

#[test]
fn local_store_unlock_reasons_have_stable_codes_and_labels() {
    let cases = [
        (LocalStoreUnlockReason::Accepted, 0, "ACCEPTED"),
        (
            LocalStoreUnlockReason::UnsupportedStoreVersion,
            1,
            "UNSUPPORTED_STORE_VERSION",
        ),
        (
            LocalStoreUnlockReason::PlaintextCacheForbidden,
            2,
            "PLAINTEXT_CACHE_FORBIDDEN",
        ),
        (
            LocalStoreUnlockReason::RecoveryRequired,
            3,
            "RECOVERY_REQUIRED",
        ),
        (
            LocalStoreUnlockReason::KeychainUnavailable,
            4,
            "KEYCHAIN_UNAVAILABLE",
        ),
        (
            LocalStoreUnlockReason::DeviceSecretMissing,
            5,
            "DEVICE_SECRET_MISSING",
        ),
        (
            LocalStoreUnlockReason::DeviceSecretCorrupt,
            6,
            "DEVICE_SECRET_CORRUPT",
        ),
        (
            LocalStoreUnlockReason::PlaintextSecretForbidden,
            7,
            "PLAINTEXT_SECRET_FORBIDDEN",
        ),
        (
            LocalStoreUnlockReason::DatabaseHeaderMissing,
            8,
            "DATABASE_HEADER_MISSING",
        ),
        (
            LocalStoreUnlockReason::DatabaseHeaderCorrupt,
            9,
            "DATABASE_HEADER_CORRUPT",
        ),
        (
            LocalStoreUnlockReason::DatabaseHeaderAuthenticationFailed,
            10,
            "DATABASE_HEADER_AUTHENTICATION_FAILED",
        ),
        (
            LocalStoreUnlockReason::AppLockRequired,
            11,
            "APP_LOCK_REQUIRED",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> LocalStoreUnlockInput {
    LocalStoreUnlockInput {
        store_version: MERCURY_LOCAL_STORE_VERSION,
        keychain_available: true,
        device_secret: LocalStoreUnlockSecretState::PresentSealed,
        database_header: LocalStoreUnlockDatabaseHeaderState::Authenticated,
        app_lock_satisfied: true,
        recovery_required: false,
        plaintext_cache_records: 0,
    }
}

fn assert_rejected(
    decision: mercury_core::LocalStoreUnlockDecision,
    reason: LocalStoreUnlockReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_open_database);
    assert!(!decision.can_unseal_device_secret);
    assert!(!decision.can_load_message_keys);
    assert_eq!(decision.reason, reason);
}
