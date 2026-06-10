use mercury_core::{
    ClientBootstrapDecision, ClientBootstrapInput, ClientBootstrapReason,
    ClientBootstrapSecretState, ClientReplayCheckpointState, ClientSyncState, DeviceTrustDecision,
    DeviceTrustReason, KeyTransparencyDecision, KeyTransparencyReason, KeyTransparencyState,
    evaluate_client_bootstrap,
};

#[test]
fn accepted_bootstrap_can_sync_decrypt_and_open_message_ui() {
    let decision = evaluate_client_bootstrap(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_start_sync);
    assert!(decision.can_decrypt_local_store);
    assert!(decision.can_open_message_ui);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_recovery);
    assert!(!decision.requires_user_action);
    assert_eq!(decision.reason, ClientBootstrapReason::Accepted);
}

#[test]
fn account_and_device_identity_must_be_present() {
    let mut missing_account = valid_input();
    missing_account.account_id_len = 0;
    assert_rejected(
        evaluate_client_bootstrap(missing_account),
        ClientBootstrapReason::MissingAccountId,
        false,
        false,
        false,
        true,
    );

    let mut missing_device = valid_input();
    missing_device.device_id_len = 0;
    assert_rejected(
        evaluate_client_bootstrap(missing_device),
        ClientBootstrapReason::MissingDeviceId,
        false,
        false,
        false,
        true,
    );
}

#[test]
fn recovery_or_plaintext_cache_blocks_startup_before_ui() {
    let mut recovery = valid_input();
    recovery.pending_recovery = true;
    assert_rejected(
        evaluate_client_bootstrap(recovery),
        ClientBootstrapReason::RecoveryRequired,
        false,
        false,
        true,
        true,
    );

    let mut plaintext_cache = valid_input();
    plaintext_cache.plaintext_cache_records = 1;
    assert_rejected(
        evaluate_client_bootstrap(plaintext_cache),
        ClientBootstrapReason::PlaintextCacheForbidden,
        false,
        false,
        false,
        true,
    );
}

#[test]
fn local_device_trust_and_key_transparency_must_be_ready() {
    let mut tofu_device = valid_input();
    tofu_device.local_device_trust = tofu_trust();
    assert_rejected(
        evaluate_client_bootstrap(tofu_device),
        ClientBootstrapReason::LocalDeviceTrustRejected,
        false,
        false,
        false,
        true,
    );

    let mut stale_key_transparency = valid_input();
    stale_key_transparency.key_transparency = key_transparency(KeyTransparencyState::StaleProof);
    assert_rejected(
        evaluate_client_bootstrap(stale_key_transparency),
        ClientBootstrapReason::KeyTransparencyNotReady,
        true,
        true,
        false,
        true,
    );

    let mut inconsistent_key_transparency = valid_input();
    inconsistent_key_transparency.key_transparency =
        key_transparency(KeyTransparencyState::Inconsistent);
    assert_rejected(
        evaluate_client_bootstrap(inconsistent_key_transparency),
        ClientBootstrapReason::KeyTransparencyRejected,
        false,
        false,
        false,
        true,
    );
}

#[test]
fn account_and_device_secrets_must_be_sealed_and_available() {
    let mut missing_account = valid_input();
    missing_account.account_secret = ClientBootstrapSecretState::Missing;
    assert_rejected(
        evaluate_client_bootstrap(missing_account),
        ClientBootstrapReason::AccountSecretMissing,
        false,
        false,
        true,
        true,
    );

    let mut corrupt_device = valid_input();
    corrupt_device.device_secret = ClientBootstrapSecretState::Corrupt;
    assert_rejected(
        evaluate_client_bootstrap(corrupt_device),
        ClientBootstrapReason::DeviceSecretCorrupt,
        false,
        false,
        true,
        true,
    );

    let mut plaintext_secret = valid_input();
    plaintext_secret.device_secret = ClientBootstrapSecretState::PlaintextPresent;
    assert_rejected(
        evaluate_client_bootstrap(plaintext_secret),
        ClientBootstrapReason::PlaintextSecretForbidden,
        false,
        false,
        false,
        true,
    );
}

