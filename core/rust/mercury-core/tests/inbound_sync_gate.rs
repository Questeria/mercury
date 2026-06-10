use mercury_core::{
    ClientBootstrapDecision, ClientBootstrapReason, InboundSyncDecision, InboundSyncInput,
    InboundSyncReason, InboundSyncSourceState, evaluate_inbound_sync,
};

#[test]
fn ready_inbound_sync_can_poll_and_run_receive_session() {
    let decision = evaluate_inbound_sync(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_poll_relay);
    assert!(decision.can_run_receive_session);
    assert!(decision.can_update_replay_checkpoint);
    assert!(!decision.requires_network_retry);
    assert!(!decision.requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, InboundSyncReason::DeliveryReady);
}

#[test]
fn idle_inbound_sync_can_poll_without_running_receive_session() {
    let mut input = valid_input();
    input.pending_delivery = false;
    input.route_id_len = 0;

    let decision = input.evaluate();

    assert!(decision.accepted);
    assert!(decision.can_poll_relay);
    assert!(!decision.can_run_receive_session);
    assert!(decision.can_update_replay_checkpoint);
    assert_eq!(decision.reason, InboundSyncReason::Idle);
}

#[test]
fn plaintext_notification_preview_blocks_before_bootstrap_state() {
    let mut input = valid_input();
    input.bootstrap = blocked_bootstrap(true);
    input.plaintext_notification_preview_len = 12;

    assert_rejected(
        input.evaluate(),
        InboundSyncReason::PlaintextNotificationPreviewForbidden,
        false,
        true,
    );
}

#[test]
fn bootstrap_must_allow_sync_before_relay_poll() {
    let mut input = valid_input();
    input.bootstrap = blocked_bootstrap(true);

    assert_rejected(
        input.evaluate(),
        InboundSyncReason::BootstrapBlocked,
        false,
        true,
    );
}

#[test]
fn transport_state_controls_network_retry_and_user_action() {
    let mut offline = valid_input();
    offline.source_state = InboundSyncSourceState::Offline;
    assert_rejected(
        offline.evaluate(),
        InboundSyncReason::TransportOffline,
        true,
        false,
    );

    let mut backoff = valid_input();
    backoff.source_state = InboundSyncSourceState::BackoffRequired;
    assert_rejected(
        backoff.evaluate(),
        InboundSyncReason::BackoffRequired,
        true,
        false,
    );

    let mut auth = valid_input();
    auth.source_state = InboundSyncSourceState::AuthRejected;
    assert_rejected(
        auth.evaluate(),
        InboundSyncReason::TransportAuthRejected,
        false,
        true,
    );
}

#[test]
fn sync_contract_rejects_bad_batch_limits_and_delivery_routes() {
    let mut zero_batch = valid_input();
    zero_batch.poll_batch_limit = 0;
    assert_rejected(
        zero_batch.evaluate(),
        InboundSyncReason::BadPollBatchLimit,
        false,
        false,
    );

    let mut oversized_batch = valid_input();
    oversized_batch.poll_batch_limit = 101;
    assert_rejected(
        oversized_batch.evaluate(),
        InboundSyncReason::BadPollBatchLimit,
        false,
        false,
    );

    let mut bad_route = valid_input();
    bad_route.route_id_len = 16;
    assert_rejected(
        bad_route.evaluate(),
        InboundSyncReason::BadRouteIdLength,
        false,
        false,
    );
}

#[test]
fn inbound_sync_reasons_have_stable_codes_and_labels() {
    assert_eq!(InboundSyncReason::DeliveryReady.code(), 0);
    assert_eq!(InboundSyncReason::DeliveryReady.label(), "DELIVERY_READY");
    assert_eq!(InboundSyncReason::Idle.code(), 1);
    assert_eq!(InboundSyncReason::Idle.label(), "IDLE");
    assert_eq!(InboundSyncReason::BootstrapBlocked.code(), 2);
    assert_eq!(
        InboundSyncReason::BootstrapBlocked.label(),
        "BOOTSTRAP_BLOCKED"
    );
    assert_eq!(
        InboundSyncReason::PlaintextNotificationPreviewForbidden.code(),
        8
    );
    assert_eq!(
        InboundSyncReason::PlaintextNotificationPreviewForbidden.label(),
        "PLAINTEXT_NOTIFICATION_PREVIEW_FORBIDDEN"
    );
}

fn valid_input() -> InboundSyncInput {
    InboundSyncInput {
        bootstrap: accepted_bootstrap(),
        source_state: InboundSyncSourceState::Ready,
        pending_delivery: true,
        route_id_len: 32,
        poll_batch_limit: 25,
        plaintext_notification_preview_len: 0,
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

fn blocked_bootstrap(requires_user_action: bool) -> ClientBootstrapDecision {
    ClientBootstrapDecision {
        accepted: false,
        can_start_sync: false,
        can_decrypt_local_store: false,
        can_open_message_ui: false,
        requires_sync: false,
        requires_recovery: true,
        requires_user_action,
        reason: ClientBootstrapReason::RecoveryRequired,
    }
}

fn assert_rejected(
    decision: InboundSyncDecision,
    reason: InboundSyncReason,
    requires_network_retry: bool,
    requires_user_action: bool,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_poll_relay);
    assert!(!decision.can_run_receive_session);
    assert!(!decision.can_update_replay_checkpoint);
    assert_eq!(decision.requires_network_retry, requires_network_retry);
    assert_eq!(decision.requires_user_action, requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
