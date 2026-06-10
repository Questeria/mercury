use mercury_core::{
    AiConnectorInput, AiConnectorReason, AiConnectorRuntimeKind, AiGrantFacts, AiLifecycleFacts,
    AiParticipantAction, AiParticipantReason, AiParticipantRequest, AiPolicyFacts,
};

#[test]
fn ai_connector_accepts_user_selected_local_draft_only_runtime() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason, AiConnectorReason::Accepted);
    assert_eq!(decision.runtime_kind, AiConnectorRuntimeKind::LocalDevice);
    assert!(decision.participant.accepted);
    assert!(decision.can_call_model);
    assert!(decision.can_emit_draft);
    assert!(!decision.can_send_message);
    assert!(!decision.can_use_tool);
    assert!(decision.requires_user_review);
    assert!(!decision.requires_user_setup);
    assert!(!decision.requires_model_setup);
    assert!(decision.forbids_prompt_retention);
    assert!(decision.forbids_training);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn ai_connector_rejects_participant_failures_and_non_draft_actions() {
    let mut hidden_grant = valid_input();
    hidden_grant.participant_request.grant_visible = false;
    let hidden_grant_decision = hidden_grant.evaluate();
    assert_rejected(
        hidden_grant_decision,
        AiConnectorReason::ParticipantRejected,
    );
    assert_eq!(
        hidden_grant_decision.participant.reason,
        AiParticipantReason::GrantVisibilityRequired
    );

    let mut read_context = valid_input();
    read_context.participant_request =
        request(AiParticipantAction::ReadSelectedContext, valid_ai());
    let read_context_decision = read_context.evaluate();
    assert_rejected(
        read_context_decision,
        AiConnectorReason::DraftActionRequired,
    );
    assert!(read_context_decision.participant.accepted);
    assert!(!read_context_decision.participant.can_emit_draft);
}

#[test]
fn ai_connector_requires_user_selected_runtime_and_model() {
    let mut runtime_missing = valid_input();
    runtime_missing.runtime_user_selected = false;
    let runtime_missing_decision = runtime_missing.evaluate();
    assert_rejected(
        runtime_missing_decision,
        AiConnectorReason::RuntimeNotUserSelected,
    );
    assert!(runtime_missing_decision.requires_user_setup);

    let mut model_missing = valid_input();
    model_missing.model_user_selected = false;
    let model_missing_decision = model_missing.evaluate();
    assert_rejected(
        model_missing_decision,
        AiConnectorReason::ModelNotUserSelected,
    );
    assert!(model_missing_decision.requires_model_setup);
}

#[test]
fn ai_connector_rejects_unapproved_runtime_classes() {
    let mut development = valid_input();
    development.runtime_kind = AiConnectorRuntimeKind::DevelopmentStub;
    assert_rejected(
        development.evaluate(),
        AiConnectorReason::DevelopmentRuntimeForbidden,
    );

    let mut remote = valid_input();
    remote.runtime_kind = AiConnectorRuntimeKind::RemoteProvider;
    assert_rejected(remote.evaluate(), AiConnectorReason::RemoteRuntimeForbidden);

    let mut remote_allowed = valid_input();
    remote_allowed.runtime_kind = AiConnectorRuntimeKind::RemoteProvider;
    remote_allowed.allow_remote_runtime = true;
    assert!(remote_allowed.evaluate().accepted);

    let mut high_security = valid_input();
    high_security.runtime_kind = AiConnectorRuntimeKind::UserHostedLocalNetwork;
    high_security.high_security_room = true;
    assert_rejected(
        high_security.evaluate(),
        AiConnectorReason::HighSecurityRequiresLocalRuntime,
    );
}

#[test]
fn ai_connector_requires_authenticated_integrity_checked_digest_only_bridge() {
    let mut unauthenticated = valid_input();
    unauthenticated.connector_authenticated = false;
    let unauthenticated_decision = unauthenticated.evaluate();
    assert_rejected(
        unauthenticated_decision,
        AiConnectorReason::ConnectorAuthenticationMissing,
    );
    assert!(unauthenticated_decision.requires_user_setup);

    let mut unverified_model = valid_input();
    unverified_model.model_integrity_verified = false;
    let unverified_model_decision = unverified_model.evaluate();
    assert_rejected(
        unverified_model_decision,
        AiConnectorReason::ModelIntegrityUnverified,
    );
    assert!(unverified_model_decision.requires_model_setup);

    let mut missing_context_digest = valid_input();
    missing_context_digest.context_digest_len = 0;
    assert_rejected(
        missing_context_digest.evaluate(),
        AiConnectorReason::ContextDigestRequired,
    );

    let mut missing_output_digest = valid_input();
    missing_output_digest.draft_output_digest_len = 0;
    assert_rejected(
        missing_output_digest.evaluate(),
        AiConnectorReason::DraftOutputDigestRequired,
    );

    let mut plaintext_bridge = valid_input();
    plaintext_bridge.plaintext_bridge_fields = 1;
    let plaintext_bridge_decision = plaintext_bridge.evaluate();
    assert_rejected(
        plaintext_bridge_decision,
        AiConnectorReason::PlaintextBridgeForbidden,
    );
    assert!(!plaintext_bridge_decision.plaintext_bytes_exposed);
}

