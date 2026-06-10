#!/usr/bin/env python3
"""Check that Mercury policy constants and vectors agree with the manifest."""

from __future__ import annotations

import json
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
ENVELOPE_MANIFEST_PATH = REPO_ROOT / "policy" / "envelope_policy_v1.json"
AI_GRANT_MANIFEST_PATH = REPO_ROOT / "policy" / "ai_grant_policy_v1.json"
AI_GRANT_LIFECYCLE_MANIFEST_PATH = (
    REPO_ROOT / "policy" / "ai_grant_lifecycle_policy_v1.json"
)
ROOM_EPOCH_MANIFEST_PATH = REPO_ROOT / "policy" / "room_epoch_policy_v1.json"
POLICY_PIPELINE_MANIFEST_PATH = REPO_ROOT / "policy" / "policy_pipeline_v1.json"
RELAY_SUBMIT_MANIFEST_PATH = REPO_ROOT / "policy" / "relay_submit_policy_v1.json"


HELIX_REASON_FUNCTIONS = {
    "ACCEPT": "mercury_accept",
    "BAD_VERSION": "mercury_bad_version",
    "UNSUPPORTED_SUITE": "mercury_unsupported_suite",
    "DOWNGRADED_SUITE": "mercury_downgraded_suite",
    "BAD_CONVERSATION_ID_LEN": "mercury_bad_conversation_id_len",
    "BAD_SENDER_ACCOUNT_ID_LEN": "mercury_bad_sender_account_id_len",
    "BAD_SENDER_DEVICE_ID_LEN": "mercury_bad_sender_device_id_len",
    "BAD_EPOCH": "mercury_bad_epoch",
    "BAD_SEQUENCE": "mercury_bad_sequence",
    "BAD_MESSAGE_KIND": "mercury_bad_message_kind",
    "PAYLOAD_TOO_LARGE": "mercury_payload_too_large",
    "UNKNOWN_CRITICAL_FLAG": "mercury_unknown_critical_flag",
    "RESERVED_AI_KIND": "mercury_reserved_ai_kind",
}

HELIX_AUDIT_FUNCTIONS = {
    "ACCEPTED_MESSAGE": "mercury_audit_accepted_message",
    "POLICY_REJECT": "mercury_audit_policy_reject",
    "DOWNGRADE_ATTEMPT": "mercury_audit_downgrade_attempt",
    "SIZE_REJECT": "mercury_audit_size_reject",
}

AI_GRANT_HELIX_REASON_FUNCTIONS = {
    "ACCEPT": "mercury_ai_grant_accept",
    "BAD_VERSION": "mercury_ai_grant_bad_version",
    "BAD_PRINCIPAL_KIND": "mercury_ai_grant_bad_principal_kind",
    "BAD_AI_MODE": "mercury_ai_grant_bad_ai_mode",
    "AI_BLOCKED_ROOM": "mercury_ai_grant_ai_blocked_room",
    "BAD_TTL": "mercury_ai_grant_bad_ttl",
    "TTL_TOO_LONG": "mercury_ai_grant_ttl_too_long",
    "INSUFFICIENT_APPROVERS": "mercury_ai_grant_insufficient_approvers",
    "BAD_READ_SCOPE": "mercury_ai_grant_bad_read_scope",
    "READ_SCOPE_TOO_BROAD": "mercury_ai_grant_read_scope_too_broad",
    "BAD_WRITE_SCOPE": "mercury_ai_grant_bad_write_scope",
    "WRITE_SCOPE_TOO_BROAD": "mercury_ai_grant_write_scope_too_broad",
    "BAD_TOOL_SCOPE": "mercury_ai_grant_bad_tool_scope",
    "TOOL_SCOPE_TOO_BROAD": "mercury_ai_grant_tool_scope_too_broad",
    "BAD_RETENTION_MODE": "mercury_ai_grant_bad_retention_mode",
    "PROMPT_STORE_FORBIDDEN": "mercury_ai_grant_prompt_store_forbidden",
    "TRAINING_FORBIDDEN": "mercury_ai_grant_training_forbidden",
    "REMOTE_PROVIDER_FORBIDDEN": "mercury_ai_grant_remote_provider_forbidden",
    "HIGHSEC_LOCAL_ONLY": "mercury_ai_grant_highsec_local_only",
    "HIGHSEC_CONFIRM_SEND_REQUIRED": "mercury_ai_grant_highsec_confirm_send_required",
    "HIGHSEC_TOOLS_FORBIDDEN": "mercury_ai_grant_highsec_tools_forbidden",
}

