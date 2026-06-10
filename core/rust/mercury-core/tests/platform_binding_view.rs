use mercury_core::{
    ClientBootstrapDecision, ClientBootstrapReason, ClientReceiveDecision, ClientReceiveReason,
    ComponentReasons, OutboundSendDecision, OutboundSendReason, PlatformDecisionView,
    PolicyDecision,
};

#[test]
fn bootstrap_view_keeps_message_ui_closed_during_sync() {
    let view = PlatformDecisionView::from_bootstrap(ClientBootstrapDecision {
        accepted: false,
        can_start_sync: true,
        can_decrypt_local_store: false,
        can_open_message_ui: false,
        requires_sync: true,
        requires_recovery: false,
        requires_user_action: false,
        reason: ClientBootstrapReason::SyncIncomplete,
    });

    assert_eq!(view.source, "client_bootstrap");
    assert!(!view.accepted);
    assert_eq!(view.reason_code, 18);
    assert_eq!(view.reason_label, "SYNC_INCOMPLETE");
    assert!(view.can_start_sync);
    assert!(!view.can_open_message_ui);
    assert!(view.requires_sync);
}

#[test]
fn send_and_receive_views_project_binding_capabilities() {
    let send = PlatformDecisionView::from_outbound_send(OutboundSendDecision {
        accepted: true,
        can_send: true,
        can_persist_ciphertext: true,
        requires_user_action: false,
        reason: OutboundSendReason::Accepted,
    });

    assert_eq!(send.source, "outbound_send");
    assert_eq!(send.reason_label, "ACCEPTED");
    assert!(send.can_send);
    assert!(send.can_persist_ciphertext);
    assert!(!send.can_receive);

    let receive = PlatformDecisionView::from_client_receive(ClientReceiveDecision {
        accepted: false,
        can_decrypt: false,
        can_persist_ciphertext: false,
        can_expose_to_ui: false,
        requires_client_retry: true,
        requires_user_action: false,
        reason: ClientReceiveReason::OrderingGap,
    });

    assert_eq!(receive.source, "client_receive");
    assert_eq!(receive.reason_code, 10);
    assert_eq!(receive.reason_label, "ORDERING_GAP");
    assert!(receive.requires_client_retry);
    assert!(!receive.can_open_message_ui);
    assert!(!receive.can_receive);
}

#[test]
fn policy_view_uses_existing_policy_labels() {
    let policy = PlatformDecisionView::from_policy(PolicyDecision {
        accepted: false,
        reason_code: 8,
        audit_class: 3,
        components: ComponentReasons {
            envelope_reason: 10,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    });

    assert_eq!(policy.source, "policy_pipeline");
    assert!(!policy.accepted);
    assert_eq!(policy.reason_code, 8);
    assert_eq!(policy.reason_label, "ENVELOPE_REJECT");
    assert!(!policy.can_open_message_ui);
    assert!(!policy.can_send);
}
