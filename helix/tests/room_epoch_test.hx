// Mercury Phase 1 room epoch and device membership policy tests.

@pure
fn mercury_room_epoch_accept() -> i32 { 0 }

@pure
fn mercury_room_epoch_bad_version() -> i32 { 1 }

@pure
fn mercury_room_epoch_bad_room_mode() -> i32 { 2 }

@pure
fn mercury_room_epoch_bad_epoch() -> i32 { 3 }

@pure
fn mercury_room_epoch_stale_epoch() -> i32 { 4 }

@pure
fn mercury_room_epoch_future_epoch() -> i32 { 5 }

@pure
fn mercury_room_epoch_bad_device_kind() -> i32 { 6 }

@pure
fn mercury_room_epoch_bad_device_state() -> i32 { 7 }

@pure
fn mercury_room_epoch_bad_revoked_at_epoch() -> i32 { 8 }

@pure
fn mercury_room_epoch_bad_access_kind() -> i32 { 9 }

@pure
fn mercury_room_epoch_device_removed() -> i32 { 10 }

@pure
fn mercury_room_epoch_device_compromised() -> i32 { 11 }

@pure
fn mercury_room_epoch_highsec_epoch_rotation_required() -> i32 { 12 }

@pure
fn mercury_room_epoch_ai_device_blocked_room() -> i32 { 13 }

@pure
fn mercury_room_epoch_audit_accepted() -> i32 { 1 }

@pure
fn mercury_room_epoch_audit_policy_reject() -> i32 { 2 }

@pure
fn mercury_room_epoch_audit_replay_reject() -> i32 { 3 }

@pure
fn mercury_room_epoch_audit_membership_reject() -> i32 { 4 }

@pure
fn mercury_room_epoch_audit_highsec_rotation_reject() -> i32 { 5 }

@pure
fn mercury_room_epoch_audit_ai_policy_reject() -> i32 { 6 }

@pure
fn mercury_room_epoch_validate_epoch_tail(
    current_epoch: i32,
    message_epoch: i32,
    min_accepted_epoch: i32,
) -> i32 {
    if current_epoch < min_accepted_epoch {
        mercury_room_epoch_bad_epoch()
    } else {
        if message_epoch < min_accepted_epoch {
            mercury_room_epoch_stale_epoch()
        } else {
            if message_epoch < current_epoch {
                mercury_room_epoch_stale_epoch()
            } else {
                if message_epoch > current_epoch {
                    mercury_room_epoch_future_epoch()
                } else {
                    mercury_room_epoch_accept()
                }
            }
        }
    }
}

@pure
fn mercury_room_epoch_validate_epoch_v1(
    version: i32,
    room_mode: i32,
    current_epoch: i32,
    message_epoch: i32,
    min_accepted_epoch: i32,
) -> i32 {
    if version != 1 {
        mercury_room_epoch_bad_version()
    } else {
        if room_mode < 1 {
            mercury_room_epoch_bad_room_mode()
        } else {
            if room_mode > 4 {
                mercury_room_epoch_bad_room_mode()
            } else {
                if current_epoch < 1 {
                    mercury_room_epoch_bad_epoch()
                } else {
                    if min_accepted_epoch < 1 {
                        mercury_room_epoch_bad_epoch()
                    } else {
                        mercury_room_epoch_validate_epoch_tail(current_epoch, message_epoch, min_accepted_epoch)
                    }
                }
            }
        }
    }
}

@pure
fn mercury_room_epoch_validate_device_state(
    device_state: i32,
    revoked_at_epoch: i32,
    current_epoch: i32,
) -> i32 {
    if device_state == 1 {
        if revoked_at_epoch != 0 {
            mercury_room_epoch_bad_revoked_at_epoch()
        } else {
            mercury_room_epoch_accept()
        }
    } else {
        if revoked_at_epoch < 1 {
            mercury_room_epoch_bad_revoked_at_epoch()
        } else {
            if revoked_at_epoch > current_epoch {
                mercury_room_epoch_bad_revoked_at_epoch()
            } else {
                if device_state == 2 {
                    mercury_room_epoch_device_removed()
                } else {
                    mercury_room_epoch_device_compromised()
                }
            }
        }
    }
}

@pure
fn mercury_room_epoch_validate_device_access(
    device_state: i32,
    revoked_at_epoch: i32,
    current_epoch: i32,
    access_kind: i32,
) -> i32 {
    if access_kind < 0 {
        mercury_room_epoch_bad_access_kind()
    } else {
        if access_kind > 2 {
            mercury_room_epoch_bad_access_kind()
        } else {
            mercury_room_epoch_validate_device_state(device_state, revoked_at_epoch, current_epoch)
        }
    }
}

