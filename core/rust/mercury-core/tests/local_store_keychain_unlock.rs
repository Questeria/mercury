use mercury_core::{
    LocalStoreKeychainBackend, LocalStoreKeychainProtection, LocalStoreKeychainReason,
    LocalStoreKeychainUnlockInput, LocalStoreUnlockDatabaseHeaderState, LocalStoreUnlockReason,
    LocalStoreUnlockSecretState, MERCURY_LOCAL_STORE_VERSION,
};

#[test]
fn keychain_unlock_accepts_hardware_backed_sealed_secret() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_build_unlock_input);
    assert!(!decision.requires_user_auth);
    assert!(!decision.requires_recovery);
    assert!(!decision.requires_destructive_repair);
    assert_eq!(decision.backend_code, 2);
    assert_eq!(decision.backend_label, "android_keystore");
    assert_eq!(decision.protection_code, 1);
    assert_eq!(decision.protection_label, "hardware_backed");
    assert_eq!(decision.reason, LocalStoreKeychainReason::Accepted);

    let unlock = decision.unlock_input.evaluate();
    assert!(unlock.accepted);
    assert_eq!(unlock.reason, LocalStoreUnlockReason::Accepted);
}

#[test]
fn keychain_unlock_rejects_unavailable_or_dev_only_backends() {
    let mut unavailable = valid_input();
    unavailable.backend_available = false;
    let unavailable_decision = unavailable.evaluate();
    assert_rejected(
        unavailable_decision,
        LocalStoreKeychainReason::BackendUnavailable,
    );
    assert!(unavailable_decision.requires_user_auth);
    assert!(!unavailable_decision.requires_destructive_repair);

    let mut dev_backend = valid_input();
    dev_backend.backend = LocalStoreKeychainBackend::DevelopmentMemory;
    dev_backend.protection = LocalStoreKeychainProtection::DevelopmentOnly;
    let dev_backend_decision = dev_backend.evaluate();
    assert_rejected(
        dev_backend_decision,
        LocalStoreKeychainReason::DevelopmentBackendForbidden,
    );
    assert!(dev_backend_decision.requires_destructive_repair);

    let mut allowed_dev = dev_backend;
    allowed_dev.allow_development_backend = true;
    let allowed_dev_decision = allowed_dev.evaluate();
    assert!(allowed_dev_decision.accepted);
    assert_eq!(allowed_dev_decision.backend_label, "development_memory");
    assert_eq!(allowed_dev_decision.protection_label, "development_only");
}

#[test]
fn keychain_unlock_rejects_missing_corrupt_plaintext_or_exportable_secrets() {
    let mut missing = valid_input();
    missing.device_secret = LocalStoreUnlockSecretState::Missing;
    let missing_decision = missing.evaluate();
    assert_rejected(
        missing_decision,
        LocalStoreKeychainReason::DeviceSecretMissing,
    );
    assert!(missing_decision.requires_recovery);

    let mut corrupt = valid_input();
    corrupt.device_secret = LocalStoreUnlockSecretState::Corrupt;
    let corrupt_decision = corrupt.evaluate();
    assert_rejected(
        corrupt_decision,
        LocalStoreKeychainReason::DeviceSecretCorrupt,
    );
    assert!(corrupt_decision.requires_recovery);

    let mut plaintext = valid_input();
    plaintext.device_secret = LocalStoreUnlockSecretState::PlaintextPresent;
    let plaintext_decision = plaintext.evaluate();
    assert_rejected(
        plaintext_decision,
        LocalStoreKeychainReason::PlaintextSecretForbidden,
    );
    assert!(plaintext_decision.requires_destructive_repair);

    let mut exportable = valid_input();
    exportable.device_secret_exportable = true;
    let exportable_decision = exportable.evaluate();
    assert_rejected(
        exportable_decision,
        LocalStoreKeychainReason::ExportableSecretForbidden,
    );
    assert!(exportable_decision.requires_destructive_repair);
}

