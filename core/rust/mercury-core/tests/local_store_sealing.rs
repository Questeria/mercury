use mercury_core::{
    ComponentReasons, LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope,
    LocalStorePayloadKind, LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreSealRequest,
    LocalStoreSealingReason, LocalStoreSealingSuite, PolicyDecision,
    build_sealed_local_store_write_request,
};

#[test]
fn sealing_accepts_room_epoch_message_ciphertext() {
    let request = seal_request(
        LocalStoreRecordKind::MessageCiphertext,
        key(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreKeyBinding::room_epoch(32, 32, 7),
        ),
        Some(policy_decision(true)),
    );

    let decision = request.evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason, LocalStoreSealingReason::Accepted);
}

#[test]
fn sealing_rejects_wrong_key_scope_for_message_ciphertext() {
    let request = seal_request(
        LocalStoreRecordKind::MessageCiphertext,
        key(
            LocalStoreKeyScope::Conversation,
            LocalStoreKeyBinding::conversation(32, 32),
        ),
        Some(policy_decision(true)),
    );

    let decision = request.evaluate();

    assert!(!decision.accepted);
    assert_eq!(decision.reason, LocalStoreSealingReason::KeyScopeMismatch);
}

#[test]
fn sealing_rejects_unbound_room_epoch_key() {
    let request = seal_request(
        LocalStoreRecordKind::MessageCiphertext,
        key(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreKeyBinding::conversation(32, 32),
        ),
        Some(policy_decision(true)),
    );

    let decision = request.evaluate();

    assert!(!decision.accepted);
    assert_eq!(decision.reason, LocalStoreSealingReason::KeyBindingMismatch);
}

#[test]
fn sealing_rejects_plaintext_and_hash_only_record_kinds() {
    let plaintext = seal_request(
        LocalStoreRecordKind::MessagePlaintext,
        key(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreKeyBinding::room_epoch(32, 32, 7),
        ),
        Some(policy_decision(true)),
    )
    .evaluate();

    assert!(!plaintext.accepted);
    assert_eq!(
        plaintext.reason,
        LocalStoreSealingReason::RecordCannotBeSealed
    );

    let audit = seal_request(
        LocalStoreRecordKind::PolicyDecisionAudit,
        key(LocalStoreKeyScope::Audit, LocalStoreKeyBinding::audit(32)),
        Some(policy_decision(false)),
    )
    .evaluate();

    assert!(!audit.accepted);
    assert_eq!(audit.reason, LocalStoreSealingReason::RecordCannotBeSealed);
}

#[test]
fn sealing_enforces_nonce_generation_and_policy() {
    let bad_nonce = LocalStoreSealRequest::new(
        locator(),
        LocalStoreRecordKind::MessageCiphertext,
        key(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreKeyBinding::room_epoch(32, 32, 7),
        ),
        12,
        64,
        Some(policy_decision(true)),
    )
    .evaluate();
    assert!(!bad_nonce.accepted);
    assert_eq!(bad_nonce.reason, LocalStoreSealingReason::BadNonceLength);

    let bad_generation = LocalStoreSealRequest::new(
        locator(),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStoreKeyDescriptor::new(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreSealingSuite::MercuryLocalStoreV1,
            0,
            LocalStoreKeyBinding::room_epoch(32, 32, 7),
        ),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        64,
        Some(policy_decision(true)),
    )
    .evaluate();
    assert!(!bad_generation.accepted);
    assert_eq!(
        bad_generation.reason,
        LocalStoreSealingReason::BadKeyGeneration
    );

    let rejected_policy = seal_request(
        LocalStoreRecordKind::MessageCiphertext,
        key(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreKeyBinding::room_epoch(32, 32, 7),
        ),
        Some(policy_decision(false)),
    )
    .evaluate();
    assert!(!rejected_policy.accepted);
    assert_eq!(
        rejected_policy.reason,
        LocalStoreSealingReason::PolicyDecisionRejected
    );
}

#[test]
fn validated_seal_request_builds_sealed_write_request() {
    let request = seal_request(
        LocalStoreRecordKind::MessageCiphertext,
        key(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreKeyBinding::room_epoch(32, 32, 7),
        ),
        Some(policy_decision(true)),
    );

    let write = build_sealed_local_store_write_request(request, b"sealed bytes")
        .expect("valid seal request should build a store request");

    assert_eq!(write.record_kind, LocalStoreRecordKind::MessageCiphertext);
    assert_eq!(write.payload.kind(), LocalStorePayloadKind::Sealed);
    assert_eq!(write.payload.bytes(), b"sealed bytes");
}

#[test]
fn conversation_secret_uses_conversation_key_without_policy() {
    let request = seal_request(
        LocalStoreRecordKind::ConversationSecret,
        key(
            LocalStoreKeyScope::Conversation,
            LocalStoreKeyBinding::conversation(32, 32),
        ),
        None,
    );

    let decision = request.evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason, LocalStoreSealingReason::Accepted);
}

fn seal_request(
    record_kind: LocalStoreRecordKind,
    key: LocalStoreKeyDescriptor,
    policy_decision: Option<PolicyDecision>,
) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        locator(),
        record_kind,
        key,
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        64,
        policy_decision,
    )
}

fn locator() -> LocalStoreRecordLocator<'static> {
    LocalStoreRecordLocator::new("conversation-7", "record-42")
}

fn key(scope: LocalStoreKeyScope, binding: LocalStoreKeyBinding) -> LocalStoreKeyDescriptor {
    LocalStoreKeyDescriptor::new(
        scope,
        LocalStoreSealingSuite::MercuryLocalStoreV1,
        1,
        binding,
    )
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
