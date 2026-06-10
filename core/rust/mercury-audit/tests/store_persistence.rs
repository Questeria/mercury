//! End-to-end persistence: this crate's store-write PRODUCER composed with
//! mercury-core's `PrototypeSealedAuditEventStore` ADAPTER.
//!
//! The unit tests prove each `SealedAuditStoreWrite` is gate-accepted in isolation;
//! this proves the produced writes actually PERSIST through the real durable-store
//! path (`put_sealed_audit_event_record`) and that the store's replay / rollback
//! guards genuinely reject a re-submitted or out-of-order event. Only the
//! already-built LocalHashChain path + audited crypto are exercised (the witnessed
//! verifier chain requires a post-quantum checkpoint signature the workspace has no
//! audited primitive for, so it is intentionally out of scope here).

use ed25519_dalek::SigningKey;

use mercury_audit::{AuditEvent, AuditEventLog, StorageAttestation};
use mercury_core::{
    PrototypeSealedAuditEventStore, SealedAuditEventKind, SealedAuditEventStoreReason,
};

fn log() -> AuditEventLog {
    AuditEventLog::new(SigningKey::from_bytes(&[9u8; 32]))
}

fn event(record: &[u8]) -> AuditEvent<'_> {
    AuditEvent {
        sealed_record: record,
        kind: SealedAuditEventKind::DeviceKeyChange,
        device_binding: b"device-key-material",
        actor_binding: b"actor-id-material",
        epoch_binding: b"epoch-material",
        room_epoch_binding: b"room-epoch-material",
    }
}

#[test]
fn produced_store_writes_persist_and_advance_the_store() {
    let mut log = log();
    let mut store = PrototypeSealedAuditEventStore::default();

    for i in 0..4 {
        let record = format!("sealed event {i}");
        let write = log
            .stage(
                &event(record.as_bytes()),
                &StorageAttestation::local_sealed_store(),
                1_700_000_000 + i as i64,
            )
            .local_store_write(true);

        let decision = store.put(write.as_gate_input());
        assert!(
            decision.accepted,
            "event {i} reason = {:?}",
            decision.reason
        );
        assert_eq!(decision.reason, SealedAuditEventStoreReason::Accepted);
        assert!(decision.persisted_record);
        assert_eq!(decision.event_sequence, i as i64);
        // The store grew by exactly this record.
        assert_eq!(decision.record_count, i + 1);
    }

    assert_eq!(store.len(), 4);
    assert_eq!(store.highest_event_sequence(), Some(3));
    // Every persisted event is retrievable by sequence and by its real event hash.
    for i in 0..4 {
        let record = store
            .get_by_sequence(i)
            .expect("a persisted event is retrievable by sequence");
        assert_eq!(record.event_sequence, i);
        assert!(store.get_by_hash(&record.event_hash).is_some());
        assert_eq!(record.event_hash.len(), 32);
        assert!(store.checkpoint_recorded(&record.checkpoint_id));
    }
}

#[test]
fn a_replayed_event_is_rejected_by_the_store() {
    // Re-submitting the SAME produced write is a replay: the store rejects it as a
    // duplicate sequence and does NOT grow.
    let mut log = log();
    let mut store = PrototypeSealedAuditEventStore::default();
    let write = log
        .stage(
            &event(b"sealed event 0"),
            &StorageAttestation::local_sealed_store(),
            1_700_000_000,
        )
        .local_store_write(true);

    let first = store.put(write.as_gate_input());
    assert!(first.accepted && first.persisted_record);
    assert_eq!(store.len(), 1);

    let replay = store.put(write.as_gate_input());
    assert!(!replay.accepted);
    assert_eq!(
        replay.reason,
        SealedAuditEventStoreReason::DuplicateSequence
    );
    assert_eq!(store.len(), 1, "a replay must not grow the store");
}

#[test]
fn an_out_of_order_event_below_the_high_water_mark_is_rejected() {
    // Persist sequences 0 and 2 (leaving a gap), so the store's high-water mark is 2.
    // A later attempt to insert sequence 1 — below the mark and not already present —
    // is a rollback the store must reject (monotonicity / anti-rewind guard).
    let mut log = log();
    let writes: Vec<_> = (0..3)
        .map(|i| {
            let record = format!("sealed event {i}");
            log.stage(
                &event(record.as_bytes()),
                &StorageAttestation::local_sealed_store(),
                1_700_000_000 + i as i64,
            )
            .local_store_write(true)
        })
        .collect();

    let mut store = PrototypeSealedAuditEventStore::default();
    assert!(store.put(writes[0].as_gate_input()).accepted);
    assert!(store.put(writes[2].as_gate_input()).accepted);
    assert_eq!(store.highest_event_sequence(), Some(2));

    let rolled_back = store.put(writes[1].as_gate_input());
    assert!(!rolled_back.accepted);
    assert_eq!(
        rolled_back.reason,
        SealedAuditEventStoreReason::RollbackSequence
    );
}