AI_GRANT_HELIX_AUDIT_FUNCTIONS = {
    "ACCEPTED_AI_GRANT": "mercury_ai_grant_audit_accepted",
    "AI_POLICY_REJECT": "mercury_ai_grant_audit_policy_reject",
    "AI_PRIVACY_REJECT": "mercury_ai_grant_audit_privacy_reject",
    "AI_HIGHSEC_REJECT": "mercury_ai_grant_audit_highsec_reject",
    "AI_RETENTION_REJECT": "mercury_ai_grant_audit_retention_reject",
    "AI_TOOL_REJECT": "mercury_ai_grant_audit_tool_reject",
}

AI_GRANT_LIFECYCLE_HELIX_REASON_FUNCTIONS = {
    "ACCEPT": "mercury_lifecycle_accept",
    "BAD_VERSION": "mercury_lifecycle_bad_version",
    "BAD_GRANT_STATE": "mercury_lifecycle_bad_grant_state",
    "BAD_REVOKE_REASON": "mercury_lifecycle_bad_revoke_reason",
    "BAD_TIME": "mercury_lifecycle_bad_time",
    "GRANT_EXPIRED": "mercury_lifecycle_grant_expired",
    "GRANT_REVOKED": "mercury_lifecycle_grant_revoked",
    "READ_AFTER_REVOKE": "mercury_lifecycle_read_after_revoke",
    "WRITE_AFTER_REVOKE": "mercury_lifecycle_write_after_revoke",
    "HIGHSEC_EPOCH_ROTATION_REQUIRED": (
        "mercury_lifecycle_highsec_epoch_rotation_required"
    ),
    "BAD_ACCESS_KIND": "mercury_lifecycle_bad_access_kind",
}

AI_GRANT_LIFECYCLE_HELIX_AUDIT_FUNCTIONS = {
    "ACCEPTED_AI_GRANT_LIFECYCLE": "mercury_lifecycle_audit_accepted",
    "AI_LIFECYCLE_POLICY_REJECT": "mercury_lifecycle_audit_policy_reject",
    "AI_GRANT_EXPIRED_REJECT": "mercury_lifecycle_audit_expired_reject",
    "AI_GRANT_REVOKED_REJECT": "mercury_lifecycle_audit_revoked_reject",
    "AI_POST_REVOKE_ACCESS_REJECT": (
        "mercury_lifecycle_audit_post_revoke_access_reject"
    ),
    "AI_HIGHSEC_EPOCH_ROTATION_REQUIRED": (
        "mercury_lifecycle_audit_highsec_epoch_rotation_required"
    ),
}

ROOM_EPOCH_HELIX_REASON_FUNCTIONS = {
    "ACCEPT": "mercury_room_epoch_accept",
    "BAD_VERSION": "mercury_room_epoch_bad_version",
    "BAD_ROOM_MODE": "mercury_room_epoch_bad_room_mode",
    "BAD_EPOCH": "mercury_room_epoch_bad_epoch",
    "STALE_EPOCH": "mercury_room_epoch_stale_epoch",
    "FUTURE_EPOCH": "mercury_room_epoch_future_epoch",
    "BAD_DEVICE_KIND": "mercury_room_epoch_bad_device_kind",
    "BAD_DEVICE_STATE": "mercury_room_epoch_bad_device_state",
    "BAD_REVOKED_AT_EPOCH": "mercury_room_epoch_bad_revoked_at_epoch",
    "BAD_ACCESS_KIND": "mercury_room_epoch_bad_access_kind",
    "DEVICE_REMOVED": "mercury_room_epoch_device_removed",
    "DEVICE_COMPROMISED": "mercury_room_epoch_device_compromised",
    "HIGHSEC_EPOCH_ROTATION_REQUIRED": (
        "mercury_room_epoch_highsec_epoch_rotation_required"
    ),
    "AI_DEVICE_BLOCKED_ROOM": "mercury_room_epoch_ai_device_blocked_room",
}

