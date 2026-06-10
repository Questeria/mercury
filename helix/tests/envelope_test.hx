// Mercury Phase 1 envelope policy tests.
//
// This file duplicates helix/policy/envelope.hx until the Helix build lane has
// a stable Mercury-local module/import story. It should remain scalar-only and
// vector-friendly so Rust and host-language mirrors can match it exactly.

@pure
fn mercury_accept() -> i32 { 0 }

@pure
fn mercury_bad_version() -> i32 { 1 }

@pure
fn mercury_unsupported_suite() -> i32 { 2 }

@pure
fn mercury_downgraded_suite() -> i32 { 3 }

@pure
fn mercury_bad_conversation_id_len() -> i32 { 4 }

@pure
fn mercury_bad_sender_account_id_len() -> i32 { 5 }

@pure
fn mercury_bad_sender_device_id_len() -> i32 { 6 }

@pure
fn mercury_bad_epoch() -> i32 { 7 }

@pure
fn mercury_bad_sequence() -> i32 { 8 }

@pure
fn mercury_bad_message_kind() -> i32 { 9 }

@pure
fn mercury_payload_too_large() -> i32 { 10 }

@pure
fn mercury_unknown_critical_flag() -> i32 { 11 }

@pure
fn mercury_reserved_ai_kind() -> i32 { 12 }

@pure
fn mercury_audit_accepted_message() -> i32 { 1 }

@pure
fn mercury_audit_policy_reject() -> i32 { 2 }

@pure
fn mercury_audit_downgrade_attempt() -> i32 { 3 }

@pure
fn mercury_audit_size_reject() -> i32 { 4 }

@pure
fn mercury_supported_version(version: i32) -> bool {
    version == 1
}

@pure
fn mercury_supported_suite(suite_id: i32) -> bool {
    if suite_id == 257 {
        true
    } else {
        suite_id == 513
    }
}

@pure
fn mercury_valid_id_len(id_len: i32) -> bool {
    if id_len < 8 {
        false
    } else {
        id_len <= 128
    }
}

@pure
fn mercury_known_message_kind(message_kind: i32) -> bool {
    if message_kind == 1 {
        true
    } else {
        if message_kind == 2 {
            true
        } else {
            message_kind == 3
        }
    }
}

@pure
fn mercury_valid_payload_len(payload_len: i32, max_payload_len: i32) -> bool {
    if payload_len < 0 {
        false
    } else {
        if max_payload_len < 0 {
            false
        } else {
            payload_len <= max_payload_len
        }
    }
}

@pure
fn mercury_known_critical_flags(critical_flags: i32) -> bool {
    if critical_flags < 0 {
        false
    } else {
        critical_flags <= 3
    }
}

@pure
fn mercury_validate_identity_v1(
    version: i32,
    suite_id: i32,
    min_suite_id: i32,
    conversation_id_len: i32,
    sender_account_id_len: i32,
    sender_device_id_len: i32,
) -> i32 {
    if mercury_supported_version(version) == false {
        mercury_bad_version()
    } else {
        if mercury_supported_suite(suite_id) == false {
            mercury_unsupported_suite()
        } else {
            if suite_id < min_suite_id {
                mercury_downgraded_suite()
            } else {
                if mercury_valid_id_len(conversation_id_len) == false {
                    mercury_bad_conversation_id_len()
                } else {
                    if mercury_valid_id_len(sender_account_id_len) == false {
                        mercury_bad_sender_account_id_len()
                    } else {
                        if mercury_valid_id_len(sender_device_id_len) == false {
                            mercury_bad_sender_device_id_len()
                        } else {
                            mercury_accept()
                        }
                    }
                }
            }
        }
    }
}

@pure
fn mercury_validate_order_v1(
    epoch: i32,
    sequence: i32,
    expected_epoch: i32,
    expected_sequence: i32,
) -> i32 {
    if epoch == expected_epoch {
        if sequence == expected_sequence {
            mercury_accept()
        } else {
            mercury_bad_sequence()
        }
    } else {
        mercury_bad_epoch()
    }
}

