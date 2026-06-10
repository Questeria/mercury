use std::convert::Infallible;

use mercury_core::{
    AcceptedPlatformLocalStoreAdapter, PlatformLocalStoreAdapterFactory,
    PlatformLocalStoreAdapterInput, PlatformLocalStoreAdapterKind, PlatformLocalStoreAdapterReason,
    PlatformLocalStoreRuntime, open_platform_local_store_adapter,
};

#[test]
fn platform_adapter_accepts_desktop_production_database() {
    let decision = valid_input(PlatformLocalStoreRuntime::Desktop).evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason, PlatformLocalStoreAdapterReason::Accepted);
    assert_eq!(decision.reason_code(), 0);
    assert_eq!(decision.reason_label(), "ACCEPTED");
    assert!(decision.can_open_adapter);
    assert!(decision.forbids_plaintext_storage);
    assert!(!decision.requires_user_auth);
    assert!(!decision.requires_install_setup);
    assert!(!decision.requires_hardware_backing);
}

#[test]
fn platform_adapter_requires_mobile_hardware_backing() {
    let mut input = valid_input(PlatformLocalStoreRuntime::Mobile);
    input.hardware_backed_key_store = false;
    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        PlatformLocalStoreAdapterReason::HardwareBackingRequired
    );
    assert!(!decision.can_open_adapter);
    assert!(decision.requires_hardware_backing);
    assert!(decision.forbids_plaintext_storage);
}

#[test]
fn platform_adapter_rejects_plaintext_and_unapproved_development_adapters() {
    let mut plaintext = valid_input(PlatformLocalStoreRuntime::Desktop);
    plaintext.adapter_kind = PlatformLocalStoreAdapterKind::PlaintextFileStore;
    let plaintext_decision = plaintext.evaluate();

    assert!(!plaintext_decision.accepted);
    assert_eq!(
        plaintext_decision.reason,
        PlatformLocalStoreAdapterReason::PlaintextAdapterForbidden
    );
    assert!(plaintext_decision.forbids_plaintext_storage);

    let mut prototype = valid_input(PlatformLocalStoreRuntime::Desktop);
    prototype.adapter_kind = PlatformLocalStoreAdapterKind::PrototypeFileStore;
    prototype.allow_development_adapters = false;
    let prototype_decision = prototype.evaluate();

    assert!(!prototype_decision.accepted);
    assert_eq!(
        prototype_decision.reason,
        PlatformLocalStoreAdapterReason::DevelopmentAdapterForbidden
    );
}

#[test]
fn platform_adapter_reports_setup_and_auth_requirements() {
    let mut missing_root = valid_input(PlatformLocalStoreRuntime::Desktop);
    missing_root.database_root_present = false;
    let missing_root_decision = missing_root.evaluate();

    assert!(!missing_root_decision.accepted);
    assert_eq!(
        missing_root_decision.reason,
        PlatformLocalStoreAdapterReason::DatabaseRootMissing
    );
    assert!(missing_root_decision.requires_install_setup);

    let mut app_lock = valid_input(PlatformLocalStoreRuntime::Desktop);
    app_lock.app_lock_satisfied = false;
    let app_lock_decision = app_lock.evaluate();

    assert!(!app_lock_decision.accepted);
    assert_eq!(
        app_lock_decision.reason,
        PlatformLocalStoreAdapterReason::AppLockRequired
    );
    assert!(app_lock_decision.requires_user_auth);
}

#[test]
fn platform_adapter_factory_opens_only_after_accepted_gate() {
    let mut factory = RecordingPlatformAdapterFactory::default();

    let accepted = open_platform_local_store_adapter(
        &mut factory,
        valid_input(PlatformLocalStoreRuntime::Desktop),
    )
    .expect("accepted platform adapter should not fail");

    assert!(accepted.accepted);
    assert_eq!(factory.open_calls, 1);
    assert_eq!(
        factory.last_reason,
        Some(PlatformLocalStoreAdapterReason::Accepted)
    );

    let mut rejected_input = valid_input(PlatformLocalStoreRuntime::Desktop);
    rejected_input.os_keychain_available = false;
    let rejected = open_platform_local_store_adapter(&mut factory, rejected_input)
        .expect("rejected platform adapter should not fail");

    assert!(!rejected.accepted);
    assert_eq!(
        rejected.reason,
        PlatformLocalStoreAdapterReason::KeychainUnavailable
    );
    assert_eq!(factory.open_calls, 1);
}