ROOM_EPOCH_HELIX_AUDIT_FUNCTIONS = {
    "ACCEPTED_ROOM_EPOCH": "mercury_room_epoch_audit_accepted",
    "ROOM_EPOCH_POLICY_REJECT": "mercury_room_epoch_audit_policy_reject",
    "ROOM_EPOCH_REPLAY_REJECT": "mercury_room_epoch_audit_replay_reject",
    "ROOM_DEVICE_MEMBERSHIP_REJECT": "mercury_room_epoch_audit_membership_reject",
    "ROOM_HIGHSEC_ROTATION_REJECT": (
        "mercury_room_epoch_audit_highsec_rotation_reject"
    ),
    "ROOM_AI_POLICY_REJECT": "mercury_room_epoch_audit_ai_policy_reject",
}

POLICY_PIPELINE_HELIX_REASON_FUNCTIONS = {
    "ACCEPT": "mercury_pipeline_accept",
    "BAD_VERSION": "mercury_pipeline_bad_version",
    "BAD_ACTOR_KIND": "mercury_pipeline_bad_actor_kind",
    "BAD_ENVELOPE_REASON": "mercury_pipeline_bad_envelope_reason",
    "BAD_ROOM_EPOCH_REASON": "mercury_pipeline_bad_room_epoch_reason",
    "BAD_AI_GRANT_REASON": "mercury_pipeline_bad_ai_grant_reason",
    "BAD_AI_LIFECYCLE_REASON": "mercury_pipeline_bad_ai_lifecycle_reason",
    "AI_COMPONENT_FOR_HUMAN": "mercury_pipeline_ai_component_for_human",
    "ENVELOPE_REJECT": "mercury_pipeline_envelope_reject",
    "ROOM_EPOCH_REJECT": "mercury_pipeline_room_epoch_reject",
    "AI_GRANT_REJECT": "mercury_pipeline_ai_grant_reject",
    "AI_LIFECYCLE_REJECT": "mercury_pipeline_ai_lifecycle_reject",
    "AI_POST_REVOKE_ACCESS_REJECT": (
        "mercury_pipeline_ai_post_revoke_access_reject"
    ),
    "AI_HIGHSEC_ROTATION_REQUIRED": "mercury_pipeline_ai_highsec_rotation_required",
}

POLICY_PIPELINE_HELIX_AUDIT_FUNCTIONS = {
    "ACCEPTED_POLICY_DECISION": "mercury_pipeline_audit_accepted",
    "PIPELINE_CONTRACT_REJECT": "mercury_pipeline_audit_contract_reject",
    "MESSAGE_POLICY_REJECT": "mercury_pipeline_audit_message_policy_reject",
    "ROOM_POLICY_REJECT": "mercury_pipeline_audit_room_policy_reject",
    "AI_POLICY_REJECT": "mercury_pipeline_audit_ai_policy_reject",
    "AI_LIFECYCLE_POLICY_REJECT": (
        "mercury_pipeline_audit_ai_lifecycle_policy_reject"
    ),
    "AI_POST_REVOKE_REJECT": "mercury_pipeline_audit_ai_post_revoke_reject",
    "AI_HIGHSEC_ROTATION_REJECT": (
        "mercury_pipeline_audit_ai_highsec_rotation_reject"
    ),
}

RELAY_SUBMIT_HELIX_REASON_FUNCTIONS = {
    "ACCEPT": "mercury_relay_accept",
    "BAD_VERSION": "mercury_relay_bad_version",
    "BAD_SEND_GATE_REASON": "mercury_relay_bad_send_gate_reason",
    "SEND_GATE_REJECTED": "mercury_relay_send_gate_rejected",
    "BAD_ROUTE_ID_LEN": "mercury_relay_bad_route_id_len",
    "BAD_REPLAY_TOKEN_LEN": "mercury_relay_bad_replay_token_len",
    "BAD_TTL": "mercury_relay_bad_ttl",
    "TTL_TOO_LONG": "mercury_relay_ttl_too_long",
    "BAD_CIPHERTEXT_LEN": "mercury_relay_bad_ciphertext_len",
    "CIPHERTEXT_TOO_LARGE": "mercury_relay_ciphertext_too_large",
    "BAD_SEALED_HEADER_LEN": "mercury_relay_bad_sealed_header_len",
    "SEALED_HEADER_TOO_LARGE": "mercury_relay_sealed_header_too_large",
    "PLAINTEXT_IDENTITY_FORBIDDEN": "mercury_relay_plaintext_identity_forbidden",
    "BAD_PADDING_BUCKET": "mercury_relay_bad_padding_bucket",
}

