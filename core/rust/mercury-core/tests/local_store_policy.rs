use mercury_core::{
    ComponentReasons, LocalStoreKeyScope, LocalStorePlaintextClass, LocalStoreRecordKind,
    LocalStoreRetentionClass, LocalStoreWriteIntent, LocalStoreWriteReason, PolicyDecision,
};

#[test]
fn message_plaintext_is_never_writable() {
    let decision = LocalStoreWriteIntent::new(
        LocalStoreRecordKind::MessagePlaintext,
        true,
        Some(policy_decision(true)),
    )
    .evaluate();

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason,
        LocalStoreWriteReason::PlaintextRecordForbidden
    );
    assert_eq!(
        decision.record_policy.plaintext_class,
        LocalStorePlaintextClass::NeverStore
    );
    assert_eq!(
        decision.record_policy.retention_class,
        LocalStoreRetentionClass::EphemeralOnly
    );
}

#[test]
fn message_ciphertext_requires_encryption_and_accepted_policy() {
    let unencrypted = LocalStoreWriteIntent::new(
        LocalStoreRecordKind::MessageCiphertext,
        false,
        Some(policy_decision(true)),
    )
    .evaluate();
    assert!(!unencrypted.accepted);
    assert_eq!(
        unencrypted.reason,
        LocalStoreWriteReason::RequiresEncryptionAtRest
    );

    let rejected_policy = LocalStoreWriteIntent::new(
        LocalStoreRecordKind::MessageCiphertext,
        true,
        Some(policy_decision(false)),
    )
    .evaluate();
    assert!(!rejected_policy.accepted);
    assert_eq!(
        rejected_policy.reason,
        LocalStoreWriteReason::PolicyDecisionRejected
    );

    let accepted = LocalStoreWriteIntent::new(
        LocalStoreRecordKind::MessageCiphertext,
        true,
        Some(policy_decision(true)),
    )
    .evaluate();
    assert!(accepted.accepted);
    assert_eq!(accepted.reason, LocalStoreWriteReason::Accepted);
    assert_eq!(
        accepted.record_policy.key_scope,
        LocalStoreKeyScope::RoomEpoch
    );
}

#[test]
fn audit_hash_can_record_rejected_policy_decision() {
    let missing_decision =
        LocalStoreWriteIntent::new(LocalStoreRecordKind::PolicyDecisionAudit, false, None)
            .evaluate();
    assert!(!missing_decision.accepted);
    assert_eq!(
        missing_decision.reason,
        LocalStoreWriteReason::PolicyDecisionRequired
    );

    let rejected_decision = LocalStoreWriteIntent::new(
        LocalStoreRecordKind::PolicyDecisionAudit,
        false,
        Some(policy_decision(false)),
    )
    .evaluate();
    assert!(rejected_decision.accepted);
    assert_eq!(
        rejected_decision.record_policy.plaintext_class,
        LocalStorePlaintextClass::HashOnly
    );
    assert_eq!(
        rejected_decision.record_policy.key_scope,
        LocalStoreKeyScope::Audit
    );
}

#[test]
fn ai_plaintext_is_blocked_but_transcript_digest_is_hash_only() {
    let prompt = LocalStoreWriteIntent::new(
        LocalStoreRecordKind::AiPromptPlaintext,
        true,
        Some(policy_decision(true)),
    )
    .evaluate();
    assert!(!prompt.accepted);
    assert_eq!(
        prompt.reason,
        LocalStoreWriteReason::PlaintextRecordForbidden
    );

    let transcript = LocalStoreWriteIntent::new(
        LocalStoreRecordKind::AiTranscriptPlaintext,
        true,
        Some(policy_decision(true)),
    )
    .evaluate();
    assert!(!transcript.accepted);
    assert_eq!(
        transcript.reason,
        LocalStoreWriteReason::PlaintextRecordForbidden
    );

    let digest = LocalStoreWriteIntent::new(
        LocalStoreRecordKind::AiTranscriptDigest,
        false,
        Some(policy_decision(true)),
    )
    .evaluate();
    assert!(digest.accepted);
    assert_eq!(
        digest.record_policy.plaintext_class,
        LocalStorePlaintextClass::HashOnly
    );
    assert_eq!(digest.record_policy.key_scope, LocalStoreKeyScope::Audit);
}

#[test]
fn account_secrets_require_encryption_but_not_message_policy() {
    let unencrypted =
        LocalStoreWriteIntent::new(LocalStoreRecordKind::AccountSecret, false, None).evaluate();
    assert!(!unencrypted.accepted);
    assert_eq!(
        unencrypted.reason,
        LocalStoreWriteReason::RequiresEncryptionAtRest
    );

    let encrypted =
        LocalStoreWriteIntent::new(LocalStoreRecordKind::AccountSecret, true, None).evaluate();
    assert!(encrypted.accepted);
    assert_eq!(encrypted.reason, LocalStoreWriteReason::Accepted);
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
