// Mercury receive_decide policy.
//
// Helix mirrors mercury-core's evaluate_client_receive(ClientReceiveInput) -> ClientReceiveDecision
// (core/rust/mercury-core/src/lib.rs:17813): the combinator that turns the inbound-receive
// sub-decisions into the ClientReceiveReason + decision fields. It is an 11-gate priority chain
// over 12 input scalars, so (the pinned Helix -O1 backend caps at 6 int params per fn) it is
// STAGED into three stage-reason functions + a first-non-zero composer + an output derivation
// (the same composition pattern as policy_pipeline), each fn <=4 params. It does NOT make the
// sub-decisions, parse anything, or do crypto -- pure scalar.
//
// Stages (verbatim priority): relay (gates 1-4) -> content (5-7) -> trust (8-10) -> accept.
// reason = first non-zero of (relay, content, trust). requires_client_retry: reasons 1/2/10 true,
// reason 3 = ack_requires_client_retry (derived), else false. requires_user_action: reasons 5/6/7
// true, reason 0 = dt_requires_user_action (derived), else false. accept => accepted/can_decrypt/
// can_persist_ciphertext/can_expose_to_ui all true; every reject => all false.
// Reason codes = ClientReceiveReason. replay_state: 0 NewInOrder / 1 Duplicate / 2 Stale / 3 FutureGap.
// core/rust/mercury-core/tests/receive_decide_vectors.rs reconstructs the REAL sub-decisions and
// pins to evaluate_client_receive.

// relay+ack gates 1-4.
@pure
fn mercury_rd_relay(
    relay_accepted: i32,
    relay_delivered: i32,
    ack_duplicate: i32,
    ack_accepted: i32,
) -> i32 {
    if relay_accepted == 0 { 1 } else {
        if relay_delivered == 0 { 2 } else {
            if ack_duplicate == 1 { 4 } else {
                if ack_accepted == 0 { 3 } else { 0 }
            }
        }
    }
}

// content gates 5-7 (digest length, plaintext-identity, replay state).
@pure
fn mercury_rd_content(digest_ok: i32, plaintext_ok: i32, replay_state: i32) -> i32 {
    if digest_ok == 0 { 11 } else {
        if plaintext_ok == 0 { 12 } else {
            if replay_state == 1 { 8 } else {
                if replay_state == 2 { 9 } else {
                    if replay_state == 3 { 10 } else { 0 }
                }
            }
        }
    }
}

// trust gates 8-10 (sender device trust, message policy, ciphertext sealing).
@pure
fn mercury_rd_trust(dt_can_send: i32, policy_accepted: i32, sealing_accepted: i32) -> i32 {
    if dt_can_send == 0 { 5 } else {
        if policy_accepted == 0 { 6 } else {
            if sealing_accepted == 0 { 7 } else { 0 }
        }
    }
}

// first non-zero of (relay, content, trust) -- the verbatim priority order.
@pure
fn mercury_rd_reason(relay_r: i32, content_r: i32, trust_r: i32) -> i32 {
    if relay_r == 0 {
        if content_r == 0 { trust_r } else { content_r }
    } else { relay_r }
}

// accepted (== can_decrypt == can_persist_ciphertext == can_expose_to_ui) = reason is ACCEPTED.
@pure
fn mercury_rd_acc(reason: i32) -> i32 {
    if reason == 0 { 1 } else { 0 }
}

// requires_client_retry: reasons 1/2/10 -> true; reason 3 -> ack_requires_client_retry; else false.
@pure
fn mercury_rd_rcr(reason: i32, ack_rcr: i32) -> i32 {
    if reason == 1 { 1 } else {
        if reason == 2 { 1 } else {
            if reason == 10 { 1 } else {
                if reason == 3 { ack_rcr } else { 0 }
            }
        }
    }
}

// requires_user_action: reasons 5/6/7 -> true; reason 0 (accept) -> dt_requires_user_action; else false.
@pure
fn mercury_rd_rua(reason: i32, dt_rua: i32) -> i32 {
    if reason == 5 { 1 } else {
        if reason == 6 { 1 } else {
            if reason == 7 { 1 } else {
                if reason == 0 { dt_rua } else { 0 }
            }
        }
    }
}

// lossless pack of the ClientReceiveDecision given the composed reason + the two derived bits:
// reason*64 + (accepted,can_decrypt,can_persist_ciphertext,can_expose_to_ui = acc, hence acc*60)
// + requires_client_retry*2 + requires_user_action. Mirrors check_receive_decide_vectors.py::pack.
@pure
fn mercury_rd_pack(reason: i32, ack_rcr: i32, dt_rua: i32) -> i32 {
    reason * 64
    + mercury_rd_acc(reason) * 60
    + mercury_rd_rcr(reason, ack_rcr) * 2
    + mercury_rd_rua(reason, dt_rua)
}

fn main() -> i32 {
    // Smoke: clean accept (reason 0) -> acc=1 -> 0*64 + 60 + 0 + 0 = 60.
    mercury_rd_pack(0, 0, 0)
}
