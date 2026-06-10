// Mercury Phase 1 policy pipeline tests.

@pure
fn mercury_pipeline_accept() -> i32 { 0 }

@pure
fn mercury_pipeline_bad_version() -> i32 { 1 }

@pure
fn mercury_pipeline_bad_actor_kind() -> i32 { 2 }

@pure
fn mercury_pipeline_bad_envelope_reason() -> i32 { 3 }

@pure
fn mercury_pipeline_bad_room_epoch_reason() -> i32 { 4 }

@pure
fn mercury_pipeline_bad_ai_grant_reason() -> i32 { 5 }

@pure
fn mercury_pipeline_bad_ai_lifecycle_reason() -> i32 { 6 }

@pure
fn mercury_pipeline_ai_component_for_human() -> i32 { 7 }

@pure
fn mercury_pipeline_envelope_reject() -> i32 { 8 }

@pure
fn mercury_pipeline_room_epoch_reject() -> i32 { 9 }

@pure
fn mercury_pipeline_ai_grant_reject() -> i32 { 10 }

@pure
fn mercury_pipeline_ai_lifecycle_reject() -> i32 { 11 }

@pure
fn mercury_pipeline_ai_post_revoke_access_reject() -> i32 { 12 }

@pure
fn mercury_pipeline_ai_highsec_rotation_required() -> i32 { 13 }

@pure
fn mercury_pipeline_audit_accepted() -> i32 { 1 }

@pure
fn mercury_pipeline_audit_contract_reject() -> i32 { 2 }

@pure
fn mercury_pipeline_audit_message_policy_reject() -> i32 { 3 }

@pure
fn mercury_pipeline_audit_room_policy_reject() -> i32 { 4 }

@pure
fn mercury_pipeline_audit_ai_policy_reject() -> i32 { 5 }

@pure
fn mercury_pipeline_audit_ai_lifecycle_policy_reject() -> i32 { 6 }

@pure
fn mercury_pipeline_audit_ai_post_revoke_reject() -> i32 { 7 }

@pure
fn mercury_pipeline_audit_ai_highsec_rotation_reject() -> i32 { 8 }

@pure
fn mercury_pipeline_validate_ai_component_reasons(ai_grant_reason: i32, ai_lifecycle_reason: i32) -> i32 {
    if ai_grant_reason < 0 {
        mercury_pipeline_bad_ai_grant_reason()
    } else {
        if ai_grant_reason > 20 {
            mercury_pipeline_bad_ai_grant_reason()
        } else {
            if ai_lifecycle_reason < 0 {
                mercury_pipeline_bad_ai_lifecycle_reason()
            } else {
                if ai_lifecycle_reason > 10 {
                    mercury_pipeline_bad_ai_lifecycle_reason()
                } else {
                    mercury_pipeline_accept()
                }
            }
        }
    }
}

@pure
fn mercury_pipeline_validate_component_reasons(
    envelope_reason: i32,
    room_epoch_reason: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
) -> i32 {
    if envelope_reason < 0 {
        mercury_pipeline_bad_envelope_reason()
    } else {
        if envelope_reason > 12 {
            mercury_pipeline_bad_envelope_reason()
        } else {
            if room_epoch_reason < 0 {
                mercury_pipeline_bad_room_epoch_reason()
            } else {
                if room_epoch_reason > 13 {
                    mercury_pipeline_bad_room_epoch_reason()
                } else {
                    mercury_pipeline_validate_ai_component_reasons(ai_grant_reason, ai_lifecycle_reason)
                }
            }
        }
    }
}

@pure
fn mercury_pipeline_validate_inputs_v1(
    version: i32,
    actor_kind: i32,
    envelope_reason: i32,
    room_epoch_reason: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
) -> i32 {
    if version != 1 {
        mercury_pipeline_bad_version()
    } else {
        if actor_kind < 1 {
            mercury_pipeline_bad_actor_kind()
        } else {
            if actor_kind > 3 {
                mercury_pipeline_bad_actor_kind()
            } else {
                mercury_pipeline_validate_component_reasons(envelope_reason, room_epoch_reason, ai_grant_reason, ai_lifecycle_reason)
            }
        }
    }
}

