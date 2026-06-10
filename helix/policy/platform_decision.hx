// Mercury platform_decision policy.
//
// Helix mirrors mercury-core's PlatformDecisionView capability projection: for a
// decision mercury-core ALREADY made (source + reason_code), what the 13-field
// platform view exposes. It does NOT make the decision, parse JSON, inspect
// ciphertext, verify signatures, or implement cryptography -- pure scalar projection.
//
// source: 0 = client_bootstrap, 1 = outbound_send, 2 = client_receive.
// reason_code: the per-source reason enum discriminant (bootstrap 0..20, outbound
//   0..5, receive 0..12); 0 == ACCEPTED for every source.
// derived: the single bit copied from a deeper decision input for the four
//   (source,reason) pairs that are not constant (else ignored):
//     (0,6) KEY_TRANSPARENCY_NOT_READY -> requires_user_action
//     (1,0) ACCEPTED                   -> requires_user_action
//     (2,0) ACCEPTED                   -> requires_user_action
//     (2,3) DELIVERY_ACK_REJECTED      -> requires_client_retry
//
// Source of truth: core/rust/mercury-core/src/lib.rs (PlatformDecisionView 30292-30307,
// from_* 30309-30362, reason enums + reject/accept helpers cited in
// tools/check_platform_decision_vectors.py). Set membership uses nested if/else and each
// @pure fn takes <=6 scalar args, matching the existing Mercury policy style (e.g. envelope.hx)
// and keeping every fn explicitly total.

// ---- bootstrap set-membership helpers (reason_code subsets) ----

// can_start_sync rejects + replay/room/sync states: 6,12,13,15,16,17,18,19,20.
// (== the requires_sync set; can_start_sync adds the accepted case separately.)
@pure
fn mercury_pd_bs_sync_capable(rc: i32) -> i32 {
    if rc == 6 { 1 } else {
    if rc == 12 { 1 } else {
    if rc == 13 { 1 } else {
    if rc == 15 { 1 } else {
    if rc == 16 { 1 } else {
    if rc == 17 { 1 } else {
    if rc == 18 { 1 } else {
    if rc == 19 { 1 } else {
    if rc == 20 { 1 } else { 0 } } } } } } } } }
}

// requires_recovery set: RECOVERY_REQUIRED(3), ACCOUNT/DEVICE secret missing/corrupt(8,9,10,11).
@pure
fn mercury_pd_bs_recovery(rc: i32) -> i32 {
    if rc == 3 { 1 } else {
    if rc == 8 { 1 } else {
    if rc == 9 { 1 } else {
    if rc == 10 { 1 } else {
    if rc == 11 { 1 } else { 0 } } } } }
}

// bootstrap requires_user_action == FALSE set (excluding the derived KT_NOT_READY=6):
// ACCEPTED(0) + passive room/replay/sync states 12,15,16,17,18,19.
@pure
fn mercury_pd_bs_no_user_action(rc: i32) -> i32 {
    if rc == 0 { 1 } else {
    if rc == 12 { 1 } else {
    if rc == 15 { 1 } else {
    if rc == 16 { 1 } else {
    if rc == 17 { 1 } else {
    if rc == 18 { 1 } else {
    if rc == 19 { 1 } else { 0 } } } } } } }
}

// ---- the 10 boolean view fields (1 = true, 0 = false) ----

@pure
fn mercury_pd_accepted(rc: i32) -> i32 {
    if rc == 0 { 1 } else { 0 }
}

// can_open_message_ui: accepted bootstrap or accepted receive (NOT outbound).
@pure
fn mercury_pd_can_open_message_ui(source: i32, rc: i32) -> i32 {
    if rc == 0 {
        if source == 1 { 0 } else { 1 }
    } else { 0 }
}

// can_start_sync: bootstrap only; accepted or a sync-capable reject.
@pure
fn mercury_pd_can_start_sync(source: i32, rc: i32) -> i32 {
    if source == 0 {
        if rc == 0 { 1 } else { mercury_pd_bs_sync_capable(rc) }
    } else { 0 }
}