#[test]
fn ai_connector_rejects_retention_training_direct_send_and_tool_execution() {
    let mut retention = valid_input();
    retention.prompt_retention_enabled = true;
    assert_rejected(
        retention.evaluate(),
        AiConnectorReason::PromptRetentionForbidden,
    );

    let mut training = valid_input();
    training.training_enabled = true;
    assert_rejected(training.evaluate(), AiConnectorReason::TrainingForbidden);

    let mut direct_send = valid_input();
    direct_send.direct_send_enabled = true;
    assert_rejected(
        direct_send.evaluate(),
        AiConnectorReason::DirectSendForbidden,
    );

    let mut tool_execution = valid_input();
    tool_execution.tool_execution_enabled = true;
    assert_rejected(
        tool_execution.evaluate(),
        AiConnectorReason::ToolExecutionForbidden,
    );
}

#[test]
fn ai_connector_reasons_and_runtime_kinds_have_stable_codes_and_labels() {
    let runtimes = [
        (AiConnectorRuntimeKind::LocalDevice, 1, "local_device"),
        (
            AiConnectorRuntimeKind::UserHostedLocalNetwork,
            2,
            "user_hosted_local_network",
        ),
        (AiConnectorRuntimeKind::RemoteProvider, 3, "remote_provider"),
        (
            AiConnectorRuntimeKind::DevelopmentStub,
            4,
            "development_stub",
        ),
    ];

    for (runtime, code, label) in runtimes {
        assert_eq!(runtime.code(), code);
        assert_eq!(runtime.label(), label);
    }

    let reasons = [
        (AiConnectorReason::Accepted, 0, "ACCEPTED"),
        (
            AiConnectorReason::ParticipantRejected,
            1,
            "PARTICIPANT_REJECTED",
        ),
        (
            AiConnectorReason::DraftActionRequired,
            2,
            "DRAFT_ACTION_REQUIRED",
        ),
        (
            AiConnectorReason::RuntimeNotUserSelected,
            3,
            "RUNTIME_NOT_USER_SELECTED",
        ),
        (
            AiConnectorReason::ModelNotUserSelected,
            4,
            "MODEL_NOT_USER_SELECTED",
        ),
        (
            AiConnectorReason::DevelopmentRuntimeForbidden,
            5,
            "DEVELOPMENT_RUNTIME_FORBIDDEN",
        ),
        (
            AiConnectorReason::RemoteRuntimeForbidden,
            6,
            "REMOTE_RUNTIME_FORBIDDEN",
        ),
        (
            AiConnectorReason::HighSecurityRequiresLocalRuntime,
            7,
            "HIGH_SECURITY_REQUIRES_LOCAL_RUNTIME",
        ),
        (
            AiConnectorReason::ConnectorAuthenticationMissing,
            8,
            "CONNECTOR_AUTHENTICATION_MISSING",
        ),
        (
            AiConnectorReason::ModelIntegrityUnverified,
            9,
            "MODEL_INTEGRITY_UNVERIFIED",
        ),
        (
            AiConnectorReason::ContextDigestRequired,
            10,
            "CONTEXT_DIGEST_REQUIRED",
        ),
        (
            AiConnectorReason::DraftOutputDigestRequired,
            11,
            "DRAFT_OUTPUT_DIGEST_REQUIRED",
        ),
        (
            AiConnectorReason::PlaintextBridgeForbidden,
            12,
            "PLAINTEXT_BRIDGE_FORBIDDEN",
        ),
        (
            AiConnectorReason::PromptRetentionForbidden,
            13,
            "PROMPT_RETENTION_FORBIDDEN",
        ),
        (
            AiConnectorReason::TrainingForbidden,
            14,
            "TRAINING_FORBIDDEN",
        ),
        (
            AiConnectorReason::DirectSendForbidden,
            15,
            "DIRECT_SEND_FORBIDDEN",
        ),
        (
            AiConnectorReason::ToolExecutionForbidden,
            16,
            "TOOL_EXECUTION_FORBIDDEN",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> AiConnectorInput {
    AiConnectorInput {
        participant_request: request(AiParticipantAction::DraftReply, valid_ai()),
        runtime_kind: AiConnectorRuntimeKind::LocalDevice,
        runtime_user_selected: true,
        model_user_selected: true,
        connector_authenticated: true,
        model_integrity_verified: true,
        allow_development_runtime: false,
        allow_remote_runtime: false,
        high_security_room: false,
        context_digest_len: 32,
        draft_output_digest_len: 32,
        plaintext_bridge_fields: 0,
        prompt_retention_enabled: false,
        training_enabled: false,
        direct_send_enabled: false,
        tool_execution_enabled: false,
    }
}

fn request(action: AiParticipantAction, ai: AiPolicyFacts) -> AiParticipantRequest {
    AiParticipantRequest::new(action, ai, true, true, 3, 32, 32, 0)
}

fn valid_ai() -> AiPolicyFacts {
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

fn assert_rejected(decision: mercury_core::AiConnectorDecision, reason: AiConnectorReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_call_model);
    assert!(!decision.can_emit_draft);
    assert!(!decision.can_send_message);
    assert!(!decision.can_use_tool);
    assert!(!decision.requires_user_review);
    assert!(decision.forbids_prompt_retention);
    assert!(decision.forbids_training);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
