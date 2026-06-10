use mercury_core::{
    PrototypeSealedAuditEventStore, SealedAuditAnchorKind, SealedAuditEnvelopeSuite,
    SealedAuditEventChainDecision, SealedAuditEventChainInput, SealedAuditEventKind,
    SealedAuditEventStoreDecision, SealedAuditEventStoreReason, SealedAuditEventStoreWrite,
    evaluate_sealed_audit_event_chain, evaluate_sealed_audit_event_store_write,
    put_sealed_audit_event_record,
};

const EVENT_HASH: [u8; 32] = [0x81; 32];
const OTHER_EVENT_HASH: [u8; 32] = [0x82; 32];
const PREVIOUS_EVENT_HASH: [u8; 32] = [0x83; 32];
const RECORD_DIGEST: [u8; 32] = [0x84; 32];
const OTHER_RECORD_DIGEST: [u8; 32] = [0x85; 32];
const MERKLE_ROOT_HASH: [u8; 32] = [0x86; 32];
const CHECKPOINT_ID: [u8; 32] = [0x87; 32];
const OTHER_CHECKPOINT_ID: [u8; 32] = [0x88; 32];
const CHECKPOINT_SIGNATURE: [u8; 64] = [0x89; 64];
const TRANSPARENCY_RECEIPT: [u8; 96] = [0x8a; 96];
const WITNESS_RECEIPT: [u8; 96] = [0x8b; 96];
const SHORT_DIGEST: [u8; 16] = [0x8c; 16];