// can_send: outbound ACCEPTED only.
@pure
fn mercury_pd_can_send(source: i32, rc: i32) -> i32 {
    if source == 1 {
        if rc == 0 { 1 } else { 0 }
    } else { 0 }
}

// can_receive: receive ACCEPTED only (decision.can_decrypt).
@pure
fn mercury_pd_can_receive(source: i32, rc: i32) -> i32 {
    if source == 2 {
        if rc == 0 { 1 } else { 0 }
    } else { 0 }
}

// can_persist_ciphertext: outbound or receive ACCEPTED (NOT bootstrap).
@pure
fn mercury_pd_can_persist_ciphertext(source: i32, rc: i32) -> i32 {
    if rc == 0 {
        if source == 0 { 0 } else { 1 }
    } else { 0 }
}

// requires_sync: bootstrap sync-capable rejects.
@pure
fn mercury_pd_requires_sync(source: i32, rc: i32) -> i32 {
    if source == 0 { mercury_pd_bs_sync_capable(rc) } else { 0 }
}

// requires_recovery: bootstrap recovery set.
@pure
fn mercury_pd_requires_recovery(source: i32, rc: i32) -> i32 {
    if source == 0 { mercury_pd_bs_recovery(rc) } else { 0 }
}

// requires_client_retry: receive RELAY_DELIVERY_REJECTED(1)/NOT_DELIVERED(2)/ORDERING_GAP(10),
// and DELIVERY_ACK_REJECTED(3) is derived.
@pure
fn mercury_pd_requires_client_retry(source: i32, rc: i32, derived: i32) -> i32 {
    if source == 2 {
        if rc == 1 { 1 } else {
        if rc == 2 { 1 } else {
        if rc == 10 { 1 } else {
        if rc == 3 { derived } else { 0 } } } }
    } else { 0 }
}

// requires_user_action: per-source. bootstrap = not-in-no-action-set (KT_NOT_READY=6 derived);
// outbound = every reject true, ACCEPTED derived; receive = SENDER_DEVICE_TRUST(5)/
// MESSAGE_POLICY(6)/LOCAL_STORE(7) true, ACCEPTED derived, else false.
@pure
fn mercury_pd_requires_user_action(source: i32, rc: i32, derived: i32) -> i32 {
    if source == 0 {
        if rc == 6 { derived } else {
            if mercury_pd_bs_no_user_action(rc) == 1 { 0 } else { 1 }
        }
    } else {
        if source == 1 {
            if rc == 0 { derived } else { 1 }
        } else {
            if rc == 0 { derived } else {
            if rc == 5 { 1 } else {
            if rc == 6 { 1 } else {
            if rc == 7 { 1 } else { 0 } } } }
        }
    }
}

// ---- lossless packed projection: 10 booleans MSB->LSB (accepted=bit9 .. RUA=bit0) ----
@pure
fn mercury_pd_pack(source: i32, rc: i32, derived: i32) -> i32 {
    mercury_pd_accepted(rc) * 512
    + mercury_pd_can_open_message_ui(source, rc) * 256
    + mercury_pd_can_start_sync(source, rc) * 128
    + mercury_pd_can_send(source, rc) * 64
    + mercury_pd_can_receive(source, rc) * 32
    + mercury_pd_can_persist_ciphertext(source, rc) * 16
    + mercury_pd_requires_sync(source, rc) * 8
    + mercury_pd_requires_recovery(source, rc) * 4
    + mercury_pd_requires_client_retry(source, rc, derived) * 2
    + mercury_pd_requires_user_action(source, rc, derived) * 1
}

fn main() -> i32 {
    // Smoke: outbound ACCEPTED with no user action -> can_send|can_persist|accepted.
    // accepted(512) + can_send(64) + can_persist(16) = 592.
    mercury_pd_pack(1, 0, 0)
}
