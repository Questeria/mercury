use mercury_core::{
    ActorKind, AiGrantFacts, AiLifecycleFacts, AiPolicyFacts, EnvelopeFacts, PolicyEvaluationInput,
    RoomEpochFacts, evaluate_policy,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: CorePolicyInputJson,
    expected: ExpectedDecision,
}

#[derive(Debug, Deserialize)]
struct CorePolicyInputJson {
    actor_kind: i32,
    envelope: EnvelopeFactsJson,
    room_epoch: RoomEpochFactsJson,
    #[serde(default)]
    ai: Option<AiPolicyFactsJson>,
}

#[derive(Debug, Deserialize)]
struct EnvelopeFactsJson {
    version: i32,
    suite_id: i32,
    conversation_id_len: i32,
    sender_account_id_len: i32,
    sender_device_id_len: i32,
    epoch: i32,
    sequence: i32,
    message_kind: i32,
    payload_len: i32,
    critical_flags: i32,
    noncritical_flags: i32,
    expected_epoch: i32,
    expected_sequence: i32,
    min_suite_id: i32,
    max_payload_len: i32,
}

#[derive(Debug, Deserialize)]
struct RoomEpochFactsJson {
    version: i32,
    room_mode: i32,
    device_kind: i32,
    device_state: i32,
    current_epoch: i32,
    message_epoch: i32,
    min_accepted_epoch: i32,
    revoked_at_epoch: i32,
    access_kind: i32,
}

#[derive(Debug, Deserialize)]
struct AiPolicyFactsJson {
    grant: AiGrantFactsJson,
    lifecycle: AiLifecycleFactsJson,
}

#[derive(Debug, Deserialize)]
struct AiGrantFactsJson {
    version: i32,
    principal_kind: i32,
    room_mode: i32,
    ai_mode: i32,
    ttl_s: i32,
    approver_count: i32,
    read_scope: i32,
    write_scope: i32,
    tool_scope: i32,
    retention_mode: i32,
    training_allowed: i32,
    prompt_store_allowed: i32,
}

#[derive(Debug, Deserialize)]
struct AiLifecycleFactsJson {
    version: i32,
    grant_state: i32,
    revoke_reason: i32,
    now_s: i32,
    expires_at_s: i32,
    room_mode: i32,
    access_kind: i32,
    epoch_rotated: i32,
}

#[derive(Debug, Deserialize)]
struct ExpectedDecision {
    accepted: bool,
    reason_code: i32,
    audit_class: i32,
    envelope_reason: i32,
    room_epoch_reason: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
}

impl CorePolicyInputJson {
    fn into_input(self) -> PolicyEvaluationInput {
        PolicyEvaluationInput {
            actor_kind: ActorKind::try_from(self.actor_kind)
                .unwrap_or_else(|err| panic!("invalid actor kind {}", err.code)),
            envelope: self.envelope.into(),
            room_epoch: self.room_epoch.into(),
            ai: self.ai.map(Into::into),
        }
    }
}

impl From<EnvelopeFactsJson> for EnvelopeFacts {
    fn from(input: EnvelopeFactsJson) -> Self {
        Self {
            version: input.version,
            suite_id: input.suite_id,
            conversation_id_len: input.conversation_id_len,
            sender_account_id_len: input.sender_account_id_len,
            sender_device_id_len: input.sender_device_id_len,
            epoch: input.epoch,
            sequence: input.sequence,
            message_kind: input.message_kind,
            payload_len: input.payload_len,
            critical_flags: input.critical_flags,
            noncritical_flags: input.noncritical_flags,
            expected_epoch: input.expected_epoch,
            expected_sequence: input.expected_sequence,
            min_suite_id: input.min_suite_id,
            max_payload_len: input.max_payload_len,
        }
    }
}

impl From<RoomEpochFactsJson> for RoomEpochFacts {
    fn from(input: RoomEpochFactsJson) -> Self {
        Self {
            version: input.version,
            room_mode: input.room_mode,
            device_kind: input.device_kind,
            device_state: input.device_state,
            current_epoch: input.current_epoch,
            message_epoch: input.message_epoch,
            min_accepted_epoch: input.min_accepted_epoch,
            revoked_at_epoch: input.revoked_at_epoch,
            access_kind: input.access_kind,
        }
    }
}

