//! Differential anchor for the Helix `platform_decision` policy.
//!
//! For every one of the 44 spec-derived golden vectors (vectors/platform_decision/), this
//! constructs the input decision (public fields) from the vector's expected view + the reason
//! ENUM VARIANT for its reason_code, calls the REAL `PlatformDecisionView::from_bootstrap` /
//! `from_outbound_send` / `from_client_receive`, and asserts the produced 13-field view and its
//! packed projection equal the vector exactly.
//!
//! What this pins to REAL mercury-core code (a mirror error here FAILS):
//!   * `reason_code` + `reason_label` are taken from the real reason enums' `code()`/`label()`;
//!   * `source` + the per-source CONSTANT fields + the `can_decrypt -> can_receive` /
//!     `can_expose_to_ui -> can_open_message_ui` renames come from the real constructors;
//!   * the lossless `pack` bit-order matches the manifest + the Helix `mercury_pd_pack`.
//!
//! Honest scope: the per-reason values of the *passthrough* booleans are echoed from the vector
//! here; their LOGIC is independently re-encoded in helix/policy/platform_decision.hx and the
//! Python mirror (tools/check_platform_decision_vectors.py), verbatim-extracted from the private
//! reject-helper/accepted-path source cited there. The deeper evaluate_* -> decision fixtures
//! (`platform_fixture_view`) live in mercury-bindings and are exercised by that crate's tests.

use mercury_core::{
    ClientBootstrapDecision, ClientBootstrapReason, ClientReceiveDecision, ClientReceiveReason,
    OutboundSendDecision, OutboundSendReason, PlatformDecisionView,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: InputJson,
    expected: ExpectedView,
    expected_pack: i32,
}

#[derive(Debug, Deserialize)]
struct InputJson {
    source: i32,
    reason_code: i32,
    // `derived` (present in the JSON for the four derived (source,reason) pairs) is already
    // reflected in `expected`, which this test echoes into the decision, so it is not read here;
    // the Helix policy is where `derived` independently drives the projection. serde ignores it.
}

#[derive(Debug, Deserialize)]
struct ExpectedView {
    source: String,
    accepted: bool,
    reason_code: i32,
    reason_label: String,
    can_open_message_ui: bool,
    can_start_sync: bool,
    can_send: bool,
    can_receive: bool,
    can_persist_ciphertext: bool,
    requires_sync: bool,
    requires_recovery: bool,
    requires_client_retry: bool,
    requires_user_action: bool,
}

fn bootstrap_reason(code: i32) -> ClientBootstrapReason {
    use ClientBootstrapReason::*;
    match code {
        0 => Accepted,
        1 => MissingAccountId,
        2 => MissingDeviceId,
        3 => RecoveryRequired,
        4 => PlaintextCacheForbidden,
        5 => LocalDeviceTrustRejected,
        6 => KeyTransparencyNotReady,
        7 => KeyTransparencyRejected,
        8 => AccountSecretMissing,
        9 => AccountSecretCorrupt,
        10 => DeviceSecretMissing,
        11 => DeviceSecretCorrupt,
        12 => RoomStateMissing,
        13 => RoomStateCorrupt,
        14 => PlaintextSecretForbidden,
        15 => ReplayCheckpointMissing,
        16 => ReplayCheckpointStale,
        17 => ReplayGapDetected,
        18 => SyncIncomplete,
        19 => SyncGapDetected,
        20 => SyncFailed,
        _ => panic!("unknown bootstrap reason_code {code}"),
    }
}

fn outbound_reason(code: i32) -> OutboundSendReason {
    use OutboundSendReason::*;
    match code {
        0 => Accepted,
        1 => DeviceTrustRejected,
        2 => RoomMembershipTransitionRejected,
        3 => RoomMembershipEpochNotRotated,
        4 => MessagePolicyRejected,
        5 => LocalStoreSealingRejected,
        _ => panic!("unknown outbound reason_code {code}"),
    }
}

