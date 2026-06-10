// Mercury bootstrap_decide policy.
//
// Helix mirrors mercury-core's evaluate_client_bootstrap(ClientBootstrapInput) ->
// ClientBootstrapDecision (core/rust/mercury-core/src/lib.rs:21528): the combinator that turns the
// client-bootstrap gate inputs into the ClientBootstrapReason + decision fields. It is the richest
// decider (21 reasons), a ~13-gate priority chain over 12 input scalars, so (the pinned Helix -O1
// backend caps at 6 int params per fn) it is STAGED into four stage-reason functions + a
// first-non-zero composer + an output derivation (the same composition pattern as policy_pipeline /
// receive_decide), each fn <=5 params. It does NOT parse anything or do crypto -- pure scalar.
//
// Stages (verbatim priority): pre (gates 1-5: account_id/device_id/pending_recovery/plaintext_cache/
// local_trust) -> kt (key_transparency, gates 6-7) -> secrets (account/device/room sealed-secret
// state, gates 8-14) -> rs (replay_checkpoint then sync_state, gates 15-20) -> accept.
// reason = first non-zero of (pre, kt, secrets, rs). The richer state enums are reduced to small
// codes the real combinator branches identically on:
//   kt_code      0 Consistent (pass) / 1 Inconsistent (->7) / 2 NotReady (->6) [= NotChecked|
//                MissingProof|StaleProof, which the real body treats alike].
//   secret state 0 PresentSealed / 1 Missing / 2 Corrupt / 3 PlaintextPresent (per account/device/room).
//   replay       0 Ready / 1 Missing / 2 Stale / 3 GapDetected.
//   sync         0 CaughtUp / 1 Offline / 2 CatchingUp / 3 GapDetected / 4 Failed.
// Output ClientBootstrapDecision (7 bools): accepted = can_decrypt_local_store = can_open_message_ui
// = (reason==0); can_start_sync / requires_sync / requires_recovery per reason; requires_user_action
// per reason with reason 6 (KEY_TRANSPARENCY_NOT_READY) -> key_transparency.requires_user_action
// (derived). Per-reason output bits are IDENTICAL to the platform_decision bootstrap table.
// core/rust/mercury-core/tests/bootstrap_decide_vectors.rs reconstructs the REAL ClientBootstrapInput
// and pins to evaluate_client_bootstrap.

// pre gates 1-5 (identity, recovery, plaintext-cache, local device trust).
@pure
fn mercury_bd_pre(
    account_id_ok: i32,
    device_id_ok: i32,
    pending_recovery: i32,
    plaintext_cache_ok: i32,
    local_trust: i32,
) -> i32 {
    if account_id_ok == 0 { 1 } else {
        if device_id_ok == 0 { 2 } else {
            if pending_recovery == 1 { 3 } else {
                if plaintext_cache_ok == 0 { 4 } else {
                    if local_trust == 0 { 5 } else { 0 }
                }
            }
        }
    }
}

// key-transparency gate 6/7 (kt_code reduction of KeyTransparencyState).
@pure
fn mercury_bd_kt(kt_code: i32) -> i32 {
    if kt_code == 1 { 7 } else {
        if kt_code == 2 { 6 } else { 0 }
    }
}