impl From<AiPolicyFactsJson> for AiPolicyFacts {
    fn from(input: AiPolicyFactsJson) -> Self {
        Self {
            grant: input.grant.into(),
            lifecycle: input.lifecycle.into(),
        }
    }
}

impl From<AiGrantFactsJson> for AiGrantFacts {
    fn from(input: AiGrantFactsJson) -> Self {
        Self {
            version: input.version,
            principal_kind: input.principal_kind,
            room_mode: input.room_mode,
            ai_mode: input.ai_mode,
            ttl_s: input.ttl_s,
            approver_count: input.approver_count,
            read_scope: input.read_scope,
            write_scope: input.write_scope,
            tool_scope: input.tool_scope,
            retention_mode: input.retention_mode,
            training_allowed: input.training_allowed,
            prompt_store_allowed: input.prompt_store_allowed,
        }
    }
}

impl From<AiLifecycleFactsJson> for AiLifecycleFacts {
    fn from(input: AiLifecycleFactsJson) -> Self {
        Self {
            version: input.version,
            grant_state: input.grant_state,
            revoke_reason: input.revoke_reason,
            now_s: input.now_s,
            expires_at_s: input.expires_at_s,
            room_mode: input.room_mode,
            access_kind: input.access_kind,
            epoch_rotated: input.epoch_rotated,
        }
    }
}

