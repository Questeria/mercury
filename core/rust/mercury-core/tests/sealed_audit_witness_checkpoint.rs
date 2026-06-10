use mercury_core::{
    SealedAuditAnchorKind, SealedAuditCheckpointSignatureAlgorithm, SealedAuditEnvelopeSuite,
    SealedAuditEventChainInput, SealedAuditEventKind, SealedAuditEventStoreDecision,
    SealedAuditEventStoreReason, SealedAuditEventStoreWrite, SealedAuditWitnessCheckpointDecision,
    SealedAuditWitnessCheckpointInput, SealedAuditWitnessCheckpointReason,
    evaluate_sealed_audit_event_chain, evaluate_sealed_audit_witness_checkpoint,
    put_sealed_audit_event_record,
};

const EVENT_HASH: [u8; 32] = [0x91; 32];
const PREVIOUS_EVENT_HASH: [u8; 32] = [0x92; 32];
const RECORD_DIGEST: [u8; 32] = [0x93; 32];
const MERKLE_ROOT_HASH: [u8; 32] = [0x94; 32];
const CHECKPOINT_ID: [u8; 32] = [0x95; 32];
const CHECKPOINT_SIGNATURE: [u8; 2484] = [0x96; 2484];
const TRANSPARENCY_RECEIPT: [u8; 96] = [0x97; 96];
const WITNESS_RECEIPT: [u8; 96] = [0x98; 96];

