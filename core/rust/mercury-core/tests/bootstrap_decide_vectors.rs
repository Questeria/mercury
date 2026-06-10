//! Differential anchor for the Helix `bootstrap_decide` policy.
//!
//! For every bootstrap_decide golden vector this reconstructs the REAL ClientBootstrapInput from the
//! 12 scalar inputs (real DeviceTrustDecision / KeyTransparencyDecision / ClientBootstrapSecretState
//! x3 / ClientReplayCheckpointState / ClientSyncState, plus the account/device id lengths, the
//! pending-recovery flag, and the plaintext-cache count), calls the REAL `evaluate_client_bootstrap`,
//! and asserts the produced ClientBootstrapDecision (7 bools + reason code/label) and its packed
//! projection equal the vector. This pins the ~13-gate priority chain and the css/rsy/rrc/rua
//! derivation to mercury-core. The kt_code reduction (2 = any NotReady state) is faithful because the
//! real body treats NotChecked/MissingProof/StaleProof identically. Exhaustive input agreement: the
//! FU-4 exhaustive differential.

use mercury_core::{
    ClientBootstrapDecision, ClientBootstrapInput, ClientBootstrapSecretState,
    ClientReplayCheckpointState, ClientSyncState, DeviceTrustDecision, DeviceTrustReason,
    KeyTransparencyDecision, KeyTransparencyReason, KeyTransparencyState,
    evaluate_client_bootstrap,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: InputJson,
    expected: ExpectedDecision,
    expected_pack: i32,
}

#[derive(Debug, Deserialize)]
struct InputJson {
    account_id_ok: i32,
    device_id_ok: i32,
    pending_recovery: i32,
    plaintext_cache_ok: i32,
    local_trust: i32,
    kt_code: i32,
    kt_requires_user_action: i32,
    account_secret: i32,
    device_secret: i32,
    room_state: i32,
    replay_checkpoint: i32,
    sync_state: i32,
}

#[derive(Debug, Deserialize)]
struct ExpectedDecision {
    accepted: bool,
    can_start_sync: bool,
    can_decrypt_local_store: bool,
    can_open_message_ui: bool,
    requires_sync: bool,
    requires_recovery: bool,
    requires_user_action: bool,
    reason_code: i32,
    reason_label: String,
}

fn kt_state(code: i32) -> KeyTransparencyState {
    match code {
        0 => KeyTransparencyState::Consistent,
        1 => KeyTransparencyState::Inconsistent,
        // 2 = NotReady: NotChecked|MissingProof|StaleProof all branch to KEY_TRANSPARENCY_NOT_READY.
        2 => KeyTransparencyState::NotChecked,
        other => panic!("unknown kt_code {other}"),
    }
}

fn secret(code: i32) -> ClientBootstrapSecretState {
    match code {
        0 => ClientBootstrapSecretState::PresentSealed,
        1 => ClientBootstrapSecretState::Missing,
        2 => ClientBootstrapSecretState::Corrupt,
        3 => ClientBootstrapSecretState::PlaintextPresent,
        other => panic!("unknown secret state {other}"),
    }
}

fn replay(code: i32) -> ClientReplayCheckpointState {
    match code {
        0 => ClientReplayCheckpointState::Ready,
        1 => ClientReplayCheckpointState::Missing,
        2 => ClientReplayCheckpointState::Stale,
        3 => ClientReplayCheckpointState::GapDetected,
        other => panic!("unknown replay_checkpoint {other}"),
    }
}

fn sync(code: i32) -> ClientSyncState {
    match code {
        0 => ClientSyncState::CaughtUp,
        1 => ClientSyncState::Offline,
        2 => ClientSyncState::CatchingUp,
        3 => ClientSyncState::GapDetected,
        4 => ClientSyncState::Failed,
        other => panic!("unknown sync_state {other}"),
    }
}

fn real_decision(i: &InputJson) -> ClientBootstrapDecision {
    let input = ClientBootstrapInput {
        account_id_len: if i.account_id_ok == 1 { 32 } else { 0 },
        device_id_len: if i.device_id_ok == 1 { 32 } else { 0 },
        local_device_trust: DeviceTrustDecision {
            trusted: i.local_trust == 1,
            can_send: true,
            requires_user_action: false,
            reason: DeviceTrustReason::Trusted,
        },
        key_transparency: KeyTransparencyDecision {
            state: kt_state(i.kt_code),
            reason: KeyTransparencyReason::Consistent, // ignored by evaluate_client_bootstrap
            requires_user_action: i.kt_requires_user_action == 1,
        },
        account_secret: secret(i.account_secret),
        device_secret: secret(i.device_secret),
        room_state: secret(i.room_state),
        replay_checkpoint: replay(i.replay_checkpoint),
        sync_state: sync(i.sync_state),
        pending_recovery: i.pending_recovery == 1,
        plaintext_cache_records: if i.plaintext_cache_ok == 1 { 0 } else { 1 },
    };
    evaluate_client_bootstrap(input)
}

/// reason_code (high) then accepted, can_start_sync, can_decrypt_local_store, can_open_message_ui,
/// requires_sync, requires_recovery, requires_user_action.
fn pack(d: &ClientBootstrapDecision) -> i32 {
    let bits = [
        d.accepted,
        d.can_start_sync,
        d.can_decrypt_local_store,
        d.can_open_message_ui,
        d.requires_sync,
        d.requires_recovery,
        d.requires_user_action,
    ];
    let mut out = d.reason.code();
    for b in bits {
        out = (out << 1) | (b as i32);
    }
    out
}

#[test]
fn bootstrap_decide_vectors_match_real_combinator() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/bootstrap_decide");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    let mut count = 0usize;
    for entry in entries {
        let path = entry.expect("vector entry").path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        let v: Vector = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));

        let got = real_decision(&v.input);
        let e = &v.expected;
        let at = format!("vector {} ({})", v.name, path.display());

        assert_eq!(got.accepted, e.accepted, "accepted {at}");
        assert_eq!(got.can_start_sync, e.can_start_sync, "can_start_sync {at}");
        assert_eq!(
            got.can_decrypt_local_store, e.can_decrypt_local_store,
            "can_decrypt_local_store {at}"
        );
        assert_eq!(
            got.can_open_message_ui, e.can_open_message_ui,
            "can_open_message_ui {at}"
        );
        assert_eq!(got.requires_sync, e.requires_sync, "requires_sync {at}");
        assert_eq!(
            got.requires_recovery, e.requires_recovery,
            "requires_recovery {at}"
        );
        assert_eq!(
            got.requires_user_action, e.requires_user_action,
            "requires_user_action {at}"
        );
        assert_eq!(got.reason.code(), e.reason_code, "reason_code {at}");
        assert_eq!(
            got.reason.label(),
            e.reason_label.as_str(),
            "reason_label {at}"
        );
        assert_eq!(pack(&got), v.expected_pack, "pack {at}");

        count += 1;
    }

    assert_eq!(count, 29, "expected all 29 bootstrap_decide vectors to run");
}
