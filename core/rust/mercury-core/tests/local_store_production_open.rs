use mercury_core::{
    LocalStoreCrashRecoveryState, LocalStoreKeyScope, LocalStoreProductionOpenInput,
    LocalStoreProductionOpenReason, LocalStoreSealingSuite, LocalStoreUnlockDatabaseHeaderState,
    LocalStoreUnlockInput, LocalStoreUnlockReason, LocalStoreUnlockSecretState,
    MERCURY_LOCAL_STORE_VERSION,
};

#[test]
fn production_open_accepts_clean_ready_manifest() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_open_database);
    assert!(!decision.can_replay_wal);
    assert!(decision.can_load_records);
    assert!(decision.can_load_message_keys);
    assert!(!decision.requires_user_auth);
    assert!(!decision.requires_recovery);
    assert!(!decision.requires_migration);
    assert!(!decision.requires_crash_recovery);
    assert!(!decision.requires_destructive_repair);
    assert_eq!(decision.reason, LocalStoreProductionOpenReason::Accepted);
    assert_eq!(decision.reason.code(), 0);
    assert_eq!(decision.reason.label(), "ACCEPTED");
    assert_eq!(
        decision.unlock_decision.reason,
        LocalStoreUnlockReason::Accepted
    );
}

#[test]
fn production_open_propagates_unlock_rejection_flags() {
    let mut app_lock = valid_input();
    app_lock.unlock.app_lock_satisfied = false;
    let app_lock_decision = app_lock.evaluate();

    assert_rejected(
        app_lock_decision,
        LocalStoreProductionOpenReason::UnlockRejected,
    );
    assert!(app_lock_decision.requires_user_auth);
    assert!(!app_lock_decision.requires_destructive_repair);
    assert_eq!(
        app_lock_decision.unlock_decision.reason,
        LocalStoreUnlockReason::AppLockRequired
    );

    let mut plaintext_cache = valid_input();
    plaintext_cache.unlock.plaintext_cache_records = 1;
    let plaintext_cache_decision = plaintext_cache.evaluate();

    assert_rejected(
        plaintext_cache_decision,
        LocalStoreProductionOpenReason::UnlockRejected,
    );
    assert!(!plaintext_cache_decision.requires_user_auth);
    assert!(plaintext_cache_decision.requires_destructive_repair);
    assert_eq!(
        plaintext_cache_decision.unlock_decision.reason,
        LocalStoreUnlockReason::PlaintextCacheForbidden
    );
}

#[test]
fn production_open_rejects_bad_header_shape() {
    let mut bad_magic = valid_input();
    bad_magic.header_magic_matches = false;
    let bad_magic_decision = bad_magic.evaluate();
    assert_rejected(
        bad_magic_decision,
        LocalStoreProductionOpenReason::HeaderMagicMismatch,
    );
    assert!(bad_magic_decision.requires_destructive_repair);

    let mut bad_suite = valid_input();
    bad_suite.header_suite_code = 999;
    let bad_suite_decision = bad_suite.evaluate();
    assert_rejected(
        bad_suite_decision,
        LocalStoreProductionOpenReason::HeaderSuiteMismatch,
    );
    assert!(bad_suite_decision.requires_migration);

    let mut bad_nonce = valid_input();
    bad_nonce.header_nonce_len = LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len() - 1;
    let bad_nonce_decision = bad_nonce.evaluate();
    assert_rejected(
        bad_nonce_decision,
        LocalStoreProductionOpenReason::BadHeaderNonceLength,
    );
    assert!(bad_nonce_decision.requires_destructive_repair);

    let mut bad_tag = valid_input();
    bad_tag.header_tag_len =
        LocalStoreSealingSuite::MercuryLocalStoreV1.authentication_tag_len() - 1;
    let bad_tag_decision = bad_tag.evaluate();
    assert_rejected(
        bad_tag_decision,
        LocalStoreProductionOpenReason::BadHeaderTagLength,
    );
    assert!(bad_tag_decision.requires_destructive_repair);
}

#[test]
fn production_open_rejects_bad_key_manifest() {
    let mut missing_slot = valid_input();
    missing_slot.sealed_key_slots = 0;
    let missing_slot_decision = missing_slot.evaluate();
    assert_rejected(
        missing_slot_decision,
        LocalStoreProductionOpenReason::KeySlotMissing,
    );
    assert!(missing_slot_decision.requires_recovery);

    let mut plaintext_slot = valid_input();
    plaintext_slot.plaintext_key_slots = 1;
    let plaintext_slot_decision = plaintext_slot.evaluate();
    assert_rejected(
        plaintext_slot_decision,
        LocalStoreProductionOpenReason::PlaintextKeySlotForbidden,
    );
    assert!(plaintext_slot_decision.requires_destructive_repair);

    let mut bad_scope = valid_input();
    bad_scope.root_key_scope = LocalStoreKeyScope::Conversation;
    let bad_scope_decision = bad_scope.evaluate();
    assert_rejected(
        bad_scope_decision,
        LocalStoreProductionOpenReason::RootKeyScopeMismatch,
    );
    assert!(bad_scope_decision.requires_destructive_repair);

    let mut bad_generation = valid_input();
    bad_generation.root_key_generation = 0;
    let bad_generation_decision = bad_generation.evaluate();
    assert_rejected(
        bad_generation_decision,
        LocalStoreProductionOpenReason::RootKeyGenerationInvalid,
    );
    assert!(bad_generation_decision.requires_destructive_repair);
}

