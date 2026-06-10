use std::{fs, path::PathBuf};

use mercury_bindings::{
    PLATFORM_FIXTURES, platform_fixture_by_name, platform_fixture_json, platform_fixture_view,
};

#[test]
fn fixture_names_are_resolvable() {
    for descriptor in PLATFORM_FIXTURES {
        assert_eq!(
            platform_fixture_by_name(descriptor.name),
            Some(descriptor.fixture)
        );
    }

    assert_eq!(platform_fixture_by_name("does_not_exist"), None);
}

#[test]
fn fixture_json_matches_checked_in_payloads() {
    let fixture_dir = repo_root().join("fixtures").join("platform");

    for descriptor in PLATFORM_FIXTURES {
        let actual = serde_json::to_value(platform_fixture_view(descriptor.fixture))
            .expect("fixture view should serialize");
        let fixture_path = fixture_dir.join(format!("{}.json", descriptor.name));
        let expected: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&fixture_path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_path.display())),
        )
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", fixture_path.display()));

        assert_eq!(actual, expected, "fixture drift: {}", descriptor.name);
    }
}

#[test]
fn fixture_json_helper_emits_pretty_json() {
    let json =
        platform_fixture_json(PLATFORM_FIXTURES[0].fixture).expect("fixture json should serialize");

    assert!(json.contains("\"source\": \"client_bootstrap\""));
    assert!(json.contains('\n'));
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}
