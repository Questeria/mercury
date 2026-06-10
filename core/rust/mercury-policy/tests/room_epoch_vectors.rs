use mercury_policy::{RoomEpochInput, room_epoch_audit_class_for_reason, validate_room_epoch};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: RoomEpochInputJson,
    expected_reason: i32,
    expected_audit_class: i32,
}

#[derive(Debug, Deserialize)]
struct RoomEpochInputJson {
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

impl From<RoomEpochInputJson> for RoomEpochInput {
    fn from(input: RoomEpochInputJson) -> Self {
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

#[test]
fn room_epoch_vectors_match_policy() {
    let vector_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/room_epoch");
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

        let reason = validate_room_epoch(vector.input.into());
        assert_eq!(
            reason,
            vector.expected_reason,
            "reason mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        let audit_class = room_epoch_audit_class_for_reason(reason);
        assert_eq!(
            audit_class,
            vector.expected_audit_class,
            "audit class mismatch for vector {} at {}",
            vector.name,
            path.display()
        );

        count += 1;
    }

    assert_eq!(count, 19, "expected all 19 room epoch vectors to run");
}
