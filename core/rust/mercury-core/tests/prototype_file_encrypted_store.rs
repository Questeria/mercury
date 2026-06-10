use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mercury_core::{
    ComponentReasons, LocalStorePayload, LocalStorePayloadKind, LocalStoreRecordKind,
    LocalStoreRecordLocator, LocalStoreWriteReason, LocalStoreWriteRequest, PolicyDecision,
    PrototypeFileEncryptedLocalStore,
};

#[test]
fn file_store_persists_accepted_sealed_records_across_reopen() {
    let root = temp_root("persist");
    let locator = record_locator();
    let path;

    {
        let mut store = PrototypeFileEncryptedLocalStore::new(root.clone());
        path = store.record_path(locator);
        let decision = store
            .put_record(LocalStoreWriteRequest::new(
                locator,
                LocalStoreRecordKind::MessageCiphertext,
                LocalStorePayload::sealed(b"sealed-message-bytes"),
                Some(policy_decision(true)),
            ))
            .expect("file store write should succeed");

        assert!(decision.accepted);
        assert!(path.exists());
    }

    let reopened = PrototypeFileEncryptedLocalStore::new(root.clone());
    let stored = reopened
        .get_record(locator)
        .expect("file store read should succeed")
        .expect("accepted record should exist after reopen");

    assert_eq!(stored.namespace, "conversation-7");
    assert_eq!(stored.record_id, "message-42");
    assert_eq!(stored.record_kind, LocalStoreRecordKind::MessageCiphertext);
    assert_eq!(stored.record_kind.code(), 6);
    assert_eq!(stored.record_kind.label(), "message_ciphertext");
    assert_eq!(stored.payload_kind, LocalStorePayloadKind::Sealed);
    assert_eq!(stored.payload_kind.code(), 1);
    assert_eq!(stored.payload_kind.label(), "sealed");
    assert_eq!(stored.bytes, b"sealed-message-bytes");
    assert!(stored.write_decision.accepted);

    cleanup(root);
}

#[test]
fn file_store_rejected_plaintext_record_does_not_create_file() {
    let root = temp_root("reject");
    let locator = record_locator();
    let mut store = PrototypeFileEncryptedLocalStore::new(root.clone());
    let path = store.record_path(locator);

    let decision = store
        .put_record(LocalStoreWriteRequest::new(
            locator,
            LocalStoreRecordKind::MessagePlaintext,
            LocalStorePayload::public_metadata(b"plaintext must not persist"),
            Some(policy_decision(true)),
        ))
        .expect("rejected write should still return a decision");

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        LocalStoreWriteReason::PlaintextRecordForbidden
    );
    assert!(!path.exists());
    assert!(
        store
            .get_record(locator)
            .expect("missing rejected record should be readable")
            .is_none()
    );

    cleanup(root);
}

#[test]
fn file_store_rejected_policy_does_not_replace_existing_record() {
    let root = temp_root("replace");
    let locator = record_locator();
    let mut store = PrototypeFileEncryptedLocalStore::new(root.clone());

    let accepted = store
        .put_record(LocalStoreWriteRequest::new(
            locator,
            LocalStoreRecordKind::MessageCiphertext,
            LocalStorePayload::sealed(b"first-sealed-message"),
            Some(policy_decision(true)),
        ))
        .expect("accepted write should succeed");
    assert!(accepted.accepted);

    let rejected = store
        .put_record(LocalStoreWriteRequest::new(
            locator,
            LocalStoreRecordKind::MessageCiphertext,
            LocalStorePayload::sealed(b"rejected-replacement"),
            Some(policy_decision(false)),
        ))
        .expect("rejected write should return a decision");
    assert!(!rejected.accepted);
    assert_eq!(
        rejected.reason,
        LocalStoreWriteReason::PolicyDecisionRejected
    );

    let stored = store
        .get_record(locator)
        .expect("file store read should succeed")
        .expect("original record should remain");
    assert_eq!(stored.bytes, b"first-sealed-message");

    cleanup(root);
}

#[test]
fn file_store_delete_removes_durable_record() {
    let root = temp_root("delete");
    let locator = record_locator();
    let mut store = PrototypeFileEncryptedLocalStore::new(root.clone());

    store
        .put_record(LocalStoreWriteRequest::new(
            locator,
            LocalStoreRecordKind::PolicyDecisionAudit,
            LocalStorePayload::hash_digest(b"decision-digest"),
            Some(policy_decision(false)),
        ))
        .expect("audit digest write should succeed");

    assert!(
        store
            .get_record(locator)
            .expect("file store read should succeed")
            .is_some()
    );
    assert!(store.delete(locator).expect("delete should succeed"));
    assert!(!store.delete(locator).expect("second delete should succeed"));
    assert!(
        store
            .get_record(locator)
            .expect("deleted record lookup should succeed")
            .is_none()
    );

    cleanup(root);
}

fn record_locator() -> LocalStoreRecordLocator<'static> {
    LocalStoreRecordLocator::new("conversation-7", "message-42")
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

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "mercury-file-store-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn cleanup(root: PathBuf) {
    let _ = fs::remove_dir_all(root);
}
