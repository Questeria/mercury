// Mercury Phase 1 policy pipeline.
//
// This policy composes the existing policy slices into one stable decision.
// It does not inspect raw messages or cryptographic material; callers provide
// component reason codes from the envelope, room epoch, AI grant, and AI grant
// lifecycle validators.

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

fn main() -> i32 {
    mercury_pipeline_decide_v1(1, 1, 0, 0, 0, 0)
}
