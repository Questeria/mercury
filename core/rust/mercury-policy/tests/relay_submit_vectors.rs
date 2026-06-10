use mercury_policy::{
    RelaySubmitInput, relay_submit_audit_class_for_reason, relay_submit_decide_v1,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: RelaySubmitInputJson,
    expected_reason: i32,
    expected_audit_class: i32,
}

#[derive(Debug, Deserialize)]
struct RelaySubmitInputJson {
    version: i32,
    send_gate_reason: i32,
    route_id_len: i32,
    replay_token_len: i32,
    queue_ttl_s: i32,
    max_queue_ttl_s: i32,
    ciphertext_len: i32,
    max_ciphertext_len: i32,
    sealed_header_len: i32,
    plaintext_identity_fields: i32,
    padding_bucket: i32,
}

impl From<RelaySubmitInputJson> for RelaySubmitInput {
    fn from(input: RelaySubmitInputJson) -> Self {
        Self {
            version: input.version,
            send_gate_reason: input.send_gate_reason,
            route_id_len: input.route_id_len,
            replay_token_len: input.replay_token_len,
            queue_ttl_s: input.queue_ttl_s,
            max_queue_ttl_s: input.max_queue_ttl_s,
            ciphertext_len: input.ciphertext_len,
            max_ciphertext_len: input.max_ciphertext_len,
            sealed_header_len: input.sealed_header_len,
            plaintext_identity_fields: input.plaintext_identity_fields,
            padding_bucket: input.padding_bucket,
        }
    }
}

#[test]
fn relay_submit_vectors_match_policy() {
    let vector_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/relay_submit");
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

        let reason = relay_submit_decide_v1(vector.input.into());
        assert_eq!(
            reason,
            vector.expected_reason,
            "reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        let audit_class = relay_submit_audit_class_for_reason(reason);
        assert_eq!(
            audit_class,
            vector.expected_audit_class,
            "audit class mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        count += 1;
    }

    assert_eq!(count, 15, "expected all 15 relay submit vectors to run");
}
