use mercury_core::{
    AiGrantFacts, AiLifecycleFacts, AiParticipantAction, AiParticipantReason, AiParticipantRequest,
    AiPolicyFacts, ClientBootstrapDecision, ClientBootstrapReason, ComponentReasons,
    LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope, LocalStoreRecordKind,
    LocalStoreRecordLocator, LocalStoreSealRequest, LocalStoreSealingReason,
    LocalStoreSealingSuite, OutboundSendDecision, OutboundSendReason, PolicyDecision,
    PrototypeBackendSession, PrototypeBackendSessionEventKind, PrototypeBackendSessionInput,
    PrototypeBackendSessionReason, RelayQueueItemState,
};

#[test]
fn prototype_session_completes_happy_path_without_plaintext_surface() {
    let mut session = PrototypeBackendSession::default();
    let outcome = session.run(valid_session_input());

    assert!(outcome.completed);
    assert_eq!(outcome.reason, PrototypeBackendSessionReason::Completed);
    assert!(outcome.bootstrap.accepted);
    assert!(outcome.seal.expect("seal decision").accepted);
    assert!(outcome.store_write.expect("store decision").accepted);
    assert!(outcome.relay_submission.expect("relay submission").accepted);
    assert!(outcome.relay_queue.expect("relay queue").accepted);
    assert!(outcome.relay_delivery.expect("relay delivery").accepted);
    assert!(outcome.open.expect("open decision").accepted);
    assert!(outcome.ai.expect("ai decision").accepted);
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.relay_items, 1);
    assert_eq!(outcome.ai_audit_records, 1);
    assert_eq!(outcome.crypto_seal_calls, 1);
    assert_eq!(outcome.crypto_open_calls, 1);
    assert_eq!(outcome.delivered_ciphertext_len, PLAINTEXT.len() + 16);
    assert_eq!(outcome.delivered_sealed_header_len, SEALED_HEADER.len());
    assert_eq!(outcome.opened_plaintext_len, PLAINTEXT.len());
    assert_eq!(session.events().len(), 9);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeBackendSessionEventKind::SessionStarted
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeBackendSessionEventKind::SessionFinished
    );
    assert_eq!(
        session
            .events()
            .last()
            .expect("last event")
            .view()
            .kind_label,
        "session_finished"
    );
    assert_eq!(
        session
            .events()
            .last()
            .expect("last event")
            .view()
            .reason_label,
        "completed"
    );
    assert!(session.events().last().expect("last event").terminal);
    assert!(
        session
            .events()
            .iter()
            .all(|event| !event.plaintext_bytes_exposed)
    );

    let relay_item = session
        .relay()
        .get_item(&ROUTE_ID)
        .expect("delivered relay metadata should remain");
    assert_eq!(relay_item.state, RelayQueueItemState::Delivered);
    assert!(!relay_item.has_ciphertext());
}

#[test]
fn prototype_session_blocks_before_side_effects_when_bootstrap_rejects() {
    let mut input = valid_session_input();
    input.bootstrap = ClientBootstrapDecision {
        accepted: false,
        can_start_sync: true,
        can_decrypt_local_store: false,
        can_open_message_ui: false,
        requires_sync: true,
        requires_recovery: false,
        requires_user_action: false,
        reason: ClientBootstrapReason::SyncIncomplete,
    };
    let mut session = PrototypeBackendSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeBackendSessionReason::BootstrapBlocked
    );
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.relay_items, 0);
    assert_eq!(outcome.ai_audit_records, 0);
    assert_eq!(outcome.crypto_seal_calls, 0);
    assert_eq!(outcome.crypto_open_calls, 0);
    assert_eq!(session.events().len(), 2);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeBackendSessionEventKind::BootstrapChecked
    );
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn prototype_session_rejects_bad_seal_before_relay_or_ai() {
    let mut input = valid_session_input();
    input.seal_request = seal_request(99, Some(policy_decision(true)));
    let mut session = PrototypeBackendSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeBackendSessionReason::LocalStoreSealRejected
    );
    assert_eq!(
        outcome.seal.expect("seal decision").reason,
        LocalStoreSealingReason::BadPlaintextLength
    );
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.relay_items, 0);
    assert_eq!(outcome.ai_audit_records, 0);
    assert_eq!(outcome.crypto_seal_calls, 0);
    assert_eq!(outcome.crypto_open_calls, 0);
    assert_eq!(
        session.events().last().expect("terminal event").reason,
        PrototypeBackendSessionReason::LocalStoreSealRejected
    );
}

