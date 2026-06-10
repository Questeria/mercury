use mercury_core::{
    ComponentReasons, LocalStoreCrashRecoveryState, LocalStoreKeyScope, LocalStoreKeychainBackend,
    LocalStoreKeychainProtection, LocalStoreKeychainUnlockInput, LocalStorePayload,
    LocalStoreProductionOpenReason, LocalStoreRecordKind, LocalStoreRecordLocator,
    LocalStoreSealingSuite, LocalStoreUnlockDatabaseHeaderState, LocalStoreUnlockSecretState,
    LocalStoreWriteReason, MERCURY_LOCAL_STORE_VERSION, PolicyDecision,
    PrototypeEncryptedLocalStore, PrototypeProductionStoreSessionInput,
    PrototypeProductionStoreSessionReason, run_prototype_production_store_session,
};

#[test]
fn production_store_session_completes_without_plaintext_exposure() {
    let mut store = PrototypeEncryptedLocalStore::default();

    let outcome = run_prototype_production_store_session(&mut store, valid_session_input())
        .expect("prototype session should not fail");

    assert!(outcome.accepted);
    assert_eq!(
        outcome.reason,
        PrototypeProductionStoreSessionReason::Accepted
    );
    assert!(outcome.open_attempted);
    assert!(!outcome.wal_replayed);
    assert!(!outcome.plaintext_exposed);
    assert!(outcome.keychain_decision.accepted);
    assert!(outcome.unlock_decision.accepted);
    assert!(outcome.production_open_decision.accepted);
    assert!(outcome.write_decision.expect("write decision").accepted);

    let read = outcome.read_record.expect("read record");
    assert_eq!(read.record_kind, LocalStoreRecordKind::MessageCiphertext);
    assert_eq!(read.bytes, b"sealed-message");
}

#[test]
fn production_store_session_stops_when_keychain_rejects() {
    let mut store = PrototypeEncryptedLocalStore::default();
    let mut input = valid_session_input();
    input.keychain.device_secret_exportable = true;

    let outcome = run_prototype_production_store_session(&mut store, input)
        .expect("prototype session should not fail");

    assert!(!outcome.accepted);
    assert_eq!(
        outcome.reason,
        PrototypeProductionStoreSessionReason::KeychainRejected
    );
    assert!(!outcome.open_attempted);
    assert!(!outcome.wal_replayed);
    assert!(outcome.write_decision.is_none());
    assert!(outcome.read_record.is_none());
}

#[test]
fn production_store_session_replays_wal_before_open_or_write() {
    let mut store = PrototypeEncryptedLocalStore::default();
    let mut input = valid_session_input();
    input.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;

    let outcome = run_prototype_production_store_session(&mut store, input)
        .expect("prototype session should not fail");

    assert!(!outcome.accepted);
    assert_eq!(
        outcome.reason,
        PrototypeProductionStoreSessionReason::WalReplayRequired
    );
    assert!(!outcome.open_attempted);
    assert!(outcome.wal_replayed);
    assert_eq!(
        outcome.production_open_decision.reason,
        LocalStoreProductionOpenReason::WalReplayRequired
    );
    assert!(outcome.write_decision.is_none());
}

#[test]
fn production_store_session_reports_rejected_store_write() {
    let mut store = PrototypeEncryptedLocalStore::default();
    let mut input = valid_session_input();
    input.write_request = mercury_core::LocalStoreWriteRequest::new(
        locator(),
        LocalStoreRecordKind::MessagePlaintext,
        LocalStorePayload::public_metadata(b"plaintext"),
        Some(policy_decision(true)),
    );

    let outcome = run_prototype_production_store_session(&mut store, input)
        .expect("prototype session should not fail");

    assert!(!outcome.accepted);
    assert_eq!(
        outcome.reason,
        PrototypeProductionStoreSessionReason::StoreWriteRejected
    );
    assert!(outcome.open_attempted);
    assert_eq!(
        outcome.write_decision.expect("write decision").reason,
        LocalStoreWriteReason::PlaintextRecordForbidden
    );
    assert!(outcome.read_record.is_none());
}

#[test]
fn production_store_session_reasons_have_stable_codes_and_labels() {
    let cases = [
        (
            PrototypeProductionStoreSessionReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            PrototypeProductionStoreSessionReason::KeychainRejected,
            1,
            "KEYCHAIN_REJECTED",
        ),
        (
            PrototypeProductionStoreSessionReason::UnlockRejected,
            2,
            "UNLOCK_REJECTED",
        ),
        (
            PrototypeProductionStoreSessionReason::ProductionOpenRejected,
            3,
            "PRODUCTION_OPEN_REJECTED",
        ),
        (
            PrototypeProductionStoreSessionReason::WalReplayRequired,
            4,
            "WAL_REPLAY_REQUIRED",
        ),
        (
            PrototypeProductionStoreSessionReason::StoreWriteRejected,
            5,
            "STORE_WRITE_REJECTED",
        ),
        (
            PrototypeProductionStoreSessionReason::StoreReadMissing,
            6,
            "STORE_READ_MISSING",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_session_input() -> PrototypeProductionStoreSessionInput<'static> {
    PrototypeProductionStoreSessionInput {
        keychain: LocalStoreKeychainUnlockInput {
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
        write_request: mercury_core::LocalStoreWriteRequest::new(
            locator(),
            LocalStoreRecordKind::MessageCiphertext,
            LocalStorePayload::sealed(b"sealed-message"),
            Some(policy_decision(true)),
        ),
    }
}

fn locator() -> LocalStoreRecordLocator<'static> {
    LocalStoreRecordLocator::new("conversation-7", "message-42")
}

fn policy_decision(accepted: bool) -> PolicyDecision {
    PolicyDecision {
        accepted,
        reason_code: if accepted { 0 } else { 1 },
        audit_class: 0,
        components: ComponentReasons {
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    }
}
