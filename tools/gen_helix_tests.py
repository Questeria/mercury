#!/usr/bin/env python3
"""Generate Helix policy test files from the shared ``vectors/`` (Track 0b).

Helix programs have no runtime input channel -- a ``.hx`` test bakes its inputs
into source and signals success via exit code 42. The Rust mirror
(``tests/<policy>_vectors.rs``) and the Python checkers consume the JSON vectors
directly, but the Helix ``*_test.hx`` were hand-written in parallel and not
mechanically linked -- so the Helix lane could silently drift from the vectors.

This generator regenerates each Helix test's ``main()`` from
``vectors/<policy>/*.json``. The hand-maintained ``@pure`` policy prelude
(everything before ``fn main()``) is preserved; only ``main()`` is generated,
one ``expect`` + one ``expect_audit`` per vector. Crucially the generated calls
feed EVERY vector input field into the full policy validators (no baked
constants, no hand-curated helper dispatch), so Rust, the Python checkers, and
Helix provably evaluate the same inputs.

Usage:
    python3 tools/gen_helix_tests.py --write           # rewrite all test .hx
    python3 tools/gen_helix_tests.py --check            # fail (exit 1) on drift
    python3 tools/gen_helix_tests.py --write envelope   # subset

CI runs ``--check``; with the compile+execute in run_helix_checks.sh, a vector
edit without regen fails the diff and a policy-logic change that breaks a vector
makes the ELF exit != 42 -- divergence fails CI either way.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import sys
from typing import Callable

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MAIN_MARKER = "fn main()"


def load_vectors(policy: str) -> list[dict]:
    files = sorted(glob.glob(os.path.join(ROOT, "vectors", policy, "*.json")))
    if not files:
        sys.exit(f"error: no vectors under vectors/{policy}")
    out = []
    for path in files:
        with open(path, encoding="utf-8") as fh:
            out.append(json.load(fh))
    return out


def prelude_of(policy: str) -> str:
    """Everything before ``fn main()`` -- the hand-maintained @pure library."""
    path = os.path.join(ROOT, "helix", "tests", f"{policy}_test.hx")
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    idx = text.find(MAIN_MARKER)
    if idx == -1:
        sys.exit(f"error: no '{MAIN_MARKER}' in {policy}_test.hx")
    return text[:idx]


def ml(name: str, args: list[str]) -> str:
    """A multi-line combinator call, matching the existing test formatting."""
    body = "".join(f"        {a},\n" for a in args)
    return f"{name}(\n{body}    )"


# ---- per-policy expression builders (vector input -> combinator call string) ----


def envelope_expr(i: dict) -> str:
    return ml(
        "mercury_first_reject",
        [
            f"mercury_validate_identity_v1({i['version']}, {i['suite_id']}, {i['min_suite_id']}, {i['conversation_id_len']}, {i['sender_account_id_len']}, {i['sender_device_id_len']})",
            f"mercury_validate_order_v1({i['epoch']}, {i['sequence']}, {i['expected_epoch']}, {i['expected_sequence']})",
            f"mercury_validate_content_v1({i['message_kind']}, {i['payload_len']}, {i['critical_flags']}, {i['noncritical_flags']}, {i['max_payload_len']})",
        ],
    )


def ai_grant_expr(i: dict) -> str:
    return ml(
        "mercury_ai_grant_first_reject",
        [
            f"mercury_ai_grant_validate_subject_v1({i['version']}, {i['principal_kind']}, {i['room_mode']}, {i['ai_mode']}, {i['ttl_s']}, {i['approver_count']})",
            f"mercury_ai_grant_validate_scopes_v1({i['read_scope']}, {i['write_scope']}, {i['tool_scope']}, {i['retention_mode']}, {i['training_allowed']}, {i['prompt_store_allowed']})",
            f"mercury_ai_grant_validate_highsec_v1({i['room_mode']}, {i['principal_kind']}, {i['ai_mode']}, {i['write_scope']}, {i['tool_scope']}, {i['approver_count']})",
        ],
    )


def policy_pipeline_expr(i: dict) -> str:
    return (
        f"mercury_pipeline_decide_v1({i['version']}, {i['actor_kind']}, "
        f"{i['envelope_reason']}, {i['room_epoch_reason']}, {i['ai_grant_reason']}, "
        f"{i['ai_lifecycle_reason']})"
    )


def relay_submit_expr(i: dict) -> str:
    return ml(
        "mercury_relay_decide_v1",
        [
            f"{i['version']}",
            f"{i['send_gate_reason']}",
            f"mercury_relay_validate_metadata({i['route_id_len']}, {i['replay_token_len']}, {i['sealed_header_len']}, {i['plaintext_identity_fields']}, {i['padding_bucket']})",
            f"mercury_relay_validate_lifetime({i['queue_ttl_s']}, {i['max_queue_ttl_s']})",
            f"mercury_relay_validate_ciphertext({i['ciphertext_len']}, {i['max_ciphertext_len']})",
        ],
    )


def ai_grant_lifecycle_expr(i: dict) -> str:
    state = (
        f"mercury_lifecycle_validate_state_v1({i['version']}, {i['grant_state']}, "
        f"{i['revoke_reason']}, {i['now_s']}, {i['expires_at_s']}, {i['room_mode']})"
    )
    access = (
        f"mercury_lifecycle_validate_access_v1({state}, {i['access_kind']}, "
        f"{i['room_mode']}, {i['epoch_rotated']})"
    )
    return ml("mercury_lifecycle_first_reject", [state, access])


def room_epoch_expr(i: dict) -> str:
    return ml(
        "mercury_room_epoch_first_reject",
        [
            f"mercury_room_epoch_validate_epoch_v1({i['version']}, {i['room_mode']}, {i['current_epoch']}, {i['message_epoch']}, {i['min_accepted_epoch']})",
            f"mercury_room_epoch_validate_device_v1({i['device_kind']}, {i['device_state']}, {i['revoked_at_epoch']}, {i['current_epoch']}, {i['access_kind']})",
            f"mercury_room_epoch_validate_highsec_v1({i['room_mode']}, {i['device_kind']}, {i['device_state']}, {i['revoked_at_epoch']}, {i['current_epoch']}, {i['access_kind']})",
        ],
    )


def fn(prefix: str, mapping: dict[int, str]) -> dict[int, str]:
    return {code: f"{prefix}{name}" for code, name in mapping.items()}


POLICIES: dict[str, dict] = {
    "envelope": {
        "expr": envelope_expr,
        "reason": {
            0: "mercury_accept", 1: "mercury_bad_version", 2: "mercury_unsupported_suite",
            3: "mercury_downgraded_suite", 4: "mercury_bad_conversation_id_len",
            5: "mercury_bad_sender_account_id_len", 6: "mercury_bad_sender_device_id_len",
            7: "mercury_bad_epoch", 8: "mercury_bad_sequence", 9: "mercury_bad_message_kind",
            10: "mercury_payload_too_large", 11: "mercury_unknown_critical_flag",
            12: "mercury_reserved_ai_kind",
        },
        "audit": {
            1: "mercury_audit_accepted_message", 2: "mercury_audit_policy_reject",
            3: "mercury_audit_downgrade_attempt", 4: "mercury_audit_size_reject",
        },
    },
    "ai_grant": {
        "expr": ai_grant_expr,
        "reason": fn("mercury_ai_grant_", {
            0: "accept", 1: "bad_version", 2: "bad_principal_kind", 3: "bad_ai_mode",
            4: "ai_blocked_room", 5: "bad_ttl", 6: "ttl_too_long", 7: "insufficient_approvers",
            8: "bad_read_scope", 9: "read_scope_too_broad", 10: "bad_write_scope",
            11: "write_scope_too_broad", 12: "bad_tool_scope", 13: "tool_scope_too_broad",
            14: "bad_retention_mode", 15: "prompt_store_forbidden", 16: "training_forbidden",
            17: "remote_provider_forbidden", 18: "highsec_local_only",
            19: "highsec_confirm_send_required", 20: "highsec_tools_forbidden",
        }),
        "audit": fn("mercury_ai_grant_audit_", {
            1: "accepted", 2: "policy_reject", 3: "privacy_reject", 4: "highsec_reject",
            5: "retention_reject", 6: "tool_reject",
        }),
    },
    "ai_grant_lifecycle": {
        "expr": ai_grant_lifecycle_expr,
        "reason": fn("mercury_lifecycle_", {
            0: "accept", 1: "bad_version", 2: "bad_grant_state", 3: "bad_revoke_reason",
            4: "bad_time", 5: "grant_expired", 6: "grant_revoked", 7: "read_after_revoke",
            8: "write_after_revoke", 9: "highsec_epoch_rotation_required", 10: "bad_access_kind",
        }),
        "audit": fn("mercury_lifecycle_audit_", {
            1: "accepted", 2: "policy_reject", 3: "expired_reject", 4: "revoked_reject",
            5: "post_revoke_access_reject", 6: "highsec_epoch_rotation_required",
        }),
    },
    "room_epoch": {
        "expr": room_epoch_expr,
        "reason": fn("mercury_room_epoch_", {
            0: "accept", 1: "bad_version", 2: "bad_room_mode", 3: "bad_epoch", 4: "stale_epoch",
            5: "future_epoch", 6: "bad_device_kind", 7: "bad_device_state",
            8: "bad_revoked_at_epoch", 9: "bad_access_kind", 10: "device_removed",
            11: "device_compromised", 12: "highsec_epoch_rotation_required",
            13: "ai_device_blocked_room",
        }),
        "audit": fn("mercury_room_epoch_audit_", {
            1: "accepted", 2: "policy_reject", 3: "replay_reject", 4: "membership_reject",
            5: "highsec_rotation_reject", 6: "ai_policy_reject",
        }),
    },
    "policy_pipeline": {
        "expr": policy_pipeline_expr,
        "reason": fn("mercury_pipeline_", {
            0: "accept", 1: "bad_version", 2: "bad_actor_kind", 3: "bad_envelope_reason",
            4: "bad_room_epoch_reason", 5: "bad_ai_grant_reason", 6: "bad_ai_lifecycle_reason",
            7: "ai_component_for_human", 8: "envelope_reject", 9: "room_epoch_reject",
            10: "ai_grant_reject", 11: "ai_lifecycle_reject", 12: "ai_post_revoke_access_reject",
            13: "ai_highsec_rotation_required",
        }),
        "audit": fn("mercury_pipeline_audit_", {
            1: "accepted", 2: "contract_reject", 3: "message_policy_reject",
            4: "room_policy_reject", 5: "ai_policy_reject", 6: "ai_lifecycle_policy_reject",
            7: "ai_post_revoke_reject", 8: "ai_highsec_rotation_reject",
        }),
    },
    "relay_submit": {
        "expr": relay_submit_expr,
        "reason": fn("mercury_relay_", {
            0: "accept", 1: "bad_version", 2: "bad_send_gate_reason", 3: "send_gate_rejected",
            4: "bad_route_id_len", 5: "bad_replay_token_len", 6: "bad_ttl", 7: "ttl_too_long",
            8: "bad_ciphertext_len", 9: "ciphertext_too_large", 10: "bad_sealed_header_len",
            11: "sealed_header_too_large", 12: "plaintext_identity_forbidden",
            13: "bad_padding_bucket",
        }),
        "audit": fn("mercury_relay_audit_", {
            1: "accepted", 2: "contract_reject", 3: "client_send_reject", 4: "metadata_reject",
            5: "retention_reject", 6: "size_reject",
        }),
    },
}


def build_main(policy: str) -> str:
    cfg = POLICIES[policy]
    expr_fn: Callable[[dict], str] = cfg["expr"]
    reason_fn, audit_fn = cfg["reason"], cfg["audit"]
    out = [
        "fn main() -> i32 {",
        "    let mut failed: i32 = 0;",
        "",
        f"    // GENERATED from vectors/{policy}/*.json by tools/gen_helix_tests.py.",
        "    // Do not edit by hand; run `python3 tools/gen_helix_tests.py --write`",
        "    // after changing the vectors (Track 0b: Rust<->Helix differential).",
    ]
    for v in load_vectors(policy):
        expr = expr_fn(v["input"])
        reason = reason_fn[v["expected_reason"]]
        audit = audit_fn[v["expected_audit_class"]]
        out += [
            "",
            f"    // {v['name']}",
            f"    failed = failed + expect({expr}, {reason}());",
            f"    failed = failed + expect_audit({expr}, {audit}());",
        ]
    out += ["", "    if failed == 0 { 42 } else { failed }", "}", ""]
    return "\n".join(out)


def render(policy: str) -> tuple[str, str]:
    path = os.path.join(ROOT, "helix", "tests", f"{policy}_test.hx")
    return path, prelude_of(policy) + build_main(policy)


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate Helix tests from vectors.")
    grp = ap.add_mutually_exclusive_group(required=True)
    grp.add_argument("--write", action="store_true", help="rewrite the .hx files")
    grp.add_argument("--check", action="store_true", help="fail on drift")
    ap.add_argument("policies", nargs="*", help="subset (default: all)")
    args = ap.parse_args()

    names = args.policies or list(POLICIES)
    drift = []
    for name in names:
        if name not in POLICIES:
            sys.exit(f"error: unknown policy '{name}'")
        path, content = render(name)
        if args.write:
            with open(path, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(content)
            print(f"wrote {os.path.relpath(path, ROOT)}")
        else:
            with open(path, encoding="utf-8") as fh:
                if fh.read() != content:
                    drift.append(name)
                    print(f"DRIFT: helix/tests/{name}_test.hx differs from vectors", file=sys.stderr)

    if args.check and drift:
        print("\nRun: python3 tools/gen_helix_tests.py --write", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
