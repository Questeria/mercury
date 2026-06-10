// Mercury Phase 1 AI grant policy tests.

@pure
fn mercury_ai_grant_accept() -> i32 { 0 }

@pure
fn mercury_ai_grant_bad_version() -> i32 { 1 }

@pure
fn mercury_ai_grant_bad_principal_kind() -> i32 { 2 }

@pure
fn mercury_ai_grant_bad_ai_mode() -> i32 { 3 }

@pure
fn mercury_ai_grant_ai_blocked_room() -> i32 { 4 }

@pure
fn mercury_ai_grant_bad_ttl() -> i32 { 5 }

@pure
fn mercury_ai_grant_ttl_too_long() -> i32 { 6 }

@pure
fn mercury_ai_grant_insufficient_approvers() -> i32 { 7 }

@pure
fn mercury_ai_grant_bad_read_scope() -> i32 { 8 }

@pure
fn mercury_ai_grant_read_scope_too_broad() -> i32 { 9 }

@pure
fn mercury_ai_grant_bad_write_scope() -> i32 { 10 }

@pure
fn mercury_ai_grant_write_scope_too_broad() -> i32 { 11 }

@pure
fn mercury_ai_grant_bad_tool_scope() -> i32 { 12 }

@pure
fn mercury_ai_grant_tool_scope_too_broad() -> i32 { 13 }

@pure
fn mercury_ai_grant_bad_retention_mode() -> i32 { 14 }

@pure
fn mercury_ai_grant_prompt_store_forbidden() -> i32 { 15 }

@pure
fn mercury_ai_grant_training_forbidden() -> i32 { 16 }

@pure
fn mercury_ai_grant_remote_provider_forbidden() -> i32 { 17 }

@pure
fn mercury_ai_grant_highsec_local_only() -> i32 { 18 }

@pure
fn mercury_ai_grant_highsec_confirm_send_required() -> i32 { 19 }

@pure
fn mercury_ai_grant_highsec_tools_forbidden() -> i32 { 20 }

@pure
fn mercury_ai_grant_audit_accepted() -> i32 { 1 }

@pure
fn mercury_ai_grant_audit_policy_reject() -> i32 { 2 }

@pure
fn mercury_ai_grant_audit_privacy_reject() -> i32 { 3 }

@pure
fn mercury_ai_grant_audit_highsec_reject() -> i32 { 4 }

@pure
fn mercury_ai_grant_audit_retention_reject() -> i32 { 5 }

@pure
fn mercury_ai_grant_audit_tool_reject() -> i32 { 6 }

@pure
fn mercury_ai_grant_valid_ai_mode(ai_mode: i32) -> bool {
    if ai_mode < 1 { false } else { ai_mode <= 3 }
}

@pure
fn mercury_ai_grant_validate_subject_tail(room_mode: i32, ai_mode: i32, ttl_s: i32, approver_count: i32) -> i32 {
    if room_mode < 1 {
        mercury_ai_grant_ai_blocked_room()
    } else {
        if room_mode > 4 {
            mercury_ai_grant_ai_blocked_room()
        } else {
            if room_mode == 4 {
                mercury_ai_grant_ai_blocked_room()
            } else {
                if ttl_s < 1 {
                    mercury_ai_grant_bad_ttl()
                } else {
                    if ttl_s > 900 {
                        mercury_ai_grant_ttl_too_long()
                    } else {
                        if approver_count < 1 {
                            mercury_ai_grant_insufficient_approvers()
                        } else {
                            if ai_mode == 3 {
                                if room_mode > 1 {
                                    mercury_ai_grant_remote_provider_forbidden()
                                } else {
                                    mercury_ai_grant_accept()
                                }
                            } else {
                                mercury_ai_grant_accept()
                            }
                        }
                    }
                }
            }
        }
    }
}