#[test]
fn prototype_session_stops_after_store_when_relay_rejects() {
    let mut input = valid_session_input();
    input.outbound_send = OutboundSendDecision {
        accepted: false,
        can_send: false,
        can_persist_ciphertext: false,
        requires_user_action: true,
        reason: OutboundSendReason::MessagePolicyRejected,
    };
    let mut session = PrototypeBackendSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeBackendSessionReason::RelaySubmitRejected
    );
    assert!(outcome.store_write.expect("store decision").accepted);
    assert!(!outcome.relay_submission.expect("relay submission").accepted);
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.relay_items, 0);
    assert_eq!(outcome.ai_audit_records, 0);
    assert_eq!(outcome.crypto_seal_calls, 1);
    assert_eq!(outcome.crypto_open_calls, 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeBackendSessionEventKind::RelaySubmitEvaluated
    );
}

#[test]
fn prototype_session_records_ai_rejection_after_secure_delivery_flow() {
    let mut input = valid_session_input();
    input.ai_request.grant_visible = false;
    let mut session = PrototypeBackendSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeBackendSessionReason::AiParticipantRejected
    );
    assert_eq!(
        outcome.ai.expect("ai decision").reason,
        AiParticipantReason::GrantVisibilityRequired
    );
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.relay_items, 1);
    assert_eq!(outcome.ai_audit_records, 1);
    assert_eq!(outcome.crypto_seal_calls, 1);
    assert_eq!(outcome.crypto_open_calls, 1);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeBackendSessionEventKind::AiParticipantEvaluated
    );
}

fn valid_session_input() -> PrototypeBackendSessionInput<'static> {
    PrototypeBackendSessionInput {
        bootstrap: ClientBootstrapDecision {
            accepted: true,
            can_start_sync: true,
            can_decrypt_local_store: true,
            can_open_message_ui: true,
            requires_sync: false,
            requires_recovery: false,
            requires_user_action: false,
            reason: ClientBootstrapReason::Accepted,
        },
        seal_request: seal_request(PLAINTEXT.len() as i32, Some(policy_decision(true))),
        plaintext: PLAINTEXT,
        outbound_send: OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        },
        route_id: &ROUTE_ID,
        replay_token: &REPLAY_TOKEN,
        queue_ttl_s: 300,
        max_queue_ttl_s: 86400,
        max_ciphertext_len: 1048576,
        sealed_header: &SEALED_HEADER,
        plaintext_identity_fields: 0,
        padding_bucket: 3,
        created_at_s: 100,
        now_s: 120,
        delivery_now_s: 130,
        ai_request: AiParticipantRequest::new(
            AiParticipantAction::DraftReply,
            valid_ai_policy(),
            true,
            true,
            3,
            32,
            32,
            0,
        ),
    }
}

fn seal_request(
    plaintext_len: i32,
    policy_decision: Option<PolicyDecision>,
) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        LocalStoreRecordLocator::new("conversation-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        local_store_key(LocalStoreKeyScope::RoomEpoch),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        plaintext_len,
        policy_decision,
    )
}

fn local_store_key(scope: LocalStoreKeyScope) -> LocalStoreKeyDescriptor {
    LocalStoreKeyDescriptor::new(
        scope,
        LocalStoreSealingSuite::MercuryLocalStoreV1,
        1,
        LocalStoreKeyBinding::room_epoch(32, 32, 7),
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

fn valid_ai_policy() -> AiPolicyFacts {
    AiPolicyFacts {
        grant: AiGrantFacts {
            version: 1,
            principal_kind: 2,
            room_mode: 1,
            ai_mode: 1,
            ttl_s: 300,
            approver_count: 1,
            read_scope: 1,
            write_scope: 1,
            tool_scope: 1,
            retention_mode: 0,
            training_allowed: 0,
            prompt_store_allowed: 0,
        },
        lifecycle: AiLifecycleFacts {
            version: 1,
            grant_state: 1,
            revoke_reason: 0,
            now_s: 100,
            expires_at_s: 400,
            room_mode: 1,
            access_kind: 0,
            epoch_rotated: 0,
        },
    }
}

const PLAINTEXT: &[u8] = b"session payload to seal";
const ROUTE_ID: [u8; 32] = [7; 32];
const REPLAY_TOKEN: [u8; 32] = [9; 32];
const SEALED_HEADER: [u8; 96] = [5; 96];
