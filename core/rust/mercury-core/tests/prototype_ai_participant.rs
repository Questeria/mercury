use mercury_core::{
    AiGrantFacts, AiLifecycleFacts, AiParticipantAction, AiParticipantReason, AiParticipantRequest,
    AiPolicyFacts, PrototypeAiParticipantBackend,
};

#[test]
fn prototype_ai_accepts_visible_local_draft_without_plaintext_audit() {
    let mut backend = PrototypeAiParticipantBackend::default();
    let decision = backend.handle(request(AiParticipantAction::DraftReply, valid_ai()));

    assert!(decision.accepted);
    assert!(decision.can_receive_selected_context);
    assert!(decision.can_emit_draft);
    assert!(!decision.can_send_message);
    assert!(!decision.can_store_prompt);
    assert!(!decision.can_train);
    assert_eq!(decision.reason, AiParticipantReason::Accepted);
    assert_eq!(backend.len(), 1);
    assert_eq!(backend.audit_records()[0].input_digest_len, 32);
    assert_eq!(backend.audit_records()[0].output_digest_len, 32);
}

#[test]
fn prototype_ai_requires_visible_participant_and_visible_grant() {
    let mut hidden_participant = request(AiParticipantAction::DraftReply, valid_ai());
    hidden_participant.participant_visible = false;
    assert_rejected(
        hidden_participant.evaluate(),
        AiParticipantReason::ParticipantVisibilityRequired,
    );

    let mut hidden_grant = request(AiParticipantAction::DraftReply, valid_ai());
    hidden_grant.grant_visible = false;
    assert_rejected(
        hidden_grant.evaluate(),
        AiParticipantReason::GrantVisibilityRequired,
    );
}

#[test]
fn prototype_ai_rejects_hidden_plaintext_bridge_and_bad_audit_digests() {
    let mut plaintext_identity = request(AiParticipantAction::DraftReply, valid_ai());
    plaintext_identity.plaintext_identity_fields = 1;
    assert_rejected(
        plaintext_identity.evaluate(),
        AiParticipantReason::PlaintextIdentityForbidden,
    );

    let mut bad_digest = request(AiParticipantAction::DraftReply, valid_ai());
    bad_digest.input_digest_len = 16;
    assert_rejected(
        bad_digest.evaluate(),
        AiParticipantReason::BadAuditDigestLength,
    );
}

#[test]
fn prototype_ai_rejects_policy_and_lifecycle_failures() {
    let mut bad_grant = valid_ai();
    bad_grant.grant.read_scope = 4;
    assert_rejected(
        request(AiParticipantAction::DraftReply, bad_grant).evaluate(),
        AiParticipantReason::GrantPolicyRejected,
    );

    let mut expired = valid_ai();
    expired.lifecycle.now_s = expired.lifecycle.expires_at_s;
    assert_rejected(
        request(AiParticipantAction::DraftReply, expired).evaluate(),
        AiParticipantReason::GrantLifecycleRejected,
    );
}

#[test]
fn prototype_ai_enforces_action_scopes() {
    let mut no_context = request(AiParticipantAction::DraftReply, valid_ai());
    no_context.selected_message_count = 0;
    assert_rejected(
        no_context.evaluate(),
        AiParticipantReason::SelectedContextRequired,
    );

    let mut read_only = valid_ai();
    read_only.grant.write_scope = 0;
    assert_rejected(
        request(AiParticipantAction::DraftReply, read_only).evaluate(),
        AiParticipantReason::WriteScopeRequired,
    );

    let mut confirmed_send_ai = valid_ai();
    confirmed_send_ai.grant.write_scope = 2;
    let confirmed_send = request(
        AiParticipantAction::SendMessageWithConfirmation,
        confirmed_send_ai,
    )
    .evaluate();
    assert!(confirmed_send.accepted);
    assert!(confirmed_send.can_send_message);
    assert!(confirmed_send.requires_user_confirmation);

    assert_rejected(
        request(AiParticipantAction::AutonomousSend, confirmed_send_ai).evaluate(),
        AiParticipantReason::AutonomousSendForbidden,
    );
}

#[test]
fn prototype_ai_enforces_tool_and_retention_boundaries() {
    assert_rejected(
        request(AiParticipantAction::UseRoomSearchSelectedTool, valid_ai()).evaluate(),
        AiParticipantReason::ToolScopeRequired,
    );

    let mut tool_ai = valid_ai();
    tool_ai.grant.tool_scope = 2;
    let tool = request(AiParticipantAction::UseRoomSearchSelectedTool, tool_ai).evaluate();
    assert!(tool.accepted);
    assert!(tool.can_use_tool);

    assert_rejected(
        request(AiParticipantAction::UseOpenWorldExternalTool, tool_ai).evaluate(),
        AiParticipantReason::ToolScopeRequired,
    );
    assert_rejected(
        request(AiParticipantAction::StorePrompt, valid_ai()).evaluate(),
        AiParticipantReason::PromptStoreForbidden,
    );
    assert_rejected(
        request(AiParticipantAction::TrainOnContext, valid_ai()).evaluate(),
        AiParticipantReason::TrainingForbidden,
    );
    assert_rejected(
        request(AiParticipantAction::WriteMemory, valid_ai()).evaluate(),
        AiParticipantReason::MemoryWriteForbidden,
    );
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

fn assert_rejected(decision: mercury_core::AiParticipantDecision, reason: AiParticipantReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_receive_selected_context);
    assert!(!decision.can_emit_draft);
    assert!(!decision.can_send_message);
    assert!(!decision.can_use_tool);
    assert!(!decision.can_store_prompt);
    assert!(!decision.can_train);
    assert_eq!(decision.reason, reason);
}
