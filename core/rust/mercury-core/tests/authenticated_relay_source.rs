use mercury_core::{
    AuthenticatedRelaySourceInput, AuthenticatedRelaySourceReason,
    AuthenticatedRelayTransportState, ClientBootstrapDecision, ClientBootstrapReason,
    InboundSyncReason, InboundSyncSourceState,
};

#[test]
fn authenticated_source_allows_delivery_ready_inbound_sync() {
    let decision = valid_source().evaluate();

    assert!(decision.accepted);
    assert_eq!(
        decision.reason,
        AuthenticatedRelaySourceReason::DeliveryReady
    );
    assert_eq!(decision.source_state, InboundSyncSourceState::Ready);
    assert!(decision.can_poll_relay);
    assert!(decision.can_run_receive_session);
    assert!(!decision.plaintext_bytes_exposed);

    let sync = decision
        .into_inbound_sync_input(accepted_bootstrap())
        .evaluate();
    assert!(sync.accepted);
    assert_eq!(sync.reason, InboundSyncReason::DeliveryReady);
    assert!(sync.can_run_receive_session);
}

#[test]
fn authenticated_source_idle_does_not_run_receive_session() {
    let mut input = valid_source();
    input.pending_delivery = false;
    input.route_id_len = 0;

    let decision = input.evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason, AuthenticatedRelaySourceReason::Idle);
    assert!(decision.can_poll_relay);
    assert!(!decision.can_run_receive_session);

    let sync = decision
        .into_inbound_sync_input(accepted_bootstrap())
        .evaluate();
    assert!(sync.accepted);
    assert_eq!(sync.reason, InboundSyncReason::Idle);
    assert!(!sync.can_run_receive_session);
}

#[test]
fn transport_failures_map_to_retryable_inbound_source_states() {
    let mut offline = valid_source();
    offline.transport = AuthenticatedRelayTransportState::Offline;
    let offline_decision = offline.evaluate();

    assert!(!offline_decision.accepted);
    assert_eq!(
        offline_decision.reason,
        AuthenticatedRelaySourceReason::TransportOffline
    );
    assert_eq!(
        offline_decision.source_state,
        InboundSyncSourceState::Offline
    );
    assert!(offline_decision.requires_network_retry);
    assert_eq!(
        offline_decision
            .into_inbound_sync_input(accepted_bootstrap())
            .evaluate()
            .reason,
        InboundSyncReason::TransportOffline
    );

    let mut backoff = valid_source();
    backoff.transport = AuthenticatedRelayTransportState::BackoffRequired;
    let backoff_decision = backoff.evaluate();

    assert!(!backoff_decision.accepted);
    assert_eq!(
        backoff_decision.reason,
        AuthenticatedRelaySourceReason::BackoffRequired
    );
    assert_eq!(
        backoff_decision.source_state,
        InboundSyncSourceState::BackoffRequired
    );
    assert!(backoff_decision.requires_network_retry);
}

#[test]
fn authentication_failures_require_user_action_before_sync() {
    let mut bad_ticket = valid_source();
    bad_ticket.session_ticket_len = 12;
    assert_auth_rejected(
        bad_ticket.evaluate(),
        AuthenticatedRelaySourceReason::BadSessionTicketLength,
    );

    let mut bad_credential = valid_source();
    bad_credential.device_credential_len = 16;
    assert_auth_rejected(
        bad_credential.evaluate(),
        AuthenticatedRelaySourceReason::BadDeviceCredentialLength,
    );

    let mut bad_server_tag = valid_source();
    bad_server_tag.server_auth_tag_len = 24;
    assert_auth_rejected(
        bad_server_tag.evaluate(),
        AuthenticatedRelaySourceReason::BadServerAuthTagLength,
    );

    let mut unauthenticated_server = valid_source();
    unauthenticated_server.server_authenticated = false;
    assert_auth_rejected(
        unauthenticated_server.evaluate(),
        AuthenticatedRelaySourceReason::ServerAuthenticationRejected,
    );

    let mut bad_route_key = valid_source();
    bad_route_key.route_key_authenticated = false;
    assert_auth_rejected(
        bad_route_key.evaluate(),
        AuthenticatedRelaySourceReason::RouteKeyAuthenticationRejected,
    );

    let mut replay_rejected = valid_source();
    replay_rejected.replay_window_valid = false;
    assert_auth_rejected(
        replay_rejected.evaluate(),
        AuthenticatedRelaySourceReason::ReplayWindowRejected,
    );
}