@pure
fn mercury_ai_grant_validate_subject_v1(
    version: i32,
    principal_kind: i32,
    room_mode: i32,
    ai_mode: i32,
    ttl_s: i32,
    approver_count: i32,
) -> i32 {
    if version != 1 {
        mercury_ai_grant_bad_version()
    } else {
        if principal_kind < 2 {
            mercury_ai_grant_bad_principal_kind()
        } else {
            if principal_kind > 4 {
                mercury_ai_grant_bad_principal_kind()
            } else {
                if mercury_ai_grant_valid_ai_mode(ai_mode) == false {
                    mercury_ai_grant_bad_ai_mode()
                } else {
                    if principal_kind == 4 {
                        if ai_mode == 1 {
                            mercury_ai_grant_bad_ai_mode()
                        } else {
                            mercury_ai_grant_validate_subject_tail(room_mode, ai_mode, ttl_s, approver_count)
                        }
                    } else {
                        mercury_ai_grant_validate_subject_tail(room_mode, ai_mode, ttl_s, approver_count)
                    }
                }
            }
        }
    }
}

@pure
fn mercury_ai_grant_validate_scopes_retention(retention_mode: i32, training_allowed: i32, prompt_store_allowed: i32) -> i32 {
    if retention_mode < 0 {
        mercury_ai_grant_bad_retention_mode()
    } else {
        if retention_mode > 3 {
            mercury_ai_grant_bad_retention_mode()
        } else {
            if retention_mode > 1 {
                mercury_ai_grant_prompt_store_forbidden()
            } else {
                if prompt_store_allowed != 0 {
                    mercury_ai_grant_prompt_store_forbidden()
                } else {
                    if training_allowed != 0 {
                        mercury_ai_grant_training_forbidden()
                    } else {
                        mercury_ai_grant_accept()
                    }
                }
            }
        }
    }
}

@pure
fn mercury_ai_grant_validate_scopes_tool(
    tool_scope: i32,
    retention_mode: i32,
    training_allowed: i32,
    prompt_store_allowed: i32,
) -> i32 {
    if tool_scope < 0 {
        mercury_ai_grant_bad_tool_scope()
    } else {
        if tool_scope > 4 {
            mercury_ai_grant_bad_tool_scope()
        } else {
            if tool_scope > 2 {
                mercury_ai_grant_tool_scope_too_broad()
            } else {
                mercury_ai_grant_validate_scopes_retention(retention_mode, training_allowed, prompt_store_allowed)
            }
        }
    }
}

@pure
fn mercury_ai_grant_validate_scopes_write(
    write_scope: i32,
    tool_scope: i32,
    retention_mode: i32,
    training_allowed: i32,
    prompt_store_allowed: i32,
) -> i32 {
    if write_scope < 0 {
        mercury_ai_grant_bad_write_scope()
    } else {
        if write_scope > 3 {
            mercury_ai_grant_bad_write_scope()
        } else {
            if write_scope == 3 {
                mercury_ai_grant_write_scope_too_broad()
            } else {
                mercury_ai_grant_validate_scopes_tool(tool_scope, retention_mode, training_allowed, prompt_store_allowed)
            }
        }
    }
}

@pure
fn mercury_ai_grant_validate_scopes_v1(
    read_scope: i32,
    write_scope: i32,
    tool_scope: i32,
    retention_mode: i32,
    training_allowed: i32,
    prompt_store_allowed: i32,
) -> i32 {
    if read_scope < 0 {
        mercury_ai_grant_bad_read_scope()
    } else {
        if read_scope > 4 {
            mercury_ai_grant_bad_read_scope()
        } else {
            if read_scope == 4 {
                mercury_ai_grant_read_scope_too_broad()
            } else {
                mercury_ai_grant_validate_scopes_write(write_scope, tool_scope, retention_mode, training_allowed, prompt_store_allowed)
            }
        }
    }
}

