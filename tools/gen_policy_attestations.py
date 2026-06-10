#!/usr/bin/env python3
"""Generate Mercury Helix policy attestation artifacts.

The manifest is intentionally Mercury-local. It hashes Helix policy sources,
tests, vector corpora, policy JSON files, and Rust mirror tests without writing
to or depending on the external Helix checkout.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "helix" / "policy" / "attestation.json"
TS_PATH = ROOT / "ui" / "app" / "src" / "mercury" / "policyAttestation.generated.ts"

POLICIES: list[dict[str, Any]] = [
    {
        "name": "envelope",
        "display_name": "Envelope",
        "role": "message envelope and downgrade gate",
        "policy_json": "policy/envelope_policy_v1.json",
        "vectors": "vectors/envelope",
        "rust_mirror_tests": ["core/rust/mercury-policy/tests/envelope_vectors.rs"],
        "python_vector_check": "tools/check_envelope_vectors.py",
        "decision_sources": ["policy_pipeline"],
    },
    {
        "name": "ai_grant",
        "display_name": "AI grant",
        "role": "AI principal scope gate",
        "policy_json": "policy/ai_grant_policy_v1.json",
        "vectors": "vectors/ai_grant",
        "rust_mirror_tests": ["core/rust/mercury-policy/tests/ai_grant_vectors.rs"],
        "python_vector_check": "tools/check_ai_grant_vectors.py",
        "decision_sources": ["policy_pipeline"],
    },
    {
        "name": "ai_grant_lifecycle",
        "display_name": "AI grant lifecycle",
        "role": "AI grant expiry and revoke gate",
        "policy_json": "policy/ai_grant_lifecycle_policy_v1.json",
        "vectors": "vectors/ai_grant_lifecycle",
        "rust_mirror_tests": [
            "core/rust/mercury-policy/tests/ai_grant_lifecycle_vectors.rs"
        ],
        "python_vector_check": "tools/check_ai_grant_lifecycle_vectors.py",
        "decision_sources": ["policy_pipeline"],
    },
    {
        "name": "room_epoch",
        "display_name": "Room epoch",
        "role": "room epoch and membership state gate",
        "policy_json": "policy/room_epoch_policy_v1.json",
        "vectors": "vectors/room_epoch",
        "rust_mirror_tests": ["core/rust/mercury-policy/tests/room_epoch_vectors.rs"],
        "python_vector_check": "tools/check_room_epoch_vectors.py",
        "decision_sources": ["policy_pipeline"],
    },
    {
        "name": "policy_pipeline",
        "display_name": "Policy pipeline",
        "role": "component policy composition",
        "policy_json": "policy/policy_pipeline_v1.json",
        "vectors": "vectors/policy_pipeline",
        "rust_mirror_tests": [
            "core/rust/mercury-policy/tests/policy_pipeline_vectors.rs",
            "core/rust/mercury-core/tests/core_policy_vectors.rs",
        ],
        "python_vector_check": "tools/check_policy_pipeline_vectors.py",
        "decision_sources": ["policy_pipeline", "client_policy"],
        "component_policies": [
            "envelope",
            "room_epoch",
            "ai_grant",
            "ai_grant_lifecycle",
        ],
    },
    {
        "name": "relay_submit",
        "display_name": "Relay submit",
        "role": "opaque relay submission gate",
        "policy_json": "policy/relay_submit_policy_v1.json",
        "vectors": "vectors/relay_submit",
        "rust_mirror_tests": [
            "core/rust/mercury-policy/tests/relay_submit_vectors.rs",
            "core/rust/mercury-core/tests/relay_submission.rs",
        ],
        "python_vector_check": "tools/check_relay_submit_vectors.py",
        "decision_sources": [],
    },
    {
        "name": "platform_decision",
        "display_name": "Platform decision",
        "role": "capability view projection",
        "policy_json": "policy/platform_decision_v1.json",
        "vectors": "vectors/platform_decision",
        "rust_mirror_tests": ["core/rust/mercury-core/tests/platform_decision_vectors.rs"],
        "python_vector_check": "tools/check_platform_decision_vectors.py",
        "decision_sources": [],
    },
    {
        "name": "outbound_decide",
        "display_name": "Outbound send",
        "role": "client outbound send gate",
        "policy_json": "policy/outbound_decide_v1.json",
        "vectors": "vectors/outbound_decide",
        "rust_mirror_tests": [
            "core/rust/mercury-core/tests/outbound_decide_vectors.rs",
            "core/rust/mercury-core/tests/decider_exhaustive_diff.rs",
        ],
        "python_vector_check": "tools/check_outbound_decide_vectors.py",
        "decision_sources": ["outbound_send"],
    },
    {
        "name": "receive_decide",
        "display_name": "Client receive",
        "role": "client receive gate",
        "policy_json": "policy/receive_decide_v1.json",
        "vectors": "vectors/receive_decide",
        "rust_mirror_tests": [
            "core/rust/mercury-core/tests/receive_decide_vectors.rs",
            "core/rust/mercury-core/tests/decider_exhaustive_diff.rs",
        ],
        "python_vector_check": "tools/check_receive_decide_vectors.py",
        "decision_sources": ["client_receive"],
    },
    {
        "name": "bootstrap_decide",
        "display_name": "Client bootstrap",
        "role": "client bootstrap and startup gate",
        "policy_json": "policy/bootstrap_decide_v1.json",
        "vectors": "vectors/bootstrap_decide",
        "rust_mirror_tests": [
            "core/rust/mercury-core/tests/bootstrap_decide_vectors.rs",
            "core/rust/mercury-core/tests/decider_exhaustive_diff.rs",
        ],
        "python_vector_check": "tools/check_bootstrap_decide_vectors.py",
        "decision_sources": ["client_bootstrap"],
    },
    {
        "name": "inbound_sync",
        "display_name": "Inbound sync",
        "role": "client inbound sync gate",
        "policy_json": "policy/inbound_sync_v1.json",
        "vectors": "vectors/inbound_sync",
        "rust_mirror_tests": [
            "core/rust/mercury-core/tests/inbound_sync_vectors.rs",
            "core/rust/mercury-core/tests/decider_exhaustive_diff.rs",
        ],
        "python_vector_check": "tools/check_inbound_sync_vectors.py",
        "decision_sources": [],
    },
    {
        "name": "account_recovery",
        "display_name": "Account recovery",
        "role": "account recovery gate",
        "policy_json": "policy/account_recovery_v1.json",
        "vectors": "vectors/account_recovery",
        "rust_mirror_tests": [
            "core/rust/mercury-core/tests/account_recovery_vectors.rs",
            "core/rust/mercury-core/tests/decider_exhaustive_diff.rs",
        ],
        "python_vector_check": "tools/check_account_recovery_vectors.py",
        "decision_sources": [],
    },
]


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def _normalize_newlines(data: bytes) -> bytes:
    # Hash line-ending-independently so Windows checkouts (autocrlf -> CRLF) and the Linux CI runner
    # (LF) produce IDENTICAL digests for the same source. Without this, `--check` drifts across
    # platforms (the manifest is generated on one and verified on the other).
    return data.replace(b"\r\n", b"\n").replace(b"\r", b"\n")


def file_sha256(path: Path) -> str | None:
    if not path.exists():
        return None
    return hashlib.sha256(_normalize_newlines(path.read_bytes())).hexdigest()


def directory_sha256(path: Path) -> tuple[str | None, int]:
    if not path.exists():
        return None, 0
    h = hashlib.sha256()
    count = 0
    for child in sorted(p for p in path.rglob("*") if p.is_file()):
        count += 1
        h.update(rel(child).encode("utf-8"))
        h.update(b"\0")
        h.update(_normalize_newlines(child.read_bytes()))
        h.update(b"\0")
    return h.hexdigest(), count


def require(path: Path) -> None:
    if not path.exists():
        raise FileNotFoundError(rel(path))


def build_manifest() -> dict[str, Any]:
    policy_entries: list[dict[str, Any]] = []
    for policy in POLICIES:
        name = policy["name"]
        policy_path = ROOT / "helix" / "policy" / f"{name}.hx"
        test_path = ROOT / "helix" / "tests" / f"{name}_test.hx"
        vectors_path = ROOT / policy["vectors"]
        policy_json_path = ROOT / policy["policy_json"]
        python_check_path = ROOT / policy["python_vector_check"]
        require(policy_path)
        require(test_path)
        require(vectors_path)
        require(policy_json_path)
        require(python_check_path)
        for test in policy["rust_mirror_tests"]:
            require(ROOT / test)

        vector_hash, vector_file_count = directory_sha256(vectors_path)
        entry = {
            "name": name,
            "display_name": policy["display_name"],
            "role": policy["role"],
            "decision_sources": policy.get("decision_sources", []),
            "component_policies": policy.get("component_policies", []),
            "helix_policy": rel(policy_path),
            "helix_policy_sha256": file_sha256(policy_path),
            "helix_test": rel(test_path),
            "helix_test_sha256": file_sha256(test_path),
            "policy_json": rel(policy_json_path),
            "policy_json_sha256": file_sha256(policy_json_path),
            "vectors": rel(vectors_path),
            "vector_file_count": vector_file_count,
            "vectors_sha256": vector_hash,
            "python_vector_check": rel(python_check_path),
            "rust_mirror_tests": policy["rust_mirror_tests"],
            "required_gates": [
                "python_vector_check",
                "rust_vector_or_exhaustive_test",
                "helix_hash",
                "helix_proof_obligations",
                "helix_elf_exit_42",
            ],
            "optional_gates": ["from_raw_k1_elf_exit_42"],
            "production_runtime": "rust_mirror",
        }
        policy_entries.append(entry)

    return {
        "schema": "mercury.helix.policy.attestation.v1",
        "generated_by": "tools/gen_policy_attestations.py",
        "helix_boundary": (
            "Helix owns deterministic policy, validators, state-machine checks, "
            "and proof artifacts. Rust and reviewed libraries own production "
            "cryptography, ratchets, key storage, and platform bindings."
        ),
        "toolchains": {
            "ci_primary": {
                "kind": "pinned_python_helixc",
                "path": "third_party/helix",
                "runner": "tools/run_helix_checks.sh",
            },
            "windows_primary": {
                "kind": "pinned_python_helixc",
                "path": "third_party/helix",
                "runner": "tools/run_helix_checks.ps1",
            },
            "optional_from_raw": {
                "kind": "kovostov_native_self_hosted_k1",
                "runner": "tools/run_fromraw_helix_check.sh",
                "windows_runner": "tools/run_native_helix_checks.ps1",
                "kovostov_native_access": "read_only_copy_to_tmp",
            },
        },
        "policies": policy_entries,
    }


def compact_policy(entry: dict[str, Any]) -> dict[str, Any]:
    return {
        "name": entry["name"],
        "displayName": entry["display_name"],
        "role": entry["role"],
        "sourceHash": entry["helix_policy_sha256"],
        "sourceHashShort": entry["helix_policy_sha256"][:12],
        "testHashShort": entry["helix_test_sha256"][:12],
        "vectorHashShort": entry["vectors_sha256"][:12],
        "vectorFileCount": entry["vector_file_count"],
        "policyJson": entry["policy_json"],
        "rustMirrorTests": entry["rust_mirror_tests"],
        "requiredGates": entry["required_gates"],
        "optionalGates": entry["optional_gates"],
        "productionRuntime": entry["production_runtime"],
        "componentPolicies": entry["component_policies"],
    }


def build_ts(manifest: dict[str, Any]) -> str:
    policies = {entry["name"]: compact_policy(entry) for entry in manifest["policies"]}
    source_map: dict[str, str] = {}
    for entry in manifest["policies"]:
        for source in entry["decision_sources"]:
            source_map[source] = entry["name"]

    return (
        "// Generated by tools/gen_policy_attestations.py --write. Do not edit by hand.\n"
        "\n"
        f"export const HELIX_POLICY_BOUNDARY = {json.dumps(manifest['helix_boundary'])} as const;\n"
        "\n"
        f"export const POLICY_ATTESTATIONS = {json.dumps(policies, indent=2, sort_keys=True)} as const;\n"
        "\n"
        f"export const DECISION_SOURCE_POLICY = {json.dumps(source_map, indent=2, sort_keys=True)} as const;\n"
        "\n"
        "export type PolicyName = keyof typeof POLICY_ATTESTATIONS;\n"
        "\n"
        "export function policyForDecisionSource(source: string): PolicyName | undefined {\n"
        "  return DECISION_SOURCE_POLICY[source as keyof typeof DECISION_SOURCE_POLICY];\n"
        "}\n"
        "\n"
        "export function attestationForDecisionSource(source: string) {\n"
        "  const policy = policyForDecisionSource(source);\n"
        "  return policy ? POLICY_ATTESTATIONS[policy] : undefined;\n"
        "}\n"
        "\n"
        "export function componentAttestations(policy: PolicyName) {\n"
        "  return POLICY_ATTESTATIONS[policy].componentPolicies.map(\n"
        "    (name) => POLICY_ATTESTATIONS[name as PolicyName],\n"
        "  );\n"
        "}\n"
    )


def write_if_changed(path: Path, content: str) -> None:
    if path.exists() and path.read_text(encoding="utf-8") == content:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8", newline="\n")


def check_file(path: Path, expected: str) -> bool:
    actual = path.read_text(encoding="utf-8") if path.exists() else None
    if actual != expected:
        print(f"drift: {rel(path)}")
        return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="write generated artifacts")
    parser.add_argument("--check", action="store_true", help="fail if artifacts are stale")
    args = parser.parse_args()

    manifest = build_manifest()
    manifest_json = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    manifest_ts = build_ts(manifest)

    if args.write:
        write_if_changed(MANIFEST_PATH, manifest_json)
        write_if_changed(TS_PATH, manifest_ts)
        print(f"wrote {rel(MANIFEST_PATH)}")
        print(f"wrote {rel(TS_PATH)}")
        return 0

    if args.check:
        ok = check_file(MANIFEST_PATH, manifest_json)
        ok = check_file(TS_PATH, manifest_ts) and ok
        if not ok:
            print("Run: python tools/gen_policy_attestations.py --write")
            return 1
        print("policy attestations: OK")
        return 0

    print(manifest_json, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