@pure
fn mercury_room_epoch_validate_device_v1(
    device_kind: i32,
    device_state: i32,
    revoked_at_epoch: i32,
    current_epoch: i32,
    access_kind: i32,
) -> i32 {
    if device_kind < 1 {
        mercury_room_epoch_bad_device_kind()
    } else {
        if device_kind > 2 {
            mercury_room_epoch_bad_device_kind()
        } else {
            if device_state < 1 {
                mercury_room_epoch_bad_device_state()
            } else {
                if device_state > 3 {
                    mercury_room_epoch_bad_device_state()
                } else {
                    mercury_room_epoch_validate_device_access(device_state, revoked_at_epoch, current_epoch, access_kind)
                }
            }
        }
    }
}

@pure
fn mercury_room_epoch_validate_highsec_rotation(
    device_state: i32,
    revoked_at_epoch: i32,
    current_epoch: i32,
    access_kind: i32,
) -> i32 {
    if access_kind < 0 {
        mercury_room_epoch_accept()
    } else {
        if access_kind > 2 {
            mercury_room_epoch_accept()
        } else {
            if device_state == 1 {
                mercury_room_epoch_accept()
            } else {
                if revoked_at_epoch < 1 {
                    mercury_room_epoch_accept()
                } else {
                    if revoked_at_epoch > current_epoch {
                        mercury_room_epoch_accept()
                    } else {
                        if current_epoch <= revoked_at_epoch {
                            mercury_room_epoch_highsec_epoch_rotation_required()
                        } else {
                            mercury_room_epoch_accept()
                        }
                    }
                }
            }
        }
    }
}

@pure
fn mercury_room_epoch_validate_highsec_v1(
    room_mode: i32,
    device_kind: i32,
    device_state: i32,
    revoked_at_epoch: i32,
    current_epoch: i32,
    access_kind: i32,
) -> i32 {
    if room_mode == 4 {
        if device_kind == 2 {
            mercury_room_epoch_ai_device_blocked_room()
        } else {
            mercury_room_epoch_accept()
        }
    } else {
        if room_mode == 3 {
            mercury_room_epoch_validate_highsec_rotation(device_state, revoked_at_epoch, current_epoch, access_kind)
        } else {
            mercury_room_epoch_accept()
        }
    }
}

@pure
fn mercury_room_epoch_first_reject(epoch_reason: i32, device_reason: i32, highsec_reason: i32) -> i32 {
    if epoch_reason != mercury_room_epoch_accept() {
        epoch_reason
    } else {
        if highsec_reason == mercury_room_epoch_highsec_epoch_rotation_required() {
            highsec_reason
        } else {
            if device_reason != mercury_room_epoch_accept() {
                device_reason
            } else {
                highsec_reason
            }
        }
    }
}

@pure
fn mercury_room_epoch_audit_class_for_reason(reason_code: i32) -> i32 {
    if reason_code == mercury_room_epoch_accept() {
        mercury_room_epoch_audit_accepted()
    } else {
        if reason_code == mercury_room_epoch_stale_epoch() {
            mercury_room_epoch_audit_replay_reject()
        } else {
            if reason_code == mercury_room_epoch_future_epoch() {
                mercury_room_epoch_audit_replay_reject()
            } else {
                if reason_code == mercury_room_epoch_device_removed() {
                    mercury_room_epoch_audit_membership_reject()
                } else {
                    if reason_code == mercury_room_epoch_device_compromised() {
                        mercury_room_epoch_audit_membership_reject()
                    } else {
                        if reason_code == mercury_room_epoch_highsec_epoch_rotation_required() {
                            mercury_room_epoch_audit_highsec_rotation_reject()
                        } else {
                            if reason_code == mercury_room_epoch_ai_device_blocked_room() {
                                mercury_room_epoch_audit_ai_policy_reject()
                            } else {
                                mercury_room_epoch_audit_policy_reject()
                            }
                        }
                    }
                }
            }
        }
    }
}

@pure
fn room_epoch_case(
    version: i32,
    room_mode: i32,
    device_kind: i32,
    device_state: i32,
    current_epoch: i32,
    message_epoch: i32,
) -> i32 {
    mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(version, room_mode, current_epoch, message_epoch, 1),
        mercury_room_epoch_validate_device_v1(device_kind, device_state, 0, current_epoch, 2),
        mercury_room_epoch_validate_highsec_v1(room_mode, device_kind, device_state, 0, current_epoch, 2),
    )
}

@pure
fn room_epoch_revoked_case(
    room_mode: i32,
    device_kind: i32,
    device_state: i32,
    current_epoch: i32,
    revoked_at_epoch: i32,
    access_kind: i32,
) -> i32 {
    mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, room_mode, current_epoch, current_epoch, 1),
        mercury_room_epoch_validate_device_v1(device_kind, device_state, revoked_at_epoch, current_epoch, access_kind),
        mercury_room_epoch_validate_highsec_v1(room_mode, device_kind, device_state, revoked_at_epoch, current_epoch, access_kind),
    )
}

@pure
fn expect(actual: i32, expected: i32) -> i32 {
    if actual == expected { 0 } else { 1 }
}

@pure
fn expect_audit(reason_code: i32, expected: i32) -> i32 {
    expect(mercury_room_epoch_audit_class_for_reason(reason_code), expected)
}

