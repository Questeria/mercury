use mercury_policy::{AiGrantInput, ai_grant_audit_class_for_reason, validate_ai_grant};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: AiGrantInputJson,
    expected_reason: i32,
    expected_audit_class: i32,
}

#[derive(Debug, Deserialize)]
struct AiGrantInputJson {
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

impl From<AiGrantInputJson> for AiGrantInput {
    fn from(input: AiGrantInputJson) -> Self {
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

#[test]
fn ai_grant_vectors_match_policy() {
    let vector_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/ai_grant");
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

        let reason = validate_ai_grant(vector.input.into());
        assert_eq!(
            reason,
            vector.expected_reason,
            "reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        let audit_class = ai_grant_audit_class_for_reason(reason);
        assert_eq!(
            audit_class,
            vector.expected_audit_class,
            "audit class mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        count += 1;
    }

    assert_eq!(count, 20, "expected all 20 AI grant vectors to run");
}