#[test]
fn keychain_unlock_separates_user_auth_from_header_rejection() {
    let mut auth_required = valid_input();
    auth_required.user_auth_required = true;
    auth_required.user_auth_satisfied = false;
    let auth_decision = auth_required.evaluate();
    assert_rejected(auth_decision, LocalStoreKeychainReason::UserAuthRequired);
    assert!(auth_decision.requires_user_auth);
    assert!(!auth_decision.unlock_input.app_lock_satisfied);

    let mut bad_header = valid_input();
    bad_header.database_header = LocalStoreUnlockDatabaseHeaderState::AuthenticationFailed;
    let bad_header_decision = bad_header.evaluate();
    assert!(bad_header_decision.accepted);

    let unlock = bad_header_decision.unlock_input.evaluate();
    assert!(!unlock.accepted);
    assert_eq!(
        unlock.reason,
        LocalStoreUnlockReason::DatabaseHeaderAuthenticationFailed
    );
    assert!(unlock.requires_user_auth);
}

#[test]
fn keychain_backend_protection_and_reasons_have_stable_codes_and_labels() {
    let backends = [
        (LocalStoreKeychainBackend::IosKeychain, 1, "ios_keychain"),
        (
            LocalStoreKeychainBackend::AndroidKeystore,
            2,
            "android_keystore",
        ),
        (
            LocalStoreKeychainBackend::MacosKeychain,
            3,
            "macos_keychain",
        ),
        (
            LocalStoreKeychainBackend::WindowsCredentialVault,
            4,
            "windows_credential_vault",
        ),
        (
            LocalStoreKeychainBackend::LinuxSecretService,
            5,
            "linux_secret_service",
        ),
        (
            LocalStoreKeychainBackend::DevelopmentMemory,
            99,
            "development_memory",
        ),
    ];

    for (backend, code, label) in backends {
        assert_eq!(backend.code(), code);
        assert_eq!(backend.label(), label);
    }

    let protections = [
        (
            LocalStoreKeychainProtection::HardwareBacked,
            1,
            "hardware_backed",
        ),
        (LocalStoreKeychainProtection::OsProtected, 2, "os_protected"),
        (
            LocalStoreKeychainProtection::DevelopmentOnly,
            99,
            "development_only",
        ),
    ];

    for (protection, code, label) in protections {
        assert_eq!(protection.code(), code);
        assert_eq!(protection.label(), label);
    }

    let reasons = [
        (LocalStoreKeychainReason::Accepted, 0, "ACCEPTED"),
        (
            LocalStoreKeychainReason::BackendUnavailable,
            1,
            "BACKEND_UNAVAILABLE",
        ),
        (
            LocalStoreKeychainReason::DevelopmentBackendForbidden,
            2,
            "DEVELOPMENT_BACKEND_FORBIDDEN",
        ),
        (
            LocalStoreKeychainReason::UserAuthRequired,
            3,
            "USER_AUTH_REQUIRED",
        ),
        (
            LocalStoreKeychainReason::DeviceSecretMissing,
            4,
            "DEVICE_SECRET_MISSING",
        ),
        (
            LocalStoreKeychainReason::DeviceSecretCorrupt,
            5,
            "DEVICE_SECRET_CORRUPT",
        ),
        (
            LocalStoreKeychainReason::PlaintextSecretForbidden,
            6,
            "PLAINTEXT_SECRET_FORBIDDEN",
        ),
        (
            LocalStoreKeychainReason::ExportableSecretForbidden,
            7,
            "EXPORTABLE_SECRET_FORBIDDEN",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> LocalStoreKeychainUnlockInput {
    LocalStoreKeychainUnlockInput {
        store_version: MERCURY_LOCAL_STORE_VERSION,
        backend: LocalStoreKeychainBackend::AndroidKeystore,
        backend_available: true,
        protection: LocalStoreKeychainProtection::HardwareBacked,
        allow_development_backend: false,
        user_auth_required: false,
        user_auth_satisfied: false,
        device_secret: LocalStoreUnlockSecretState::PresentSealed,
        device_secret_exportable: false,
        database_header: LocalStoreUnlockDatabaseHeaderState::Authenticated,
        recovery_required: false,
        plaintext_cache_records: 0,
    }
}

fn assert_rejected(
    decision: mercury_core::LocalStoreKeychainUnlockDecision,
    reason: LocalStoreKeychainReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_build_unlock_input);
    assert_eq!(decision.reason, reason);
}