@pure
fn mercury_pipeline_validate_actor_components(
    actor_kind: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
) -> i32 {
    if actor_kind == 1 {
        if ai_grant_reason != 0 {
            mercury_pipeline_ai_component_for_human()
        } else {
            if ai_lifecycle_reason != 0 {
                mercury_pipeline_ai_component_for_human()
            } else {
                mercury_pipeline_accept()
            }
        }
    } else {
        mercury_pipeline_accept()
    }
}

@pure
fn mercury_pipeline_map_ai_lifecycle(ai_lifecycle_reason: i32) -> i32 {
    if ai_lifecycle_reason == 0 {
        mercury_pipeline_accept()
    } else {
        if ai_lifecycle_reason == 9 {
            mercury_pipeline_ai_highsec_rotation_required()
        } else {
            if ai_lifecycle_reason == 7 {
                mercury_pipeline_ai_post_revoke_access_reject()
            } else {
                if ai_lifecycle_reason == 8 {
                    mercury_pipeline_ai_post_revoke_access_reject()
                } else {
                    mercury_pipeline_ai_lifecycle_reject()
                }
            }
        }
    }
}

@pure
fn mercury_pipeline_component_first_reject(
    envelope_reason: i32,
    room_epoch_reason: i32,
    actor_component_reason: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
) -> i32 {
    if envelope_reason != 0 {
        mercury_pipeline_envelope_reject()
    } else {
        if room_epoch_reason != 0 {
            mercury_pipeline_room_epoch_reject()
        } else {
            if actor_component_reason != 0 {
                actor_component_reason
            } else {
                if ai_grant_reason != 0 {
                    mercury_pipeline_ai_grant_reject()
                } else {
                    mercury_pipeline_map_ai_lifecycle(ai_lifecycle_reason)
                }
            }
        }
    }
}

@pure
fn mercury_pipeline_decide_v1(
    version: i32,
    actor_kind: i32,
    envelope_reason: i32,
    room_epoch_reason: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
) -> i32 {
    let input_reason = mercury_pipeline_validate_inputs_v1(version, actor_kind, envelope_reason, room_epoch_reason, ai_grant_reason, ai_lifecycle_reason);
    if input_reason != 0 {
        input_reason
    } else {
        mercury_pipeline_component_first_reject(
            envelope_reason,
            room_epoch_reason,
            mercury_pipeline_validate_actor_components(actor_kind, ai_grant_reason, ai_lifecycle_reason),
            ai_grant_reason,
            ai_lifecycle_reason,
        )
    }
}