@pure
fn mercury_validate_content_v1(
    message_kind: i32,
    payload_len: i32,
    critical_flags: i32,
    noncritical_flags: i32,
    max_payload_len: i32,
) -> i32 {
    if mercury_known_message_kind(message_kind) == false {
        mercury_bad_message_kind()
    } else {
        if message_kind == 3 {
            mercury_reserved_ai_kind()
        } else {
            if mercury_valid_payload_len(payload_len, max_payload_len) == false {
                mercury_payload_too_large()
            } else {
                if mercury_known_critical_flags(critical_flags) == false {
                    mercury_unknown_critical_flag()
                } else {
                    mercury_accept()
                }
            }
        }
    }
}

@pure
fn mercury_first_reject(identity_reason: i32, order_reason: i32, content_reason: i32) -> i32 {
    if identity_reason == mercury_accept() {
        if order_reason == mercury_accept() {
            content_reason
        } else {
            order_reason
        }
    } else {
        identity_reason
    }
}

@pure
fn mercury_audit_class_for_reason(reason_code: i32) -> i32 {
    if reason_code == mercury_accept() {
        mercury_audit_accepted_message()
    } else {
        if reason_code == mercury_downgraded_suite() {
            mercury_audit_downgrade_attempt()
        } else {
            if reason_code == mercury_payload_too_large() {
                mercury_audit_size_reject()
            } else {
                mercury_audit_policy_reject()
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
    expect(mercury_audit_class_for_reason(reason_code), expected)
}

fn main() -> i32 {
    let mut failed: i32 = 0;

    // GENERATED from vectors/envelope/*.json by tools/gen_helix_tests.py.
    // Do not edit by hand; run `python3 tools/gen_helix_tests.py --write`
    // after changing the vectors (Track 0b: Rust<->Helix differential).

    // reject_bad_epoch
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(8, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_bad_epoch());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(8, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_audit_policy_reject());

    // reject_bad_sequence
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 41, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_bad_sequence());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 41, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_audit_policy_reject());

    // reject_bad_version_v0
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(0, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_bad_version());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(0, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_audit_policy_reject());

    // reject_downgrade_classical_below_floor
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 257, 513, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_downgraded_suite());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 257, 513, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_audit_downgrade_attempt());

    // reject_empty_sender_device_id
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 0),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_bad_sender_device_id_len());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 0),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_audit_policy_reject());

    // reject_oversize_payload
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 65537, 0, 0, 65536),
    ), mercury_payload_too_large());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 65537, 0, 0, 65536),
    ), mercury_audit_size_reject());

    // reject_reserved_ai_context_kind_phase1
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(3, 1, 0, 0, 65536),
    ), mercury_reserved_ai_kind());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(3, 1, 0, 0, 65536),
    ), mercury_audit_policy_reject());

    // reject_unknown_critical_flag
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 4, 0, 65536),
    ), mercury_unknown_critical_flag());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 4, 0, 65536),
    ), mercury_audit_policy_reject());

    // reject_unknown_suite
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 999, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_unsupported_suite());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 999, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_audit_policy_reject());

    // valid_empty_payload_control_v1
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(2, 0, 0, 0, 65536),
    ), mercury_accept());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(2, 0, 0, 0, 65536),
    ), mercury_audit_accepted_message());

    // valid_minimal_application_v1
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_accept());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 0, 65536),
    ), mercury_audit_accepted_message());

    // valid_unknown_noncritical_flag_v1
    failed = failed + expect(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 2048, 65536),
    ), mercury_accept());
    failed = failed + expect_audit(mercury_first_reject(
        mercury_validate_identity_v1(1, 513, 257, 16, 32, 32),
        mercury_validate_order_v1(7, 42, 7, 42),
        mercury_validate_content_v1(1, 1, 0, 2048, 65536),
    ), mercury_audit_accepted_message());

    if failed == 0 { 42 } else { failed }
}
