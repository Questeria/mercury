// Mercury Phase 1 AI grant lifecycle policy tests.

@pure
fn mercury_lifecycle_accept() -> i32 { 0 }

@pure
fn mercury_lifecycle_bad_version() -> i32 { 1 }

@pure
fn mercury_lifecycle_bad_grant_state() -> i32 { 2 }

@pure
fn mercury_lifecycle_bad_revoke_reason() -> i32 { 3 }

@pure
fn mercury_lifecycle_bad_time() -> i32 { 4 }

@pure
fn mercury_lifecycle_grant_expired() -> i32 { 5 }

@pure
fn mercury_lifecycle_grant_revoked() -> i32 { 6 }

@pure
fn mercury_lifecycle_read_after_revoke() -> i32 { 7 }

@pure
fn mercury_lifecycle_write_after_revoke() -> i32 { 8 }

@pure
fn mercury_lifecycle_highsec_epoch_rotation_required() -> i32 { 9 }

@pure
fn mercury_lifecycle_bad_access_kind() -> i32 { 10 }

@pure
fn mercury_lifecycle_audit_accepted() -> i32 { 1 }

@pure
fn mercury_lifecycle_audit_policy_reject() -> i32 { 2 }

@pure
fn mercury_lifecycle_audit_expired_reject() -> i32 { 3 }

@pure
fn mercury_lifecycle_audit_revoked_reject() -> i32 { 4 }

@pure
fn mercury_lifecycle_audit_post_revoke_access_reject() -> i32 { 5 }

@pure
fn mercury_lifecycle_audit_highsec_epoch_rotation_required() -> i32 { 6 }

@pure
fn mercury_lifecycle_validate_state_tail(
    grant_state: i32,
    revoke_reason: i32,
    now_s: i32,
    expires_at_s: i32,
) -> i32 {
    if grant_state == 1 {
        if revoke_reason != 0 {
            mercury_lifecycle_bad_revoke_reason()
        } else {
            if now_s >= expires_at_s {
                mercury_lifecycle_grant_expired()
            } else {
                mercury_lifecycle_accept()
            }
        }
    } else {
        if revoke_reason < 1 {
            mercury_lifecycle_bad_revoke_reason()
        } else {
            if revoke_reason > 5 {
                mercury_lifecycle_bad_revoke_reason()
            } else {
                mercury_lifecycle_grant_revoked()
            }
        }
    }
}
@pure
fn mercury_lifecycle_validate_state_v1(
    version: i32,
    grant_state: i32,
    revoke_reason: i32,
    now_s: i32,
    expires_at_s: i32,
    room_mode: i32,
) -> i32 {
    if version != 1 {
        mercury_lifecycle_bad_version()
    } else {
        if grant_state < 1 {
            mercury_lifecycle_bad_grant_state()
        } else {
            if grant_state > 2 {
                mercury_lifecycle_bad_grant_state()
            } else {
                if now_s < 0 {
                    mercury_lifecycle_bad_time()
                } else {
                    if expires_at_s <= 0 {
                        mercury_lifecycle_bad_time()
                    } else {
                        mercury_lifecycle_validate_state_tail(grant_state, revoke_reason, now_s, expires_at_s)
                    }
                }
            }
        }
    }
}

@pure
fn mercury_lifecycle_post_revoke_access(access_kind: i32) -> i32 {
    if access_kind == 0 {
        mercury_lifecycle_grant_revoked()
    } else {
        if access_kind == 1 {
            mercury_lifecycle_read_after_revoke()
        } else {
            mercury_lifecycle_write_after_revoke()
        }
    }
}

@pure
fn mercury_lifecycle_validate_access_v1(
    lifecycle_reason: i32,
    access_kind: i32,
    room_mode: i32,
    epoch_rotated: i32,
) -> i32 {
    if access_kind < 0 {
        mercury_lifecycle_bad_access_kind()
    } else {
        if access_kind > 2 {
            mercury_lifecycle_bad_access_kind()
        } else {
            if lifecycle_reason == mercury_lifecycle_grant_revoked() {
                if room_mode == 3 {
                    if epoch_rotated != 1 {
                        mercury_lifecycle_highsec_epoch_rotation_required()
                    } else {
                        mercury_lifecycle_post_revoke_access(access_kind)
                    }
                } else {
                    mercury_lifecycle_post_revoke_access(access_kind)
                }
            } else {
                mercury_lifecycle_accept()
            }
        }
    }
}

@pure
fn mercury_lifecycle_first_reject(state_reason: i32, access_reason: i32) -> i32 {
    if access_reason == mercury_lifecycle_highsec_epoch_rotation_required() {
        access_reason
    } else {
        if state_reason == mercury_lifecycle_accept() {
            access_reason
        } else {
            if access_reason == mercury_lifecycle_accept() {
                state_reason
            } else {
                access_reason
            }
        }
    }
}

@pure
fn mercury_lifecycle_audit_class_for_reason(reason_code: i32) -> i32 {
    if reason_code == mercury_lifecycle_accept() {
        mercury_lifecycle_audit_accepted()
    } else {
        if reason_code == mercury_lifecycle_grant_expired() {
            mercury_lifecycle_audit_expired_reject()
        } else {
            if reason_code == mercury_lifecycle_grant_revoked() {
                mercury_lifecycle_audit_revoked_reject()
            } else {
                if reason_code == mercury_lifecycle_read_after_revoke() {
                    mercury_lifecycle_audit_post_revoke_access_reject()
                } else {
                    if reason_code == mercury_lifecycle_write_after_revoke() {
                        mercury_lifecycle_audit_post_revoke_access_reject()
                    } else {
                        if reason_code == mercury_lifecycle_highsec_epoch_rotation_required() {
                            mercury_lifecycle_audit_highsec_epoch_rotation_required()
                        } else {
                            mercury_lifecycle_audit_policy_reject()
                        }
                    }
                }
            }
        }
    }
}

