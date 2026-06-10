//! Exhaustive differential: the staged Helix decider reductions vs the REAL mercury-core
//! `evaluate_*` combinators, over the COMPLETE reduced input domain.
//!
//! The `*_vectors.rs` pins prove agreement on a representative sample (the per-decider golden vectors).
//! This proves it for EVERY input: each test enumerates the full cross-product of the real input
//! variants (every `KeyTransparencyState`, every `ClientBootstrapSecretState`, every
//! `RoomMembershipSendState` transition shape, ...), builds the REAL input struct, calls the REAL
//! `evaluate_*`, independently computes the staged reduction in Rust (a line-for-line port of the
//! corresponding `helix/policy/*_decide.hx`), and asserts every decision bool + reason code + the
//! packed projection agree. Because it enumerates the real variants (not just the reduced codes),
//! it also proves the lossy reductions are faithful for ALL members of each collapsed class -- e.g.
//! that NotChecked / MissingProof / StaleProof all behave as `kt_code = 2`, and that every
//! `Transition{accepted,epoch_rotated}` shape behaves as its `room_state` code. This is the FU-4
//! exhaustive backstop behind the sampled vector pins.
//!
//! Honest scope note: where a reduction collapses an integer range to a predicate (e.g.
//! `recovery_key_entropy_bits >= 192/128` -> `entropy_ok`, `recovery_key_digest_len == 32` ->
//! `digest_ok`, the threshold quorum, `audit_digest_len == 32`), the enumeration tests the
//! decision-boundary representatives (min vs min-1, 32 vs 31), NOT every integer -- faithful because
//! the real fn's behaviour depends only on the predicate. So "EVERY input" above means every input
//! in the reduced (predicate/enum-code) domain, not the full integer domain. See
//! docs/HELIX_DECIDER_COVERAGE.md for the per-reduction detail.
//!
//! Counts: outbound 144, receive 20_480, bootstrap 409_600, inbound_sync 256, account_recovery
//! 20_480 input combinations.

use mercury_core::{
    AccountRecoveryDecision, AccountRecoveryInput, AccountRecoveryMethod, ClientBootstrapDecision,
    ClientBootstrapInput, ClientBootstrapReason, ClientBootstrapSecretState, ClientReceiveDecision,
    ClientReceiveInput, ClientReceiveReplayState, ClientReplayCheckpointState, ClientSyncState,
    ComponentReasons, DeliveryAckDecision, DeliveryAckReason, DeviceTrustDecision,
    DeviceTrustReason, InboundSyncDecision, InboundSyncInput, InboundSyncSourceState,
    KeyTransparencyDecision, KeyTransparencyReason, KeyTransparencyState, LocalStoreRecordKind,
    LocalStoreSealingDecision, LocalStoreSealingReason, OutboundSendDecision, OutboundSendInput,
    PolicyDecision, RelayQueueDecision, RelayQueueItemState, RelayQueueReason,
    RoomMembershipSendState, RoomMembershipTransitionDecision, RoomMembershipTransitionReason,
    evaluate_account_recovery, evaluate_client_bootstrap, evaluate_client_receive,
    evaluate_inbound_sync, evaluate_outbound_send,
};

fn b2i(b: bool) -> i32 {
    if b { 1 } else { 0 }
}

fn fnz(values: &[i32]) -> i32 {
    for &v in values {
        if v != 0 {
            return v;
        }
    }
    0
}

// ---- shared sub-decision builders (representatives; the real combinators read only the fields
// the reductions encode -- the same reconstruction the *_vectors.rs pins use). ----

fn device_trust(can_send: bool, rua: bool) -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: false,
        can_send,
        requires_user_action: rua,
        reason: DeviceTrustReason::Trusted,
    }
}

