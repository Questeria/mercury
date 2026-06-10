use mercury_core::{
    AuthenticatedRelaySourceInput, AuthenticatedRelaySourceReason,
    AuthenticatedRelayTransportState, ClientBootstrapDecision, ClientBootstrapReason,
    ClientReceiveReplayState, ComponentReasons, DeviceTrustDecision, DeviceTrustReason,
    InboundSyncInput, InboundSyncSourceState, LocalStoreRecordKind, LocalStoreRecordLocator,
    LocalStoreSealingDecision, LocalStoreSealingReason, OutboundSendDecision, OutboundSendReason,
    PolicyDecision, PrototypeAuthenticatedInboundSyncSessionInput, PrototypeInboundSyncSession,
    PrototypeInboundSyncSessionEventKind, PrototypeInboundSyncSessionInput,
    PrototypeInboundSyncSessionReason, PrototypeReceiveSessionInput, PrototypeRelaySubmitRequest,
};

const ROUTE_ID: [u8; 32] = [7; 32];
const REPLAY_TOKEN: [u8; 32] = [8; 32];
const CIPHERTEXT: [u8; 128] = [9; 128];
const SEALED_HEADER: [u8; 48] = [10; 48];

#[test]
fn inbound_sync_session_runs_receive_and_records_single_transcript() {
    let mut session = PrototypeInboundSyncSession::default();
    let outcome = session.run(valid_session_input());

    assert!(outcome.completed);
    assert_eq!(outcome.reason, PrototypeInboundSyncSessionReason::Completed);
    assert!(outcome.sync.accepted);
    assert!(outcome.receive_ran);
    assert!(outcome.receive.expect("receive outcome").completed);
    assert_eq!(outcome.local_store_records, 1);
    assert!(!outcome.plaintext_exposed);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeInboundSyncSessionEventKind::SyncGateEvaluated
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeInboundSyncSessionEventKind::SyncFinished
    );
    assert!(session.events().last().expect("last event").terminal);
    assert!(
        session
            .events()
            .iter()
            .all(|event| !event.plaintext_bytes_exposed)
    );
}

#[test]
fn idle_sync_finishes_without_receive_side_effects() {
    let mut session = PrototypeInboundSyncSession::default();
    let mut input = valid_session_input();
    input.sync.pending_delivery = false;
    input.sync.route_id_len = 0;

    let outcome = session.run(input);

    assert!(outcome.completed);
    assert_eq!(outcome.reason, PrototypeInboundSyncSessionReason::SyncIdle);
    assert!(!outcome.receive_ran);
    assert!(outcome.receive.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(session.events().len(), 1);
    assert!(session.events()[0].terminal);
}

#[test]
fn rejected_sync_does_not_run_receive_session() {
    let mut session = PrototypeInboundSyncSession::default();
    let mut input = valid_session_input();
    input.sync.bootstrap = blocked_bootstrap();

    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeInboundSyncSessionReason::SyncRejected
    );
    assert!(!outcome.receive_ran);
    assert!(outcome.receive.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(
        session.events().last().expect("terminal event").reason,
        PrototypeInboundSyncSessionReason::SyncRejected
    );
}

#[test]
fn receive_rejection_becomes_terminal_sync_session_event() {
    let mut session = PrototypeInboundSyncSession::default();
    let mut input = valid_session_input();
    input.receive.ack_token_len = 12;

    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeInboundSyncSessionReason::ReceiveRejected
    );
    assert!(outcome.receive_ran);
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeInboundSyncSessionEventKind::DeliveryAckEvaluated
    );
    assert!(session.events().last().expect("terminal event").terminal);
    assert_eq!(
        session.events().last().expect("terminal event").reason,
        PrototypeInboundSyncSessionReason::ReceiveRejected
    );
}

#[test]
fn authenticated_source_session_runs_receive_after_source_acceptance() {
    let mut session = PrototypeInboundSyncSession::default();
    let outcome = session.run_authenticated_source(valid_authenticated_session_input());

    assert!(outcome.relay_source.accepted);
    assert_eq!(
        outcome.relay_source.reason,
        AuthenticatedRelaySourceReason::DeliveryReady
    );
    assert!(outcome.session.completed);
    assert_eq!(
        outcome.session.reason,
        PrototypeInboundSyncSessionReason::Completed
    );
    assert!(outcome.session.receive_ran);
    assert_eq!(outcome.session.local_store_records, 1);
    assert!(!outcome.session.plaintext_exposed);
}

#[test]
fn authenticated_source_idle_finishes_without_receive_side_effects() {
    let mut session = PrototypeInboundSyncSession::default();
    let mut input = valid_authenticated_session_input();
    input.relay_source.pending_delivery = false;
    input.relay_source.route_id_len = 0;

    let outcome = session.run_authenticated_source(input);

    assert!(outcome.relay_source.accepted);
    assert_eq!(
        outcome.relay_source.reason,
        AuthenticatedRelaySourceReason::Idle
    );
    assert!(outcome.session.completed);
    assert_eq!(
        outcome.session.reason,
        PrototypeInboundSyncSessionReason::SyncIdle
    );
    assert!(!outcome.session.receive_ran);
    assert_eq!(session.events().len(), 1);
}

