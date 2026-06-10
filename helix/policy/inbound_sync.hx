// Mercury inbound_sync policy.
//
// Helix mirrors mercury-core's evaluate_inbound_sync(InboundSyncInput) -> InboundSyncDecision
// (core/rust/mercury-core/src/lib.rs:30212): the client-sync decider that turns the bootstrap
// decision + transport source state + poll/route shape into an InboundSyncReason + decision fields.
// A 9-reason priority chain whose reason is computable in EXACTLY 6 int params (the pinned Helix
// -O1 cap), so unlike the staged deciders it needs NO staging -- one reason fn + an output
// derivation. It does NOT make the sub-decisions, parse anything, or do crypto -- pure scalar.
//
// 6 reason-scalars (a faithful reduction of the real InboundSyncInput):
//   plaintext_preview_ok     <- (plaintext_notification_preview_len == 0)
//   bootstrap_can_start_sync <- bootstrap.can_start_sync   (the real ClientBootstrapDecision field)
//   source_state             <- InboundSyncSourceState: 0 Ready / 1 Offline / 2 AuthRejected / 3 BackoffRequired
//   poll_batch_ok            <- (1 <= poll_batch_limit <= 100)
//   pending_delivery         <- bool
//   route_id_ok              <- (route_id_len == 32)
// plus bootstrap_rua <- bootstrap.requires_user_action, fed into reason 2's output derivation only.
//
// Reasons (InboundSyncReason): 0 DELIVERY_READY, 1 IDLE, 2 BOOTSTRAP_BLOCKED, 3 TRANSPORT_OFFLINE,
// 4 TRANSPORT_AUTH_REJECTED, 5 BACKOFF_REQUIRED, 6 BAD_POLL_BATCH_LIMIT, 7 BAD_ROUTE_ID_LENGTH,
// 8 PLAINTEXT_NOTIFICATION_PREVIEW_FORBIDDEN. Output: accepted/can_poll_relay/can_update_replay_checkpoint
// on {0,1}; can_run_receive_session only on 0; requires_network_retry on {3,5}; requires_user_action
// reasons 8 & 4 true, reason 2 -> bootstrap_rua (derived), else false; plaintext_bytes_exposed always
// false. core/rust/mercury-core/tests/inbound_sync_vectors.rs reconstructs a REAL InboundSyncInput
// (incl a real ClientBootstrapDecision) and pins to evaluate_inbound_sync.

// the priority chain producing the InboundSyncReason code (6 params == the codegen cap).
@pure
fn mercury_is_reason(
    plaintext_preview_ok: i32,
    bootstrap_can_start_sync: i32,
    source_state: i32,
    poll_batch_ok: i32,
    pending_delivery: i32,
    route_id_ok: i32,
) -> i32 {
    if plaintext_preview_ok == 0 { 8 } else {
        if bootstrap_can_start_sync == 0 { 2 } else {
            if source_state == 1 { 3 } else {
                if source_state == 2 { 4 } else {
                    if source_state == 3 { 5 } else {
                        if poll_batch_ok == 0 { 6 } else {
                            if pending_delivery == 0 { 1 } else {
                                if route_id_ok == 0 { 7 } else { 0 }
                            }
                        }
                    }
                }
            }
        }
    }
}

// accepted (== can_poll_relay == can_update_replay_checkpoint) = reason is DeliveryReady or Idle.
@pure
fn mercury_is_acc(reason: i32) -> i32 {
    if reason == 0 { 1 } else {
        if reason == 1 { 1 } else { 0 }
    }
}

// can_run_receive_session: only DeliveryReady (there is pending delivery to receive).
@pure
fn mercury_is_crrs(reason: i32) -> i32 {
    if reason == 0 { 1 } else { 0 }
}

// requires_network_retry: TransportOffline (3) or BackoffRequired (5).
@pure
fn mercury_is_retry(reason: i32) -> i32 {
    if reason == 3 { 1 } else {
        if reason == 5 { 1 } else { 0 }
    }
}

// requires_user_action: PlaintextForbidden (8) and AuthRejected (4) -> true; BootstrapBlocked (2)
// -> the bootstrap decision's requires_user_action (derived); else false.
@pure
fn mercury_is_rua(reason: i32, bootstrap_rua: i32) -> i32 {
    if reason == 8 { 1 } else {
        if reason == 4 { 1 } else {
            if reason == 2 { bootstrap_rua } else { 0 }
        }
    }
}

// lossless pack of the InboundSyncDecision: reason in the high bits, then the 7 decision bools in
// InboundSyncDecision field order (accepted, can_poll_relay, can_run_receive_session,
// can_update_replay_checkpoint, requires_network_retry, requires_user_action, plaintext_bytes_exposed).
// plaintext_bytes_exposed is always 0. Mirrors check_inbound_sync_vectors.py::pack.
@pure
fn mercury_is_pack(reason: i32, bootstrap_rua: i32) -> i32 {
    reason * 128
    + mercury_is_acc(reason) * 64
    + mercury_is_acc(reason) * 32
    + mercury_is_crrs(reason) * 16
    + mercury_is_acc(reason) * 8
    + mercury_is_retry(reason) * 4
    + mercury_is_rua(reason, bootstrap_rua) * 2
}

fn main() -> i32 {
    // Smoke: clean DeliveryReady (reason 0) -> acc=1, crrs=1, retry=0, rua=0 -> 64+32+16+8 = 120.
    mercury_is_pack(0, 0)
}