fn policy(accepted: bool) -> PolicyDecision {
    PolicyDecision {
        accepted,
        reason_code: 0,
        audit_class: 0,
        components: ComponentReasons {
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    }
}

fn sealing(accepted: bool) -> LocalStoreSealingDecision {
    LocalStoreSealingDecision {
        accepted,
        reason: LocalStoreSealingReason::Accepted,
        record_policy: LocalStoreRecordKind::MessageCiphertext.policy(),
    }
}

// =====================================================================================
// outbound_decide  (helix/policy/outbound_decide.hx)
// =====================================================================================

fn outbound_staged(
    dt_can_send: i32,
    dt_rua: i32,
    room_state: i32,
    room_rua: i32,
    policy_accepted: i32,
    sealing_accepted: i32,
) -> (i32, bool, bool, bool, bool, i32) {
    let reason = if dt_can_send == 0 {
        1
    } else if room_state == 2 {
        2
    } else if room_state == 3 {
        3
    } else if policy_accepted == 0 {
        4
    } else if sealing_accepted == 0 {
        5
    } else {
        0
    };
    let acc = reason == 0;
    let rua = if reason == 0 {
        if dt_rua == 1 {
            1
        } else if room_state == 1 {
            room_rua
        } else {
            0
        }
    } else {
        1
    };
    let pack = reason * 16 + b2i(acc) * 14 + rua;
    (reason, acc, acc, acc, rua == 1, pack)
}

fn outbound_pack_real(d: &OutboundSendDecision) -> i32 {
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
fn outbound_decide_exhaustive_matches_real() {
    // Enumerate the REAL room_membership shapes so the room_state reduction is proven for every
    // Transition{accepted,epoch_rotated,rua} (not just the 4 representative codes).
    let mut rooms: Vec<RoomMembershipSendState> = vec![RoomMembershipSendState::Stable];
    for accepted in [false, true] {
        for epoch_rotated in [false, true] {
            for rua in [false, true] {
                rooms.push(RoomMembershipSendState::Transition(
                    RoomMembershipTransitionDecision {
                        accepted,
                        epoch_rotated,
                        requires_user_action: rua,
                        reason: RoomMembershipTransitionReason::Accepted,
                    },
                ));
            }
        }
    }

    let mut count = 0usize;
    for dt_can_send in [false, true] {
        for dt_rua in [false, true] {
            for room in &rooms {
                let (room_state, room_rua) = match room {
                    RoomMembershipSendState::Stable => (0, 0),
                    RoomMembershipSendState::Transition(t) => {
                        let code = if !t.accepted {
                            2
                        } else if !t.epoch_rotated {
                            3
                        } else {
                            1
                        };
                        (code, b2i(t.requires_user_action))
                    }
                };
                for policy_accepted in [false, true] {
                    for sealing_accepted in [false, true] {
                        let input = OutboundSendInput {
                            sender_device_trust: device_trust(dt_can_send, dt_rua),
                            room_membership: *room,
                            message_policy: policy(policy_accepted),
                            ciphertext_sealing: sealing(sealing_accepted),
                        };
                        let real = evaluate_outbound_send(input);
                        let (r, acc, cs, cp, rua, pack) = outbound_staged(
                            b2i(dt_can_send),
                            b2i(dt_rua),
                            room_state,
                            room_rua,
                            b2i(policy_accepted),
                            b2i(sealing_accepted),
                        );
                        let at = format!(
                            "outbound dt_cs={dt_can_send} dt_rua={dt_rua} room={room:?} pol={policy_accepted} seal={sealing_accepted}"
                        );
                        assert_eq!(real.reason.code(), r, "reason {at}");
                        assert_eq!(real.accepted, acc, "accepted {at}");
                        assert_eq!(real.can_send, cs, "can_send {at}");
                        assert_eq!(real.can_persist_ciphertext, cp, "can_persist {at}");
                        assert_eq!(real.requires_user_action, rua, "rua {at}");
                        assert_eq!(outbound_pack_real(&real), pack, "pack {at}");
                        count += 1;
                    }
                }
            }
        }
    }
    assert_eq!(count, 2 * 2 * 9 * 2 * 2, "outbound exhaustive count");
}

// =====================================================================================
// receive_decide  (helix/policy/receive_decide.hx)
// =====================================================================================

fn receive_staged(
    relay_accepted: i32,
    relay_delivered: i32,
    ack_duplicate: i32,
    ack_accepted: i32,
    ack_rcr: i32,
    digest_ok: i32,
    plaintext_ok: i32,
    replay_state: i32,
    dt_can_send: i32,
    policy_accepted: i32,
    sealing_accepted: i32,
    dt_rua: i32,
) -> (i32, bool, bool, bool, i32) {
    let relay = if relay_accepted == 0 {
        1
    } else if relay_delivered == 0 {
        2
    } else if ack_duplicate == 1 {
        4
    } else if ack_accepted == 0 {
        3
    } else {
        0
    };
    let content = if digest_ok == 0 {
        11
    } else if plaintext_ok == 0 {
        12
    } else if replay_state == 1 {
        8
    } else if replay_state == 2 {
        9
    } else if replay_state == 3 {
        10
    } else {
        0
    };
    let trust = if dt_can_send == 0 {
        5
    } else if policy_accepted == 0 {
        6
    } else if sealing_accepted == 0 {
        7
    } else {
        0
    };
    let reason = fnz(&[relay, content, trust]);
    let acc = reason == 0;
    let rcr = if reason == 1 || reason == 2 || reason == 10 {
        1
    } else if reason == 3 {
        ack_rcr
    } else {
        0
    };
    let rua = if reason == 5 || reason == 6 || reason == 7 {
        1
    } else if reason == 0 {
        dt_rua
    } else {
        0
    };
    let pack = reason * 64 + b2i(acc) * 60 + rcr * 2 + rua;
    (reason, acc, rcr == 1, rua == 1, pack)
}

fn receive_pack_real(d: &ClientReceiveDecision) -> i32 {
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

fn receive_replay(state: i32) -> ClientReceiveReplayState {
    match state {
        0 => ClientReceiveReplayState::NewInOrder,
        1 => ClientReceiveReplayState::Duplicate,
        2 => ClientReceiveReplayState::Stale,
        3 => ClientReceiveReplayState::FutureGap,
        other => panic!("replay {other}"),
    }
}

#[test]
fn receive_decide_exhaustive_matches_real() {
    let mut count = 0usize;
    for relay_accepted in [false, true] {
        // Enumerate every real RelayQueueItemState so the relay_delivered reduction (== Delivered)
        // is proven for each non-Delivered member (Absent/Pending/Expired/Deleted), not just one.
        for relay_item in [
            RelayQueueItemState::Absent,
            RelayQueueItemState::Pending,
            RelayQueueItemState::Delivered,
            RelayQueueItemState::Expired,
            RelayQueueItemState::Deleted,
        ] {
            let relay_delivered = relay_item == RelayQueueItemState::Delivered;
            for ack_duplicate in [false, true] {
                for ack_accepted in [false, true] {
                    for ack_rcr in [false, true] {
                        for digest_ok in [false, true] {
                            for plaintext_ok in [false, true] {
                                for replay in 0..4 {
                                    for dt_can_send in [false, true] {
                                        for policy_accepted in [false, true] {
                                            for sealing_accepted in [false, true] {
                                                for dt_rua in [false, true] {
                                                    let input =
                                                        ClientReceiveInput {
                                                            relay_delivery: RelayQueueDecision {
                                                                accepted: relay_accepted,
                                                                next_state: relay_item,
                                                                persist_item: false,
                                                                delete_item: false,
                                                                reason: RelayQueueReason::Accepted,
                                                            },
                                                            delivery_ack: DeliveryAckDecision {
                                                                accepted: ack_accepted,
                                                                duplicate: ack_duplicate,
                                                                retain_hash_audit: false,
                                                                delete_queue_record: false,
                                                                requires_client_retry: ack_rcr,
                                                                reason: DeliveryAckReason::Accepted,
                                                            },
                                                            sender_device_trust: device_trust(
                                                                dt_can_send,
                                                                dt_rua,
                                                            ),
                                                            message_policy: policy(policy_accepted),
                                                            ciphertext_sealing: sealing(
                                                                sealing_accepted,
                                                            ),
                                                            replay_state: receive_replay(replay),
                                                            ciphertext_digest_len: if digest_ok {
                                                                32
                                                            } else {
                                                                31
                                                            },
                                                            plaintext_identity_fields:
                                                                if plaintext_ok { 0 } else { 1 },
                                                        };
                                                    let real = evaluate_client_receive(input);
                                                    let (r, acc, rcr, rua, pack) = receive_staged(
                                                        b2i(relay_accepted),
                                                        b2i(relay_delivered),
                                                        b2i(ack_duplicate),
                                                        b2i(ack_accepted),
                                                        b2i(ack_rcr),
                                                        b2i(digest_ok),
                                                        b2i(plaintext_ok),
                                                        replay,
                                                        b2i(dt_can_send),
                                                        b2i(policy_accepted),
                                                        b2i(sealing_accepted),
                                                        b2i(dt_rua),
                                                    );
                                                    assert_eq!(
                                                        real.reason.code(),
                                                        r,
                                                        "receive reason"
                                                    );
                                                    assert_eq!(real.accepted, acc, "rx accepted");
                                                    assert_eq!(
                                                        real.can_decrypt, acc,
                                                        "rx can_decrypt"
                                                    );
                                                    assert_eq!(
                                                        real.can_persist_ciphertext, acc,
                                                        "rx can_persist"
                                                    );
                                                    assert_eq!(
                                                        real.can_expose_to_ui, acc,
                                                        "rx can_expose"
                                                    );
                                                    assert_eq!(
                                                        real.requires_client_retry, rcr,
                                                        "rx rcr"
                                                    );
                                                    assert_eq!(
                                                        real.requires_user_action, rua,
                                                        "rx rua"
                                                    );
                                                    assert_eq!(
                                                        receive_pack_real(&real),
                                                        pack,
                                                        "rx pack"
                                                    );
                                                    count += 1;
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
        }
    }
    assert_eq!(count, 20480, "receive exhaustive count");
}

// =====================================================================================
// bootstrap_decide  (helix/policy/bootstrap_decide.hx)
// =====================================================================================

const CSS_SET: [i32; 10] = [0, 6, 12, 13, 15, 16, 17, 18, 19, 20];
const RSY_SET: [i32; 9] = [6, 12, 13, 15, 16, 17, 18, 19, 20];
const RRC_SET: [i32; 5] = [3, 8, 9, 10, 11];
const NOACTION_SET: [i32; 7] = [0, 12, 15, 16, 17, 18, 19];

fn in_set(reason: i32, set: &[i32]) -> bool {
    set.contains(&reason)
}

fn bootstrap_staged(
    account_id_ok: i32,
    device_id_ok: i32,
    pending_recovery: i32,
    plaintext_cache_ok: i32,
    local_trust: i32,
    kt_code: i32,
    kt_rua: i32,
    account_secret: i32,
    device_secret: i32,
    room_state: i32,
    replay: i32,
    sync: i32,
) -> (i32, bool, bool, bool, bool, bool, bool, bool, i32) {
    let pre = if account_id_ok == 0 {
        1
    } else if device_id_ok == 0 {
        2
    } else if pending_recovery == 1 {
        3
    } else if plaintext_cache_ok == 0 {
        4
    } else if local_trust == 0 {
        5
    } else {
        0
    };
    let kt = if kt_code == 1 {
        7
    } else if kt_code == 2 {
        6
    } else {
        0
    };
    let secret_one = |s: i32, missing: i32, corrupt: i32| -> i32 {
        if s == 1 {
            missing
        } else if s == 2 {
            corrupt
        } else if s == 3 {
            14
        } else {
            0
        }
    };
    let secrets = fnz(&[
        secret_one(account_secret, 8, 9),
        secret_one(device_secret, 10, 11),
        secret_one(room_state, 12, 13),
    ]);
    let rs = {
        let replay_r = match replay {
            1 => 15,
            2 => 16,
            3 => 17,
            _ => 0,
        };
        if replay_r != 0 {
            replay_r
        } else {
            match sync {
                1 => 18,
                2 => 18,
                3 => 19,
                4 => 20,
                _ => 0,
            }
        }
    };
    let reason = fnz(&[pre, kt, secrets, rs]);
    let acc = reason == 0;
    let css = in_set(reason, &CSS_SET);
    let rsy = in_set(reason, &RSY_SET);
    let rrc = in_set(reason, &RRC_SET);
    let rua = if reason == 6 {
        kt_rua == 1
    } else {
        !in_set(reason, &NOACTION_SET)
    };
    let pack = reason * 128
        + b2i(acc) * 64
        + b2i(css) * 32
        + b2i(acc) * 16
        + b2i(acc) * 8
        + b2i(rsy) * 4
        + b2i(rrc) * 2
        + b2i(rua);
    (reason, acc, css, acc, acc, rsy, rrc, rua, pack)
}

fn bootstrap_pack_real(d: &ClientBootstrapDecision) -> i32 {
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

fn kt_state(code: i32, idx: i32) -> KeyTransparencyState {
    // code 2 = NotReady: enumerate all three real members so the collapse is proven for each.
    match code {
        0 => KeyTransparencyState::Consistent,
        1 => KeyTransparencyState::Inconsistent,
        2 => match idx {
            0 => KeyTransparencyState::NotChecked,
            1 => KeyTransparencyState::MissingProof,
            _ => KeyTransparencyState::StaleProof,
        },
        other => panic!("kt_code {other}"),
    }
}

fn secret_state(code: i32) -> ClientBootstrapSecretState {
    match code {
        0 => ClientBootstrapSecretState::PresentSealed,
        1 => ClientBootstrapSecretState::Missing,
        2 => ClientBootstrapSecretState::Corrupt,
        3 => ClientBootstrapSecretState::PlaintextPresent,
        other => panic!("secret {other}"),
    }
}

fn replay_state(code: i32) -> ClientReplayCheckpointState {
    match code {
        0 => ClientReplayCheckpointState::Ready,
        1 => ClientReplayCheckpointState::Missing,
        2 => ClientReplayCheckpointState::Stale,
        3 => ClientReplayCheckpointState::GapDetected,
        other => panic!("replay {other}"),
    }
}

fn sync_state(code: i32) -> ClientSyncState {
    match code {
        0 => ClientSyncState::CaughtUp,
        1 => ClientSyncState::Offline,
        2 => ClientSyncState::CatchingUp,
        3 => ClientSyncState::GapDetected,
        4 => ClientSyncState::Failed,
        other => panic!("sync {other}"),
    }
}

#[test]
fn bootstrap_decide_exhaustive_matches_real() {
    let mut count = 0usize;
    for account_id_ok in [false, true] {
        for device_id_ok in [false, true] {
            for pending_recovery in [false, true] {
                for plaintext_cache_ok in [false, true] {
                    for local_trust in [false, true] {
                        for kt_code in 0..3 {
                            // kt_code 2 collapses 3 real states; expand them so each is checked.
                            let kt_variants = if kt_code == 2 { 3 } else { 1 };
                            for kt_idx in 0..kt_variants {
                                for kt_rua in [false, true] {
                                    for account_secret in 0..4 {
                                        for device_secret in 0..4 {
                                            for room_state in 0..4 {
                                                for replay in 0..4 {
                                                    for sync in 0..5 {
                                                        let input = ClientBootstrapInput {
                                                            account_id_len: if account_id_ok {
                                                                32
                                                            } else {
                                                                0
                                                            },
                                                            device_id_len: if device_id_ok {
                                                                32
                                                            } else {
                                                                0
                                                            },
                                                            // bootstrap reads .trusted (not
                                                            // .can_send like outbound/receive).
                                                            local_device_trust:
                                                                DeviceTrustDecision {
                                                                    trusted: local_trust,
                                                                    can_send: true,
                                                                    requires_user_action: false,
                                                                    reason:
                                                                        DeviceTrustReason::Trusted,
                                                                },
                                                            key_transparency:
                                                                KeyTransparencyDecision {
                                                                    state: kt_state(kt_code, kt_idx),
                                                                    reason:
                                                                        KeyTransparencyReason::Consistent,
                                                                    requires_user_action: kt_rua,
                                                                },
                                                            account_secret: secret_state(
                                                                account_secret,
                                                            ),
                                                            device_secret: secret_state(
                                                                device_secret,
                                                            ),
                                                            room_state: secret_state(room_state),
                                                            replay_checkpoint: replay_state(replay),
                                                            sync_state: sync_state(sync),
                                                            pending_recovery,
                                                            plaintext_cache_records:
                                                                if plaintext_cache_ok { 0 } else { 1 },
                                                        };
                                                        let real = evaluate_client_bootstrap(input);
                                                        let (
                                                            r,
                                                            acc,
                                                            css,
                                                            cdls,
                                                            cous,
                                                            rsy,
                                                            rrc,
                                                            rua,
                                                            pack,
                                                        ) = bootstrap_staged(
                                                            b2i(account_id_ok),
                                                            b2i(device_id_ok),
                                                            b2i(pending_recovery),
                                                            b2i(plaintext_cache_ok),
                                                            b2i(local_trust),
                                                            kt_code,
                                                            b2i(kt_rua),
                                                            account_secret,
                                                            device_secret,
                                                            room_state,
                                                            replay,
                                                            sync,
                                                        );
                                                        assert_eq!(
                                                            real.reason.code(),
                                                            r,
                                                            "bs reason"
                                                        );
                                                        assert_eq!(
                                                            real.accepted, acc,
                                                            "bs accepted"
                                                        );
                                                        assert_eq!(
                                                            real.can_start_sync, css,
                                                            "bs css"
                                                        );
                                                        assert_eq!(
                                                            real.can_decrypt_local_store, cdls,
                                                            "bs cdls"
                                                        );
                                                        assert_eq!(
                                                            real.can_open_message_ui, cous,
                                                            "bs cous"
                                                        );
                                                        assert_eq!(
                                                            real.requires_sync, rsy,
                                                            "bs rsy"
                                                        );
                                                        assert_eq!(
                                                            real.requires_recovery, rrc,
                                                            "bs rrc"
                                                        );
                                                        assert_eq!(
                                                            real.requires_user_action, rua,
                                                            "bs rua"
                                                        );
                                                        assert_eq!(
                                                            bootstrap_pack_real(&real),
                                                            pack,
                                                            "bs pack"
                                                        );
                                                        count += 1;
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
            }
        }
    }
    // 2^5 prefixes * (Consistent + Inconsistent + 3 NotReady) kt-variants * 2 kt_rua * 4^3 secrets
    // * 4 replay * 5 sync = 32 * 5 * 2 * 64 * 4 * 5.
    assert_eq!(count, 32 * 5 * 2 * 64 * 4 * 5, "bootstrap exhaustive count");
}

// =====================================================================================
// inbound_sync  (helix/policy/inbound_sync.hx)
// =====================================================================================

fn inbound_sync_staged(
    plaintext_preview_ok: i32,
    bootstrap_can_start_sync: i32,
    bootstrap_rua: i32,
    source_state: i32,
    poll_batch_ok: i32,
    pending_delivery: i32,
    route_id_ok: i32,
) -> (i32, bool, bool, bool, bool, bool, bool, bool, i32) {
    let reason = if plaintext_preview_ok == 0 {
        8
    } else if bootstrap_can_start_sync == 0 {
        2
    } else if source_state == 1 {
        3
    } else if source_state == 2 {
        4
    } else if source_state == 3 {
        5
    } else if poll_batch_ok == 0 {
        6
    } else if pending_delivery == 0 {
        1
    } else if route_id_ok == 0 {
        7
    } else {
        0
    };
    let acc = reason == 0 || reason == 1;
    let crrs = reason == 0;
    let retry = reason == 3 || reason == 5;
    let rua = if reason == 8 || reason == 4 {
        true
    } else if reason == 2 {
        bootstrap_rua == 1
    } else {
        false
    };
    let exposed = false;
    let pack = reason * 128
        + b2i(acc) * 64
        + b2i(acc) * 32
        + b2i(crrs) * 16
        + b2i(acc) * 8
        + b2i(retry) * 4
        + b2i(rua) * 2
        + b2i(exposed);
    (reason, acc, acc, crrs, acc, retry, rua, exposed, pack)
}

fn inbound_sync_pack_real(d: &InboundSyncDecision) -> i32 {
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

fn inbound_source(state: i32) -> InboundSyncSourceState {
    match state {
        0 => InboundSyncSourceState::Ready,
        1 => InboundSyncSourceState::Offline,
        2 => InboundSyncSourceState::AuthRejected,
        3 => InboundSyncSourceState::BackoffRequired,
        other => panic!("source {other}"),
    }
}

#[test]
fn inbound_sync_exhaustive_matches_real() {
    let mut count = 0usize;
    for plaintext_preview_ok in [false, true] {
        for bootstrap_can_start_sync in [false, true] {
            for bootstrap_rua in [false, true] {
                for source_state in 0..4 {
                    for poll_batch_ok in [false, true] {
                        for pending_delivery in [false, true] {
                            for route_id_ok in [false, true] {
                                let bootstrap = ClientBootstrapDecision {
                                    accepted: false,
                                    can_start_sync: bootstrap_can_start_sync,
                                    can_decrypt_local_store: false,
                                    can_open_message_ui: false,
                                    requires_sync: false,
                                    requires_recovery: false,
                                    requires_user_action: bootstrap_rua,
                                    reason: ClientBootstrapReason::Accepted,
                                };
                                let input = InboundSyncInput {
                                    bootstrap,
                                    source_state: inbound_source(source_state),
                                    pending_delivery,
                                    route_id_len: if route_id_ok { 32 } else { 16 },
                                    poll_batch_limit: if poll_batch_ok { 50 } else { 0 },
                                    plaintext_notification_preview_len: if plaintext_preview_ok {
                                        0
                                    } else {
                                        1
                                    },
                                };
                                let real = evaluate_inbound_sync(input);
                                let (r, acc, cpr, crrs, curc, retry, rua, exposed, pack) =
                                    inbound_sync_staged(
                                        b2i(plaintext_preview_ok),
                                        b2i(bootstrap_can_start_sync),
                                        b2i(bootstrap_rua),
                                        source_state,
                                        b2i(poll_batch_ok),
                                        b2i(pending_delivery),
                                        b2i(route_id_ok),
                                    );
                                assert_eq!(real.reason.code(), r, "is reason");
                                assert_eq!(real.accepted, acc, "is accepted");
                                assert_eq!(real.can_poll_relay, cpr, "is can_poll_relay");
                                assert_eq!(real.can_run_receive_session, crrs, "is crrs");
                                assert_eq!(
                                    real.can_update_replay_checkpoint, curc,
                                    "is can_update_replay"
                                );
                                assert_eq!(real.requires_network_retry, retry, "is retry");
                                assert_eq!(real.requires_user_action, rua, "is rua");
                                assert_eq!(
                                    real.plaintext_bytes_exposed, exposed,
                                    "is plaintext_exposed"
                                );
                                assert_eq!(inbound_sync_pack_real(&real), pack, "is pack");
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        count,
        2 * 2 * 2 * 4 * 2 * 2 * 2,
        "inbound_sync exhaustive count"
    );
}

// =====================================================================================
// account_recovery  (helix/policy/account_recovery.hx)
// =====================================================================================

#[allow(clippy::too_many_arguments)]
fn account_recovery_staged(
    recovery_requested: i32,
    method_code: i32,
    high_security_account: i32,
    entropy_ok: i32,
    digest_ok: i32,
    threshold_ok: i32,
    device_approval_present: i32,
    server_authenticated: i32,
    server_rate_limited: i32,
    backup_encrypted: i32,
    plaintext_backup_ok: i32,
    rotates_device_secret: i32,
    audit_ok: i32,
) -> (i32, bool, bool, bool, bool, i32) {
    let a = if recovery_requested == 0 {
        1
    } else if method_code == 4 {
        2
    } else if entropy_ok == 0 {
        3
    } else if digest_ok == 0 {
        4
    } else {
        0
    };
    let b = if method_code == 2 {
        if threshold_ok == 0 { 5 } else { 0 }
    } else if method_code == 3 {
        if device_approval_present == 0 { 6 } else { 0 }
    } else {
        0
    };
    let c = if server_authenticated == 0 {
        7
    } else if server_rate_limited == 0 {
        8
    } else if backup_encrypted == 0 {
        9
    } else if plaintext_backup_ok == 0 {
        10
    } else {
        0
    };
    let d = if high_security_account == 1 {
        if rotates_device_secret == 0 {
            11
        } else if audit_ok == 0 {
            12
        } else {
            0
        }
    } else if audit_ok == 0 {
        12
    } else {
        0
    };
    let reason = fnz(&[a, b, c, d]);
    let acc = reason == 0;
    let rua = matches!(reason, 2 | 3 | 4 | 5 | 6 | 12);
    let server = matches!(reason, 7 | 8);
    let rotation = reason == 11;
    let pack = reason * 512
        + b2i(acc) * 256
        + b2i(acc) * 128
        + b2i(acc) * 64
        + b2i(rua) * 32
        + b2i(server) * 16
        + b2i(rotation) * 8
        + 4
        + 2;
    (reason, acc, rua, server, rotation, pack)
}

fn account_recovery_pack_real(d: &AccountRecoveryDecision) -> i32 {
    let bits = [
        d.accepted,
        d.can_start_recovery,
        d.can_restore_device_secret,
        d.requires_user_action,
        d.requires_server_setup,
        d.requires_key_rotation,
        d.forbids_low_entropy_recovery,
        d.forbids_plaintext_backup,
        d.plaintext_bytes_exposed,
    ];
    let mut out = d.reason.code();
    for b in bits {
        out = (out << 1) | (b as i32);
    }
    out
}

fn account_recovery_method(code: i32) -> AccountRecoveryMethod {
    match code {
        1 => AccountRecoveryMethod::HighEntropyRecoveryKey,
        2 => AccountRecoveryMethod::ThresholdRecovery,
        3 => AccountRecoveryMethod::HardwareDeviceTransfer,
        4 => AccountRecoveryMethod::LowEntropyPin,
        5 => AccountRecoveryMethod::CloudBackup,
        other => panic!("method {other}"),
    }
}

#[test]
fn account_recovery_exhaustive_matches_real() {
    let mut count = 0usize;
    for recovery_requested in [false, true] {
        for method_code in 1..=5 {
            for high_security_account in [false, true] {
                for entropy_ok in [false, true] {
                    for digest_ok in [false, true] {
                        for threshold_ok in [false, true] {
                            for device_approval_present in [false, true] {
                                for server_authenticated in [false, true] {
                                    for server_rate_limited in [false, true] {
                                        for backup_encrypted in [false, true] {
                                            for plaintext_backup_ok in [false, true] {
                                                for rotates_device_secret in [false, true] {
                                                    for audit_ok in [false, true] {
                                                        let min_entropy = if high_security_account {
                                                            192
                                                        } else {
                                                            128
                                                        };
                                                        let input =
                                                            AccountRecoveryInput {
                                                                recovery_requested,
                                                                method: account_recovery_method(
                                                                    method_code,
                                                                ),
                                                                high_security_account,
                                                                recovery_key_entropy_bits:
                                                                    if entropy_ok {
                                                                        min_entropy
                                                                    } else {
                                                                        min_entropy - 1
                                                                    },
                                                                recovery_key_digest_len:
                                                                    if digest_ok { 32 } else { 31 },
                                                                threshold_shares: if threshold_ok {
                                                                    2
                                                                } else {
                                                                    1
                                                                },
                                                                threshold_required: 2,
                                                                threshold_approvals: 2,
                                                                device_approval_present,
                                                                server_authenticated,
                                                                server_rate_limited,
                                                                backup_encrypted,
                                                                plaintext_backup_fields:
                                                                    if plaintext_backup_ok {
                                                                        0
                                                                    } else {
                                                                        1
                                                                    },
                                                                rotates_device_secret,
                                                                audit_digest_len: if audit_ok {
                                                                    32
                                                                } else {
                                                                    31
                                                                },
                                                            };
                                                        let real = evaluate_account_recovery(input);
                                                        let (r, acc, rua, server, rotation, pack) =
                                                            account_recovery_staged(
                                                                b2i(recovery_requested),
                                                                method_code,
                                                                b2i(high_security_account),
                                                                b2i(entropy_ok),
                                                                b2i(digest_ok),
                                                                b2i(threshold_ok),
                                                                b2i(device_approval_present),
                                                                b2i(server_authenticated),
                                                                b2i(server_rate_limited),
                                                                b2i(backup_encrypted),
                                                                b2i(plaintext_backup_ok),
                                                                b2i(rotates_device_secret),
                                                                b2i(audit_ok),
                                                            );
                                                        assert_eq!(
                                                            real.reason.code(),
                                                            r,
                                                            "ar reason"
                                                        );
                                                        assert_eq!(
                                                            real.accepted, acc,
                                                            "ar accepted"
                                                        );
                                                        assert_eq!(
                                                            real.can_start_recovery, acc,
                                                            "ar can_start"
                                                        );
                                                        assert_eq!(
                                                            real.can_restore_device_secret, acc,
                                                            "ar can_restore"
                                                        );
                                                        assert_eq!(
                                                            real.requires_user_action, rua,
                                                            "ar rua"
                                                        );
                                                        assert_eq!(
                                                            real.requires_server_setup, server,
                                                            "ar server"
                                                        );
                                                        assert_eq!(
                                                            real.requires_key_rotation, rotation,
                                                            "ar rotation"
                                                        );
                                                        assert!(
                                                            real.forbids_low_entropy_recovery,
                                                            "ar forbids_le"
                                                        );
                                                        assert!(
                                                            real.forbids_plaintext_backup,
                                                            "ar forbids_pt"
                                                        );
                                                        assert!(
                                                            !real.plaintext_bytes_exposed,
                                                            "ar exposed"
                                                        );
                                                        assert_eq!(
                                                            real.method.code(),
                                                            method_code,
                                                            "ar method passthrough"
                                                        );
                                                        assert_eq!(
                                                            account_recovery_pack_real(&real),
                                                            pack,
                                                            "ar pack"
                                                        );
                                                        count += 1;
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
            }
        }
    }
    assert_eq!(count, 5 * 4096, "account_recovery exhaustive count");
}