@pure
fn mercury_pipeline_audit_class_for_reason(reason_code: i32) -> i32 {
    if reason_code == mercury_pipeline_accept() {
        mercury_pipeline_audit_accepted()
    } else {
        if reason_code < mercury_pipeline_ai_component_for_human() {
            mercury_pipeline_audit_contract_reject()
        } else {
            if reason_code == mercury_pipeline_ai_component_for_human() {
                mercury_pipeline_audit_ai_policy_reject()
            } else {
                if reason_code == mercury_pipeline_envelope_reject() {
                    mercury_pipeline_audit_message_policy_reject()
                } else {
                    if reason_code == mercury_pipeline_room_epoch_reject() {
                        mercury_pipeline_audit_room_policy_reject()
                    } else {
                        if reason_code == mercury_pipeline_ai_grant_reject() {
                            mercury_pipeline_audit_ai_policy_reject()
                        } else {
                            if reason_code == mercury_pipeline_ai_lifecycle_reject() {
                                mercury_pipeline_audit_ai_lifecycle_policy_reject()
                            } else {
                                if reason_code == mercury_pipeline_ai_post_revoke_access_reject() {
                                    mercury_pipeline_audit_ai_post_revoke_reject()
                                } else {
                                    if reason_code == mercury_pipeline_ai_highsec_rotation_required() {
                                        mercury_pipeline_audit_ai_highsec_rotation_reject()
                                    } else {
                                        mercury_pipeline_audit_contract_reject()
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

@pure
fn expect(actual: i32, expected: i32) -> i32 {
    if actual == expected { 0 } else { 1 }
}

@pure
fn expect_audit(reason_code: i32, expected: i32) -> i32 {
    expect(mercury_pipeline_audit_class_for_reason(reason_code), expected)
}

fn main() -> i32 {
    let mut failed: i32 = 0;

    // GENERATED from vectors/policy_pipeline/*.json by tools/gen_helix_tests.py.
    // Do not edit by hand; run `python3 tools/gen_helix_tests.py --write`
    // after changing the vectors (Track 0b: Rust<->Helix differential).

    // reject_pipeline_ai_component_for_human
    failed = failed + expect(mercury_pipeline_decide_v1(1, 1, 0, 0, 5, 0), mercury_pipeline_ai_component_for_human());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 1, 0, 0, 5, 0), mercury_pipeline_audit_ai_policy_reject());

    // reject_pipeline_ai_grant_policy_reject
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 0, 0, 17, 0), mercury_pipeline_ai_grant_reject());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 0, 0, 17, 0), mercury_pipeline_audit_ai_policy_reject());

    // reject_pipeline_ai_highsec_rotation_required
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 9), mercury_pipeline_ai_highsec_rotation_required());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 9), mercury_pipeline_audit_ai_highsec_rotation_reject());

    // reject_pipeline_ai_lifecycle_expired
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 5), mercury_pipeline_ai_lifecycle_reject());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 5), mercury_pipeline_audit_ai_lifecycle_policy_reject());

    // reject_pipeline_ai_post_revoke_read
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 7), mercury_pipeline_ai_post_revoke_access_reject());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 7), mercury_pipeline_audit_ai_post_revoke_reject());

    // reject_pipeline_bad_actor_kind
    failed = failed + expect(mercury_pipeline_decide_v1(1, 4, 0, 0, 0, 0), mercury_pipeline_bad_actor_kind());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 4, 0, 0, 0, 0), mercury_pipeline_audit_contract_reject());

    // reject_pipeline_bad_ai_grant_reason
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 0, 0, 21, 0), mercury_pipeline_bad_ai_grant_reason());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 0, 0, 21, 0), mercury_pipeline_audit_contract_reject());

    // reject_pipeline_bad_ai_lifecycle_reason
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 11), mercury_pipeline_bad_ai_lifecycle_reason());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 11), mercury_pipeline_audit_contract_reject());

    // reject_pipeline_bad_envelope_reason
    failed = failed + expect(mercury_pipeline_decide_v1(1, 1, 13, 0, 0, 0), mercury_pipeline_bad_envelope_reason());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 1, 13, 0, 0, 0), mercury_pipeline_audit_contract_reject());

    // reject_pipeline_bad_room_epoch_reason
    failed = failed + expect(mercury_pipeline_decide_v1(1, 1, 0, 14, 0, 0), mercury_pipeline_bad_room_epoch_reason());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 1, 0, 14, 0, 0), mercury_pipeline_audit_contract_reject());

    // reject_pipeline_bad_version_v0
    failed = failed + expect(mercury_pipeline_decide_v1(0, 1, 0, 0, 0, 0), mercury_pipeline_bad_version());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(0, 1, 0, 0, 0, 0), mercury_pipeline_audit_contract_reject());

    // reject_pipeline_envelope_policy_reject
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 8, 0, 0, 0), mercury_pipeline_envelope_reject());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 8, 0, 0, 0), mercury_pipeline_audit_message_policy_reject());

    // reject_pipeline_room_epoch_policy_reject
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 0, 10, 0, 0), mercury_pipeline_room_epoch_reject());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 0, 10, 0, 0), mercury_pipeline_audit_room_policy_reject());

    // valid_pipeline_ai_message_v1
    failed = failed + expect(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 0), mercury_pipeline_accept());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 2, 0, 0, 0, 0), mercury_pipeline_audit_accepted());

    // valid_pipeline_human_message_v1
    failed = failed + expect(mercury_pipeline_decide_v1(1, 1, 0, 0, 0, 0), mercury_pipeline_accept());
    failed = failed + expect_audit(mercury_pipeline_decide_v1(1, 1, 0, 0, 0, 0), mercury_pipeline_audit_accepted());

    if failed == 0 { 42 } else { failed }
}
