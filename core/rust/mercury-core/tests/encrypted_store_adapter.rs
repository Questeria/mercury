use std::convert::Infallible;

use mercury_core::{
    AcceptedLocalStoreWrite, ComponentReasons, EncryptedLocalStoreAdapter, LocalStorePayload,
    LocalStorePayloadKind, LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreWriteDecision,
    LocalStoreWriteReason, LocalStoreWriteRequest, PolicyDecision, put_local_store_record,
};

#[test]
fn adapter_writes_only_policy_accepted_sealed_records() {
    let mut store = MemoryStore::default();
    let request = LocalStoreWriteRequest::new(
        locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStorePayload::sealed(b"sealed-message"),
        Some(policy_decision(true)),
    );

    let decision = put_local_store_record(&mut store, request).expect("infallible memory store");

    assert!(decision.accepted);
    assert_eq!(store.writes.len(), 1);
    assert_eq!(
        store.writes[0].record_kind,
        LocalStoreRecordKind::MessageCiphertext
    );
    assert_eq!(store.writes[0].payload_kind, LocalStorePayloadKind::Sealed);
    assert_eq!(store.writes[0].bytes, b"sealed-message");
}

#[test]
fn adapter_does_not_call_store_for_rejected_policy() {
    let mut store = MemoryStore::default();
    let request = LocalStoreWriteRequest::new(
        locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStorePayload::sealed(b"sealed-message"),
        Some(policy_decision(false)),
    );

    let decision = put_local_store_record(&mut store, request).expect("infallible memory store");

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        LocalStoreWriteReason::PolicyDecisionRejected
    );
    assert!(store.writes.is_empty());
}

#[test]
fn adapter_does_not_call_store_for_plaintext_record_kind() {
    let mut store = MemoryStore::default();
    let request = LocalStoreWriteRequest::new(
        locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessagePlaintext,
        LocalStorePayload::sealed(b"still-not-allowed"),
        Some(policy_decision(true)),
    );

    let decision = put_local_store_record(&mut store, request).expect("infallible memory store");

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        LocalStoreWriteReason::PlaintextRecordForbidden
    );
    assert!(store.writes.is_empty());
}

#[test]
fn adapter_rejects_hash_only_records_with_sealed_payloads() {
    let mut store = MemoryStore::default();
    let request = LocalStoreWriteRequest::new(
        locator("audit", "decision-9"),
        LocalStoreRecordKind::PolicyDecisionAudit,
        LocalStorePayload::sealed(b"encrypted-but-not-a-digest"),
        Some(policy_decision(false)),
    );

    let decision = put_local_store_record(&mut store, request).expect("infallible memory store");

    assert!(!decision.accepted);
    assert_eq!(decision.reason, LocalStoreWriteReason::PayloadClassMismatch);
    assert!(store.writes.is_empty());
}

#[test]
fn adapter_accepts_hash_digest_audit_for_rejected_decision() {
    let mut store = MemoryStore::default();
    let request = LocalStoreWriteRequest::new(
        locator("audit", "decision-9"),
        LocalStoreRecordKind::PolicyDecisionAudit,
        LocalStorePayload::hash_digest(b"hash-of-rejected-decision"),
        Some(policy_decision(false)),
    );

    let decision = put_local_store_record(&mut store, request).expect("infallible memory store");

    assert!(decision.accepted);
    assert_eq!(store.writes.len(), 1);
    assert_eq!(
        store.writes[0].record_kind,
        LocalStoreRecordKind::PolicyDecisionAudit
    );
    assert_eq!(
        store.writes[0].payload_kind,
        LocalStorePayloadKind::HashDigest
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoredWrite {
    namespace: String,
    record_id: String,
    record_kind: LocalStoreRecordKind,
    payload_kind: LocalStorePayloadKind,
    bytes: Vec<u8>,
    decision: LocalStoreWriteDecision,
}

#[derive(Default)]
struct MemoryStore {
    writes: Vec<StoredWrite>,
}

impl EncryptedLocalStoreAdapter for MemoryStore {
    type Error = Infallible;

    fn put_accepted_record(
        &mut self,
        write: AcceptedLocalStoreWrite<'_>,
    ) -> Result<(), Self::Error> {
        let request = write.request();
        self.writes.push(StoredWrite {
            namespace: request.locator.namespace.to_string(),
            record_id: request.locator.record_id.to_string(),
            record_kind: request.record_kind,
            payload_kind: request.payload.kind(),
            bytes: request.payload.bytes().to_vec(),
            decision: write.decision(),
        });
        Ok(())
    }

    fn delete_record(&mut self, locator: LocalStoreRecordLocator<'_>) -> Result<(), Self::Error> {
        self.writes.retain(|write| {
            write.namespace != locator.namespace || write.record_id != locator.record_id
        });
        Ok(())
    }
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