RELAY_SUBMIT_HELIX_AUDIT_FUNCTIONS = {
    "ACCEPTED_RELAY_SUBMISSION": "mercury_relay_audit_accepted",
    "RELAY_CONTRACT_REJECT": "mercury_relay_audit_contract_reject",
    "CLIENT_SEND_REJECT": "mercury_relay_audit_client_send_reject",
    "RELAY_METADATA_REJECT": "mercury_relay_audit_metadata_reject",
    "RELAY_RETENTION_REJECT": "mercury_relay_audit_retention_reject",
    "RELAY_SIZE_REJECT": "mercury_relay_audit_size_reject",
}


def fail(message: str) -> None:
    raise SystemExit(f"policy contract check failed: {message}")


def load_manifest(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def check_unique_values(name: str, mapping: dict[str, int]) -> None:
    values = list(mapping.values())
    if len(values) != len(set(values)):
        fail(f"{name} contains duplicate numeric values")


def check_vectors(manifest: dict, vector_subdir: str) -> None:
    reason_values = set(manifest["reason_codes"].values())
    audit_values = set(manifest["audit_classes"].values())
    expected_names = set(manifest["vectors"])
    seen_names: set[str] = set()

    for path in sorted((REPO_ROOT / "vectors" / vector_subdir).glob("*.json")):
        vector = json.loads(path.read_text(encoding="utf-8"))
        name = vector["name"]
        seen_names.add(name)
        if name not in expected_names:
            fail(f"unexpected vector {name} in {path}")
        if vector["expected_reason"] not in reason_values:
            fail(f"{name} has unknown reason {vector['expected_reason']}")
        if vector["expected_audit_class"] not in audit_values:
            fail(f"{name} has unknown audit class {vector['expected_audit_class']}")

    missing = sorted(expected_names - seen_names)
    if missing:
        fail(f"missing vectors: {', '.join(missing)}")


def read_rust_constants() -> dict[str, int]:
    rust = (REPO_ROOT / "core" / "rust" / "mercury-policy" / "src" / "lib.rs").read_text(
        encoding="utf-8"
    )

    return {
        name: int(value)
        for name, value in re.findall(r"pub const ([A-Z_]+): i32 = (-?\d+);", rust)
    }


def check_rust_constants(manifest: dict) -> None:
    constants = read_rust_constants()
    for name, expected in manifest["reason_codes"].items():
        actual = constants.get(name)
        if actual != expected:
            fail(f"Rust reason {name} is {actual}, expected {expected}")
    for name, expected in manifest["audit_classes"].items():
        rust_name = f"AUDIT_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(f"Rust audit {rust_name} is {actual}, expected {expected}")


def check_ai_grant_rust_constants(manifest: dict) -> None:
    constants = read_rust_constants()
    for name, expected in manifest["reason_codes"].items():
        rust_name = f"AI_GRANT_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(f"Rust AI grant reason {rust_name} is {actual}, expected {expected}")
    for name, expected in manifest["audit_classes"].items():
        rust_name = f"AI_GRANT_AUDIT_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(f"Rust AI grant audit {rust_name} is {actual}, expected {expected}")


def check_ai_grant_lifecycle_rust_constants(manifest: dict) -> None:
    constants = read_rust_constants()
    for name, expected in manifest["reason_codes"].items():
        rust_name = f"AI_GRANT_LIFECYCLE_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(
                f"Rust AI grant lifecycle reason {rust_name} is {actual}, "
                f"expected {expected}"
            )
    for name, expected in manifest["audit_classes"].items():
        rust_name = f"AI_GRANT_LIFECYCLE_AUDIT_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(
                f"Rust AI grant lifecycle audit {rust_name} is {actual}, "
                f"expected {expected}"
            )


