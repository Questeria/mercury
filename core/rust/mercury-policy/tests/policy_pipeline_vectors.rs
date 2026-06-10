use mercury_policy::{
    PolicyPipelineInput, policy_pipeline_audit_class_for_reason, policy_pipeline_decide_v1,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: PolicyPipelineInputJson,
    expected_reason: i32,
    expected_audit_class: i32,
}

#[derive(Debug, Deserialize)]
struct PolicyPipelineInputJson {
    version: i32,
    actor_kind: i32,
    envelope_reason: i32,
    room_epoch_reason: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
}

impl From<PolicyPipelineInputJson> for PolicyPipelineInput {
    fn from(input: PolicyPipelineInputJson) -> Self {
        Self {
            version: input.version,
            actor_kind: input.actor_kind,
            envelope_reason: input.envelope_reason,
            room_epoch_reason: input.room_epoch_reason,
            ai_grant_reason: input.ai_grant_reason,
            ai_lifecycle_reason: input.ai_lifecycle_reason,
        }
    }
}

#[test]
fn policy_pipeline_vectors_match_policy() {
    let vector_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/policy_pipeline");
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

        let reason = policy_pipeline_decide_v1(vector.input.into());
        assert_eq!(
            reason,
            vector.expected_reason,
            "reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        let audit_class = policy_pipeline_audit_class_for_reason(reason);
        assert_eq!(
            audit_class,
            vector.expected_audit_class,
            "audit class mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        count += 1;
    }

    assert_eq!(count, 15, "expected all 15 policy pipeline vectors to run");
}
