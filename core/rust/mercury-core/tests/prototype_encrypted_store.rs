use mercury_core::{
    ComponentReasons, LocalStorePayload, LocalStorePayloadKind, LocalStoreRecordKind,
    LocalStoreRecordLocator, LocalStoreWriteReason, LocalStoreWriteRequest, PolicyDecision,
    PrototypeEncryptedLocalStore,
};

#[test]
fn prototype_store_persists_accepted_sealed_records() {
    let mut store = PrototypeEncryptedLocalStore::default();
    let decision = store.put_record(LocalStoreWriteRequest::new(
        locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStorePayload::sealed(b"sealed-message"),
        Some(policy_decision(true)),
    ));

    let stored = store
        .get_record(locator("conversation-7", "message-42"))
        .expect("accepted record should be stored");

    assert!(decision.accepted);
    assert_eq!(store.len(), 1);
    assert_eq!(stored.namespace, "conversation-7");
    assert_eq!(stored.record_id, "message-42");
    assert_eq!(stored.record_kind, LocalStoreRecordKind::MessageCiphertext);
    assert_eq!(stored.payload_kind, LocalStorePayloadKind::Sealed);
    assert_eq!(stored.bytes, b"sealed-message");
    assert!(stored.write_decision.accepted);
}

#[test]
fn prototype_store_rejected_write_does_not_replace_existing_record() {
    let mut store = PrototypeEncryptedLocalStore::default();
    store.put_record(LocalStoreWriteRequest::new(
        locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStorePayload::sealed(b"first-sealed-message"),
        Some(policy_decision(true)),
    ));

    let rejected = store.put_record(LocalStoreWriteRequest::new(
        locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStorePayload::sealed(b"rejected-replacement"),
        Some(policy_decision(false)),
    ));

    let stored = store
        .get_record(locator("conversation-7", "message-42"))
        .expect("existing accepted record should remain");

    assert!(!rejected.accepted);
    assert_eq!(
        rejected.reason,
        LocalStoreWriteReason::PolicyDecisionRejected
    );
    assert_eq!(store.len(), 1);
    assert_eq!(stored.bytes, b"first-sealed-message");
}

#[test]
fn prototype_store_can_keep_hash_only_audit_records() {
    let mut store = PrototypeEncryptedLocalStore::default();
    let decision = store.put_record(LocalStoreWriteRequest::new(
        locator("audit", "decision-9"),
        LocalStoreRecordKind::PolicyDecisionAudit,
        LocalStorePayload::hash_digest(b"hash-of-rejected-decision"),
        Some(policy_decision(false)),
    ));

    let stored = store
        .get_record(locator("audit", "decision-9"))
        .expect("audit digest should be stored");

    assert!(decision.accepted);
    assert_eq!(stored.payload_kind, LocalStorePayloadKind::HashDigest);
    assert_eq!(stored.bytes, b"hash-of-rejected-decision");
}

#[test]
fn prototype_store_delete_removes_only_matching_locator() {
    let mut store = PrototypeEncryptedLocalStore::default();
    store.put_record(LocalStoreWriteRequest::new(
        locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStorePayload::sealed(b"sealed-message"),
        Some(policy_decision(true)),
    ));
    store.put_record(LocalStoreWriteRequest::new(
        locator("conversation-8", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStorePayload::sealed(b"other-sealed-message"),
        Some(policy_decision(true)),
    ));

    assert!(store.delete(locator("conversation-7", "message-42")));
    assert!(!store.delete(locator("conversation-7", "message-42")));
    assert!(
        store
            .get_record(locator("conversation-7", "message-42"))
            .is_none()
    );
    assert!(
        store
            .get_record(locator("conversation-8", "message-42"))
            .is_some()
    );
    assert_eq!(store.len(), 1);
}

fn locator<'a>(namespace: &'a str, record_id: &'a str) -> LocalStoreRecordLocator<'a> {
    LocalStoreRecordLocator::new(namespace, record_id)
}

fn policy_decision(accepted: bool) -> PolicyDecision {
    PolicyDecision {
        accepted,
        reason_code: if accepted { 0 } else { 1 },
        audit_class: 0,
        components: ComponentReasons {
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    }
}