#[test]
fn sealed_audit_event_store_persists_only_accepted_digest_records() {
    let mut store = PrototypeSealedAuditEventStore::default();
    let decision = put_sealed_audit_event_record(&mut store, valid_write())
        .expect("prototype store is infallible");

    assert!(decision.accepted);
    assert_eq!(decision.reason, SealedAuditEventStoreReason::Accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert_eq!(decision.event_sequence, 42);
    assert!(decision.can_publish_receipt);
    assert!(decision.can_detect_replay);
    assert!(decision.append_only);
    assert!(decision.keeps_digest_only);
    assert!(!decision.keeps_plaintext_metadata);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(store.len(), 1);

    let record = store
        .get_by_sequence(42)
        .expect("accepted sealed audit record should persist");
    assert_eq!(record.event_sequence, 42);
    assert_eq!(record.event_hash, EVENT_HASH);
    assert_eq!(record.previous_event_hash, PREVIOUS_EVENT_HASH);
    assert_eq!(record.record_digest, RECORD_DIGEST);
    assert_eq!(record.merkle_root_hash, MERKLE_ROOT_HASH);
    assert_eq!(record.checkpoint_id, CHECKPOINT_ID);
    assert_eq!(record.checkpoint_signature, CHECKPOINT_SIGNATURE);
    assert_eq!(record.transparency_receipt, TRANSPARENCY_RECEIPT);
    assert_eq!(record.witness_receipt, WITNESS_RECEIPT);
    assert_eq!(record.event_kind, SealedAuditEventKind::MlsCommit);
    assert_eq!(
        record.anchor_kind,
        SealedAuditAnchorKind::WitnessedTransparencyLog
    );
    assert_eq!(record.sealed_payload_len, 512);
    assert!(record.checkpoint_binds_chain);
    assert!(record.receipt_binds_checkpoint);
    assert!(!record.plaintext_bytes_exposed);
    assert!(store.get_by_hash(&EVENT_HASH).is_some());
    assert!(store.checkpoint_recorded(&CHECKPOINT_ID));
    assert_eq!(store.highest_event_sequence(), Some(42));
}

#[test]
fn sealed_audit_event_store_rejects_chain_gate_and_bad_shapes_without_mutation() {
    let mut store = PrototypeSealedAuditEventStore::default();

    let rejected_chain = SealedAuditEventStoreWrite {
        chain_decision: rejected_chain_decision(),
        ..valid_write()
    };
    assert_rejected(
        store.put(rejected_chain),
        SealedAuditEventStoreReason::ChainRejected,
    );
    assert!(store.is_empty());

    let bad_event_hash = SealedAuditEventStoreWrite {
        event_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_sealed_audit_event_store_write(bad_event_hash),
        SealedAuditEventStoreReason::BadDigestShape,
    );

    let bad_signature = SealedAuditEventStoreWrite {
        checkpoint_signature: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_sealed_audit_event_store_write(bad_signature),
        SealedAuditEventStoreReason::BadDigestShape,
    );

    assert!(store.is_empty());
}

#[test]
fn sealed_audit_event_store_rejects_duplicates_and_rollbacks_without_mutation() {
    let mut store = PrototypeSealedAuditEventStore::default();
    assert!(store.put(valid_write()).accepted);

    let duplicate_sequence = SealedAuditEventStoreWrite {
        event_hash: &OTHER_EVENT_HASH,
        checkpoint_id: &OTHER_CHECKPOINT_ID,
        ..valid_write()
    };
    assert_rejected(
        store.put(duplicate_sequence),
        SealedAuditEventStoreReason::DuplicateSequence,
    );
    assert_eq!(store.len(), 1);

    let duplicate_hash = SealedAuditEventStoreWrite {
        event_sequence: 43,
        chain_decision: accepted_chain_decision_for_sequence(43),
        checkpoint_id: &OTHER_CHECKPOINT_ID,
        record_digest: &OTHER_RECORD_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        store.put(duplicate_hash),
        SealedAuditEventStoreReason::DuplicateEventHash,
    );
    assert_eq!(store.len(), 1);

    let duplicate_checkpoint = SealedAuditEventStoreWrite {
        event_sequence: 43,
        chain_decision: accepted_chain_decision_for_sequence(43),
        event_hash: &OTHER_EVENT_HASH,
        record_digest: &OTHER_RECORD_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        store.put(duplicate_checkpoint),
        SealedAuditEventStoreReason::DuplicateCheckpoint,
    );
    assert_eq!(store.len(), 1);

    let rollback = SealedAuditEventStoreWrite {
        event_sequence: 41,
        chain_decision: accepted_chain_decision_for_sequence(41),
        event_hash: &OTHER_EVENT_HASH,
        record_digest: &OTHER_RECORD_DIGEST,
        checkpoint_id: &OTHER_CHECKPOINT_ID,
        ..valid_write()
    };
    assert_rejected(
        store.put(rollback),
        SealedAuditEventStoreReason::RollbackSequence,
    );
    assert_eq!(store.len(), 1);
}

#[test]
fn sealed_audit_event_store_rejects_plaintext_and_missing_bindings_without_mutation() {
    let mut store = PrototypeSealedAuditEventStore::default();

    let plaintext = SealedAuditEventStoreWrite {
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    assert_rejected(
        store.put(plaintext),
        SealedAuditEventStoreReason::PlaintextMetadataForbidden,
    );

    let checkpoint = SealedAuditEventStoreWrite {
        checkpoint_binds_chain: false,
        ..valid_write()
    };
    assert_rejected(
        store.put(checkpoint),
        SealedAuditEventStoreReason::CheckpointBindingMissing,
    );

    let receipt = SealedAuditEventStoreWrite {
        transparency_receipt: &[],
        receipt_binds_checkpoint: false,
        ..valid_write()
    };
    assert_rejected(
        store.put(receipt),
        SealedAuditEventStoreReason::TransparencyReceiptMissing,
    );

    let append_only = SealedAuditEventStoreWrite {
        append_only_guard: false,
        ..valid_write()
    };
    assert_rejected(
        store.put(append_only),
        SealedAuditEventStoreReason::AppendOnlyGuardMissing,
    );

    assert!(store.is_empty());
}

#[test]
fn sealed_audit_event_store_allows_next_sequence_with_fresh_hash_and_checkpoint() {
    let mut store = PrototypeSealedAuditEventStore::default();
    assert!(store.put(valid_write()).accepted);

    let next = SealedAuditEventStoreWrite {
        event_sequence: 43,
        chain_decision: accepted_chain_decision_for_sequence(43),
        event_hash: &OTHER_EVENT_HASH,
        record_digest: &OTHER_RECORD_DIGEST,
        checkpoint_id: &OTHER_CHECKPOINT_ID,
        ..valid_write()
    };
    let decision = store.put(next);

    assert!(decision.accepted);
    assert_eq!(decision.record_count, 2);
    assert_eq!(store.len(), 2);
    assert_eq!(store.highest_event_sequence(), Some(43));
}

#[test]
fn sealed_audit_event_store_reasons_have_stable_codes_and_labels() {
    let cases = [
        (SealedAuditEventStoreReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditEventStoreReason::ChainRejected,
            1,
            "CHAIN_REJECTED",
        ),
        (
            SealedAuditEventStoreReason::DuplicateSequence,
            2,
            "DUPLICATE_SEQUENCE",
        ),
        (
            SealedAuditEventStoreReason::DuplicateEventHash,
            3,
            "DUPLICATE_EVENT_HASH",
        ),
        (
            SealedAuditEventStoreReason::DuplicateCheckpoint,
            4,
            "DUPLICATE_CHECKPOINT",
        ),
        (
            SealedAuditEventStoreReason::RollbackSequence,
            5,
            "ROLLBACK_SEQUENCE",
        ),
        (
            SealedAuditEventStoreReason::BadDigestShape,
            6,
            "BAD_DIGEST_SHAPE",
        ),
        (
            SealedAuditEventStoreReason::PlaintextMetadataForbidden,
            7,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditEventStoreReason::CheckpointBindingMissing,
            8,
            "CHECKPOINT_BINDING_MISSING",
        ),
        (
            SealedAuditEventStoreReason::TransparencyReceiptMissing,
            9,
            "TRANSPARENCY_RECEIPT_MISSING",
        ),
        (
            SealedAuditEventStoreReason::AppendOnlyGuardMissing,
            10,
            "APPEND_ONLY_GUARD_MISSING",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(decision: SealedAuditEventStoreDecision, reason: SealedAuditEventStoreReason) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.persisted_record);
    assert!(!decision.can_publish_receipt);
    assert!(decision.can_detect_replay);
    assert!(decision.append_only);
    assert!(decision.keeps_digest_only);
    assert!(!decision.keeps_plaintext_metadata);
    assert!(!decision.plaintext_bytes_exposed);
}

fn valid_write() -> SealedAuditEventStoreWrite<'static> {
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

fn accepted_chain_decision_for_sequence(sequence: i64) -> SealedAuditEventChainDecision {
    let mut input = valid_chain_input();
    input.event_sequence = sequence;
    input.previous_chain_size = sequence;
    input.previous_checkpoint_size = sequence;
    input.checkpoint_size = sequence + 1;
    evaluate_sealed_audit_event_chain(input)
}

fn rejected_chain_decision() -> SealedAuditEventChainDecision {
    let mut input = valid_chain_input();
    input.storage_append_only = false;
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