def check_room_epoch_rust_constants(manifest: dict) -> None:
    constants = read_rust_constants()
    for name, expected in manifest["reason_codes"].items():
        rust_name = f"ROOM_EPOCH_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(f"Rust room epoch reason {rust_name} is {actual}, expected {expected}")
    for name, expected in manifest["audit_classes"].items():
        rust_name = f"ROOM_EPOCH_AUDIT_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(f"Rust room epoch audit {rust_name} is {actual}, expected {expected}")


def check_policy_pipeline_rust_constants(manifest: dict) -> None:
    constants = read_rust_constants()
    for name, expected in manifest["reason_codes"].items():
        rust_name = f"POLICY_PIPELINE_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(
                f"Rust policy pipeline reason {rust_name} is {actual}, "
                f"expected {expected}"
            )
    for name, expected in manifest["audit_classes"].items():
        rust_name = f"POLICY_PIPELINE_AUDIT_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(
                f"Rust policy pipeline audit {rust_name} is {actual}, "
                f"expected {expected}"
            )


def check_relay_submit_rust_constants(manifest: dict) -> None:
    constants = read_rust_constants()
    for name, expected in manifest["reason_codes"].items():
        rust_name = f"RELAY_SUBMIT_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(
                f"Rust relay submit reason {rust_name} is {actual}, "
                f"expected {expected}"
            )
    for name, expected in manifest["audit_classes"].items():
        rust_name = f"RELAY_SUBMIT_AUDIT_{name}"
        actual = constants.get(rust_name)
        if actual != expected:
            fail(
                f"Rust relay submit audit {rust_name} is {actual}, "
                f"expected {expected}"
            )


def check_helix_functions(
    manifest: dict,
    files: tuple[str, str],
    reason_functions: dict[str, str],
    audit_functions: dict[str, str],
) -> None:
    for rel in files:
        text = (REPO_ROOT / rel).read_text(encoding="utf-8")
        for name, expected in manifest["reason_codes"].items():
            fn_name = reason_functions[name]
            pattern = rf"fn\s+{fn_name}\(\)\s*->\s*i32\s*\{{\s*{expected}\s*\}}"
            if not re.search(pattern, text):
                fail(f"{rel} does not define {fn_name} -> {expected}")
        for name, expected in manifest["audit_classes"].items():
            fn_name = audit_functions[name]
            pattern = rf"fn\s+{fn_name}\(\)\s*->\s*i32\s*\{{\s*{expected}\s*\}}"
            if not re.search(pattern, text):
                fail(f"{rel} does not define {fn_name} -> {expected}")


def check_python_checker(manifest: dict) -> None:
    text = (REPO_ROOT / "tools" / "check_envelope_vectors.py").read_text(encoding="utf-8")
    constants = {
        name: int(value)
        for name, value in re.findall(r"^([A-Z_]+)\s*=\s*(-?\d+)$", text, re.MULTILINE)
    }
    for name, expected in manifest["reason_codes"].items():
        actual = constants.get(name)
        if actual != expected:
            fail(f"Python reason {name} is {actual}, expected {expected}")
    for name, expected in manifest["audit_classes"].items():
        py_name = f"AUDIT_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(f"Python audit {py_name} is {actual}, expected {expected}")


def check_ai_grant_python_checker(manifest: dict) -> None:
    text = (REPO_ROOT / "tools" / "check_ai_grant_vectors.py").read_text(encoding="utf-8")
    constants = {
        name: int(value)
        for name, value in re.findall(r"^([A-Z_]+)\s*=\s*(-?\d+)$", text, re.MULTILINE)
    }
    for name, expected in manifest["reason_codes"].items():
        py_name = f"AI_GRANT_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(f"Python AI grant reason {py_name} is {actual}, expected {expected}")
    for name, expected in manifest["audit_classes"].items():
        py_name = f"AI_GRANT_AUDIT_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(f"Python AI grant audit {py_name} is {actual}, expected {expected}")


