#![cfg(feature = "serde")]

use mercury_core::{
    ClientBootstrapDecision, ClientBootstrapReason, ComponentReasons, PlatformDecisionView,
    PolicyDecision,
};

#[test]
fn decision_view_serializes_compact_fields() {
    let decision = PolicyDecision {
        accepted: false,
        reason_code: 8,
        audit_class: 3,
        components: ComponentReasons {
            envelope_reason: 10,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    };

    let json = serde_json::to_value(decision.view()).expect("decision view should serialize");

    assert_eq!(json["accepted"], false);
    assert_eq!(json["reason"]["source"], "policy_pipeline");
    assert_eq!(json["reason"]["code"], 8);
    assert_eq!(json["reason"]["label"], "ENVELOPE_REJECT");
    assert_eq!(json["audit_class"]["label"], "MESSAGE_POLICY_REJECT");
    assert_eq!(json["primary_source"], "envelope");
    assert_eq!(json["primary_component"]["source"], "envelope");
    assert_eq!(json["primary_component"]["code"], 10);
    assert_eq!(json["primary_component"]["label"], "PAYLOAD_TOO_LARGE");
    assert_eq!(
        json["components"]["envelope_reason"]["label"],
        "PAYLOAD_TOO_LARGE"
    );
    assert_eq!(json["components"]["room_epoch_reason"]["label"], "ACCEPT");
    assert_eq!(json["components"]["ai_grant_reason"]["label"], "ACCEPT");
    assert_eq!(json["components"]["ai_lifecycle_reason"]["label"], "ACCEPT");
}

#[test]
fn platform_decision_view_serializes_binding_fields() {
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

    let json = serde_json::to_value(view).expect("platform view should serialize");

    assert_eq!(json["source"], "client_bootstrap");
    assert_eq!(json["accepted"], false);
    assert_eq!(json["reason_code"], 18);
    assert_eq!(json["reason_label"], "SYNC_INCOMPLETE");
    assert_eq!(json["can_start_sync"], true);
    assert_eq!(json["can_open_message_ui"], false);
    assert_eq!(json["requires_sync"], true);
}
