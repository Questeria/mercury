//! Differential anchor for the Helix `receive_decide` policy.
//!
//! For every receive_decide golden vector this reconstructs the REAL sub-decisions from the 12
//! scalar inputs (a real RelayQueueDecision/next_state, DeliveryAckDecision, ClientReceiveReplayState,
//! DeviceTrustDecision, PolicyDecision, LocalStoreSealingDecision, plus the digest-length and
//! plaintext-identity scalars), calls the REAL `evaluate_client_receive`, and asserts the produced
//! ClientReceiveDecision (6 bools + reason code/label) and its packed projection equal the vector.
//! This pins the 11-gate priority chain and the rcr/rua derivation to mercury-core. Exhaustive
//! input agreement: the FU-4 exhaustive differential.

use mercury_core::ClientReceiveDecision;
use mercury_core::{
    ClientReceiveInput, ClientReceiveReplayState, ComponentReasons, DeliveryAckDecision,
    DeliveryAckReason, DeviceTrustDecision, DeviceTrustReason, LocalStoreRecordKind,
    LocalStoreSealingDecision, LocalStoreSealingReason, PolicyDecision, RelayQueueDecision,
    RelayQueueItemState, RelayQueueReason, evaluate_client_receive,
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
    relay_accepted: i32,
    relay_delivered: i32,
    ack_duplicate: i32,
    ack_accepted: i32,
    ack_requires_client_retry: i32,
    ciphertext_digest_ok: i32,
    plaintext_identity_ok: i32,
    replay_state: i32,
    dt_can_send: i32,
    message_policy_accepted: i32,
    ciphertext_sealing_accepted: i32,
    dt_requires_user_action: i32,
}

#[derive(Debug, Deserialize)]
struct ExpectedDecision {
    accepted: bool,
    can_decrypt: bool,
    can_persist_ciphertext: bool,
    can_expose_to_ui: bool,
    requires_client_retry: bool,
    requires_user_action: bool,
    reason_code: i32,
    reason_label: String,
}

fn replay(state: i32) -> ClientReceiveReplayState {
    match state {
        0 => ClientReceiveReplayState::NewInOrder,
        1 => ClientReceiveReplayState::Duplicate,
        2 => ClientReceiveReplayState::Stale,
        3 => ClientReceiveReplayState::FutureGap,
        other => panic!("unknown replay_state {other}"),
    }
}

fn real_decision(i: &InputJson) -> ClientReceiveDecision {
    let input = ClientReceiveInput {
        relay_delivery: RelayQueueDecision {
            accepted: i.relay_accepted == 1,
            next_state: if i.relay_delivered == 1 {
                RelayQueueItemState::Delivered
            } else {
                RelayQueueItemState::Pending // any non-Delivered triggers the gate
            },
            persist_item: false,
            delete_item: false,
            reason: RelayQueueReason::Accepted, // ignored by evaluate_client_receive
        },
        delivery_ack: DeliveryAckDecision {
            accepted: i.ack_accepted == 1,
            duplicate: i.ack_duplicate == 1,
            retain_hash_audit: false,
            delete_queue_record: false,
            requires_client_retry: i.ack_requires_client_retry == 1,
            reason: DeliveryAckReason::Accepted,
        },
        sender_device_trust: DeviceTrustDecision {
            trusted: false,
            can_send: i.dt_can_send == 1,
            requires_user_action: i.dt_requires_user_action == 1,
            reason: DeviceTrustReason::Trusted,
        },
        message_policy: PolicyDecision {
            accepted: i.message_policy_accepted == 1,
            reason_code: 0,
            audit_class: 0,
            components: ComponentReasons {
                envelope_reason: 0,
                room_epoch_reason: 0,
                ai_grant_reason: 0,
                ai_lifecycle_reason: 0,
            },
        },
        ciphertext_sealing: LocalStoreSealingDecision {
            accepted: i.ciphertext_sealing_accepted == 1,
            reason: LocalStoreSealingReason::Accepted,
            record_policy: LocalStoreRecordKind::MessageCiphertext.policy(),
        },
        replay_state: replay(i.replay_state),
        ciphertext_digest_len: if i.ciphertext_digest_ok == 1 { 32 } else { 31 },
        plaintext_identity_fields: if i.plaintext_identity_ok == 1 { 0 } else { 1 },
    };
    evaluate_client_receive(input)
}

/// reason_code (high) then accepted, can_decrypt, can_persist_ciphertext, can_expose_to_ui, rcr, rua.
fn pack(d: &ClientReceiveDecision) -> i32 {
    let bits = [
        d.accepted,
        d.can_decrypt,
        d.can_persist_ciphertext,
        d.can_expose_to_ui,
        d.requires_client_retry,
        d.requires_user_action,
    ];
    let mut out = d.reason.code();
    for b in bits {
        out = (out << 1) | (b as i32);
    }
    out
}

#[test]
fn receive_decide_vectors_match_real_combinator() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/receive_decide");
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
        assert_eq!(got.can_decrypt, e.can_decrypt, "can_decrypt {at}");
        assert_eq!(
            got.can_persist_ciphertext, e.can_persist_ciphertext,
            "can_persist_ciphertext {at}"
        );
        assert_eq!(
            got.can_expose_to_ui, e.can_expose_to_ui,
            "can_expose_to_ui {at}"
        );
        assert_eq!(
            got.requires_client_retry, e.requires_client_retry,
            "requires_client_retry {at}"
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

    assert_eq!(count, 19, "expected all 19 receive_decide vectors to run");
}