#[test]
fn witness_checkpoint_accepts_persisted_pq_hybrid_checkpoint_publication() {
    let decision = evaluate_sealed_audit_witness_checkpoint(valid_input());

    assert!(decision.accepted);
    assert_eq!(
        decision.reason,
        SealedAuditWitnessCheckpointReason::Accepted
    );
    assert_eq!(
        decision.anchor_kind,
        SealedAuditAnchorKind::WitnessedTransparencyLog
    );
    assert_eq!(
        decision.signature_algorithm,
        SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44
    );
    assert_eq!(decision.store_event_sequence, 42);
    assert_eq!(decision.checkpoint_size, 43);
    assert_eq!(decision.witness_threshold, 2);
    assert!(decision.can_publish_checkpoint);
    assert!(decision.can_request_witness_cosignature);
    assert!(decision.can_monitor_privately);
    assert!(decision.can_detect_split_view);
    assert!(!decision.requires_witness_repair);
    assert!(!decision.requires_key_rotation);
    assert!(!decision.requires_user_warning);
    assert!(!decision.requires_local_recovery);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn witness_checkpoint_rejects_unpersisted_store_and_non_witness_anchors() {
    let store_rejected = SealedAuditWitnessCheckpointInput {
        store_decision: rejected_store_decision(),
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_witness_checkpoint(store_rejected),
        SealedAuditWitnessCheckpointReason::StoreRejected,
    );

    for anchor_kind in [
        SealedAuditAnchorKind::LocalHashChain,
        SealedAuditAnchorKind::PrivateMerkleLog,
        SealedAuditAnchorKind::Unknown,
    ] {
        let local_anchor = SealedAuditWitnessCheckpointInput {
            anchor_kind,
            ..valid_input()
        };
        assert_rejected(
            evaluate_sealed_audit_witness_checkpoint(local_anchor),
            SealedAuditWitnessCheckpointReason::AnchorRejected,
        );
    }
}

#[test]
fn witness_checkpoint_rejects_bad_checkpoint_shape_and_stale_checkpoints() {
    let bad_origin = SealedAuditWitnessCheckpointInput {
        checkpoint_origin_len: 0,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_witness_checkpoint(bad_origin),
        SealedAuditWitnessCheckpointReason::LogOriginRejected,
    );

    let bad_root = SealedAuditWitnessCheckpointInput {
        checkpoint_root_hash_len: 16,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_witness_checkpoint(bad_root),
        SealedAuditWitnessCheckpointReason::CheckpointShapeRejected,
    );

    let stale = SealedAuditWitnessCheckpointInput {
        checkpoint_size: 43,
        previous_checkpoint_size: 43,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_witness_checkpoint(stale),
        SealedAuditWitnessCheckpointReason::StaleCheckpoint,
    );
}

#[test]
fn witness_checkpoint_rejects_weak_or_bad_signing_keys() {
    let classical_only = SealedAuditWitnessCheckpointInput {
        signature_algorithm: SealedAuditCheckpointSignatureAlgorithm::Ed25519,
        checkpoint_signature_len: 64,
        witness_cosignature_bytes: 152,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_checkpoint(classical_only),
        SealedAuditWitnessCheckpointReason::SigningKeyRejected,
        false,
        true,
        false,
    );

    let expired = SealedAuditWitnessCheckpointInput {
        signing_key_not_expired: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_checkpoint(expired),
        SealedAuditWitnessCheckpointReason::SigningKeyRejected,
        false,
        true,
        false,
    );

    let rotation = SealedAuditWitnessCheckpointInput {
        previous_signing_key_retained_for_verification: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_checkpoint(rotation),
        SealedAuditWitnessCheckpointReason::KeyRotationRejected,
        false,
        true,
        false,
    );
}

#[test]
fn witness_checkpoint_rejects_consistency_and_quorum_failures() {
    let consistency = SealedAuditWitnessCheckpointInput {
        consistency_proof_verified: false,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_witness_checkpoint(consistency),
        SealedAuditWitnessCheckpointReason::ConsistencyProofRejected,
    );

    let too_many_hashes = SealedAuditWitnessCheckpointInput {
        consistency_proof_hash_count: 64,
        ..valid_input()
    };
    assert_rejected(
        evaluate_sealed_audit_witness_checkpoint(too_many_hashes),
        SealedAuditWitnessCheckpointReason::ConsistencyProofRejected,
    );

    let quorum = SealedAuditWitnessCheckpointInput {
        witness_count: 1,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_checkpoint(quorum),
        SealedAuditWitnessCheckpointReason::WitnessQuorumRejected,
        true,
        false,
        false,
    );

    let pins = SealedAuditWitnessCheckpointInput {
        witness_key_pins_present: false,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_checkpoint(pins),
        SealedAuditWitnessCheckpointReason::WitnessQuorumRejected,
        true,
        false,
        false,
    );
}

#[test]
fn witness_checkpoint_rejects_split_view_and_privacy_leaking_monitors() {
    let split_view = SealedAuditWitnessCheckpointInput {
        split_view_evidence_present: true,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_checkpoint(split_view),
        SealedAuditWitnessCheckpointReason::SplitViewEvidence,
        true,
        false,
        true,
    );

    let privacy = SealedAuditWitnessCheckpointInput {
        monitor_query_uses_private_retrieval: false,
        monitor_query_plaintext_selectors: 1,
        monitor_receives_only_digests: false,
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_witness_checkpoint(privacy);
    assert_rejected(
        decision,
        SealedAuditWitnessCheckpointReason::MonitorPrivacyRejected,
    );
    assert!(decision.plaintext_bytes_exposed);
}

#[test]
fn witness_checkpoint_accepts_authenticated_local_recovery_only() {
    let recoverable = SealedAuditWitnessCheckpointInput {
        local_latest_checkpoint_available: false,
        recovery_checkpoint_authenticated: true,
        recovery_requires_user_verification: true,
        ..valid_input()
    };
    let decision = evaluate_sealed_audit_witness_checkpoint(recoverable);
    assert!(decision.accepted);
    assert!(decision.requires_local_recovery);
    assert!(decision.can_publish_checkpoint);

    let unauthenticated = SealedAuditWitnessCheckpointInput {
        local_latest_checkpoint_available: false,
        recovery_checkpoint_authenticated: false,
        recovery_requires_user_verification: true,
        ..valid_input()
    };
    assert_rejected_with(
        evaluate_sealed_audit_witness_checkpoint(unauthenticated),
        SealedAuditWitnessCheckpointReason::RecoveryStateRejected,
        false,
        false,
        true,
    );
}

#[test]
fn witness_checkpoint_reasons_and_algorithms_have_stable_codes_and_labels() {
    let reasons = [
        (SealedAuditWitnessCheckpointReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditWitnessCheckpointReason::StoreRejected,
            1,
            "STORE_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::AnchorRejected,
            2,
            "ANCHOR_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::CheckpointShapeRejected,
            3,
            "CHECKPOINT_SHAPE_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::LogOriginRejected,
            4,
            "LOG_ORIGIN_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::SigningKeyRejected,
            5,
            "SIGNING_KEY_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::KeyRotationRejected,
            6,
            "KEY_ROTATION_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::ConsistencyProofRejected,
            7,
            "CONSISTENCY_PROOF_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::WitnessQuorumRejected,
            8,
            "WITNESS_QUORUM_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::StaleCheckpoint,
            9,
            "STALE_CHECKPOINT",
        ),
        (
            SealedAuditWitnessCheckpointReason::SplitViewEvidence,
            10,
            "SPLIT_VIEW_EVIDENCE",
        ),
        (
            SealedAuditWitnessCheckpointReason::MonitorPrivacyRejected,
            11,
            "MONITOR_PRIVACY_REJECTED",
        ),
        (
            SealedAuditWitnessCheckpointReason::RecoveryStateRejected,
            12,
            "RECOVERY_STATE_REJECTED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let algorithms = [
        (
            SealedAuditCheckpointSignatureAlgorithm::Ed25519,
            1,
            "ed25519",
        ),
        (
            SealedAuditCheckpointSignatureAlgorithm::EcdsaP256Sha256,
            2,
            "ecdsa_p256_sha256",
        ),
        (
            SealedAuditCheckpointSignatureAlgorithm::MlDsa44,
            3,
            "ml_dsa_44",
        ),
        (
            SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44,
            4,
            "hybrid_ed25519_ml_dsa_44",
        ),
        (
            SealedAuditCheckpointSignatureAlgorithm::Unknown,
            5,
            "unknown",
        ),
    ];

    for (algorithm, code, label) in algorithms {
        assert_eq!(algorithm.code(), code);
        assert_eq!(algorithm.label(), label);
    }
}

fn assert_rejected(
    decision: SealedAuditWitnessCheckpointDecision,
    reason: SealedAuditWitnessCheckpointReason,
) {
    assert_rejected_with(decision, reason, false, false, false);
}

fn assert_rejected_with(
    decision: SealedAuditWitnessCheckpointDecision,
    reason: SealedAuditWitnessCheckpointReason,
    requires_witness_repair: bool,
    requires_key_rotation: bool,
    requires_user_warning: bool,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.can_publish_checkpoint);
    assert!(!decision.can_request_witness_cosignature);
    assert!(!decision.can_monitor_privately);
    assert!(decision.can_detect_split_view);
    assert_eq!(decision.requires_witness_repair, requires_witness_repair);
    assert_eq!(decision.requires_key_rotation, requires_key_rotation);
    assert_eq!(decision.requires_user_warning, requires_user_warning);
}

fn valid_input() -> SealedAuditWitnessCheckpointInput {
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
        local_latest_checkpoint_available: true,
        recovery_checkpoint_authenticated: false,
        recovery_requires_user_verification: false,
    }
}

fn accepted_store_decision() -> SealedAuditEventStoreDecision {
    let mut store = mercury_core::PrototypeSealedAuditEventStore::default();
    put_sealed_audit_event_record(&mut store, valid_store_write())
        .expect("prototype sealed audit event store is infallible")
}

fn rejected_store_decision() -> SealedAuditEventStoreDecision {
    SealedAuditEventStoreDecision {
        accepted: false,
        reason: SealedAuditEventStoreReason::ChainRejected,
        persisted_record: false,
        record_count: 0,
        event_sequence: 42,
        can_publish_receipt: false,
        can_detect_replay: true,
        append_only: true,
        keeps_digest_only: true,
        keeps_plaintext_metadata: false,
        plaintext_bytes_exposed: false,
    }
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
