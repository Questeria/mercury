use mercury_policy::{
    AiGrantLifecycleInput, ai_grant_lifecycle_audit_class_for_reason, validate_ai_grant_lifecycle,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: AiGrantLifecycleInputJson,
    expected_reason: i32,
    expected_audit_class: i32,
}

#[derive(Debug, Deserialize)]
struct AiGrantLifecycleInputJson {
    version: i32,
    grant_state: i32,
    revoke_reason: i32,
    now_s: i32,
    expires_at_s: i32,
    room_mode: i32,
    access_kind: i32,
    epoch_rotated: i32,
}

impl From<AiGrantLifecycleInputJson> for AiGrantLifecycleInput {
    fn from(input: AiGrantLifecycleInputJson) -> Self {
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
fn ai_grant_lifecycle_vectors_match_policy() {
    let vector_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/ai_grant_lifecycle");
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

        let reason = validate_ai_grant_lifecycle(vector.input.into());
        assert_eq!(
            reason,
            vector.expected_reason,
            "reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        let audit_class = ai_grant_lifecycle_audit_class_for_reason(reason);
        assert_eq!(
            audit_class,
            vector.expected_audit_class,
            "audit class mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        count += 1;
    }

    assert_eq!(
        count, 15,
        "expected all 15 AI grant lifecycle vectors to run"
    );
}
