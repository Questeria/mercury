#!/usr/bin/env python3
"""Check Mercury Phase 1 AI grant vectors without external dependencies."""

from __future__ import annotations

import json
from pathlib import Path


AI_GRANT_ACCEPT = 0
AI_GRANT_BAD_VERSION = 1
AI_GRANT_BAD_PRINCIPAL_KIND = 2
AI_GRANT_BAD_AI_MODE = 3
AI_GRANT_AI_BLOCKED_ROOM = 4
AI_GRANT_BAD_TTL = 5
AI_GRANT_TTL_TOO_LONG = 6
AI_GRANT_INSUFFICIENT_APPROVERS = 7
AI_GRANT_BAD_READ_SCOPE = 8
AI_GRANT_READ_SCOPE_TOO_BROAD = 9
AI_GRANT_BAD_WRITE_SCOPE = 10
AI_GRANT_WRITE_SCOPE_TOO_BROAD = 11
AI_GRANT_BAD_TOOL_SCOPE = 12
AI_GRANT_TOOL_SCOPE_TOO_BROAD = 13
AI_GRANT_BAD_RETENTION_MODE = 14
AI_GRANT_PROMPT_STORE_FORBIDDEN = 15
AI_GRANT_TRAINING_FORBIDDEN = 16
AI_GRANT_REMOTE_PROVIDER_FORBIDDEN = 17
AI_GRANT_HIGHSEC_LOCAL_ONLY = 18
AI_GRANT_HIGHSEC_CONFIRM_SEND_REQUIRED = 19
AI_GRANT_HIGHSEC_TOOLS_FORBIDDEN = 20

AI_GRANT_AUDIT_ACCEPTED_AI_GRANT = 1
AI_GRANT_AUDIT_AI_POLICY_REJECT = 2
AI_GRANT_AUDIT_AI_PRIVACY_REJECT = 3
AI_GRANT_AUDIT_AI_HIGHSEC_REJECT = 4
AI_GRANT_AUDIT_AI_RETENTION_REJECT = 5
AI_GRANT_AUDIT_AI_TOOL_REJECT = 6


def validate_subject(input_: dict[str, int]) -> int:
    if input_["version"] != 1:
        return AI_GRANT_BAD_VERSION
    if input_["principal_kind"] < 2 or input_["principal_kind"] > 4:
        return AI_GRANT_BAD_PRINCIPAL_KIND
    if input_["ai_mode"] < 1 or input_["ai_mode"] > 3:
        return AI_GRANT_BAD_AI_MODE
    if input_["principal_kind"] == 4 and input_["ai_mode"] == 1:
        return AI_GRANT_BAD_AI_MODE
    if input_["room_mode"] < 1 or input_["room_mode"] > 4 or input_["room_mode"] == 4:
        return AI_GRANT_AI_BLOCKED_ROOM
    if input_["ttl_s"] < 1:
        return AI_GRANT_BAD_TTL
    if input_["ttl_s"] > 900:
        return AI_GRANT_TTL_TOO_LONG
    if input_["approver_count"] < 1:
        return AI_GRANT_INSUFFICIENT_APPROVERS
    if input_["ai_mode"] == 3 and input_["room_mode"] > 1:
        return AI_GRANT_REMOTE_PROVIDER_FORBIDDEN
    return AI_GRANT_ACCEPT