#[test]
fn core_policy_vectors_match_decision_layer() {
    let vector_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/core_policy");
    let entries = fs::read_dir(&vector_dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", vector_dir.display()));

    let mut count = 0usize;
    for entry in entries {
        let path = entry.expect("failed to read vector entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let vector: Vector = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));

        let decision = evaluate_policy(vector.input.into_input());
        assert_eq!(
            decision.accepted,
            vector.expected.accepted,
            "accepted mismatch for vector {} at {}",
            vector.name,
            path.display()
        );
        assert_eq!(
            decision.reason_code,
            vector.expected.reason_code,
            "reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );
        assert_eq!(
            decision.audit_class,
            vector.expected.audit_class,
            "audit class mismatch for vector {} at {}",
            vector.name,
            path.display()
        );
        assert_eq!(
            decision.components.envelope_reason,
            vector.expected.envelope_reason,
            "envelope reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );
        assert_eq!(
            decision.components.room_epoch_reason,
            vector.expected.room_epoch_reason,
            "room epoch reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );
        assert_eq!(
            decision.components.ai_grant_reason,
            vector.expected.ai_grant_reason,
            "AI grant reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );
        assert_eq!(
            decision.components.ai_lifecycle_reason,
            vector.expected.ai_lifecycle_reason,
            "AI lifecycle reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );
        assert_stable_labels(&vector.name, decision);

        count += 1;
    }

    assert_eq!(count, 9, "expected all 9 core policy vectors to run");
}

fn assert_stable_labels(name: &str, decision: mercury_core::PolicyDecision) {
    let expected = expected_labels(name);
    let labels = decision.labels();
    let view = decision.view();
    assert_eq!(labels.reason.source.label(), "policy_pipeline");
    assert_eq!(labels.reason.label, expected.reason_label, "{name}");
    assert_eq!(
        labels.audit_class.label, expected.audit_class_label,
        "{name}"
    );
    assert_eq!(
        labels.primary_source.label(),
        expected.primary_source_label,
        "{name}"
    );

    let primary_component = labels
        .primary_component
        .map(|component| (component.source.label(), component.label));
    assert_eq!(primary_component, expected.primary_component, "{name}");
    assert_eq!(view.accepted, decision.accepted, "{name}");
    assert_eq!(view.reason.source, "policy_pipeline", "{name}");
    assert_eq!(view.reason.code, decision.reason_code, "{name}");
    assert_eq!(view.reason.label, expected.reason_label, "{name}");
    assert_eq!(view.audit_class.code, decision.audit_class, "{name}");
    assert_eq!(view.audit_class.label, expected.audit_class_label, "{name}");
    assert_eq!(view.primary_source, expected.primary_source_label, "{name}");
    assert_eq!(
        view.primary_component
            .map(|component| (component.source, component.label)),
        expected.primary_component,
        "{name}"
    );

    assert_eq!(
        labels.components.envelope_reason.code,
        decision.components.envelope_reason
    );
    assert_eq!(
        labels.components.room_epoch_reason.code,
        decision.components.room_epoch_reason
    );
    assert_eq!(
        labels.components.ai_grant_reason.code,
        decision.components.ai_grant_reason
    );
    assert_eq!(
        labels.components.ai_lifecycle_reason.code,
        decision.components.ai_lifecycle_reason
    );
    assert_eq!(
        view.components.envelope_reason.code,
        decision.components.envelope_reason
    );
    assert_eq!(
        view.components.room_epoch_reason.code,
        decision.components.room_epoch_reason
    );
    assert_eq!(
        view.components.ai_grant_reason.code,
        decision.components.ai_grant_reason
    );
    assert_eq!(
        view.components.ai_lifecycle_reason.code,
        decision.components.ai_lifecycle_reason
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedLabels {
    reason_label: &'static str,
    audit_class_label: &'static str,
    primary_source_label: &'static str,
    primary_component: Option<(&'static str, &'static str)>,
}

fn expected_labels(name: &str) -> ExpectedLabels {
    match name {
        "valid_core_human_message_v1" | "valid_core_local_ai_message_v1" => ExpectedLabels {
            reason_label: "ACCEPT",
            audit_class_label: "ACCEPTED_POLICY_DECISION",
            primary_source_label: "policy_pipeline",
            primary_component: None,
        },
        "reject_core_envelope_payload_too_large" => ExpectedLabels {
            reason_label: "ENVELOPE_REJECT",
            audit_class_label: "MESSAGE_POLICY_REJECT",
            primary_source_label: "envelope",
            primary_component: Some(("envelope", "PAYLOAD_TOO_LARGE")),
        },
        "reject_core_room_stale_epoch" => ExpectedLabels {
            reason_label: "ROOM_EPOCH_REJECT",
            audit_class_label: "ROOM_POLICY_REJECT",
            primary_source_label: "room_epoch",
            primary_component: Some(("room_epoch", "STALE_EPOCH")),
        },
        "reject_core_human_with_ai_component" => ExpectedLabels {
            reason_label: "AI_COMPONENT_FOR_HUMAN",
            audit_class_label: "AI_POLICY_REJECT",
            primary_source_label: "policy_pipeline",
            primary_component: Some(("ai_grant", "BAD_PRINCIPAL_KIND")),
        },
        "reject_core_ai_grant_remote_sensitive" => ExpectedLabels {
            reason_label: "AI_GRANT_REJECT",
            audit_class_label: "AI_POLICY_REJECT",
            primary_source_label: "ai_grant",
            primary_component: Some(("ai_grant", "REMOTE_PROVIDER_FORBIDDEN")),
        },
        "reject_core_ai_lifecycle_expired" => ExpectedLabels {
            reason_label: "AI_LIFECYCLE_REJECT",
            audit_class_label: "AI_LIFECYCLE_POLICY_REJECT",
            primary_source_label: "ai_grant_lifecycle",
            primary_component: Some(("ai_grant_lifecycle", "GRANT_EXPIRED")),
        },
        "reject_core_ai_highsec_rotation_required" => ExpectedLabels {
            reason_label: "AI_HIGHSEC_ROTATION_REQUIRED",
            audit_class_label: "AI_HIGHSEC_ROTATION_REJECT",
            primary_source_label: "ai_grant_lifecycle",
            primary_component: Some(("ai_grant_lifecycle", "HIGHSEC_EPOCH_ROTATION_REQUIRED")),
        },
        "reject_core_ai_missing_ai_facts" => ExpectedLabels {
            reason_label: "AI_GRANT_REJECT",
            audit_class_label: "AI_POLICY_REJECT",
            primary_source_label: "ai_grant",
            primary_component: Some(("ai_grant", "BAD_PRINCIPAL_KIND")),
        },
        _ => panic!("missing label expectations for {name}"),
    }
}