#[test]
fn plaintext_or_malformed_delivery_metadata_never_reaches_receive_path() {
    let mut plaintext_preview = valid_source();
    plaintext_preview.plaintext_notification_preview_len = 1;
    assert_auth_rejected(
        plaintext_preview.evaluate(),
        AuthenticatedRelaySourceReason::PlaintextNotificationPreviewForbidden,
    );

    let mut plaintext_identity = valid_source();
    plaintext_identity.plaintext_identity_fields = 1;
    assert_auth_rejected(
        plaintext_identity.evaluate(),
        AuthenticatedRelaySourceReason::PlaintextIdentityForbidden,
    );

    let mut bad_batch = valid_source();
    bad_batch.poll_batch_limit = 101;
    let bad_batch_decision = bad_batch.evaluate();
    assert!(!bad_batch_decision.accepted);
    assert_eq!(
        bad_batch_decision.reason,
        AuthenticatedRelaySourceReason::BadPollBatchLimit
    );
    assert!(!bad_batch_decision.requires_user_action);
    assert_eq!(
        bad_batch_decision
            .into_inbound_sync_input(accepted_bootstrap())
            .evaluate()
            .reason,
        InboundSyncReason::BadPollBatchLimit
    );

    let mut bad_route = valid_source();
    bad_route.route_id_len = 31;
    let bad_route_decision = bad_route.evaluate();
    assert!(!bad_route_decision.accepted);
    assert_eq!(
        bad_route_decision.reason,
        AuthenticatedRelaySourceReason::BadRouteIdLength
    );
    assert!(!bad_route_decision.can_run_receive_session);
    assert_eq!(
        bad_route_decision
            .into_inbound_sync_input(accepted_bootstrap())
            .evaluate()
            .reason,
        InboundSyncReason::BadRouteIdLength
    );
}

#[test]
fn authenticated_source_reasons_and_transport_states_have_stable_codes_and_labels() {
    let transports = [
        (AuthenticatedRelayTransportState::Ready, 1, "ready"),
        (AuthenticatedRelayTransportState::Offline, 2, "offline"),
        (
            AuthenticatedRelayTransportState::BackoffRequired,
            3,
            "backoff_required",
        ),
    ];

    for (transport, code, label) in transports {
        assert_eq!(transport.code(), code);
        assert_eq!(transport.label(), label);
    }

    let reasons = [
        (
            AuthenticatedRelaySourceReason::DeliveryReady,
            0,
            "DELIVERY_READY",
        ),
        (AuthenticatedRelaySourceReason::Idle, 1, "IDLE"),
        (
            AuthenticatedRelaySourceReason::TransportOffline,
            2,
            "TRANSPORT_OFFLINE",
        ),
        (
            AuthenticatedRelaySourceReason::BackoffRequired,
            3,
            "BACKOFF_REQUIRED",
        ),
        (
            AuthenticatedRelaySourceReason::BadSessionTicketLength,
            4,
            "BAD_SESSION_TICKET_LENGTH",
        ),
        (
            AuthenticatedRelaySourceReason::BadDeviceCredentialLength,
            5,
            "BAD_DEVICE_CREDENTIAL_LENGTH",
        ),
        (
            AuthenticatedRelaySourceReason::BadServerAuthTagLength,
            6,
            "BAD_SERVER_AUTH_TAG_LENGTH",
        ),
        (
            AuthenticatedRelaySourceReason::ServerAuthenticationRejected,
            7,
            "SERVER_AUTHENTICATION_REJECTED",
        ),
        (
            AuthenticatedRelaySourceReason::RouteKeyAuthenticationRejected,
            8,
            "ROUTE_KEY_AUTHENTICATION_REJECTED",
        ),
        (
            AuthenticatedRelaySourceReason::ReplayWindowRejected,
            9,
            "REPLAY_WINDOW_REJECTED",
        ),
        (
            AuthenticatedRelaySourceReason::PlaintextIdentityForbidden,
            10,
            "PLAINTEXT_IDENTITY_FORBIDDEN",
        ),
        (
            AuthenticatedRelaySourceReason::PlaintextNotificationPreviewForbidden,
            11,
            "PLAINTEXT_NOTIFICATION_PREVIEW_FORBIDDEN",
        ),
        (
            AuthenticatedRelaySourceReason::BadPollBatchLimit,
            12,
            "BAD_POLL_BATCH_LIMIT",
        ),
        (
            AuthenticatedRelaySourceReason::BadRouteIdLength,
            13,
            "BAD_ROUTE_ID_LENGTH",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_auth_rejected(
    decision: mercury_core::AuthenticatedRelaySourceDecision,
    reason: AuthenticatedRelaySourceReason,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert_eq!(decision.source_state, InboundSyncSourceState::AuthRejected);
    assert!(!decision.can_poll_relay);
    assert!(!decision.can_run_receive_session);
    assert!(decision.requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
}

fn valid_source() -> AuthenticatedRelaySourceInput {
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