def validate_scopes(input_: dict[str, int]) -> int:
    if input_["read_scope"] < 0 or input_["read_scope"] > 4:
        return AI_GRANT_BAD_READ_SCOPE
    if input_["read_scope"] == 4:
        return AI_GRANT_READ_SCOPE_TOO_BROAD
    if input_["write_scope"] < 0 or input_["write_scope"] > 3:
        return AI_GRANT_BAD_WRITE_SCOPE
    if input_["write_scope"] == 3:
        return AI_GRANT_WRITE_SCOPE_TOO_BROAD
    if input_["tool_scope"] < 0 or input_["tool_scope"] > 4:
        return AI_GRANT_BAD_TOOL_SCOPE
    if input_["tool_scope"] > 2:
        return AI_GRANT_TOOL_SCOPE_TOO_BROAD
    if input_["retention_mode"] < 0 or input_["retention_mode"] > 3:
        return AI_GRANT_BAD_RETENTION_MODE
    if input_["retention_mode"] > 1 or input_["prompt_store_allowed"] != 0:
        return AI_GRANT_PROMPT_STORE_FORBIDDEN
    if input_["training_allowed"] != 0:
        return AI_GRANT_TRAINING_FORBIDDEN
    return AI_GRANT_ACCEPT


def validate_highsec(input_: dict[str, int]) -> int:
    if input_["room_mode"] != 3:
        return AI_GRANT_ACCEPT
    if input_["ai_mode"] != 1:
        return AI_GRANT_HIGHSEC_LOCAL_ONLY
    if input_["approver_count"] < 2:
        return AI_GRANT_INSUFFICIENT_APPROVERS
    if input_["write_scope"] == 3:
        return AI_GRANT_HIGHSEC_CONFIRM_SEND_REQUIRED
    if input_["tool_scope"] > 1:
        return AI_GRANT_HIGHSEC_TOOLS_FORBIDDEN
    return AI_GRANT_ACCEPT


def first_reject(subject_reason: int, scope_reason: int, highsec_reason: int) -> int:
    if subject_reason != AI_GRANT_ACCEPT:
        return subject_reason
    if scope_reason != AI_GRANT_ACCEPT:
        return scope_reason
    return highsec_reason


def audit_class_for_reason(reason_code: int) -> int:
    if reason_code == AI_GRANT_ACCEPT:
        return AI_GRANT_AUDIT_ACCEPTED_AI_GRANT
    if reason_code in (AI_GRANT_REMOTE_PROVIDER_FORBIDDEN, AI_GRANT_READ_SCOPE_TOO_BROAD):
        return AI_GRANT_AUDIT_AI_PRIVACY_REJECT
    if reason_code in (
        AI_GRANT_HIGHSEC_LOCAL_ONLY,
        AI_GRANT_HIGHSEC_CONFIRM_SEND_REQUIRED,
        AI_GRANT_HIGHSEC_TOOLS_FORBIDDEN,
    ):
        return AI_GRANT_AUDIT_AI_HIGHSEC_REJECT
    if reason_code in (AI_GRANT_PROMPT_STORE_FORBIDDEN, AI_GRANT_TRAINING_FORBIDDEN):
        return AI_GRANT_AUDIT_AI_RETENTION_REJECT
    if reason_code in (AI_GRANT_BAD_TOOL_SCOPE, AI_GRANT_TOOL_SCOPE_TOO_BROAD):
        return AI_GRANT_AUDIT_AI_TOOL_REJECT
    return AI_GRANT_AUDIT_AI_POLICY_REJECT


def validate(input_: dict[str, int]) -> int:
    return first_reject(
        validate_subject(input_),
        validate_scopes(input_),
        validate_highsec(input_),
    )


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "ai_grant"
    count = 0

    for path in sorted(vector_dir.glob("*.json")):
        vector = json.loads(path.read_text(encoding="utf-8"))
        reason = validate(vector["input"])
        audit_class = audit_class_for_reason(reason)

        if reason != vector["expected_reason"]:
            print(
                f"{path.name}: reason mismatch, got {reason}, "
                f"expected {vector['expected_reason']}"
            )
            return 1
        if audit_class != vector["expected_audit_class"]:
            print(
                f"{path.name}: audit mismatch, got {audit_class}, "
                f"expected {vector['expected_audit_class']}"
            )
            return 1
        count += 1

    if count != 20:
        print(f"expected 20 vectors, checked {count}")
        return 1

    print("AI grant vectors: OK (20 checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

