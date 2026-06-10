// Mercury relay submission policy.
//
// This policy defines the metadata and final client decision required before
// the server relay may accept an encrypted queue item. It does not inspect
// plaintext, decrypt payloads, or decide message policy.

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
