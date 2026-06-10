use mercury_policy::{EnvelopeInput, audit_class_for_reason, validate_envelope};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: EnvelopeInputJson,
    expected_reason: i32,
    expected_audit_class: i32,
}

#[derive(Debug, Deserialize)]
struct EnvelopeInputJson {
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

impl From<EnvelopeInputJson> for EnvelopeInput {
    fn from(input: EnvelopeInputJson) -> Self {
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

#[test]
fn envelope_vectors_match_policy() {
    let vector_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/envelope");
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

        let reason = validate_envelope(vector.input.into());
        assert_eq!(
            reason,
            vector.expected_reason,
            "reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        let audit_class = audit_class_for_reason(reason);
        assert_eq!(
            audit_class,
            vector.expected_audit_class,
            "audit class mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        count += 1;
    }

    assert_eq!(count, 12, "expected all 12 envelope vectors to run");
}