@pure
fn mercury_ai_grant_validate_highsec_v1(
    room_mode: i32,
    principal_kind: i32,
    ai_mode: i32,
    write_scope: i32,
    tool_scope: i32,
    approver_count: i32,
) -> i32 {
    if room_mode == 3 {
        if ai_mode != 1 {
            mercury_ai_grant_highsec_local_only()
        } else {
            if approver_count < 2 {
                mercury_ai_grant_insufficient_approvers()
            } else {
                if write_scope == 3 {
                    mercury_ai_grant_highsec_confirm_send_required()
                } else {
                    if tool_scope > 1 {
                        mercury_ai_grant_highsec_tools_forbidden()
                    } else {
                        mercury_ai_grant_accept()
                    }
                }
            }
        }
    } else {
        mercury_ai_grant_accept()
    }
}

@pure
fn mercury_ai_grant_first_reject(subject_reason: i32, scope_reason: i32, highsec_reason: i32) -> i32 {
    if subject_reason == mercury_ai_grant_accept() {
        if scope_reason == mercury_ai_grant_accept() { highsec_reason } else { scope_reason }
    } else {
        subject_reason
    }
}

@pure
fn mercury_ai_grant_audit_class_for_reason(reason_code: i32) -> i32 {
    if reason_code == mercury_ai_grant_accept() {
        mercury_ai_grant_audit_accepted()
    } else {
        if reason_code == mercury_ai_grant_remote_provider_forbidden() {
            mercury_ai_grant_audit_privacy_reject()
        } else {
            if reason_code == mercury_ai_grant_read_scope_too_broad() {
                mercury_ai_grant_audit_privacy_reject()
            } else {
                if reason_code >= mercury_ai_grant_highsec_local_only() {
                    mercury_ai_grant_audit_highsec_reject()
                } else {
                    if reason_code == mercury_ai_grant_prompt_store_forbidden() {
                        mercury_ai_grant_audit_retention_reject()
                    } else {
                        if reason_code == mercury_ai_grant_training_forbidden() {
                            mercury_ai_grant_audit_retention_reject()
                        } else {
                            if reason_code == mercury_ai_grant_bad_tool_scope() {
                                mercury_ai_grant_audit_tool_reject()
                            } else {
                                if reason_code == mercury_ai_grant_tool_scope_too_broad() {
                                    mercury_ai_grant_audit_tool_reject()
                                } else {
                                    mercury_ai_grant_audit_policy_reject()
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
    expect(mercury_ai_grant_audit_class_for_reason(reason_code), expected)
}

fn main() -> i32 {
    let mut failed: i32 = 0;

    // GENERATED from vectors/ai_grant/*.json by tools/gen_helix_tests.py.
    // Do not edit by hand; run `python3 tools/gen_helix_tests.py --write`
    // after changing the vectors (Track 0b: Rust<->Helix differential).

    // reject_ai_blocked_room
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 4, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(4, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_ai_blocked_room());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 4, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(4, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_policy_reject());

    // reject_ai_grant_bad_version_v0
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(0, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_bad_version());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(0, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_policy_reject());

    // reject_autonomous_send_phase1
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 3, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 3, 0, 1),
    ), mercury_ai_grant_write_scope_too_broad());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 3, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 3, 0, 1),
    ), mercury_ai_grant_audit_policy_reject());

    // reject_full_room_history_phase1
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(4, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_read_scope_too_broad());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(4, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_privacy_reject());

    // reject_highsec_external_tool
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 1, 900, 2),
        mercury_ai_grant_validate_scopes_v1(1, 1, 2, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 1, 1, 2, 2),
    ), mercury_ai_grant_highsec_tools_forbidden());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 1, 900, 2),
        mercury_ai_grant_validate_scopes_v1(1, 1, 2, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 1, 1, 2, 2),
    ), mercury_ai_grant_audit_highsec_reject());

    // reject_highsec_remote_enclave
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 2, 900, 2),
        mercury_ai_grant_validate_scopes_v1(1, 1, 1, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 2, 1, 1, 2),
    ), mercury_ai_grant_highsec_local_only());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 2, 900, 2),
        mercury_ai_grant_validate_scopes_v1(1, 1, 1, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 2, 1, 1, 2),
    ), mercury_ai_grant_audit_highsec_reject());

    // reject_highsec_single_approver
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 1, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 1, 1, 1, 1),
    ), mercury_ai_grant_insufficient_approvers());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 1, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 1, 1, 1, 1),
    ), mercury_ai_grant_audit_policy_reject());

    // reject_human_principal_for_ai_grant
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 1, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 1, 1, 1, 0, 1),
    ), mercury_ai_grant_bad_principal_kind());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 1, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 1, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_policy_reject());

    // reject_no_approvers
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 0),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 0),
    ), mercury_ai_grant_insufficient_approvers());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 0),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 0),
    ), mercury_ai_grant_audit_policy_reject());

    // reject_open_world_tool_phase1
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 4, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 4, 1),
    ), mercury_ai_grant_tool_scope_too_broad());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 4, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 4, 1),
    ), mercury_ai_grant_audit_tool_reject());

    // reject_prompt_store_phase1
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 2, 0, 1),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_prompt_store_forbidden());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 2, 0, 1),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_retention_reject());

    // reject_remote_provider_highsec_room
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 3, 900, 2),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 3, 1, 0, 2),
    ), mercury_ai_grant_remote_provider_forbidden());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 3, 900, 2),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 3, 1, 0, 2),
    ), mercury_ai_grant_audit_privacy_reject());

    // reject_remote_provider_sensitive_room
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 2, 3, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(2, 2, 3, 1, 0, 1),
    ), mercury_ai_grant_remote_provider_forbidden());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 2, 3, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(2, 2, 3, 1, 0, 1),
    ), mercury_ai_grant_audit_privacy_reject());

    // reject_training_allowed
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 1, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_training_forbidden());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 1, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_retention_reject());

    // reject_ttl_over_900
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 901, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_ttl_too_long());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 901, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_policy_reject());

    // reject_zero_ttl
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 0, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_bad_ttl());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 0, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_policy_reject());

    // valid_local_selected_draft_highsec_two_approvers_v1
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 1, 300, 2),
        mercury_ai_grant_validate_scopes_v1(1, 1, 1, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 1, 1, 1, 2),
    ), mercury_ai_grant_accept());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 3, 1, 300, 2),
        mercury_ai_grant_validate_scopes_v1(1, 1, 1, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(3, 2, 1, 1, 1, 2),
    ), mercury_ai_grant_audit_accepted());

    // valid_local_selected_draft_standard_v1
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_accept());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 1, 1, 900, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_accepted());

    // valid_remote_enclave_selected_readonly_standard_v1
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 3, 1, 2, 600, 1),
        mercury_ai_grant_validate_scopes_v1(1, 0, 1, 1, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 3, 2, 0, 1, 1),
    ), mercury_ai_grant_accept());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 3, 1, 2, 600, 1),
        mercury_ai_grant_validate_scopes_v1(1, 0, 1, 1, 0, 0),
        mercury_ai_grant_validate_highsec_v1(1, 3, 2, 0, 1, 1),
    ), mercury_ai_grant_audit_accepted());

    // valid_sensitive_local_selected_no_tools_v1
    failed = failed + expect(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 2, 1, 300, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(2, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_accept());
    failed = failed + expect_audit(mercury_ai_grant_first_reject(
        mercury_ai_grant_validate_subject_v1(1, 2, 2, 1, 300, 1),
        mercury_ai_grant_validate_scopes_v1(1, 1, 0, 0, 0, 0),
        mercury_ai_grant_validate_highsec_v1(2, 2, 1, 1, 0, 1),
    ), mercury_ai_grant_audit_accepted());

    if failed == 0 { 42 } else { failed }
}
