use mercury_core::{
    SealedAuditAnchorKind, SealedAuditEnvelopeSuite, SealedAuditEventChainDecision,
    SealedAuditEventChainInput, SealedAuditEventChainReason, SealedAuditEventKind,
    evaluate_sealed_audit_event_chain,
};

#[test]
fn sealed_audit_accepts_witnessed_mls_commit_chain_event() {
    let decision = evaluate_sealed_audit_event_chain(valid_input());

    assert!(decision.accepted);
    assert_eq!(decision.reason, SealedAuditEventChainReason::Accepted);
    assert_eq!(decision.event_kind, SealedAuditEventKind::MlsCommit);
    assert_eq!(
        decision.anchor_kind,
        SealedAuditAnchorKind::WitnessedTransparencyLog
    );
    assert_eq!(
        decision.envelope_suite,
        SealedAuditEnvelopeSuite::XChaCha20Poly1305Blake3
    );
    assert!(decision.can_append_event);
    assert!(decision.can_verify_inclusion);
    assert!(decision.can_publish_transparency_receipt);
    assert!(decision.can_detect_rollback);
    assert!(decision.tamper_evident);
    assert!(decision.append_only);
    assert!(!decision.requires_storage_repair);
    assert!(!decision.requires_transparency_setup);
    assert!(!decision.requires_redaction);
    assert!(!decision.requires_key_rotation);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn sealed_audit_rejects_unknown_kind_anchor_and_unsealed_envelope() {
    let mut kind = valid_input();
    kind.event_kind = SealedAuditEventKind::Unknown;
    assert_rejected(
        evaluate_sealed_audit_event_chain(kind),
        SealedAuditEventChainReason::EventKindRejected,
    );

    let mut anchor = valid_input();
    anchor.anchor_kind = SealedAuditAnchorKind::Unknown;
    let anchor_decision = evaluate_sealed_audit_event_chain(anchor);
    assert_rejected(anchor_decision, SealedAuditEventChainReason::AnchorRejected);
    assert!(anchor_decision.requires_transparency_setup);

    let mut envelope = valid_input();
    envelope.envelope_suite = SealedAuditEnvelopeSuite::HmacSha256Only;
    let envelope_decision = evaluate_sealed_audit_event_chain(envelope);
    assert_rejected(
        envelope_decision,
        SealedAuditEventChainReason::EnvelopeSuiteRejected,
    );
    assert!(envelope_decision.requires_key_rotation);
}

#[test]
fn sealed_audit_forbids_plaintext_and_weak_digest_shapes() {
    let mut plaintext = valid_input();
    plaintext.plaintext_field_count = 1;
    let plaintext_decision = evaluate_sealed_audit_event_chain(plaintext);
    assert_rejected(
        plaintext_decision,
        SealedAuditEventChainReason::PlaintextEventForbidden,
    );
    assert!(plaintext_decision.requires_redaction);
    assert!(!plaintext_decision.plaintext_bytes_exposed);

    let mut payload = valid_input();
    payload.plaintext_payload_bytes = 32;
    assert_rejected(
        evaluate_sealed_audit_event_chain(payload),
        SealedAuditEventChainReason::PlaintextEventForbidden,
    );

    let mut digest = valid_input();
    digest.merkle_root_hash_len = 16;
    assert_rejected(
        evaluate_sealed_audit_event_chain(digest),
        SealedAuditEventChainReason::DigestShapeRejected,
    );
}

#[test]
fn sealed_audit_requires_sequence_previous_hash_and_monotonic_counter() {
    let mut sequence = valid_input();
    sequence.event_sequence = 41;
    assert_rejected(
        evaluate_sealed_audit_event_chain(sequence),
        SealedAuditEventChainReason::SequenceRejected,
    );

    let mut previous_hash = valid_input();
    previous_hash.previous_event_hash_len = 0;
    assert_rejected(
        evaluate_sealed_audit_event_chain(previous_hash),
        SealedAuditEventChainReason::PreviousHashMissing,
    );

    let mut counter = valid_input();
    counter.monotonic_counter_increases = false;
    let counter_decision = evaluate_sealed_audit_event_chain(counter);
    assert_rejected(
        counter_decision,
        SealedAuditEventChainReason::MonotonicCounterRejected,
    );
    assert!(counter_decision.requires_storage_repair);
}

#[test]
fn sealed_audit_requires_context_binding_seal_and_signed_checkpoint() {
    let mut binding = valid_input();
    binding.room_epoch_digest_len = 0;
    assert_rejected(
        evaluate_sealed_audit_event_chain(binding),
        SealedAuditEventChainReason::EventBindingMissing,
    );

    let mut critical = valid_input();
    critical.critical_event_bound = false;
    assert_rejected(
        evaluate_sealed_audit_event_chain(critical),
        SealedAuditEventChainReason::EventBindingMissing,
    );

    let mut seal = valid_input();
    seal.aad_binds_event_context = false;
    let seal_decision = evaluate_sealed_audit_event_chain(seal);
    assert_rejected(seal_decision, SealedAuditEventChainReason::SealMissing);
    assert!(seal_decision.requires_key_rotation);

    let mut checkpoint = valid_input();
    checkpoint.signed_checkpoint_present = false;
    let checkpoint_decision = evaluate_sealed_audit_event_chain(checkpoint);
    assert_rejected(
        checkpoint_decision,
        SealedAuditEventChainReason::CheckpointMissing,
    );
    assert!(checkpoint_decision.requires_transparency_setup);

    let mut signature = valid_input();
    signature.checkpoint_signature_len = 32;
    assert_rejected(
        evaluate_sealed_audit_event_chain(signature),
        SealedAuditEventChainReason::CheckpointSignatureMissing,
    );
}

#[test]
fn sealed_audit_requires_merkle_receipt_and_witness_quorum() {
    let mut proof = valid_input();
    proof.consistency_proof_verified = false;
    let proof_decision = evaluate_sealed_audit_event_chain(proof);
    assert_rejected(
        proof_decision,
        SealedAuditEventChainReason::MerkleProofMissing,
    );
    assert!(proof_decision.requires_transparency_setup);

    let mut receipt = valid_input();
    receipt.transparency_receipt_present = false;
    assert_rejected(
        evaluate_sealed_audit_event_chain(receipt),
        SealedAuditEventChainReason::TransparencyReceiptMissing,
    );

    let mut witness = valid_input();
    witness.witness_count = 1;
    let witness_decision = evaluate_sealed_audit_event_chain(witness);
    assert_rejected(
        witness_decision,
        SealedAuditEventChainReason::WitnessQuorumMissing,
    );
    assert!(witness_decision.requires_transparency_setup);
}

#[test]
fn sealed_audit_requires_append_only_storage_rollback_defense_and_forward_secrecy() {
    let mut append_only = valid_input();
    append_only.storage_append_only = false;
    let append_decision = evaluate_sealed_audit_event_chain(append_only);
    assert_rejected(
        append_decision,
        SealedAuditEventChainReason::AppendOnlyStorageMissing,
    );
    assert!(append_decision.requires_storage_repair);

    let mut rollback = valid_input();
    rollback.rollback_resistant_store = false;
    let rollback_decision = evaluate_sealed_audit_event_chain(rollback);
    assert_rejected(
        rollback_decision,
        SealedAuditEventChainReason::RollbackProtectionMissing,
    );
    assert!(rollback_decision.requires_storage_repair);

    let mut forward_secret = valid_input();
    forward_secret.previous_key_material_deleted = false;
    let forward_secret_decision = evaluate_sealed_audit_event_chain(forward_secret);
    assert_rejected(
        forward_secret_decision,
        SealedAuditEventChainReason::ForwardSecrecyMissing,
    );
    assert!(forward_secret_decision.requires_key_rotation);
}

#[test]
fn sealed_audit_codes_and_labels_are_stable() {
    let kinds = [
        (
            SealedAuditEventKind::DeviceKeyChange,
            1,
            "device_key_change",
        ),
        (SealedAuditEventKind::MlsCommit, 2, "mls_commit"),
        (SealedAuditEventKind::AccountRecovery, 3, "account_recovery"),
        (SealedAuditEventKind::BackupRestore, 4, "backup_restore"),
        (SealedAuditEventKind::AiGrant, 5, "ai_grant"),
        (
            SealedAuditEventKind::RelaySourceChange,
            6,
            "relay_source_change",
        ),
        (SealedAuditEventKind::MediaRetention, 7, "media_retention"),
        (SealedAuditEventKind::Unknown, 8, "unknown"),
    ];
    for (kind, code, label) in kinds {
        assert_eq!(kind.code(), code);
        assert_eq!(kind.label(), label);
    }

    let anchors = [
        (SealedAuditAnchorKind::LocalHashChain, 1, "local_hash_chain"),
        (
            SealedAuditAnchorKind::PrivateMerkleLog,
            2,
            "private_merkle_log",
        ),
        (
            SealedAuditAnchorKind::WitnessedTransparencyLog,
            3,
            "witnessed_transparency_log",
        ),
        (
            SealedAuditAnchorKind::PublicTransparencyLog,
            4,
            "public_transparency_log",
        ),
        (SealedAuditAnchorKind::Unknown, 5, "unknown"),
    ];
    for (anchor, code, label) in anchors {
        assert_eq!(anchor.code(), code);
        assert_eq!(anchor.label(), label);
    }

    let suites = [
        (
            SealedAuditEnvelopeSuite::XChaCha20Poly1305Blake3,
            1,
            "xchacha20_poly1305_blake3",
        ),
        (
            SealedAuditEnvelopeSuite::Aes256GcmHkdfSha256,
            2,
            "aes256_gcm_hkdf_sha256",
        ),
        (
            SealedAuditEnvelopeSuite::HmacSha256Only,
            3,
            "hmac_sha256_only",
        ),
        (SealedAuditEnvelopeSuite::Plaintext, 4, "plaintext"),
        (SealedAuditEnvelopeSuite::Unknown, 5, "unknown"),
    ];
    for (suite, code, label) in suites {
        assert_eq!(suite.code(), code);
        assert_eq!(suite.label(), label);
    }

    let reasons = [
        (SealedAuditEventChainReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditEventChainReason::EventKindRejected,
            1,
            "EVENT_KIND_REJECTED",
        ),
        (
            SealedAuditEventChainReason::AnchorRejected,
            2,
            "ANCHOR_REJECTED",
        ),
        (
            SealedAuditEventChainReason::EnvelopeSuiteRejected,
            3,
            "ENVELOPE_SUITE_REJECTED",
        ),
        (
            SealedAuditEventChainReason::PlaintextEventForbidden,
            4,
            "PLAINTEXT_EVENT_FORBIDDEN",
        ),
        (
            SealedAuditEventChainReason::DigestShapeRejected,
            5,
            "DIGEST_SHAPE_REJECTED",
        ),
        (
            SealedAuditEventChainReason::SequenceRejected,
            6,
            "SEQUENCE_REJECTED",
        ),
        (
            SealedAuditEventChainReason::PreviousHashMissing,
            7,
            "PREVIOUS_HASH_MISSING",
        ),
        (
            SealedAuditEventChainReason::MonotonicCounterRejected,
            8,
            "MONOTONIC_COUNTER_REJECTED",
        ),
        (
            SealedAuditEventChainReason::EventBindingMissing,
            9,
            "EVENT_BINDING_MISSING",
        ),
        (SealedAuditEventChainReason::SealMissing, 10, "SEAL_MISSING"),
        (
            SealedAuditEventChainReason::CheckpointMissing,
            11,
            "CHECKPOINT_MISSING",
        ),
        (
            SealedAuditEventChainReason::CheckpointSignatureMissing,
            12,
            "CHECKPOINT_SIGNATURE_MISSING",
        ),
        (
            SealedAuditEventChainReason::MerkleProofMissing,
            13,
            "MERKLE_PROOF_MISSING",
        ),
        (
            SealedAuditEventChainReason::TransparencyReceiptMissing,
            14,
            "TRANSPARENCY_RECEIPT_MISSING",
        ),
        (
            SealedAuditEventChainReason::WitnessQuorumMissing,
            15,
            "WITNESS_QUORUM_MISSING",
        ),
        (
            SealedAuditEventChainReason::AppendOnlyStorageMissing,
            16,
            "APPEND_ONLY_STORAGE_MISSING",
        ),
        (
            SealedAuditEventChainReason::RollbackProtectionMissing,
            17,
            "ROLLBACK_PROTECTION_MISSING",
        ),
        (
            SealedAuditEventChainReason::ForwardSecrecyMissing,
            18,
            "FORWARD_SECRECY_MISSING",
        ),
    ];
    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(decision: SealedAuditEventChainDecision, reason: SealedAuditEventChainReason) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.can_append_event);
    assert!(!decision.can_verify_inclusion);
    assert!(!decision.can_detect_rollback);
    assert!(!decision.tamper_evident);
    assert!(!decision.append_only);
    assert!(!decision.plaintext_bytes_exposed);
}

fn valid_input() -> SealedAuditEventChainInput {
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
        checkpoint_signature_len: 64,
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
