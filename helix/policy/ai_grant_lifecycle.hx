// Mercury Phase 1 AI grant lifecycle policy.
//
// This policy decides whether an already-granted AI access token is still
// usable. Expiry is derived from now_s >= expires_at_s. Revoked high-security
// grants report the required epoch rotation before post-revoke access reasons.

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

fn main() -> i32 {
    let state = mercury_lifecycle_validate_state_v1(1, 1, 0, 100, 200, 1);
    mercury_lifecycle_first_reject(
        state,
        mercury_lifecycle_validate_access_v1(state, 1, 1, 0),
    )
}