#[test]
fn room_state_and_replay_checkpoint_can_require_sync_without_ui() {
    let mut missing_room = valid_input();
    missing_room.room_state = ClientBootstrapSecretState::Missing;
    assert_rejected(
        evaluate_client_bootstrap(missing_room),
        ClientBootstrapReason::RoomStateMissing,
        true,
        true,
        false,
        false,
    );

    let mut stale_replay = valid_input();
    stale_replay.replay_checkpoint = ClientReplayCheckpointState::Stale;
    assert_rejected(
        evaluate_client_bootstrap(stale_replay),
        ClientBootstrapReason::ReplayCheckpointStale,
        true,
        true,
        false,
        false,
    );

    let mut replay_gap = valid_input();
    replay_gap.replay_checkpoint = ClientReplayCheckpointState::GapDetected;
    assert_rejected(
        evaluate_client_bootstrap(replay_gap),
        ClientBootstrapReason::ReplayGapDetected,
        true,
        true,
        false,
        false,
    );
}

#[test]
fn sync_must_be_caught_up_before_message_ui_opens() {
    let mut catching_up = valid_input();
    catching_up.sync_state = ClientSyncState::CatchingUp;
    assert_rejected(
        catching_up.evaluate(),
        ClientBootstrapReason::SyncIncomplete,
        true,
        true,
        false,
        false,
    );

    let mut sync_gap = valid_input();
    sync_gap.sync_state = ClientSyncState::GapDetected;
    assert_rejected(
        sync_gap.evaluate(),
        ClientBootstrapReason::SyncGapDetected,
        true,
        true,
        false,
        false,
    );

    let mut failed = valid_input();
    failed.sync_state = ClientSyncState::Failed;
    assert_rejected(
        failed.evaluate(),
        ClientBootstrapReason::SyncFailed,
        true,
        true,
        false,
        true,
    );
}

fn valid_input() -> ClientBootstrapInput {
    ClientBootstrapInput {
        account_id_len: 32,
        device_id_len: 32,
        local_device_trust: full_trust(),
        key_transparency: key_transparency(KeyTransparencyState::Consistent),
        account_secret: ClientBootstrapSecretState::PresentSealed,
        device_secret: ClientBootstrapSecretState::PresentSealed,
        room_state: ClientBootstrapSecretState::PresentSealed,
        replay_checkpoint: ClientReplayCheckpointState::Ready,
        sync_state: ClientSyncState::CaughtUp,
        pending_recovery: false,
        plaintext_cache_records: 0,
    }
}

fn full_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: true,
        can_send: true,
        requires_user_action: false,
        reason: DeviceTrustReason::Trusted,
    }
}

fn tofu_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: false,
        can_send: true,
        requires_user_action: true,
        reason: DeviceTrustReason::TrustOnFirstUse,
    }
}

fn key_transparency(state: KeyTransparencyState) -> KeyTransparencyDecision {
    KeyTransparencyDecision {
        state,
        reason: if state == KeyTransparencyState::Consistent {
            KeyTransparencyReason::Consistent
        } else {
            KeyTransparencyReason::StaleProof
        },
        requires_user_action: state != KeyTransparencyState::Consistent,
    }
}

fn assert_rejected(
    decision: ClientBootstrapDecision,
    reason: ClientBootstrapReason,
    can_start_sync: bool,
    requires_sync: bool,
    requires_recovery: bool,
    requires_user_action: bool,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.can_start_sync, can_start_sync);
    assert!(!decision.can_decrypt_local_store);
    assert!(!decision.can_open_message_ui);
    assert_eq!(decision.requires_sync, requires_sync);
    assert_eq!(decision.requires_recovery, requires_recovery);
    assert_eq!(decision.requires_user_action, requires_user_action);
    assert_eq!(decision.reason, reason);
}