fn receive_reason(code: i32) -> ClientReceiveReason {
    use ClientReceiveReason::*;
    match code {
        0 => Accepted,
        1 => RelayDeliveryRejected,
        2 => RelayDeliveryNotDelivered,
        3 => DeliveryAckRejected,
        4 => DuplicateDeliveryAck,
        5 => SenderDeviceTrustRejected,
        6 => MessagePolicyRejected,
        7 => LocalStoreSealingRejected,
        8 => ReplayDuplicate,
        9 => ReplayStale,
        10 => OrderingGap,
        11 => BadCiphertextDigestLength,
        12 => PlaintextIdentityForbidden,
        _ => panic!("unknown receive reason_code {code}"),
    }
}

fn real_view(input: &InputJson, e: &ExpectedView) -> PlatformDecisionView {
    match input.source {
        0 => PlatformDecisionView::from_bootstrap(ClientBootstrapDecision {
            accepted: e.accepted,
            can_start_sync: e.can_start_sync,
            can_decrypt_local_store: false, // not consumed by from_bootstrap
            can_open_message_ui: e.can_open_message_ui,
            requires_sync: e.requires_sync,
            requires_recovery: e.requires_recovery,
            requires_user_action: e.requires_user_action,
            reason: bootstrap_reason(input.reason_code),
        }),
        1 => PlatformDecisionView::from_outbound_send(OutboundSendDecision {
            accepted: e.accepted,
            can_send: e.can_send,
            can_persist_ciphertext: e.can_persist_ciphertext,
            requires_user_action: e.requires_user_action,
            reason: outbound_reason(input.reason_code),
        }),
        2 => PlatformDecisionView::from_client_receive(ClientReceiveDecision {
            accepted: e.accepted,
            can_decrypt: e.can_receive, // view.can_receive <- decision.can_decrypt
            can_persist_ciphertext: e.can_persist_ciphertext,
            can_expose_to_ui: e.can_open_message_ui, // view.can_open_message_ui <- decision.can_expose_to_ui
            requires_client_retry: e.requires_client_retry,
            requires_user_action: e.requires_user_action,
            reason: receive_reason(input.reason_code),
        }),
        other => panic!("unknown source {other}"),
    }
}

/// MSB->LSB: accepted .. requires_user_action (matches manifest + Helix mercury_pd_pack).
fn pack(v: &PlatformDecisionView) -> i32 {
    let bits = [
        v.accepted,
        v.can_open_message_ui,
        v.can_start_sync,
        v.can_send,
        v.can_receive,
        v.can_persist_ciphertext,
        v.requires_sync,
        v.requires_recovery,
        v.requires_client_retry,
        v.requires_user_action,
    ];
    bits.iter().fold(0, |acc, &b| (acc << 1) | (b as i32))
}

#[test]
fn platform_decision_vectors_match_real_constructors() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/platform_decision");
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

        let got = real_view(&v.input, &v.expected);
        let e = &v.expected;
        let at = format!("vector {} ({})", v.name, path.display());

        assert_eq!(got.source, e.source.as_str(), "source {at}");
        assert_eq!(got.accepted, e.accepted, "accepted {at}");
        assert_eq!(got.reason_code, e.reason_code, "reason_code {at}");
        assert_eq!(
            got.reason_label,
            e.reason_label.as_str(),
            "reason_label {at}"
        );
        assert_eq!(
            got.can_open_message_ui, e.can_open_message_ui,
            "can_open_message_ui {at}"
        );
        assert_eq!(got.can_start_sync, e.can_start_sync, "can_start_sync {at}");
        assert_eq!(got.can_send, e.can_send, "can_send {at}");
        assert_eq!(got.can_receive, e.can_receive, "can_receive {at}");
        assert_eq!(
            got.can_persist_ciphertext, e.can_persist_ciphertext,
            "can_persist_ciphertext {at}"
        );
        assert_eq!(got.requires_sync, e.requires_sync, "requires_sync {at}");
        assert_eq!(
            got.requires_recovery, e.requires_recovery,
            "requires_recovery {at}"
        );
        assert_eq!(
            got.requires_client_retry, e.requires_client_retry,
            "requires_client_retry {at}"
        );
        assert_eq!(
            got.requires_user_action, e.requires_user_action,
            "requires_user_action {at}"
        );
        assert_eq!(pack(&got), v.expected_pack, "pack {at}");

        count += 1;
    }

    assert_eq!(
        count, 44,
        "expected all 44 platform_decision vectors to run"
    );
}