// secret gates 8-14: first-failing of account {8,9,14} -> device {10,11,14} -> room {12,13,14},
// where each secret state 1 Missing / 2 Corrupt / 3 PlaintextPresent(->14 shared).
@pure
fn mercury_bd_secrets(account_secret: i32, device_secret: i32, room_state: i32) -> i32 {
    if account_secret == 1 { 8 } else {
        if account_secret == 2 { 9 } else {
            if account_secret == 3 { 14 } else {
                if device_secret == 1 { 10 } else {
                    if device_secret == 2 { 11 } else {
                        if device_secret == 3 { 14 } else {
                            if room_state == 1 { 12 } else {
                                if room_state == 2 { 13 } else {
                                    if room_state == 3 { 14 } else { 0 }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// replay/sync gates 15-20: replay first (1/2/3 -> 15/16/17), then sync (1 Offline & 2 CatchingUp ->
// 18 SyncIncomplete, 3 -> 19 SyncGapDetected, 4 Failed -> 20).
@pure
fn mercury_bd_rs(replay_checkpoint: i32, sync_state: i32) -> i32 {
    if replay_checkpoint == 1 { 15 } else {
        if replay_checkpoint == 2 { 16 } else {
            if replay_checkpoint == 3 { 17 } else {
                if sync_state == 1 { 18 } else {
                    if sync_state == 2 { 18 } else {
                        if sync_state == 3 { 19 } else {
                            if sync_state == 4 { 20 } else { 0 }
                        }
                    }
                }
            }
        }
    }
}

// first non-zero of (pre, kt, secrets, rs) -- the verbatim priority order.
@pure
fn mercury_bd_reason(pre_r: i32, kt_r: i32, sec_r: i32, rs_r: i32) -> i32 {
    if pre_r == 0 {
        if kt_r == 0 {
            if sec_r == 0 { rs_r } else { sec_r }
        } else { kt_r }
    } else { pre_r }
}

// accepted (== can_decrypt_local_store == can_open_message_ui) = reason is ACCEPTED.
@pure
fn mercury_bd_acc(reason: i32) -> i32 {
    if reason == 0 { 1 } else { 0 }
}

// can_start_sync: accept (0) or any sync-capable reject {6,12,13,15,16,17,18,19,20}.
@pure
fn mercury_bd_css(reason: i32) -> i32 {
    if reason == 0 { 1 } else {
        if reason == 6 { 1 } else {
            if reason == 12 { 1 } else {
                if reason == 13 { 1 } else {
                    if reason == 15 { 1 } else {
                        if reason == 16 { 1 } else {
                            if reason == 17 { 1 } else {
                                if reason == 18 { 1 } else {
                                    if reason == 19 { 1 } else {
                                        if reason == 20 { 1 } else { 0 }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// requires_sync: the sync-capable rejects {6,12,13,15,16,17,18,19,20} (can_start_sync without accept).
@pure
fn mercury_bd_rsy(reason: i32) -> i32 {
    if reason == 6 { 1 } else {
        if reason == 12 { 1 } else {
            if reason == 13 { 1 } else {
                if reason == 15 { 1 } else {
                    if reason == 16 { 1 } else {
                        if reason == 17 { 1 } else {
                            if reason == 18 { 1 } else {
                                if reason == 19 { 1 } else {
                                    if reason == 20 { 1 } else { 0 }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// requires_recovery: {3,8,9,10,11} (recovery-required + account/device secret missing/corrupt).
@pure
fn mercury_bd_rrc(reason: i32) -> i32 {
    if reason == 3 { 1 } else {
        if reason == 8 { 1 } else {
            if reason == 9 { 1 } else {
                if reason == 10 { 1 } else {
                    if reason == 11 { 1 } else { 0 }
                }
            }
        }
    }
}

// requires_user_action: reason 6 -> key_transparency.requires_user_action (derived); the no-action
// set {0,12,15,16,17,18,19} -> 0; every other reason -> 1.
@pure
fn mercury_bd_rua(reason: i32, kt_rua: i32) -> i32 {
    if reason == 6 { kt_rua } else {
        if reason == 0 { 0 } else {
            if reason == 12 { 0 } else {
                if reason == 15 { 0 } else {
                    if reason == 16 { 0 } else {
                        if reason == 17 { 0 } else {
                            if reason == 18 { 0 } else {
                                if reason == 19 { 0 } else { 1 }
                            }
                        }
                    }
                }
            }
        }
    }
}

// lossless pack of the ClientBootstrapDecision given the composed reason + the derived kt rua:
// reason*128 + accepted*64 + can_start_sync*32 + can_decrypt_local_store*16 + can_open_message_ui*8
// + requires_sync*4 + requires_recovery*2 + requires_user_action, with accepted = can_decrypt_local_store
// = can_open_message_ui = acc. Mirrors check_bootstrap_decide_vectors.py::pack.
@pure
fn mercury_bd_pack(reason: i32, kt_rua: i32) -> i32 {
    reason * 128
    + mercury_bd_acc(reason) * 64
    + mercury_bd_css(reason) * 32
    + mercury_bd_acc(reason) * 16
    + mercury_bd_acc(reason) * 8
    + mercury_bd_rsy(reason) * 4
    + mercury_bd_rrc(reason) * 2
    + mercury_bd_rua(reason, kt_rua)
}

@pure
fn expect(actual: i32, expected: i32) -> i32 {
    if actual == expected { 0 } else { 1 }
}

fn main() -> i32 {
    let mut failed: i32 = 0;

    // GENERATED from vectors/bootstrap_decide/*.json by
    // tools/gen_bootstrap_decide_vectors.py --write. Do not edit by hand.

    // 01_missing_account_id__1
    failed = failed + expect(mercury_bd_pre(0, 1, 0, 1, 1), 1);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(1, 0, 0, 0), 1);
    failed = failed + expect(mercury_bd_pack(1, 0), 129);

    // 02_missing_device_id__1
    failed = failed + expect(mercury_bd_pre(1, 0, 0, 1, 1), 2);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(2, 0, 0, 0), 2);
    failed = failed + expect(mercury_bd_pack(2, 0), 257);

    // 03_recovery_required__1
    failed = failed + expect(mercury_bd_pre(1, 1, 1, 1, 1), 3);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(3, 0, 0, 0), 3);
    failed = failed + expect(mercury_bd_pack(3, 0), 387);

    // 04_plaintext_cache_forbidden__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 0, 1), 4);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(4, 0, 0, 0), 4);
    failed = failed + expect(mercury_bd_pack(4, 0), 513);

    // 05_local_device_trust_rejected__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 0), 5);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(5, 0, 0, 0), 5);
    failed = failed + expect(mercury_bd_pack(5, 0), 641);

    // 06_key_transparency_not_ready__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(2), 6);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 6, 0, 0), 6);
    failed = failed + expect(mercury_bd_pack(6, 0), 804);

    // 06_key_transparency_not_ready__2
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(2), 6);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 6, 0, 0), 6);
    failed = failed + expect(mercury_bd_pack(6, 1), 805);

    // 07_key_transparency_rejected__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(1), 7);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 7, 0, 0), 7);
    failed = failed + expect(mercury_bd_pack(7, 0), 897);

    // 08_account_secret_missing__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(1, 0, 0), 8);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 8, 0), 8);
    failed = failed + expect(mercury_bd_pack(8, 0), 1027);

    // 09_account_secret_corrupt__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(2, 0, 0), 9);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 9, 0), 9);
    failed = failed + expect(mercury_bd_pack(9, 0), 1155);

    // 10_device_secret_missing__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 1, 0), 10);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 10, 0), 10);
    failed = failed + expect(mercury_bd_pack(10, 0), 1283);

    // 11_device_secret_corrupt__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 2, 0), 11);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 11, 0), 11);
    failed = failed + expect(mercury_bd_pack(11, 0), 1411);

    // 12_room_state_missing__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 1), 12);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 12, 0), 12);
    failed = failed + expect(mercury_bd_pack(12, 0), 1572);

    // 13_room_state_corrupt__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 2), 13);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 13, 0), 13);
    failed = failed + expect(mercury_bd_pack(13, 0), 1701);

    // 14_plaintext_secret_forbidden__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(3, 0, 0), 14);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 14, 0), 14);
    failed = failed + expect(mercury_bd_pack(14, 0), 1793);

    // 14_plaintext_secret_forbidden__2
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 3, 0), 14);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 14, 0), 14);
    failed = failed + expect(mercury_bd_pack(14, 0), 1793);

    // 14_plaintext_secret_forbidden__3
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 3), 14);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 14, 0), 14);
    failed = failed + expect(mercury_bd_pack(14, 0), 1793);

    // 15_replay_checkpoint_missing__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(1, 0), 15);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 15), 15);
    failed = failed + expect(mercury_bd_pack(15, 0), 1956);

    // 16_replay_checkpoint_stale__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(2, 0), 16);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 16), 16);
    failed = failed + expect(mercury_bd_pack(16, 0), 2084);

    // 17_replay_gap_detected__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(3, 0), 17);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 17), 17);
    failed = failed + expect(mercury_bd_pack(17, 0), 2212);

    // 18_sync_incomplete__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 1), 18);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 18), 18);
    failed = failed + expect(mercury_bd_pack(18, 0), 2340);

    // 18_sync_incomplete__2
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 2), 18);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 18), 18);
    failed = failed + expect(mercury_bd_pack(18, 0), 2340);

    // 19_sync_gap_detected__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 3), 19);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 19), 19);
    failed = failed + expect(mercury_bd_pack(19, 0), 2468);

    // 20_sync_failed__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 4), 20);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 20), 20);
    failed = failed + expect(mercury_bd_pack(20, 0), 2597);

    // 00_accepted__1
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(0, 0), 0);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 0), 0);
    failed = failed + expect(mercury_bd_pack(0, 0), 120);

    // 01_missing_account_id__2
    failed = failed + expect(mercury_bd_pre(0, 1, 0, 1, 1), 1);
    failed = failed + expect(mercury_bd_kt(1), 7);
    failed = failed + expect(mercury_bd_secrets(1, 0, 0), 8);
    failed = failed + expect(mercury_bd_rs(0, 4), 20);
    failed = failed + expect(mercury_bd_reason(1, 7, 8, 20), 1);
    failed = failed + expect(mercury_bd_pack(1, 0), 129);

    // 06_key_transparency_not_ready__3
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(2), 6);
    failed = failed + expect(mercury_bd_secrets(1, 0, 0), 8);
    failed = failed + expect(mercury_bd_rs(1, 0), 15);
    failed = failed + expect(mercury_bd_reason(0, 6, 8, 15), 6);
    failed = failed + expect(mercury_bd_pack(6, 0), 804);

    // 08_account_secret_missing__2
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(1, 0, 1), 8);
    failed = failed + expect(mercury_bd_rs(1, 0), 15);
    failed = failed + expect(mercury_bd_reason(0, 0, 8, 15), 8);
    failed = failed + expect(mercury_bd_pack(8, 0), 1027);

    // 15_replay_checkpoint_missing__2
    failed = failed + expect(mercury_bd_pre(1, 1, 0, 1, 1), 0);
    failed = failed + expect(mercury_bd_kt(0), 0);
    failed = failed + expect(mercury_bd_secrets(0, 0, 0), 0);
    failed = failed + expect(mercury_bd_rs(1, 4), 15);
    failed = failed + expect(mercury_bd_reason(0, 0, 0, 15), 15);
    failed = failed + expect(mercury_bd_pack(15, 0), 1956);

    if failed == 0 { 42 } else { failed }
}