#[test]
fn authenticated_source_rejection_stops_before_receive_session() {
    let mut session = PrototypeInboundSyncSession::default();
    let mut input = valid_authenticated_session_input();
    input.relay_source.server_authenticated = false;

    let outcome = session.run_authenticated_source(input);

    assert!(!outcome.relay_source.accepted);
    assert_eq!(
        outcome.relay_source.reason,
        AuthenticatedRelaySourceReason::ServerAuthenticationRejected
    );
    assert!(!outcome.session.completed);
    assert_eq!(
        outcome.session.reason,
        PrototypeInboundSyncSessionReason::SyncRejected
    );
    assert!(!outcome.session.receive_ran);
    assert_eq!(outcome.session.local_store_records, 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeInboundSyncSessionEventKind::SyncGateEvaluated
    );
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn inbound_sync_session_reasons_and_events_have_stable_codes_and_labels() {
    assert_eq!(PrototypeInboundSyncSessionReason::Completed.code(), 0);
    assert_eq!(
        PrototypeInboundSyncSessionReason::Completed.label(),
        "completed"
    );
    assert_eq!(PrototypeInboundSyncSessionReason::SyncRejected.code(), 1);
    assert_eq!(
        PrototypeInboundSyncSessionReason::ReceiveRejected.label(),
        "receive_rejected"
    );
    assert_eq!(
        PrototypeInboundSyncSessionEventKind::SyncGateEvaluated.code(),
        1
    );
    assert_eq!(
        PrototypeInboundSyncSessionEventKind::SyncGateEvaluated.label(),
        "sync_gate_evaluated"
    );
    assert_eq!(PrototypeInboundSyncSessionEventKind::SyncFinished.code(), 9);
    assert_eq!(
        PrototypeInboundSyncSessionEventKind::SyncFinished.label(),
        "sync_finished"
    );
}

fn valid_session_input() -> PrototypeInboundSyncSessionInput<'static> {
    PrototypeInboundSyncSessionInput {
        sync: valid_sync_input(),
        receive: valid_receive_input(),
    }
}

fn valid_authenticated_session_input() -> PrototypeAuthenticatedInboundSyncSessionInput<'static> {
    PrototypeAuthenticatedInboundSyncSessionInput {
        bootstrap: accepted_bootstrap(),
        relay_source: valid_authenticated_relay_source(),
        receive: valid_receive_input(),
    }
}

fn valid_sync_input() -> InboundSyncInput {
    InboundSyncInput {
        bootstrap: accepted_bootstrap(),
        source_state: InboundSyncSourceState::Ready,
        pending_delivery: true,
        route_id_len: 32,
        poll_batch_limit: 25,
        plaintext_notification_preview_len: 0,
    }
}

fn valid_authenticated_relay_source() -> AuthenticatedRelaySourceInput {
    AuthenticatedRelaySourceInput {
        transport: AuthenticatedRelayTransportState::Ready,
        session_ticket_len: 32,
        device_credential_len: 32,
        server_auth_tag_len: 32,
        server_authenticated: true,
        route_key_authenticated: true,
        replay_window_valid: true,
        pending_delivery: true,
        route_id_len: 32,
        poll_batch_limit: 25,
        plaintext_notification_preview_len: 0,
        plaintext_identity_fields: 0,
    }
}

fn accepted_bootstrap() -> ClientBootstrapDecision {
    ClientBootstrapDecision {
        accepted: true,
        can_start_sync: true,
        can_decrypt_local_store: true,
        can_open_message_ui: true,
        requires_sync: false,
        requires_recovery: false,
        requires_user_action: false,
        reason: ClientBootstrapReason::Accepted,
    }
}

fn blocked_bootstrap() -> ClientBootstrapDecision {
    ClientBootstrapDecision {
        accepted: false,
        can_start_sync: false,
        can_decrypt_local_store: false,
        can_open_message_ui: false,
        requires_sync: false,
        requires_recovery: true,
        requires_user_action: true,
        reason: ClientBootstrapReason::RecoveryRequired,
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
        sender_device_trust: trusted_device(),
        message_policy: policy_decision(),
        ciphertext_sealing: sealing_decision(),
        store_locator: LocalStoreRecordLocator::new("conversation-7", "inbound-message-42"),
        store_record_kind: LocalStoreRecordKind::MessageCiphertext,
        plaintext_identity_fields: 0,
    }
}

fn trusted_device() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: true,
        can_send: true,
        requires_user_action: false,
        reason: DeviceTrustReason::Trusted,
    }
}

fn policy_decision() -> PolicyDecision {
    PolicyDecision {
        accepted: true,
        reason_code: 0,
        audit_class: 0,
        components: ComponentReasons {
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    }
}

fn sealing_decision() -> LocalStoreSealingDecision {
    LocalStoreSealingDecision {
        accepted: true,
        reason: LocalStoreSealingReason::Accepted,
        record_policy: LocalStoreRecordKind::MessageCiphertext.policy(),
    }
}
