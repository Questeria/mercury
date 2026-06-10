use std::{fs, path::PathBuf};

use mercury_bindings::{
    PROTOTYPE_FIXTURES, prototype_fixture_by_name, prototype_fixture_json, prototype_fixture_value,
};

#[test]
fn prototype_fixture_names_are_resolvable() {
    for descriptor in PROTOTYPE_FIXTURES {
        assert_eq!(
            prototype_fixture_by_name(descriptor.name),
            Some(descriptor.fixture)
        );
    }

    assert_eq!(prototype_fixture_by_name("does_not_exist"), None);
}

#[test]
fn prototype_fixture_json_matches_checked_in_payloads() {
    let fixture_dir = repo_root().join("fixtures").join("prototypes");

    for descriptor in PROTOTYPE_FIXTURES {
        let actual = prototype_fixture_value(descriptor.fixture);
        let fixture_path = fixture_dir.join(format!("{}.json", descriptor.name));
        let expected: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&fixture_path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_path.display())),
        )
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", fixture_path.display()));

        assert_eq!(
            actual, expected,
            "prototype fixture drift: {}",
            descriptor.name
        );
    }
}

#[test]
fn prototype_fixture_json_helper_emits_pretty_json() {
    let json = prototype_fixture_json(PROTOTYPE_FIXTURES[0].fixture)
        .expect("prototype fixture json should serialize");

    assert!(json.contains("\"surface\": \"prototype_local_store\""));
    assert!(json.contains('\n'));
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}