def check_ai_grant_lifecycle_python_checker(manifest: dict) -> None:
    text = (REPO_ROOT / "tools" / "check_ai_grant_lifecycle_vectors.py").read_text(
        encoding="utf-8"
    )
    constants = {
        name: int(value)
        for name, value in re.findall(r"^([A-Z_]+)\s*=\s*(-?\d+)$", text, re.MULTILINE)
    }
    for name, expected in manifest["reason_codes"].items():
        py_name = f"AI_GRANT_LIFECYCLE_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(
                f"Python AI grant lifecycle reason {py_name} is {actual}, "
                f"expected {expected}"
            )
    for name, expected in manifest["audit_classes"].items():
        py_name = f"AI_GRANT_LIFECYCLE_AUDIT_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(
                f"Python AI grant lifecycle audit {py_name} is {actual}, "
                f"expected {expected}"
            )


def check_room_epoch_python_checker(manifest: dict) -> None:
    text = (REPO_ROOT / "tools" / "check_room_epoch_vectors.py").read_text(
        encoding="utf-8"
    )
    constants = {
        name: int(value)
        for name, value in re.findall(r"^([A-Z_]+)\s*=\s*(-?\d+)$", text, re.MULTILINE)
    }
    for name, expected in manifest["reason_codes"].items():
        py_name = f"ROOM_EPOCH_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(f"Python room epoch reason {py_name} is {actual}, expected {expected}")
    for name, expected in manifest["audit_classes"].items():
        py_name = f"ROOM_EPOCH_AUDIT_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(f"Python room epoch audit {py_name} is {actual}, expected {expected}")


def check_policy_pipeline_python_checker(manifest: dict) -> None:
    text = (REPO_ROOT / "tools" / "check_policy_pipeline_vectors.py").read_text(
        encoding="utf-8"
    )
    constants = {
        name: int(value)
        for name, value in re.findall(r"^([A-Z_]+)\s*=\s*(-?\d+)$", text, re.MULTILINE)
    }
    for name, expected in manifest["reason_codes"].items():
        py_name = f"POLICY_PIPELINE_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(
                f"Python policy pipeline reason {py_name} is {actual}, "
                f"expected {expected}"
            )
    for name, expected in manifest["audit_classes"].items():
        py_name = f"POLICY_PIPELINE_AUDIT_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(
                f"Python policy pipeline audit {py_name} is {actual}, "
                f"expected {expected}"
            )


def check_relay_submit_python_checker(manifest: dict) -> None:
    text = (REPO_ROOT / "tools" / "check_relay_submit_vectors.py").read_text(
        encoding="utf-8"
    )
    constants = {
        name: int(value)
        for name, value in re.findall(r"^([A-Z_]+)\s*=\s*(-?\d+)$", text, re.MULTILINE)
    }
    for name, expected in manifest["reason_codes"].items():
        py_name = f"RELAY_SUBMIT_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(
                f"Python relay submit reason {py_name} is {actual}, "
                f"expected {expected}"
            )
    for name, expected in manifest["audit_classes"].items():
        py_name = f"RELAY_SUBMIT_AUDIT_{name}"
        actual = constants.get(py_name)
        if actual != expected:
            fail(
                f"Python relay submit audit {py_name} is {actual}, "
                f"expected {expected}"
            )


