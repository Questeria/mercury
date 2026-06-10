#!/usr/bin/env python3
"""Check Mercury Phase 1 AI grant lifecycle vectors without external dependencies."""

from __future__ import annotations

import json
from pathlib import Path


AI_GRANT_LIFECYCLE_ACCEPT = 0
AI_GRANT_LIFECYCLE_BAD_VERSION = 1
AI_GRANT_LIFECYCLE_BAD_GRANT_STATE = 2
AI_GRANT_LIFECYCLE_BAD_REVOKE_REASON = 3
AI_GRANT_LIFECYCLE_BAD_TIME = 4
AI_GRANT_LIFECYCLE_GRANT_EXPIRED = 5
AI_GRANT_LIFECYCLE_GRANT_REVOKED = 6
AI_GRANT_LIFECYCLE_READ_AFTER_REVOKE = 7
AI_GRANT_LIFECYCLE_WRITE_AFTER_REVOKE = 8
AI_GRANT_LIFECYCLE_HIGHSEC_EPOCH_ROTATION_REQUIRED = 9
AI_GRANT_LIFECYCLE_BAD_ACCESS_KIND = 10

AI_GRANT_LIFECYCLE_AUDIT_ACCEPTED_AI_GRANT_LIFECYCLE = 1
AI_GRANT_LIFECYCLE_AUDIT_AI_LIFECYCLE_POLICY_REJECT = 2
AI_GRANT_LIFECYCLE_AUDIT_AI_GRANT_EXPIRED_REJECT = 3
AI_GRANT_LIFECYCLE_AUDIT_AI_GRANT_REVOKED_REJECT = 4
AI_GRANT_LIFECYCLE_AUDIT_AI_POST_REVOKE_ACCESS_REJECT = 5
AI_GRANT_LIFECYCLE_AUDIT_AI_HIGHSEC_EPOCH_ROTATION_REQUIRED = 6


def validate_state(input_: dict[str, int]) -> int:
    if input_["version"] != 1:
        return AI_GRANT_LIFECYCLE_BAD_VERSION
    if input_["grant_state"] < 1 or input_["grant_state"] > 2:
        return AI_GRANT_LIFECYCLE_BAD_GRANT_STATE
    if input_["now_s"] < 0 or input_["expires_at_s"] <= 0:
        return AI_GRANT_LIFECYCLE_BAD_TIME
    if input_["grant_state"] == 1:
        if input_["revoke_reason"] != 0:
            return AI_GRANT_LIFECYCLE_BAD_REVOKE_REASON
        if input_["now_s"] >= input_["expires_at_s"]:
            return AI_GRANT_LIFECYCLE_GRANT_EXPIRED
        return AI_GRANT_LIFECYCLE_ACCEPT
    if input_["revoke_reason"] < 1 or input_["revoke_reason"] > 5:
        return AI_GRANT_LIFECYCLE_BAD_REVOKE_REASON
    return AI_GRANT_LIFECYCLE_GRANT_REVOKED


def post_revoke_access(access_kind: int) -> int:
    if access_kind == 0:
        return AI_GRANT_LIFECYCLE_GRANT_REVOKED
    if access_kind == 1:
        return AI_GRANT_LIFECYCLE_READ_AFTER_REVOKE
    return AI_GRANT_LIFECYCLE_WRITE_AFTER_REVOKE


def validate_access(input_: dict[str, int], lifecycle_reason: int) -> int:
    access_kind = input_["access_kind"]
    if access_kind < 0 or access_kind > 2:
        return AI_GRANT_LIFECYCLE_BAD_ACCESS_KIND
    if lifecycle_reason != AI_GRANT_LIFECYCLE_GRANT_REVOKED:
        return AI_GRANT_LIFECYCLE_ACCEPT
    if input_["room_mode"] == 3 and input_["epoch_rotated"] != 1:
        return AI_GRANT_LIFECYCLE_HIGHSEC_EPOCH_ROTATION_REQUIRED
    return post_revoke_access(access_kind)


def first_reject(state_reason: int, access_reason: int) -> int:
    if access_reason == AI_GRANT_LIFECYCLE_HIGHSEC_EPOCH_ROTATION_REQUIRED:
        return access_reason
    if state_reason == AI_GRANT_LIFECYCLE_ACCEPT:
        return access_reason
    if access_reason == AI_GRANT_LIFECYCLE_ACCEPT:
        return state_reason
    return access_reason


def audit_class_for_reason(reason_code: int) -> int:
    if reason_code == AI_GRANT_LIFECYCLE_ACCEPT:
        return AI_GRANT_LIFECYCLE_AUDIT_ACCEPTED_AI_GRANT_LIFECYCLE
    if reason_code == AI_GRANT_LIFECYCLE_GRANT_EXPIRED:
        return AI_GRANT_LIFECYCLE_AUDIT_AI_GRANT_EXPIRED_REJECT
    if reason_code == AI_GRANT_LIFECYCLE_GRANT_REVOKED:
        return AI_GRANT_LIFECYCLE_AUDIT_AI_GRANT_REVOKED_REJECT
    if reason_code in (
        AI_GRANT_LIFECYCLE_READ_AFTER_REVOKE,
        AI_GRANT_LIFECYCLE_WRITE_AFTER_REVOKE,
    ):
        return AI_GRANT_LIFECYCLE_AUDIT_AI_POST_REVOKE_ACCESS_REJECT
    if reason_code == AI_GRANT_LIFECYCLE_HIGHSEC_EPOCH_ROTATION_REQUIRED:
        return AI_GRANT_LIFECYCLE_AUDIT_AI_HIGHSEC_EPOCH_ROTATION_REQUIRED
    return AI_GRANT_LIFECYCLE_AUDIT_AI_LIFECYCLE_POLICY_REJECT


def validate(input_: dict[str, int]) -> int:
    state_reason = validate_state(input_)
    return first_reject(state_reason, validate_access(input_, state_reason))


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "ai_grant_lifecycle"
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

    if count != 15:
        print(f"expected 15 vectors, checked {count}")
        return 1

    print("AI grant lifecycle vectors: OK (15 checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
