//! Differential anchor for the Helix `outbound_decide` policy.
//!
//! For every outbound_decide golden vector this reconstructs the REAL sub-decisions from the 6
//! scalar inputs -- in particular a real `RoomMembershipSendState` from `room_state` -- calls the
//! REAL `evaluate_outbound_send`, and asserts the produced `OutboundSendDecision` (the 4 bools +
//! reason code/label) and its packed projection equal the vector. This pins the combinator's
//! priority order and rua derivation to mercury-core, and confirms the `room_state` reduction
//! (Transition accepted/epoch_rotated -> a state code) is faithful: a real Transition producing
//! each state yields the asserted reason. Exhaustive input agreement: the FU-4 exhaustive differential.

use mercury_core::{
    ComponentReasons, DeviceTrustDecision, DeviceTrustReason, LocalStoreRecordKind,
    LocalStoreSealingDecision, LocalStoreSealingReason, OutboundSendDecision, OutboundSendInput,
    PolicyDecision, RoomMembershipSendState, RoomMembershipTransitionDecision,
    RoomMembershipTransitionReason, evaluate_outbound_send,
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
    dt_can_send: i32,
    dt_requires_user_action: i32,
    room_state: i32,
    room_requires_user_action: i32,
    message_policy_accepted: i32,
    ciphertext_sealing_accepted: i32,
}

#[derive(Debug, Deserialize)]
struct ExpectedDecision {
    accepted: bool,
    can_send: bool,
    can_persist_ciphertext: bool,
    requires_user_action: bool,
    reason_code: i32,
    reason_label: String,
}

fn room_membership(state: i32, rua: bool) -> RoomMembershipSendState {
    // Reconstruct a REAL RoomMembershipSendState that produces `state` under the real combinator.
    let transition = |accepted: bool, epoch_rotated: bool| {
        RoomMembershipSendState::Transition(RoomMembershipTransitionDecision {
            accepted,
            epoch_rotated,
            requires_user_action: rua,
            reason: RoomMembershipTransitionReason::Accepted, // ignored by evaluate_outbound_send
        })
    };
    match state {
        0 => RoomMembershipSendState::Stable,
        1 => transition(true, true),  // Transition-ok
        2 => transition(false, true), // !accepted (epoch irrelevant; accepted checked first)
        3 => transition(true, false), // accepted, !epoch_rotated
        other => panic!("unknown room_state {other}"),
    }
}

fn real_decision(i: &InputJson) -> OutboundSendDecision {
    let input = OutboundSendInput {
        sender_device_trust: DeviceTrustDecision {
            trusted: false, // ignored by evaluate_outbound_send (reads can_send / requires_user_action)
            can_send: i.dt_can_send == 1,
            requires_user_action: i.dt_requires_user_action == 1,
            reason: DeviceTrustReason::Trusted,
        },
        room_membership: room_membership(i.room_state, i.room_requires_user_action == 1),
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
    };
    evaluate_outbound_send(input)
}

/// reason_code in the high bits, then accepted, can_send, can_persist_ciphertext, rua.
fn pack(d: &OutboundSendDecision) -> i32 {
    let bits = [
        d.accepted,
        d.can_send,
        d.can_persist_ciphertext,
        d.requires_user_action,
    ];
    let mut out = d.reason.code();
    for b in bits {
        out = (out << 1) | (b as i32);
    }
    out
}

#[test]
fn outbound_decide_vectors_match_real_combinator() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/outbound_decide");
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
        assert_eq!(got.can_send, e.can_send, "can_send {at}");
        assert_eq!(
            got.can_persist_ciphertext, e.can_persist_ciphertext,
            "can_persist_ciphertext {at}"
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

    assert_eq!(count, 16, "expected all 16 outbound_decide vectors to run");
}