def main() -> int:
    manifest = load_manifest(ENVELOPE_MANIFEST_PATH)
    check_unique_values("envelope reason_codes", manifest["reason_codes"])
    check_unique_values("envelope audit_classes", manifest["audit_classes"])
    check_vectors(manifest, "envelope")
    check_rust_constants(manifest)
    check_helix_functions(
        manifest,
        ("helix/policy/envelope.hx", "helix/tests/envelope_test.hx"),
        HELIX_REASON_FUNCTIONS,
        HELIX_AUDIT_FUNCTIONS,
    )
    check_python_checker(manifest)

    ai_grant_manifest = load_manifest(AI_GRANT_MANIFEST_PATH)
    check_unique_values("AI grant reason_codes", ai_grant_manifest["reason_codes"])
    check_unique_values("AI grant audit_classes", ai_grant_manifest["audit_classes"])
    check_vectors(ai_grant_manifest, "ai_grant")
    check_ai_grant_rust_constants(ai_grant_manifest)
    check_helix_functions(
        ai_grant_manifest,
        ("helix/policy/ai_grant.hx", "helix/tests/ai_grant_test.hx"),
        AI_GRANT_HELIX_REASON_FUNCTIONS,
        AI_GRANT_HELIX_AUDIT_FUNCTIONS,
    )
    check_ai_grant_python_checker(ai_grant_manifest)

    ai_grant_lifecycle_manifest = load_manifest(AI_GRANT_LIFECYCLE_MANIFEST_PATH)
    check_unique_values(
        "AI grant lifecycle reason_codes",
        ai_grant_lifecycle_manifest["reason_codes"],
    )
    check_unique_values(
        "AI grant lifecycle audit_classes",
        ai_grant_lifecycle_manifest["audit_classes"],
    )
    check_vectors(ai_grant_lifecycle_manifest, "ai_grant_lifecycle")
    check_ai_grant_lifecycle_rust_constants(ai_grant_lifecycle_manifest)
    check_helix_functions(
        ai_grant_lifecycle_manifest,
        (
            "helix/policy/ai_grant_lifecycle.hx",
            "helix/tests/ai_grant_lifecycle_test.hx",
        ),
        AI_GRANT_LIFECYCLE_HELIX_REASON_FUNCTIONS,
        AI_GRANT_LIFECYCLE_HELIX_AUDIT_FUNCTIONS,
    )
    check_ai_grant_lifecycle_python_checker(ai_grant_lifecycle_manifest)

    room_epoch_manifest = load_manifest(ROOM_EPOCH_MANIFEST_PATH)
    check_unique_values("room epoch reason_codes", room_epoch_manifest["reason_codes"])
    check_unique_values("room epoch audit_classes", room_epoch_manifest["audit_classes"])
    check_vectors(room_epoch_manifest, "room_epoch")
    check_room_epoch_rust_constants(room_epoch_manifest)
    check_helix_functions(
        room_epoch_manifest,
        ("helix/policy/room_epoch.hx", "helix/tests/room_epoch_test.hx"),
        ROOM_EPOCH_HELIX_REASON_FUNCTIONS,
        ROOM_EPOCH_HELIX_AUDIT_FUNCTIONS,
    )
    check_room_epoch_python_checker(room_epoch_manifest)

    policy_pipeline_manifest = load_manifest(POLICY_PIPELINE_MANIFEST_PATH)
    check_unique_values(
        "policy pipeline reason_codes",
        policy_pipeline_manifest["reason_codes"],
    )
    check_unique_values(
        "policy pipeline audit_classes",
        policy_pipeline_manifest["audit_classes"],
    )
    check_vectors(policy_pipeline_manifest, "policy_pipeline")
    check_policy_pipeline_rust_constants(policy_pipeline_manifest)
    check_helix_functions(
        policy_pipeline_manifest,
        ("helix/policy/policy_pipeline.hx", "helix/tests/policy_pipeline_test.hx"),
        POLICY_PIPELINE_HELIX_REASON_FUNCTIONS,
        POLICY_PIPELINE_HELIX_AUDIT_FUNCTIONS,
    )
    check_policy_pipeline_python_checker(policy_pipeline_manifest)

    relay_submit_manifest = load_manifest(RELAY_SUBMIT_MANIFEST_PATH)
    check_unique_values(
        "relay submit reason_codes",
        relay_submit_manifest["reason_codes"],
    )
    check_unique_values(
        "relay submit audit_classes",
        relay_submit_manifest["audit_classes"],
    )
    check_vectors(relay_submit_manifest, "relay_submit")
    check_relay_submit_rust_constants(relay_submit_manifest)
    check_helix_functions(
        relay_submit_manifest,
        ("helix/policy/relay_submit.hx", "helix/tests/relay_submit_test.hx"),
        RELAY_SUBMIT_HELIX_REASON_FUNCTIONS,
        RELAY_SUBMIT_HELIX_AUDIT_FUNCTIONS,
    )
    check_relay_submit_python_checker(relay_submit_manifest)

    print("policy contract: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
