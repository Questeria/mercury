// Mercury relay submission policy tests.

@pure
fn mercury_relay_accept() -> i32 { 0 }

@pure
fn mercury_relay_bad_version() -> i32 { 1 }

@pure
fn mercury_relay_bad_send_gate_reason() -> i32 { 2 }

@pure
fn mercury_relay_send_gate_rejected() -> i32 { 3 }

@pure
fn mercury_relay_bad_route_id_len() -> i32 { 4 }

@pure
fn mercury_relay_bad_replay_token_len() -> i32 { 5 }

@pure
fn mercury_relay_bad_ttl() -> i32 { 6 }

@pure
fn mercury_relay_ttl_too_long() -> i32 { 7 }

@pure
fn mercury_relay_bad_ciphertext_len() -> i32 { 8 }

@pure
fn mercury_relay_ciphertext_too_large() -> i32 { 9 }

@pure
fn mercury_relay_bad_sealed_header_len() -> i32 { 10 }

@pure
fn mercury_relay_sealed_header_too_large() -> i32 { 11 }

@pure
fn mercury_relay_plaintext_identity_forbidden() -> i32 { 12 }

@pure
fn mercury_relay_bad_padding_bucket() -> i32 { 13 }

@pure
fn mercury_relay_audit_accepted() -> i32 { 1 }

@pure
fn mercury_relay_audit_contract_reject() -> i32 { 2 }

@pure
fn mercury_relay_audit_client_send_reject() -> i32 { 3 }

@pure
fn mercury_relay_audit_metadata_reject() -> i32 { 4 }

@pure
fn mercury_relay_audit_retention_reject() -> i32 { 5 }

@pure
fn mercury_relay_audit_size_reject() -> i32 { 6 }

@pure
fn mercury_relay_validate_send_gate(send_gate_reason: i32) -> i32 {
    if send_gate_reason < 0 {
        mercury_relay_bad_send_gate_reason()
    } else {
        if send_gate_reason > 5 {
            mercury_relay_bad_send_gate_reason()
        } else {
            if send_gate_reason != 0 {
                mercury_relay_send_gate_rejected()
            } else {
                mercury_relay_accept()
            }
        }
    }
}