fn main() -> i32 {
    let mut failed: i32 = 0;

    // GENERATED from vectors/room_epoch/*.json by tools/gen_helix_tests.py.
    // Do not edit by hand; run `python3 tools/gen_helix_tests.py --write`
    // after changing the vectors (Track 0b: Rust<->Helix differential).

    // reject_room_epoch_ai_blocked_room
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 4, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(2, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(4, 2, 1, 0, 7, 2),
    ), mercury_room_epoch_ai_device_blocked_room());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 4, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(2, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(4, 2, 1, 0, 7, 2),
    ), mercury_room_epoch_audit_ai_policy_reject());

    // reject_room_epoch_bad_access_kind
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 3),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 3),
    ), mercury_room_epoch_bad_access_kind());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 3),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 3),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_bad_active_revoked_epoch
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 6, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 6, 7, 2),
    ), mercury_room_epoch_bad_revoked_at_epoch());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 6, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 6, 7, 2),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_bad_device_kind
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(3, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 3, 1, 0, 7, 2),
    ), mercury_room_epoch_bad_device_kind());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(3, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 3, 1, 0, 7, 2),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_bad_device_state
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 4, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 4, 0, 7, 2),
    ), mercury_room_epoch_bad_device_state());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 4, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 4, 0, 7, 2),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_bad_epoch_zero
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 0, 0, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 0, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 0, 2),
    ), mercury_room_epoch_bad_epoch());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 0, 0, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 0, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 0, 2),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_bad_removed_future_epoch
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 2, 8, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 2, 8, 7, 2),
    ), mercury_room_epoch_bad_revoked_at_epoch());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 2, 8, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 2, 8, 7, 2),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_bad_removed_no_epoch
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 2, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 2, 0, 7, 2),
    ), mercury_room_epoch_bad_revoked_at_epoch());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 2, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 2, 0, 7, 2),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_bad_room_mode
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 5, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(5, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_bad_room_mode());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 5, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(5, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_bad_version_v0
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(0, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_bad_version());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(0, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_compromised_device
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 3, 6, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 3, 6, 7, 2),
    ), mercury_room_epoch_device_compromised());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 3, 6, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 3, 6, 7, 2),
    ), mercury_room_epoch_audit_membership_reject());

    // reject_room_epoch_future_epoch
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 8, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_future_epoch());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 8, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_audit_replay_reject());

    // reject_room_epoch_highsec_bad_removed_future_epoch
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 3, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(2, 2, 8, 7, 1),
        mercury_room_epoch_validate_highsec_v1(3, 2, 2, 8, 7, 1),
    ), mercury_room_epoch_bad_revoked_at_epoch());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 3, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(2, 2, 8, 7, 1),
        mercury_room_epoch_validate_highsec_v1(3, 2, 2, 8, 7, 1),
    ), mercury_room_epoch_audit_policy_reject());

    // reject_room_epoch_highsec_removed_after_rotation
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 3, 8, 8, 1),
        mercury_room_epoch_validate_device_v1(2, 2, 7, 8, 1),
        mercury_room_epoch_validate_highsec_v1(3, 2, 2, 7, 8, 1),
    ), mercury_room_epoch_device_removed());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 3, 8, 8, 1),
        mercury_room_epoch_validate_device_v1(2, 2, 7, 8, 1),
        mercury_room_epoch_validate_highsec_v1(3, 2, 2, 7, 8, 1),
    ), mercury_room_epoch_audit_membership_reject());

    // reject_room_epoch_highsec_rotation_required
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 3, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(2, 2, 7, 7, 1),
        mercury_room_epoch_validate_highsec_v1(3, 2, 2, 7, 7, 1),
    ), mercury_room_epoch_highsec_epoch_rotation_required());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 3, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(2, 2, 7, 7, 1),
        mercury_room_epoch_validate_highsec_v1(3, 2, 2, 7, 7, 1),
    ), mercury_room_epoch_audit_highsec_rotation_reject());

    // reject_room_epoch_removed_device
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 2, 6, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 2, 6, 7, 2),
    ), mercury_room_epoch_device_removed());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 2, 6, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 2, 6, 7, 2),
    ), mercury_room_epoch_audit_membership_reject());

    // reject_room_epoch_stale_epoch
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 6, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_stale_epoch());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 6, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_audit_replay_reject());

    // valid_active_ai_standard_current_epoch_v1
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(2, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 2, 1, 0, 7, 2),
    ), mercury_room_epoch_accept());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(2, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 2, 1, 0, 7, 2),
    ), mercury_room_epoch_audit_accepted());

    // valid_active_human_current_epoch_v1
    failed = failed + expect(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_accept());
    failed = failed + expect_audit(mercury_room_epoch_first_reject(
        mercury_room_epoch_validate_epoch_v1(1, 1, 7, 7, 1),
        mercury_room_epoch_validate_device_v1(1, 1, 0, 7, 2),
        mercury_room_epoch_validate_highsec_v1(1, 1, 1, 0, 7, 2),
    ), mercury_room_epoch_audit_accepted());

    if failed == 0 { 42 } else { failed }
}
