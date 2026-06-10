use mercury_core::{
    PrototypeSealedAuditEventStore, SealedAuditAnchorKind, SealedAuditCheckpointSignatureAlgorithm,
    SealedAuditEnvelopeSuite, SealedAuditEventChainInput, SealedAuditEventKind,
    SealedAuditEventStoreWrite, SealedAuditWitnessCheckpointDecision,
    SealedAuditWitnessCheckpointInput, SealedAuditWitnessClientDecision,
    SealedAuditWitnessClientInput, SealedAuditWitnessClientReason,
    evaluate_sealed_audit_event_chain, evaluate_sealed_audit_witness_checkpoint,
    evaluate_sealed_audit_witness_client, put_sealed_audit_event_record,
};

const EVENT_HASH: [u8; 32] = [0xA1; 32];
const PREVIOUS_EVENT_HASH: [u8; 32] = [0xA2; 32];
const RECORD_DIGEST: [u8; 32] = [0xA3; 32];
const MERKLE_ROOT_HASH: [u8; 32] = [0xA4; 32];
const CHECKPOINT_ID: [u8; 32] = [0xA5; 32];
const CHECKPOINT_SIGNATURE: [u8; 2484] = [0xA6; 2484];
const TRANSPARENCY_RECEIPT: [u8; 96] = [0xA7; 96];
const WITNESS_RECEIPT: [u8; 96] = [0xA8; 96];

