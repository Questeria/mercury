use mercury_core::{
    DeliveryAckDecision, DeliveryAckInput, DeliveryAckReason, RelayQueueItemState,
    evaluate_delivery_ack,
};

#[test]
fn accepted_ack_retains_only_hash_audit_and_deletes_queue_record() {
    let decision = evaluate_delivery_ack(valid_ack());

    assert!(decision.accepted);
    assert!(!decision.duplicate);
    assert!(decision.retain_hash_audit);
    assert!(decision.delete_queue_record);
    assert!(!decision.requires_client_retry);
    assert_eq!(decision.reason, DeliveryAckReason::Accepted);
}

#[test]
fn ack_requires_delivered_queue_item() {
    let mut pending = valid_ack();
    pending.queue_state = RelayQueueItemState::Pending;
    assert_rejected(
        evaluate_delivery_ack(pending),
        DeliveryAckReason::QueueItemNotDelivered,
        true,
    );

    let mut expired = valid_ack();
    expired.queue_state = RelayQueueItemState::Expired;
    assert_rejected(
        evaluate_delivery_ack(expired),
        DeliveryAckReason::QueueItemExpired,
        false,
    );
}

#[test]
fn duplicate_ack_is_idempotent_without_retry_or_audit_rewrite() {
    let mut input = valid_ack();
    input.ack_seen = true;

    let decision = evaluate_delivery_ack(input);

    assert!(!decision.accepted);
    assert!(decision.duplicate);
    assert!(!decision.retain_hash_audit);
    assert!(!decision.delete_queue_record);
    assert!(!decision.requires_client_retry);
    assert_eq!(decision.reason, DeliveryAckReason::DuplicateAck);
}

#[test]
fn ack_time_window_must_be_well_formed_and_fresh() {
    let mut bad = valid_ack();
    bad.acknowledged_at_s = bad.delivered_at_s - 1;
    assert_rejected(
        evaluate_delivery_ack(bad),
        DeliveryAckReason::BadTimeWindow,
        true,
    );

    let mut late = valid_ack();
    late.acknowledged_at_s = late.delivered_at_s + late.max_ack_delay_s + 1;
    assert_rejected(
        evaluate_delivery_ack(late),
        DeliveryAckReason::AckTooLate,
        true,
    );
}

#[test]
fn ack_token_and_ciphertext_digest_are_fixed_length() {
    let mut bad_token = valid_ack();
    bad_token.ack_token_len = 16;
    assert_rejected(
        evaluate_delivery_ack(bad_token),
        DeliveryAckReason::BadAckTokenLength,
        true,
    );

    let mut bad_digest = valid_ack();
    bad_digest.ciphertext_digest_len = 16;
    assert_rejected(
        evaluate_delivery_ack(bad_digest),
        DeliveryAckReason::BadCiphertextDigestLength,
        true,
    );
}

#[test]
fn ack_allows_only_opaque_delivery_tags_and_no_plaintext_identity() {
    let mut bad_tag = valid_ack();
    bad_tag.delivery_tag_len = 8;
    assert_rejected(
        evaluate_delivery_ack(bad_tag),
        DeliveryAckReason::BadDeliveryTagLength,
        true,
    );

    let mut plaintext_identity = valid_ack();
    plaintext_identity.plaintext_identity_fields = 1;
    assert_rejected(
        evaluate_delivery_ack(plaintext_identity),
        DeliveryAckReason::PlaintextIdentityForbidden,
        false,
    );
}

fn valid_ack() -> DeliveryAckInput {
    DeliveryAckInput {
        queue_state: RelayQueueItemState::Delivered,
        ack_seen: false,
        delivered_at_s: 100,
        acknowledged_at_s: 130,
        max_ack_delay_s: 300,
        ack_token_len: 32,
        ciphertext_digest_len: 32,
        delivery_tag_len: 32,
        plaintext_identity_fields: 0,
    }
}

fn assert_rejected(
    decision: DeliveryAckDecision,
    reason: DeliveryAckReason,
    requires_client_retry: bool,
) {
    assert!(!decision.accepted);
    assert!(!decision.duplicate);
    assert!(!decision.retain_hash_audit);
    assert!(!decision.delete_queue_record);
    assert_eq!(decision.requires_client_retry, requires_client_retry);
    assert_eq!(decision.reason, reason);
}