@pure
fn mercury_relay_validate_metadata(
    route_id_len: i32,
    replay_token_len: i32,
    sealed_header_len: i32,
    plaintext_identity_fields: i32,
    padding_bucket: i32,
) -> i32 {
    if route_id_len < 16 {
        mercury_relay_bad_route_id_len()
    } else {
        if route_id_len > 128 {
            mercury_relay_bad_route_id_len()
        } else {
            if replay_token_len != 32 {
                mercury_relay_bad_replay_token_len()
            } else {
                if sealed_header_len < 16 {
                    mercury_relay_bad_sealed_header_len()
                } else {
                    if sealed_header_len > 4096 {
                        mercury_relay_sealed_header_too_large()
                    } else {
                        if plaintext_identity_fields != 0 {
                            mercury_relay_plaintext_identity_forbidden()
                        } else {
                            if padding_bucket < 1 {
                                mercury_relay_bad_padding_bucket()
                            } else {
                                if padding_bucket > 8 {
                                    mercury_relay_bad_padding_bucket()
                                } else {
                                    mercury_relay_accept()
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
fn mercury_relay_validate_lifetime(queue_ttl_s: i32, max_queue_ttl_s: i32) -> i32 {
    if queue_ttl_s < 1 {
        mercury_relay_bad_ttl()
    } else {
        if max_queue_ttl_s < 1 {
            mercury_relay_bad_ttl()
        } else {
            if max_queue_ttl_s > 604800 {
                mercury_relay_ttl_too_long()
            } else {
                if queue_ttl_s > max_queue_ttl_s {
                    mercury_relay_ttl_too_long()
                } else {
                    mercury_relay_accept()
                }
            }
        }
    }
}

@pure
fn mercury_relay_validate_ciphertext(ciphertext_len: i32, max_ciphertext_len: i32) -> i32 {
    if ciphertext_len < 1 {
        mercury_relay_bad_ciphertext_len()
    } else {
        if max_ciphertext_len < 1 {
            mercury_relay_bad_ciphertext_len()
        } else {
            if max_ciphertext_len > 4194304 {
                mercury_relay_ciphertext_too_large()
            } else {
                if ciphertext_len > max_ciphertext_len {
                    mercury_relay_ciphertext_too_large()
                } else {
                    mercury_relay_accept()
                }
            }
        }
    }
}

@pure
fn mercury_relay_first_reject(
    version_reason: i32,
    send_gate_reason: i32,
    metadata_reason: i32,
    lifetime_reason: i32,
    ciphertext_reason: i32,
) -> i32 {
    if version_reason != 0 {
        version_reason
    } else {
        if send_gate_reason != 0 {
            send_gate_reason
        } else {
            if metadata_reason != 0 {
                metadata_reason
            } else {
                if lifetime_reason != 0 {
                    lifetime_reason
                } else {
                    ciphertext_reason
                }
            }
        }
    }
}

@pure
fn mercury_relay_decide_v1(
    version: i32,
    send_gate_reason: i32,
    metadata_reason: i32,
    lifetime_reason: i32,
    ciphertext_reason: i32,
) -> i32 {
    let version_reason = if version == 1 { mercury_relay_accept() } else { mercury_relay_bad_version() };
    mercury_relay_first_reject(
        version_reason,
        mercury_relay_validate_send_gate(send_gate_reason),
        metadata_reason,
        lifetime_reason,
        ciphertext_reason,
    )
}

@pure
fn mercury_relay_audit_class_for_reason(reason_code: i32) -> i32 {
    if reason_code == mercury_relay_accept() {
        mercury_relay_audit_accepted()
    } else {
        if reason_code == mercury_relay_bad_version() {
            mercury_relay_audit_contract_reject()
        } else {
            if reason_code == mercury_relay_bad_send_gate_reason() {
                mercury_relay_audit_contract_reject()
            } else {
                if reason_code == mercury_relay_send_gate_rejected() {
                    mercury_relay_audit_client_send_reject()
                } else {
                    if reason_code == mercury_relay_bad_ttl() {
                        mercury_relay_audit_retention_reject()
                    } else {
                        if reason_code == mercury_relay_ttl_too_long() {
                            mercury_relay_audit_retention_reject()
                        } else {
                            if reason_code == mercury_relay_bad_ciphertext_len() {
                                mercury_relay_audit_size_reject()
                            } else {
                                if reason_code == mercury_relay_ciphertext_too_large() {
                                    mercury_relay_audit_size_reject()
                                } else {
                                    mercury_relay_audit_metadata_reject()
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
    expect(mercury_relay_audit_class_for_reason(reason_code), expected)
}

@pure
fn valid_metadata() -> i32 {
    mercury_relay_validate_metadata(32, 32, 96, 0, 3)
}

@pure
fn valid_lifetime() -> i32 {
    mercury_relay_validate_lifetime(300, 86400)
}

@pure
fn valid_ciphertext() -> i32 {
    mercury_relay_validate_ciphertext(2048, 1048576)
}

fn main() -> i32 {
    let mut failed: i32 = 0;

    // GENERATED from vectors/relay_submit/*.json by tools/gen_helix_tests.py.
    // Do not edit by hand; run `python3 tools/gen_helix_tests.py --write`
    // after changing the vectors (Track 0b: Rust<->Helix differential).

    // reject_relay_bad_ciphertext_len
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(0, 1048576),
    ), mercury_relay_bad_ciphertext_len());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(0, 1048576),
    ), mercury_relay_audit_size_reject());

    // reject_relay_bad_padding_bucket
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 9),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_bad_padding_bucket());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 9),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_metadata_reject());

    // reject_relay_bad_replay_token_len
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 16, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_bad_replay_token_len());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 16, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_metadata_reject());

    // reject_relay_bad_route_id_len
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(8, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_bad_route_id_len());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(8, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_metadata_reject());

    // reject_relay_bad_sealed_header_len
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 8, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_bad_sealed_header_len());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 8, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_metadata_reject());

    // reject_relay_bad_send_gate_reason
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        6,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_bad_send_gate_reason());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        6,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_contract_reject());

    // reject_relay_bad_ttl_zero
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(0, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_bad_ttl());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(0, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_retention_reject());

    // reject_relay_bad_version_v0
    failed = failed + expect(mercury_relay_decide_v1(
        0,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_bad_version());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        0,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_contract_reject());

    // reject_relay_ciphertext_too_large
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(1048577, 1048576),
    ), mercury_relay_ciphertext_too_large());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(1048577, 1048576),
    ), mercury_relay_audit_size_reject());

    // reject_relay_plaintext_identity
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 1, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_plaintext_identity_forbidden());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 1, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_metadata_reject());

    // reject_relay_sealed_header_too_large
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 4097, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_sealed_header_too_large());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 4097, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_metadata_reject());

    // reject_relay_send_gate_rejected
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        4,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_send_gate_rejected());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        4,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_client_send_reject());

    // reject_relay_ttl_too_long
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(90000, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_ttl_too_long());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(90000, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_retention_reject());

    // valid_relay_submit_max_ttl_v1
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(64, 32, 128, 0, 8),
        mercury_relay_validate_lifetime(604800, 604800),
        mercury_relay_validate_ciphertext(4096, 4194304),
    ), mercury_relay_accept());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(64, 32, 128, 0, 8),
        mercury_relay_validate_lifetime(604800, 604800),
        mercury_relay_validate_ciphertext(4096, 4194304),
    ), mercury_relay_audit_accepted());

    // valid_relay_submit_standard_v1
    failed = failed + expect(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_accept());
    failed = failed + expect_audit(mercury_relay_decide_v1(
        1,
        0,
        mercury_relay_validate_metadata(32, 32, 96, 0, 3),
        mercury_relay_validate_lifetime(300, 86400),
        mercury_relay_validate_ciphertext(2048, 1048576),
    ), mercury_relay_audit_accepted());

    if failed == 0 { 42 } else { failed }
}
