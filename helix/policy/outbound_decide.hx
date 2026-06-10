// Mercury outbound_decide policy.
//
// Helix mirrors mercury-core's evaluate_outbound_send(OutboundSendInput) -> OutboundSendDecision
// (core/rust/mercury-core/src/lib.rs:10841): the COMBINATOR that turns four sub-decisions into
// the outbound reason + the decision fields. This is the layer BELOW platform_decision
// (platform_decision maps a reason -> the 13-field view; this maps sub-decision inputs -> the
// reason). It does NOT make the sub-decisions, parse anything, or do crypto -- pure scalar.
//
// 6 scalar inputs (the pinned Helix -O1 backend supports <=6 int params per fn):
//   dt_can_send, dt_rua                <- sender_device_trust.{can_send, requires_user_action}
//   room_state                         <- room_membership reduced to its branch outcome:
//                                          0 Stable | 1 Transition-ok (accepted && epoch_rotated)
//                                          | 2 Transition !accepted | 3 Transition accepted !epoch
//   room_rua                           <- transition.requires_user_action (meaningful at state 1)
//   policy_accepted                    <- message_policy.accepted
//   sealing_accepted                   <- ciphertext_sealing.accepted
//
// Priority order (verbatim): device-trust -> room transition (state 2 then 3) -> message policy
// -> ciphertext sealing -> accept. Reason codes match OutboundSendReason: 0 ACCEPTED,
// 1 DEVICE_TRUST_REJECTED, 2 ROOM_MEMBERSHIP_TRANSITION_REJECTED, 3 ROOM_MEMBERSHIP_EPOCH_NOT_ROTATED,
// 4 MESSAGE_POLICY_REJECTED, 5 LOCAL_STORE_SEALING_REJECTED. The room_state encoding is a faithful
// reduction of the intra-room priority; core/rust/mercury-core/tests/outbound_decide_vectors.rs
// reconstructs a REAL RoomMembershipSendState from each state and pins to evaluate_outbound_send.

// message policy then ciphertext sealing (the shared tail after device-trust + room pass).
@pure
fn mercury_od_reason_tail(policy_accepted: i32, sealing_accepted: i32) -> i32 {
    if policy_accepted == 0 { 4 } else {
        if sealing_accepted == 0 { 5 } else { 0 }
    }
}

// the priority chain producing the OutboundSendReason code.
@pure
fn mercury_od_reason(
    dt_can_send: i32,
    room_state: i32,
    policy_accepted: i32,
    sealing_accepted: i32,
) -> i32 {
    if dt_can_send == 0 { 1 } else {
        if room_state == 2 { 2 } else {
            if room_state == 3 { 3 } else {
                mercury_od_reason_tail(policy_accepted, sealing_accepted)
            }
        }
    }
}

// accepted (== can_send == can_persist_ciphertext) = the reason is ACCEPTED.
@pure
fn mercury_od_acc(
    dt_can_send: i32,
    room_state: i32,
    policy_accepted: i32,
    sealing_accepted: i32,
) -> i32 {
    if mercury_od_reason(dt_can_send, room_state, policy_accepted, sealing_accepted) == 0 {
        1
    } else {
        0
    }
}

// requires_user_action: every reject is true; ACCEPT = device-trust rua OR (Transition-ok rua).
@pure
fn mercury_od_rua(reason: i32, dt_rua: i32, room_state: i32, room_rua: i32) -> i32 {
    if reason == 0 {
        if dt_rua == 1 { 1 } else {
            if room_state == 1 { room_rua } else { 0 }
        }
    } else { 1 }
}

// lossless pack: reason in the high bits, then accepted*8 + can_send*4 + can_persist_ciphertext*2
// (all == accepted, hence acc*14) + requires_user_action. Mirrors
// check_outbound_decide_vectors.py::pack (reason_code<<4 | acc<<3 | acc<<2 | acc<<1 | rua).
@pure
fn mercury_od_pack(
    dt_can_send: i32,
    dt_rua: i32,
    room_state: i32,
    room_rua: i32,
    policy_accepted: i32,
    sealing_accepted: i32,
) -> i32 {
    mercury_od_reason(dt_can_send, room_state, policy_accepted, sealing_accepted) * 16
    + mercury_od_acc(dt_can_send, room_state, policy_accepted, sealing_accepted) * 14
    + mercury_od_rua(
        mercury_od_reason(dt_can_send, room_state, policy_accepted, sealing_accepted),
        dt_rua, room_state, room_rua
    )
}

fn main() -> i32 {
    // Smoke: clean Stable accept, no user action -> reason 0, acc=1, rua=0 -> 0*16+14+0 = 14.
    mercury_od_pack(1, 0, 0, 0, 1, 1)
}