@pure
fn lifecycle_case(
    version: i32,
    grant_state: i32,
    revoke_reason: i32,
    now_s: i32,
    expires_at_s: i32,
    access_kind: i32,
) -> i32 {
    let state = mercury_lifecycle_validate_state_v1(version, grant_state, revoke_reason, now_s, expires_at_s, 1);
    mercury_lifecycle_first_reject(
        state,
        mercury_lifecycle_validate_access_v1(state, access_kind, 1, 0),
    )
}

@pure
fn lifecycle_highsec_case(grant_state: i32, revoke_reason: i32, access_kind: i32, epoch_rotated: i32) -> i32 {
    let state = mercury_lifecycle_validate_state_v1(1, grant_state, revoke_reason, 100, 200, 3);
    mercury_lifecycle_first_reject(
        state,
        mercury_lifecycle_validate_access_v1(state, access_kind, 3, epoch_rotated),
    )
}

@pure
fn expect(actual: i32, expected: i32) -> i32 {
    if actual == expected { 0 } else { 1 }
}

@pure
fn expect_audit(reason_code: i32, expected: i32) -> i32 {
    expect(mercury_lifecycle_audit_class_for_reason(reason_code), expected)
}

fn main() -> i32 {
    let mut failed: i32 = 0;

    // GENERATED from vectors/ai_grant_lifecycle/*.json by tools/gen_helix_tests.py.
    // Do not edit by hand; run `python3 tools/gen_helix_tests.py --write`
    // after changing the vectors (Track 0b: Rust<->Helix differential).

    // reject_lifecycle_bad_access_kind
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1), 3, 1, 0),
    ), mercury_lifecycle_bad_access_kind());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1), 3, 1, 0),
    ), mercury_lifecycle_audit_policy_reject());

    // reject_lifecycle_bad_active_revoke_reason
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 1, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 1, 100, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_bad_revoke_reason());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 1, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 1, 100, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_audit_policy_reject());

    // reject_lifecycle_bad_grant_state
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 3, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 3, 0, 100, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_bad_grant_state());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 3, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 3, 0, 100, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_audit_policy_reject());

    // reject_lifecycle_bad_revoked_no_reason
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 0, 100, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_bad_revoke_reason());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 0, 100, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_audit_policy_reject());

    // reject_lifecycle_bad_time
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, -1, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, -1, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_bad_time());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, -1, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, -1, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_audit_policy_reject());

    // reject_lifecycle_bad_version_v0
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(0, 1, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(0, 1, 0, 100, 200, 1), 1, 1, 0),
    ), mercury_lifecycle_bad_version());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(0, 1, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(0, 1, 0, 100, 200, 1), 1, 1, 0),
    ), mercury_lifecycle_audit_policy_reject());

    // reject_lifecycle_expired_after_expiry
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 201, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 201, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_grant_expired());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 201, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 201, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_audit_expired_reject());

    // reject_lifecycle_expired_at_boundary
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 200, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 200, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_grant_expired());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 200, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 200, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_audit_expired_reject());

    // reject_lifecycle_highsec_read_after_revoke_epoch_rotated
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 3),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 3), 1, 3, 1),
    ), mercury_lifecycle_read_after_revoke());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 3),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 3), 1, 3, 1),
    ), mercury_lifecycle_audit_post_revoke_access_reject());

    // reject_lifecycle_highsec_revoke_epoch_rotation_required
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 3),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 3), 1, 3, 0),
    ), mercury_lifecycle_highsec_epoch_rotation_required());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 3),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 3), 1, 3, 0),
    ), mercury_lifecycle_audit_highsec_epoch_rotation_required());

    // reject_lifecycle_read_after_revoke
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1), 1, 1, 0),
    ), mercury_lifecycle_read_after_revoke());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1), 1, 1, 0),
    ), mercury_lifecycle_audit_post_revoke_access_reject());

    // reject_lifecycle_revoked_user
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_grant_revoked());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1), 0, 1, 0),
    ), mercury_lifecycle_audit_revoked_reject());

    // reject_lifecycle_write_after_revoke
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1), 2, 1, 0),
    ), mercury_lifecycle_write_after_revoke());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 2, 1, 100, 200, 1), 2, 1, 0),
    ), mercury_lifecycle_audit_post_revoke_access_reject());

    // valid_active_read_before_expiry_v1
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1), 1, 1, 0),
    ), mercury_lifecycle_accept());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1), 1, 1, 0),
    ), mercury_lifecycle_audit_accepted());

    // valid_active_write_before_expiry_v1
    failed = failed + expect(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1), 2, 1, 0),
    ), mercury_lifecycle_accept());
    failed = failed + expect_audit(mercury_lifecycle_first_reject(
        mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1),
        mercury_lifecycle_validate_access_v1(mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1), 2, 1, 0),
    ), mercury_lifecycle_audit_accepted());

    if failed == 0 { 42 } else { failed }
}
