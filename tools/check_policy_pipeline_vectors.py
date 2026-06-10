#!/usr/bin/env python3
"""Check Mercury Phase 1 policy pipeline vectors without external dependencies."""

from __future__ import annotations

import json
from pathlib import Path


POLICY_PIPELINE_ACCEPT = 0
POLICY_PIPELINE_BAD_VERSION = 1
POLICY_PIPELINE_BAD_ACTOR_KIND = 2
POLICY_PIPELINE_BAD_ENVELOPE_REASON = 3
POLICY_PIPELINE_BAD_ROOM_EPOCH_REASON = 4
POLICY_PIPELINE_BAD_AI_GRANT_REASON = 5
POLICY_PIPELINE_BAD_AI_LIFECYCLE_REASON = 6
POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN = 7
POLICY_PIPELINE_ENVELOPE_REJECT = 8
POLICY_PIPELINE_ROOM_EPOCH_REJECT = 9
POLICY_PIPELINE_AI_GRANT_REJECT = 10
POLICY_PIPELINE_AI_LIFECYCLE_REJECT = 11
POLICY_PIPELINE_AI_POST_REVOKE_ACCESS_REJECT = 12
POLICY_PIPELINE_AI_HIGHSEC_ROTATION_REQUIRED = 13

POLICY_PIPELINE_AUDIT_ACCEPTED_POLICY_DECISION = 1
POLICY_PIPELINE_AUDIT_PIPELINE_CONTRACT_REJECT = 2
POLICY_PIPELINE_AUDIT_MESSAGE_POLICY_REJECT = 3
POLICY_PIPELINE_AUDIT_ROOM_POLICY_REJECT = 4
POLICY_PIPELINE_AUDIT_AI_POLICY_REJECT = 5
POLICY_PIPELINE_AUDIT_AI_LIFECYCLE_POLICY_REJECT = 6
POLICY_PIPELINE_AUDIT_AI_POST_REVOKE_REJECT = 7
POLICY_PIPELINE_AUDIT_AI_HIGHSEC_ROTATION_REJECT = 8


def validate_inputs(input_: dict[str, int]) -> int:
    if input_["version"] != 1:
        return POLICY_PIPELINE_BAD_VERSION
    if input_["actor_kind"] < 1 or input_["actor_kind"] > 3:
        return POLICY_PIPELINE_BAD_ACTOR_KIND
    if input_["envelope_reason"] < 0 or input_["envelope_reason"] > 12:
        return POLICY_PIPELINE_BAD_ENVELOPE_REASON
    if input_["room_epoch_reason"] < 0 or input_["room_epoch_reason"] > 13:
        return POLICY_PIPELINE_BAD_ROOM_EPOCH_REASON
    if input_["ai_grant_reason"] < 0 or input_["ai_grant_reason"] > 20:
        return POLICY_PIPELINE_BAD_AI_GRANT_REASON
    if input_["ai_lifecycle_reason"] < 0 or input_["ai_lifecycle_reason"] > 10:
        return POLICY_PIPELINE_BAD_AI_LIFECYCLE_REASON
    return POLICY_PIPELINE_ACCEPT


def validate_actor_components(input_: dict[str, int]) -> int:
    if input_["actor_kind"] == 1 and (
        input_["ai_grant_reason"] != 0 or input_["ai_lifecycle_reason"] != 0
    ):
        return POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN
    return POLICY_PIPELINE_ACCEPT


def map_ai_lifecycle(ai_lifecycle_reason: int) -> int:
    if ai_lifecycle_reason == 0:
        return POLICY_PIPELINE_ACCEPT
    if ai_lifecycle_reason == 9:
        return POLICY_PIPELINE_AI_HIGHSEC_ROTATION_REQUIRED
    if ai_lifecycle_reason in (7, 8):
        return POLICY_PIPELINE_AI_POST_REVOKE_ACCESS_REJECT
    return POLICY_PIPELINE_AI_LIFECYCLE_REJECT


def component_first_reject(input_: dict[str, int]) -> int:
    if input_["envelope_reason"] != 0:
        return POLICY_PIPELINE_ENVELOPE_REJECT
    if input_["room_epoch_reason"] != 0:
        return POLICY_PIPELINE_ROOM_EPOCH_REJECT

    actor_reason = validate_actor_components(input_)
    if actor_reason != POLICY_PIPELINE_ACCEPT:
        return actor_reason
    if input_["ai_grant_reason"] != 0:
        return POLICY_PIPELINE_AI_GRANT_REJECT
    return map_ai_lifecycle(input_["ai_lifecycle_reason"])


def validate(input_: dict[str, int]) -> int:
    input_reason = validate_inputs(input_)
    if input_reason != POLICY_PIPELINE_ACCEPT:
        return input_reason
    return component_first_reject(input_)


def audit_class_for_reason(reason_code: int) -> int:
    if reason_code == POLICY_PIPELINE_ACCEPT:
        return POLICY_PIPELINE_AUDIT_ACCEPTED_POLICY_DECISION
    if reason_code < POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN:
        return POLICY_PIPELINE_AUDIT_PIPELINE_CONTRACT_REJECT
    if reason_code == POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN:
        return POLICY_PIPELINE_AUDIT_AI_POLICY_REJECT
    if reason_code == POLICY_PIPELINE_ENVELOPE_REJECT:
        return POLICY_PIPELINE_AUDIT_MESSAGE_POLICY_REJECT
    if reason_code == POLICY_PIPELINE_ROOM_EPOCH_REJECT:
        return POLICY_PIPELINE_AUDIT_ROOM_POLICY_REJECT
    if reason_code == POLICY_PIPELINE_AI_GRANT_REJECT:
        return POLICY_PIPELINE_AUDIT_AI_POLICY_REJECT
    if reason_code == POLICY_PIPELINE_AI_LIFECYCLE_REJECT:
        return POLICY_PIPELINE_AUDIT_AI_LIFECYCLE_POLICY_REJECT
    if reason_code == POLICY_PIPELINE_AI_POST_REVOKE_ACCESS_REJECT:
        return POLICY_PIPELINE_AUDIT_AI_POST_REVOKE_REJECT
    if reason_code == POLICY_PIPELINE_AI_HIGHSEC_ROTATION_REQUIRED:
        return POLICY_PIPELINE_AUDIT_AI_HIGHSEC_ROTATION_REJECT
    return POLICY_PIPELINE_AUDIT_PIPELINE_CONTRACT_REJECT


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "policy_pipeline"
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

    print("policy pipeline vectors: OK (15 checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