#[test]
fn platform_adapter_kinds_and_reasons_have_stable_codes_and_labels() {
    let runtimes = [
        (PlatformLocalStoreRuntime::Unknown, 0, "unknown"),
        (PlatformLocalStoreRuntime::Desktop, 1, "desktop"),
        (PlatformLocalStoreRuntime::Mobile, 2, "mobile"),
    ];

    for (runtime, code, label) in runtimes {
        assert_eq!(runtime.code(), code);
        assert_eq!(runtime.label(), label);
    }

    let adapters = [
        (
            PlatformLocalStoreAdapterKind::ProductionEncryptedDatabase,
            1,
            "production_encrypted_database",
        ),
        (
            PlatformLocalStoreAdapterKind::PrototypeFileStore,
            2,
            "prototype_file_store",
        ),
        (
            PlatformLocalStoreAdapterKind::DevelopmentMemoryStore,
            3,
            "development_memory_store",
        ),
        (
            PlatformLocalStoreAdapterKind::PlaintextFileStore,
            4,
            "plaintext_file_store",
        ),
    ];

    for (adapter, code, label) in adapters {
        assert_eq!(adapter.code(), code);
        assert_eq!(adapter.label(), label);
    }

    let reasons = [
        (PlatformLocalStoreAdapterReason::Accepted, 0, "ACCEPTED"),
        (
            PlatformLocalStoreAdapterReason::UnknownRuntime,
            1,
            "UNKNOWN_RUNTIME",
        ),
        (
            PlatformLocalStoreAdapterReason::PlaintextAdapterForbidden,
            2,
            "PLAINTEXT_ADAPTER_FORBIDDEN",
        ),
        (
            PlatformLocalStoreAdapterReason::DevelopmentAdapterForbidden,
            3,
            "DEVELOPMENT_ADAPTER_FORBIDDEN",
        ),
        (
            PlatformLocalStoreAdapterReason::DatabaseRootMissing,
            4,
            "DATABASE_ROOT_MISSING",
        ),
        (
            PlatformLocalStoreAdapterReason::KeychainUnavailable,
            5,
            "KEYCHAIN_UNAVAILABLE",
        ),
        (
            PlatformLocalStoreAdapterReason::HardwareBackingRequired,
            6,
            "HARDWARE_BACKING_REQUIRED",
        ),
        (
            PlatformLocalStoreAdapterReason::AppLockRequired,
            7,
            "APP_LOCK_REQUIRED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

#[derive(Default)]
struct RecordingPlatformAdapterFactory {
    open_calls: usize,
    last_reason: Option<PlatformLocalStoreAdapterReason>,
}

impl PlatformLocalStoreAdapterFactory for RecordingPlatformAdapterFactory {
    type Error = Infallible;

    fn open_platform_adapter(
        &mut self,
        accepted: AcceptedPlatformLocalStoreAdapter,
    ) -> Result<(), Self::Error> {
        self.open_calls += 1;
        self.last_reason = Some(accepted.decision().reason);
        Ok(())
    }
}

fn valid_input(runtime: PlatformLocalStoreRuntime) -> PlatformLocalStoreAdapterInput {
    PlatformLocalStoreAdapterInput {
        runtime,
        adapter_kind: PlatformLocalStoreAdapterKind::ProductionEncryptedDatabase,
        database_root_present: true,
        os_keychain_available: true,
        hardware_backed_key_store: true,
        app_lock_satisfied: true,
        allow_development_adapters: false,
    }
}