#[test]
fn production_open_separates_crash_recovery_states() {
    let mut replay = valid_input();
    replay.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;
    let replay_decision = replay.evaluate();
    assert_rejected(
        replay_decision,
        LocalStoreProductionOpenReason::WalReplayRequired,
    );
    assert!(replay_decision.can_replay_wal);
    assert!(replay_decision.requires_crash_recovery);
    assert!(!replay_decision.requires_destructive_repair);

    let mut dirty = valid_input();
    dirty.crash_recovery = LocalStoreCrashRecoveryState::DirtyWithoutWal;
    let dirty_decision = dirty.evaluate();
    assert_rejected(
        dirty_decision,
        LocalStoreProductionOpenReason::DirtyShutdownWithoutWal,
    );
    assert!(!dirty_decision.can_replay_wal);
    assert!(dirty_decision.requires_destructive_repair);

    let mut failed = valid_input();
    failed.crash_recovery = LocalStoreCrashRecoveryState::ReplayFailed;
    let failed_decision = failed.evaluate();
    assert_rejected(
        failed_decision,
        LocalStoreProductionOpenReason::WalReplayFailed,
    );
    assert!(!failed_decision.can_replay_wal);
    assert!(failed_decision.requires_destructive_repair);
}

#[test]
fn production_open_reasons_and_crash_states_have_stable_codes_and_labels() {
    let reasons = [
        (LocalStoreProductionOpenReason::Accepted, 0, "ACCEPTED"),
        (
            LocalStoreProductionOpenReason::UnlockRejected,
            1,
            "UNLOCK_REJECTED",
        ),
        (
            LocalStoreProductionOpenReason::HeaderMagicMismatch,
            2,
            "HEADER_MAGIC_MISMATCH",
        ),
        (
            LocalStoreProductionOpenReason::HeaderSuiteMismatch,
            3,
            "HEADER_SUITE_MISMATCH",
        ),
        (
            LocalStoreProductionOpenReason::BadHeaderNonceLength,
            4,
            "BAD_HEADER_NONCE_LENGTH",
        ),
        (
            LocalStoreProductionOpenReason::BadHeaderTagLength,
            5,
            "BAD_HEADER_TAG_LENGTH",
        ),
        (
            LocalStoreProductionOpenReason::KeySlotMissing,
            6,
            "KEY_SLOT_MISSING",
        ),
        (
            LocalStoreProductionOpenReason::PlaintextKeySlotForbidden,
            7,
            "PLAINTEXT_KEY_SLOT_FORBIDDEN",
        ),
        (
            LocalStoreProductionOpenReason::RootKeyScopeMismatch,
            8,
            "ROOT_KEY_SCOPE_MISMATCH",
        ),
        (
            LocalStoreProductionOpenReason::RootKeyGenerationInvalid,
            9,
            "ROOT_KEY_GENERATION_INVALID",
        ),
        (
            LocalStoreProductionOpenReason::WalReplayRequired,
            10,
            "WAL_REPLAY_REQUIRED",
        ),
        (
            LocalStoreProductionOpenReason::DirtyShutdownWithoutWal,
            11,
            "DIRTY_SHUTDOWN_WITHOUT_WAL",
        ),
        (
            LocalStoreProductionOpenReason::WalReplayFailed,
            12,
            "WAL_REPLAY_FAILED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let crash_states = [
        (LocalStoreCrashRecoveryState::Clean, 0, "CLEAN"),
        (
            LocalStoreCrashRecoveryState::WalReplayRequired,
            1,
            "WAL_REPLAY_REQUIRED",
        ),
        (
            LocalStoreCrashRecoveryState::DirtyWithoutWal,
            2,
            "DIRTY_WITHOUT_WAL",
        ),
        (
            LocalStoreCrashRecoveryState::ReplayFailed,
            3,
            "REPLAY_FAILED",
        ),
    ];

    for (state, code, label) in crash_states {
        assert_eq!(state.code(), code);
        assert_eq!(state.label(), label);
    }
}

fn valid_input() -> LocalStoreProductionOpenInput {
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
    decision: mercury_core::LocalStoreProductionOpenDecision,
    reason: LocalStoreProductionOpenReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_open_database);
    assert!(!decision.can_load_records);
    assert!(!decision.can_load_message_keys);
    assert_eq!(decision.reason, reason);
}
