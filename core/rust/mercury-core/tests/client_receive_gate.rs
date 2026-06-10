use mercury_core::{
    ClientReceiveDecision, ClientReceiveInput, ClientReceiveReason, ClientReceiveReplayState,
    ComponentReasons, DeliveryAckDecision, DeliveryAckReason, DeviceTrustDecision,
    DeviceTrustReason, LocalStoreRecordKind, LocalStoreSealingDecision, LocalStoreSealingReason,
    PolicyDecision, RelayQueueDecision, RelayQueueItemState, RelayQueueReason,
    evaluate_client_receive,
};

#[test]
fn accepted_receive_allows_decrypt_persist_and_ui_exposure() {
    let decision = evaluate_client_receive(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_decrypt);
    assert!(decision.can_persist_ciphertext);
    assert!(decision.can_expose_to_ui);
    assert!(!decision.requires_client_retry);
    assert!(!decision.requires_user_action);
    assert_eq!(decision.reason, ClientReceiveReason::Accepted);
}

#[test]
fn sendable_tofu_sender_keeps_visible_user_action() {
    let mut input = valid_input();
    input.sender_device_trust = tofu_trust();

    let decision = input.evaluate();

    assert!(decision.accepted);
    assert!(decision.can_expose_to_ui);
    assert!(decision.requires_user_action);
    assert_eq!(decision.reason, ClientReceiveReason::Accepted);
}

#[test]
fn relay_delivery_must_be_accepted_and_delivered() {
    let mut rejected = valid_input();
    rejected.relay_delivery = relay_delivery(false, RelayQueueItemState::Pending);
    assert_rejected(
        evaluate_client_receive(rejected),
        ClientReceiveReason::RelayDeliveryRejected,
        true,
        false,
    );

    let mut not_delivered = valid_input();
    not_delivered.relay_delivery = relay_delivery(true, RelayQueueItemState::Pending);
    assert_rejected(
        evaluate_client_receive(not_delivered),
        ClientReceiveReason::RelayDeliveryNotDelivered,
        true,
        false,
    );
}

#[test]
fn delivery_ack_must_be_new_and_accepted() {
    let mut duplicate = valid_input();
    duplicate.delivery_ack = ack(false, true, false, DeliveryAckReason::DuplicateAck);
    assert_rejected(
        evaluate_client_receive(duplicate),
        ClientReceiveReason::DuplicateDeliveryAck,
        false,
        false,
    );

    let mut rejected = valid_input();
    rejected.delivery_ack = ack(false, false, true, DeliveryAckReason::AckTooLate);
    assert_rejected(
        evaluate_client_receive(rejected),
        ClientReceiveReason::DeliveryAckRejected,
        true,
        false,
    );
}

#[test]
fn replay_and_order_state_must_be_new_in_order() {
    let mut duplicate = valid_input();
    duplicate.replay_state = ClientReceiveReplayState::Duplicate;
    assert_rejected(
        evaluate_client_receive(duplicate),
        ClientReceiveReason::ReplayDuplicate,
        false,
        false,
    );

    let mut stale = valid_input();
    stale.replay_state = ClientReceiveReplayState::Stale;
    assert_rejected(
        evaluate_client_receive(stale),
        ClientReceiveReason::ReplayStale,
        false,
        false,
    );

    let mut gap = valid_input();
    gap.replay_state = ClientReceiveReplayState::FutureGap;
    assert_rejected(
        evaluate_client_receive(gap),
        ClientReceiveReason::OrderingGap,
        true,
        false,
    );
}

#[test]
fn sender_policy_and_store_sealing_must_all_pass() {
    let mut bad_sender = valid_input();
    bad_sender.sender_device_trust = rejected_trust();
    assert_rejected(
        evaluate_client_receive(bad_sender),
        ClientReceiveReason::SenderDeviceTrustRejected,
        false,
        true,
    );

    let mut bad_policy = valid_input();
    bad_policy.message_policy = policy_decision(false);
    assert_rejected(
        evaluate_client_receive(bad_policy),
        ClientReceiveReason::MessagePolicyRejected,
        false,
        true,
    );

    let mut bad_seal = valid_input();
    bad_seal.ciphertext_sealing = sealing_decision(false);
    assert_rejected(
        evaluate_client_receive(bad_seal),
        ClientReceiveReason::LocalStoreSealingRejected,
        false,
        true,
    );
}

#[test]
fn received_ciphertext_shape_cannot_expose_plaintext_identity() {
    let mut bad_digest = valid_input();
    bad_digest.ciphertext_digest_len = 16;
    assert_rejected(
        evaluate_client_receive(bad_digest),
        ClientReceiveReason::BadCiphertextDigestLength,
        false,
        false,
    );

    let mut plaintext_identity = valid_input();
    plaintext_identity.plaintext_identity_fields = 1;
    assert_rejected(
        evaluate_client_receive(plaintext_identity),
        ClientReceiveReason::PlaintextIdentityForbidden,
        false,
        false,
    );
}

fn valid_input() -> ClientReceiveInput {
    ClientReceiveInput::new(
        relay_delivery(true, RelayQueueItemState::Delivered),
        ack(true, false, false, DeliveryAckReason::Accepted),
        full_trust(),
        policy_decision(true),
        sealing_decision(true),
        ClientReceiveReplayState::NewInOrder,
        32,
        0,
    )
}

fn relay_delivery(accepted: bool, next_state: RelayQueueItemState) -> RelayQueueDecision {
    RelayQueueDecision {
        accepted,
        next_state,
        persist_item: false,
        delete_item: accepted,
        reason: if accepted {
            RelayQueueReason::Accepted
        } else {
            RelayQueueReason::ItemExpired
        },
    }
}

fn ack(
    accepted: bool,
    duplicate: bool,
    requires_client_retry: bool,
    reason: DeliveryAckReason,
) -> DeliveryAckDecision {
    DeliveryAckDecision {
        accepted,
        duplicate,
        retain_hash_audit: accepted,
        delete_queue_record: accepted,
        requires_client_retry,
        reason,
    }
}

fn full_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: true,
        can_send: true,
        requires_user_action: false,
        reason: DeviceTrustReason::Trusted,
    }
}

fn tofu_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: false,
        can_send: true,
        requires_user_action: true,
        reason: DeviceTrustReason::TrustOnFirstUse,
    }
}

fn rejected_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: false,
        can_send: false,
        requires_user_action: true,
        reason: DeviceTrustReason::KeyTransparencyRequired,
    }
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

fn sealing_decision(accepted: bool) -> LocalStoreSealingDecision {
    LocalStoreSealingDecision {
        accepted,
        reason: if accepted {
            LocalStoreSealingReason::Accepted
        } else {
            LocalStoreSealingReason::PolicyDecisionRejected
        },
        record_policy: LocalStoreRecordKind::MessageCiphertext.policy(),
    }
}

fn assert_rejected(
    decision: ClientReceiveDecision,
    reason: ClientReceiveReason,
    requires_client_retry: bool,
    requires_user_action: bool,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_decrypt);
    assert!(!decision.can_persist_ciphertext);
    assert!(!decision.can_expose_to_ui);
    assert_eq!(decision.requires_client_retry, requires_client_retry);
    assert_eq!(decision.requires_user_action, requires_user_action);
    assert_eq!(decision.reason, reason);
}
