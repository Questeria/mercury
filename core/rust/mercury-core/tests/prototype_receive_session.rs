use mercury_core::{
    ClientReceiveReason, ClientReceiveReplayState, ComponentReasons, DeliveryAckReason,
    DeviceTrustDecision, DeviceTrustReason, LocalStorePayloadKind, LocalStoreRecordKind,
    LocalStoreRecordLocator, LocalStoreSealingDecision, LocalStoreSealingReason,
    OutboundSendDecision, OutboundSendReason, PolicyDecision, PrototypeReceiveSession,
    PrototypeReceiveSessionEventKind, PrototypeReceiveSessionInput, PrototypeReceiveSessionReason,
    PrototypeRelaySubmitRequest,
};

#[test]
fn receive_session_completes_and_persists_delivered_ciphertext() {
    let mut session = PrototypeReceiveSession::default();
    let outcome = session.run(valid_receive_input());

    assert!(outcome.completed);
    assert_eq!(outcome.reason, PrototypeReceiveSessionReason::Completed);
    assert!(outcome.relay_submission.accepted);
    assert!(outcome.relay_queue.accepted);
    assert!(outcome.relay_delivery.expect("relay delivery").accepted);
    assert!(outcome.delivery_ack.expect("delivery ack").accepted);
    assert!(outcome.client_receive.expect("client receive").accepted);
    assert!(outcome.store_write.expect("store write").accepted);
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.relay_items, 1);
    assert_eq!(outcome.delivered_ciphertext_len, CIPHERTEXT.len());
    assert_eq!(outcome.delivered_sealed_header_len, SEALED_HEADER.len());
    assert!(!outcome.plaintext_exposed);
    assert_eq!(session.events().len(), 7);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeReceiveSessionEventKind::ReceiveStarted
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeReceiveSessionEventKind::ReceiveFinished
    );
    assert_eq!(
        session
            .events()
            .last()
            .expect("last event")
            .view()
            .kind_label,
        "receive_finished"
    );
    assert!(session.events().last().expect("last event").terminal);
    assert!(
        session
            .events()
            .iter()
            .all(|event| !event.plaintext_bytes_exposed)
    );

    let record = session
        .local_store()
        .get_record(locator())
        .expect("delivered ciphertext should be persisted");
    assert_eq!(record.record_kind, LocalStoreRecordKind::MessageCiphertext);
    assert_eq!(record.payload_kind, LocalStorePayloadKind::Sealed);
    assert_eq!(record.bytes, CIPHERTEXT);
}

#[test]
fn receive_session_stops_when_relay_submit_rejects() {
    let mut input = valid_receive_input();
    input.relay_submit = PrototypeRelaySubmitRequest::new(
        OutboundSendDecision {
            accepted: false,
            can_send: false,
            can_persist_ciphertext: false,
            requires_user_action: true,
            reason: OutboundSendReason::MessagePolicyRejected,
        },
        &ROUTE_ID,
        &REPLAY_TOKEN,
        300,
        86400,
        &CIPHERTEXT,
        1048576,
        &SEALED_HEADER,
        0,
        3,
        100,
        120,
    );

    let mut session = PrototypeReceiveSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeReceiveSessionReason::RelaySubmitRejected
    );
    assert!(!outcome.relay_submission.accepted);
    assert!(outcome.relay_delivery.is_none());
    assert!(outcome.delivery_ack.is_none());
    assert!(outcome.client_receive.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.relay_items, 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeReceiveSessionEventKind::RelaySubmitEvaluated
    );
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn receive_session_stops_when_delivery_ack_rejects() {
    let mut input = valid_receive_input();
    input.ack_token_len = 12;

    let mut session = PrototypeReceiveSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeReceiveSessionReason::DeliveryAckRejected
    );
    assert_eq!(
        outcome.delivery_ack.expect("delivery ack").reason,
        DeliveryAckReason::BadAckTokenLength
    );
    assert!(outcome.client_receive.is_none());
    assert!(outcome.store_write.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.relay_items, 1);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeReceiveSessionEventKind::DeliveryAckEvaluated
    );
}

#[test]
fn receive_session_stops_when_client_receive_requires_retry() {
    let mut input = valid_receive_input();
    input.receive_replay_state = ClientReceiveReplayState::FutureGap;

    let mut session = PrototypeReceiveSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeReceiveSessionReason::ClientReceiveRejected
    );
    let receive = outcome.client_receive.expect("client receive");
    assert_eq!(receive.reason, ClientReceiveReason::OrderingGap);
    assert!(receive.requires_client_retry);
    assert!(outcome.store_write.is_none());
    assert_eq!(outcome.local_store_records, 0);
}

#[test]
fn receive_session_stops_when_store_write_rejects() {
    let mut input = valid_receive_input();
    input.store_record_kind = LocalStoreRecordKind::MessagePlaintext;

    let mut session = PrototypeReceiveSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeReceiveSessionReason::LocalStoreWriteRejected
    );
    assert!(outcome.client_receive.expect("client receive").accepted);
    assert!(!outcome.store_write.expect("store write").accepted);
    assert_eq!(outcome.local_store_records, 0);
}

#[test]
fn receive_session_reasons_have_stable_codes_and_labels() {
    let cases = [
        (PrototypeReceiveSessionReason::Completed, 0, "completed"),
        (
            PrototypeReceiveSessionReason::RelaySubmitRejected,
            1,
            "relay_submit_rejected",
        ),
        (
            PrototypeReceiveSessionReason::RelayDeliveryRejected,
            2,
            "relay_delivery_rejected",
        ),
        (
            PrototypeReceiveSessionReason::DeliveryAckRejected,
            3,
            "delivery_ack_rejected",
        ),
        (
            PrototypeReceiveSessionReason::ClientReceiveRejected,
            4,
            "client_receive_rejected",
        ),
        (
            PrototypeReceiveSessionReason::LocalStoreWriteRejected,
            5,
            "local_store_write_rejected",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_receive_input() -> PrototypeReceiveSessionInput<'static> {
    PrototypeReceiveSessionInput {
        relay_submit: PrototypeRelaySubmitRequest::new(
            OutboundSendDecision {
                accepted: true,
                can_send: true,
                can_persist_ciphertext: true,
                requires_user_action: false,
                reason: OutboundSendReason::Accepted,
            },
            &ROUTE_ID,
            &REPLAY_TOKEN,
            300,
            86400,
            &CIPHERTEXT,
            1048576,
            &SEALED_HEADER,
            0,
            3,
            100,
            120,
        ),
        delivery_now_s: 130,
        ack_seen: false,
        acknowledged_at_s: 140,
        max_ack_delay_s: 300,
        ack_token_len: 32,
        ciphertext_digest_len: 32,
        delivery_tag_len: 32,
        receive_replay_state: ClientReceiveReplayState::NewInOrder,
        sender_device_trust: full_trust(),
        message_policy: policy_decision(true),
        ciphertext_sealing: sealing_decision(true),
        store_locator: locator(),
        store_record_kind: LocalStoreRecordKind::MessageCiphertext,
        plaintext_identity_fields: 0,
    }
}

fn locator() -> LocalStoreRecordLocator<'static> {
    LocalStoreRecordLocator::new("conversation-7", "inbound-message-42")
}

fn full_trust() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: true,
        can_send: true,
        requires_user_action: false,
        reason: DeviceTrustReason::Trusted,
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

const ROUTE_ID: [u8; 32] = [7; 32];
const REPLAY_TOKEN: [u8; 32] = [9; 32];
const CIPHERTEXT: [u8; 128] = [42; 128];
const SEALED_HEADER: [u8; 96] = [5; 96];
