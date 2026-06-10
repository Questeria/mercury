//! Differential anchor for the Helix `inbound_sync` policy.
//!
//! For every inbound_sync golden vector this reconstructs the REAL InboundSyncInput from the 7
//! scalar inputs (in particular a real ClientBootstrapDecision carrying can_start_sync +
//! requires_user_action, a real InboundSyncSourceState, and the length/limit fields), calls the REAL
//! `evaluate_inbound_sync`, and asserts the produced InboundSyncDecision (7 bools + reason
//! code/label) and its packed projection equal the vector. This pins the 9-reason priority chain and
//! the bootstrap/source-state reductions to mercury-core. Exhaustive input agreement: the FU-4
//! exhaustive differential.

use mercury_core::{
    ClientBootstrapDecision, ClientBootstrapReason, InboundSyncDecision, InboundSyncInput,
    InboundSyncSourceState, evaluate_inbound_sync,
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
    plaintext_preview_ok: i32,
    bootstrap_can_start_sync: i32,
    bootstrap_rua: i32,
    source_state: i32,
    poll_batch_ok: i32,
    pending_delivery: i32,
    route_id_ok: i32,
}

#[derive(Debug, Deserialize)]
struct ExpectedDecision {
    accepted: bool,
    can_poll_relay: bool,
    can_run_receive_session: bool,
    can_update_replay_checkpoint: bool,
    requires_network_retry: bool,
    requires_user_action: bool,
    plaintext_bytes_exposed: bool,
    reason_code: i32,
    reason_label: String,
}

fn source(state: i32) -> InboundSyncSourceState {
    match state {
        0 => InboundSyncSourceState::Ready,
        1 => InboundSyncSourceState::Offline,
        2 => InboundSyncSourceState::AuthRejected,
        3 => InboundSyncSourceState::BackoffRequired,
        other => panic!("unknown source_state {other}"),
    }
}

fn real_decision(i: &InputJson) -> InboundSyncDecision {
    // evaluate_inbound_sync reads only bootstrap.can_start_sync + bootstrap.requires_user_action.
    let bootstrap = ClientBootstrapDecision {
        accepted: false,
        can_start_sync: i.bootstrap_can_start_sync == 1,
        can_decrypt_local_store: false,
        can_open_message_ui: false,
        requires_sync: false,
        requires_recovery: false,
        requires_user_action: i.bootstrap_rua == 1,
        reason: ClientBootstrapReason::Accepted,
    };
    let input = InboundSyncInput {
        bootstrap,
        source_state: source(i.source_state),
        pending_delivery: i.pending_delivery == 1,
        route_id_len: if i.route_id_ok == 1 { 32 } else { 16 },
        poll_batch_limit: if i.poll_batch_ok == 1 { 50 } else { 0 },
        plaintext_notification_preview_len: if i.plaintext_preview_ok == 1 { 0 } else { 1 },
    };
    evaluate_inbound_sync(input)
}

/// reason_code (high) then accepted, can_poll_relay, can_run_receive_session,
/// can_update_replay_checkpoint, requires_network_retry, requires_user_action, plaintext_bytes_exposed.
fn pack(d: &InboundSyncDecision) -> i32 {
    let bits = [
        d.accepted,
        d.can_poll_relay,
        d.can_run_receive_session,
        d.can_update_replay_checkpoint,
        d.requires_network_retry,
        d.requires_user_action,
        d.plaintext_bytes_exposed,
    ];
    let mut out = d.reason.code();
    for b in bits {
        out = (out << 1) | (b as i32);
    }
    out
}

#[test]
fn inbound_sync_vectors_match_real_combinator() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/inbound_sync");
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
        assert_eq!(got.can_poll_relay, e.can_poll_relay, "can_poll_relay {at}");
        assert_eq!(
            got.can_run_receive_session, e.can_run_receive_session,
            "can_run_receive_session {at}"
        );
        assert_eq!(
            got.can_update_replay_checkpoint, e.can_update_replay_checkpoint,
            "can_update_replay_checkpoint {at}"
        );
        assert_eq!(
            got.requires_network_retry, e.requires_network_retry,
            "requires_network_retry {at}"
        );
        assert_eq!(
            got.requires_user_action, e.requires_user_action,
            "requires_user_action {at}"
        );
        assert_eq!(
            got.plaintext_bytes_exposed, e.plaintext_bytes_exposed,
            "plaintext_bytes_exposed {at}"
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

    assert_eq!(count, 15, "expected all 15 inbound_sync vectors to run");
}