#[test]
fn witness_client_accepts_policy_bound_atomic_private_monitoring_flow() {
    let decision = evaluate_sealed_audit_witness_client(valid_input());

    assert!(decision.accepted);
    assert_eq!(decision.reason, SealedAuditWitnessClientReason::Accepted);
    assert_eq!(decision.checkpoint_size, 43);
    assert_eq!(decision.policy_epoch, 7);
    assert_eq!(decision.witness_quorum_threshold, 2);
    assert_eq!(decision.response_status_code, 200);
    assert!(decision.can_submit_add_checkpoint);
    assert!(decision.can_publish_witnessed_checkpoint);
    assert!(decision.can_monitor_privately);
    assert!(!decision.can_retry_witness_conflict);
    assert!(decision.can_alert_split_view);
    assert!(!decision.requires_policy_rotation);
    assert!(!decision.requires_witness_repair);
    assert!(!decision.requires_operator_alert);
    assert!(!decision.requires_local_recovery);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn witness_client_rejects_failed_checkpoint_gate_and_bad_policy() {
    let bad_checkpoint = SealedAuditWitnessClientInput {
        checkpoint_decision: rejected_checkpoint_decision(),
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_witness_client(bad_checkpoint),
        SealedAuditWitnessClientReason::CheckpointGateRejected,
    );

    let stale_policy = SealedAuditWitnessClientInput {
        policy_not_expired: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(stale_policy),
        SealedAuditWitnessClientReason::PolicyRejected,
        true,
        false,
        false,
    );

    let insufficient_pins = SealedAuditWitnessClientInput {
        witness_key_pin_count: 1,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(insufficient_pins),
        SealedAuditWitnessClientReason::PolicyRejected,
        true,
        false,
        false,
    );
}

#[test]
fn witness_client_rejects_endpoint_and_request_shape_failures() {
    let endpoint = SealedAuditWitnessClientInput {
        endpoint_tls_pins_present: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(endpoint),
        SealedAuditWitnessClientReason::EndpointRejected,
        false,
        true,
        false,
    );

    let proof_too_large = SealedAuditWitnessClientInput {
        request_consistency_proof_hash_count: 64,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_witness_client(proof_too_large),
        SealedAuditWitnessClientReason::RequestShapeRejected,
    );

    let plaintext_selector = SealedAuditWitnessClientInput {
        request_body_plaintext_selector_count: 1,
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_witness_client(plaintext_selector);
    assert_rejected(
        decision,
        SealedAuditWitnessClientReason::RequestShapeRejected,
    );
    assert!(decision.plaintext_bytes_exposed);
}

#[test]
fn witness_client_maps_conflict_unavailability_and_split_view_alerts() {
    let conflict = SealedAuditWitnessClientInput {
        response_status_code: 409,
        response_latest_size: 41,
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_witness_client(conflict);
    assert_rejected_with(
        decision,
        SealedAuditWitnessClientReason::WitnessConflict,
        false,
        true,
        true,
    );
    assert!(decision.can_retry_witness_conflict);

    let unavailable = SealedAuditWitnessClientInput {
        response_status_code: 503,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(unavailable),
        SealedAuditWitnessClientReason::WitnessUnavailable,
        false,
        true,
        false,
    );

    let bad_proof = SealedAuditWitnessClientInput {
        response_status_code: 422,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(bad_proof),
        SealedAuditWitnessClientReason::SplitViewAlert,
        false,
        false,
        true,
    );
}

#[test]
fn witness_client_rejects_bad_response_or_non_atomic_persistence() {
    let unknown_cosigs = SealedAuditWitnessClientInput {
        response_known_cosignature_count: 1,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(unknown_cosigs),
        SealedAuditWitnessClientReason::WitnessResponseRejected,
        false,
        true,
        false,
    );

    let non_atomic = SealedAuditWitnessClientInput {
        persist_latest_checkpoint_atomically: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(non_atomic),
        SealedAuditWitnessClientReason::WitnessResponseRejected,
        false,
        true,
        false,
    );

    let no_alert_route = SealedAuditWitnessClientInput {
        split_view_alert_delivery_configured: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(no_alert_route),
        SealedAuditWitnessClientReason::SplitViewAlert,
        false,
        false,
        true,
    );
}

#[test]
fn witness_client_rejects_privacy_leaking_monitor_queries_and_bad_recovery() {
    let monitor = SealedAuditWitnessClientInput {
        monitor_query_uses_private_retrieval: false,
        monitor_query_uses_vrf_or_blinded_selector: false,
        monitor_query_plaintext_selectors: 1,
        monitor_receives_only_digests: false,
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_witness_client(monitor);
    assert_rejected(
        decision,
        SealedAuditWitnessClientReason::MonitorPrivacyRejected,
    );
    assert!(decision.plaintext_bytes_exposed);

    let recovery = SealedAuditWitnessClientInput {
        checkpoint_decision: recovery_checkpoint_decision(),
        recovery_checkpoint_authenticated: false,
        recovery_requires_user_verification: true,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_client(recovery),
        SealedAuditWitnessClientReason::RecoveryRejected,
        false,
        false,
        true,
    );

    let accepted_recovery = SealedAuditWitnessClientInput {
        checkpoint_decision: recovery_checkpoint_decision(),
        recovery_checkpoint_authenticated: true,
        recovery_requires_user_verification: true,
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_witness_client(accepted_recovery);
    assert!(decision.accepted);
    assert!(decision.requires_local_recovery);
}

#[test]
fn witness_client_reasons_have_stable_codes_and_labels() {
    let cases = [
        (SealedAuditWitnessClientReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditWitnessClientReason::CheckpointGateRejected,
            1,
            "CHECKPOINT_GATE_REJECTED",
        ),
        (
            SealedAuditWitnessClientReason::PolicyRejected,
            2,
            "POLICY_REJECTED",
        ),
        (
            SealedAuditWitnessClientReason::EndpointRejected,
            3,
            "ENDPOINT_REJECTED",
        ),
        (
            SealedAuditWitnessClientReason::RequestShapeRejected,
            4,
            "REQUEST_SHAPE_REJECTED",
        ),
        (
            SealedAuditWitnessClientReason::WitnessConflict,
            5,
            "WITNESS_CONFLICT",
        ),
        (
            SealedAuditWitnessClientReason::WitnessUnavailable,
            6,
            "WITNESS_UNAVAILABLE",
        ),
        (
            SealedAuditWitnessClientReason::WitnessResponseRejected,
            7,
            "WITNESS_RESPONSE_REJECTED",
        ),
        (
            SealedAuditWitnessClientReason::SplitViewAlert,
            8,
            "SPLIT_VIEW_ALERT",
        ),
        (
            SealedAuditWitnessClientReason::MonitorPrivacyRejected,
            9,
            "MONITOR_PRIVACY_REJECTED",
        ),
        (
            SealedAuditWitnessClientReason::RecoveryRejected,
            10,
            "RECOVERY_REJECTED",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(
    decision: SealedAuditWitnessClientDecision,
    reason: SealedAuditWitnessClientReason,
) {
    assert_rejected_with(decision, reason, false, false, false);
}

fn assert_rejected_with(
    decision: SealedAuditWitnessClientDecision,
    reason: SealedAuditWitnessClientReason,
    requires_policy_rotation: bool,
    requires_witness_repair: bool,
    requires_operator_alert: bool,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.can_submit_add_checkpoint);
    assert!(!decision.can_publish_witnessed_checkpoint);
    assert!(!decision.can_monitor_privately);
    assert!(decision.can_alert_split_view);
    assert_eq!(decision.requires_policy_rotation, requires_policy_rotation);
    assert_eq!(decision.requires_witness_repair, requires_witness_repair);
    assert_eq!(decision.requires_operator_alert, requires_operator_alert);
}

fn valid_input() -> SealedAuditWitnessClientInput {
    SealedAuditWitnessClientInput {
        checkpoint_decision: accepted_checkpoint_decision(),
        policy_digest_len: 32,
        policy_epoch: 7,
        policy_not_expired: true,
        policy_binds_log_origin: true,
        policy_binds_witness_operators: true,
        log_public_key_pin_count: 1,
        witness_key_pin_count: 3,
        witness_operator_count: 3,
        witness_quorum_threshold: 2,
        submission_endpoint_count: 3,
        monitor_endpoint_count: 2,
        endpoints_use_https_or_bastion: true,
        endpoint_tls_pins_present: true,
        request_old_size: 42,
        request_checkpoint_size: 43,
        request_consistency_proof_hash_count: 6,
        request_body_binds_policy_epoch: true,
        request_body_plaintext_selector_count: 0,
        response_status_code: 200,
        response_latest_size: 43,
        response_cosignature_count: 3,
        response_known_cosignature_count: 3,
        response_operator_count: 3,
        response_cosignatures_timestamped: true,
        response_cosignatures_bind_checkpoint: true,
        persist_latest_checkpoint_atomically: true,
        split_view_alert_delivery_configured: true,
        monitor_query_uses_private_retrieval: true,
        monitor_query_uses_vrf_or_blinded_selector: true,
        monitor_query_plaintext_selectors: 0,
        monitor_receives_only_digests: true,
        recovery_checkpoint_authenticated: false,
        recovery_requires_user_verification: false,
    }
}

fn accepted_checkpoint_decision() -> SealedAuditWitnessCheckpointDecision {
    evaluate_sealed_audit_witness_checkpoint(valid_checkpoint_input(false))
}

fn rejected_checkpoint_decision() -> SealedAuditWitnessCheckpointDecision {
    let mut input = valid_checkpoint_input(false);
    input.witness_count = 1;
    evaluate_sealed_audit_witness_checkpoint(input)
}

fn recovery_checkpoint_decision() -> SealedAuditWitnessCheckpointDecision {
    evaluate_sealed_audit_witness_checkpoint(valid_checkpoint_input(true))
}

fn valid_checkpoint_input(recovery: bool) -> SealedAuditWitnessCheckpointInput {
    SealedAuditWitnessCheckpointInput {
        store_decision: accepted_store_decision(),
        anchor_kind: SealedAuditAnchorKind::WitnessedTransparencyLog,
        signature_algorithm: SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44,
        checkpoint_origin_len: 32,
        log_id_digest_len: 32,
        checkpoint_timestamp_s: 1_769_990_400,
        checkpoint_size: 43,
        previous_checkpoint_size: 42,
        checkpoint_root_hash_len: 32,
        checkpoint_signature_len: 2484,
        signing_key_id_digest_len: 32,
        signing_key_not_expired: true,
        signing_key_rotation_window_valid: true,
        previous_signing_key_retained_for_verification: true,
        consistency_proof_verified: true,
        consistency_proof_hash_count: 6,
        witness_count: 3,
        witness_threshold: 2,
        witness_operator_count: 3,
        witness_key_pins_present: true,
        witness_cosignature_bytes: 5016,
        cosignatures_timestamped: true,
        cosignatures_bind_checkpoint: true,
        split_view_evidence_present: false,
        monitor_query_uses_private_retrieval: true,
        monitor_query_plaintext_selectors: 0,
        monitor_receives_only_digests: true,
        local_latest_checkpoint_available: !recovery,
        recovery_checkpoint_authenticated: recovery,
        recovery_requires_user_verification: recovery,
    }
}

fn accepted_store_decision() -> mercury_core::SealedAuditEventStoreDecision {
    let mut store = PrototypeSealedAuditEventStore::default();
    put_sealed_audit_event_record(&mut store, valid_store_write())
        .expect("prototype sealed audit event store is infallible")
}

fn valid_store_write() -> SealedAuditEventStoreWrite<'static> {
    SealedAuditEventStoreWrite {
        chain_decision: accepted_chain_decision_for_sequence(42),
        event_sequence: 42,
        event_hash: &EVENT_HASH,
        previous_event_hash: &PREVIOUS_EVENT_HASH,
        record_digest: &RECORD_DIGEST,
        merkle_root_hash: &MERKLE_ROOT_HASH,
        checkpoint_id: &CHECKPOINT_ID,
        checkpoint_signature: &CHECKPOINT_SIGNATURE,
        transparency_receipt: &TRANSPARENCY_RECEIPT,
        witness_receipt: &WITNESS_RECEIPT,
        event_kind: SealedAuditEventKind::MlsCommit,
        anchor_kind: SealedAuditAnchorKind::WitnessedTransparencyLog,
        sealed_payload_len: 512,
        plaintext_metadata_fields: 0,
        append_only_guard: true,
        checkpoint_binds_chain: true,
        receipt_binds_checkpoint: true,
    }
}

fn accepted_chain_decision_for_sequence(
    sequence: i64,
) -> mercury_core::SealedAuditEventChainDecision {
    let mut input = valid_chain_input();
    input.event_sequence = sequence;
    input.previous_chain_size = sequence;
    input.previous_checkpoint_size = sequence;
    input.checkpoint_size = sequence + 1;
    input.checkpoint_signature_len = 2484;
    evaluate_sealed_audit_event_chain(input)
}

fn valid_chain_input() -> SealedAuditEventChainInput {
    SealedAuditEventChainInput {
        event_kind: SealedAuditEventKind::MlsCommit,
        anchor_kind: SealedAuditAnchorKind::WitnessedTransparencyLog,
        envelope_suite: SealedAuditEnvelopeSuite::XChaCha20Poly1305Blake3,
        event_sequence: 42,
        previous_chain_size: 42,
        previous_event_hash_len: 32,
        event_hash_len: 32,
        record_digest_len: 32,
        merkle_leaf_hash_len: 32,
        merkle_root_hash_len: 32,
        event_sealed: true,
        aad_binds_event_context: true,
        plaintext_field_count: 0,
        plaintext_payload_bytes: 0,
        monotonic_counter_present: true,
        monotonic_counter_increases: true,
        device_binding_digest_len: 32,
        actor_binding_digest_len: 32,
        epoch_binding_digest_len: 32,
        room_epoch_digest_len: 32,
        critical_event_bound: true,
        signed_checkpoint_present: true,
        checkpoint_signature_len: 2484,
        checkpoint_timestamp_s: 1_769_990_400,
        checkpoint_size: 43,
        previous_checkpoint_size: 42,
        inclusion_proof_verified: true,
        consistency_proof_verified: true,
        transparency_receipt_present: true,
        witness_count: 3,
        witness_threshold: 2,
        witness_operator_count: 2,
        storage_append_only: true,
        storage_transactional: true,
        rollback_resistant_store: true,
        local_store_sealed: true,
        forward_secret_rotated: true,
        previous_key_material_deleted: true,
    }
}
