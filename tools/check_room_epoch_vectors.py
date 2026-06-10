#!/usr/bin/env python3
"""Check Mercury Phase 1 room epoch vectors without external dependencies."""

from __future__ import annotations

import json
from pathlib import Path


ROOM_EPOCH_ACCEPT = 0
ROOM_EPOCH_BAD_VERSION = 1
ROOM_EPOCH_BAD_ROOM_MODE = 2
ROOM_EPOCH_BAD_EPOCH = 3
ROOM_EPOCH_STALE_EPOCH = 4
ROOM_EPOCH_FUTURE_EPOCH = 5
ROOM_EPOCH_BAD_DEVICE_KIND = 6
ROOM_EPOCH_BAD_DEVICE_STATE = 7
ROOM_EPOCH_BAD_REVOKED_AT_EPOCH = 8
ROOM_EPOCH_BAD_ACCESS_KIND = 9
ROOM_EPOCH_DEVICE_REMOVED = 10
ROOM_EPOCH_DEVICE_COMPROMISED = 11
ROOM_EPOCH_HIGHSEC_EPOCH_ROTATION_REQUIRED = 12
ROOM_EPOCH_AI_DEVICE_BLOCKED_ROOM = 13

ROOM_EPOCH_AUDIT_ACCEPTED_ROOM_EPOCH = 1
ROOM_EPOCH_AUDIT_ROOM_EPOCH_POLICY_REJECT = 2
ROOM_EPOCH_AUDIT_ROOM_EPOCH_REPLAY_REJECT = 3
ROOM_EPOCH_AUDIT_ROOM_DEVICE_MEMBERSHIP_REJECT = 4
ROOM_EPOCH_AUDIT_ROOM_HIGHSEC_ROTATION_REJECT = 5
ROOM_EPOCH_AUDIT_ROOM_AI_POLICY_REJECT = 6


def validate_epoch(input_: dict[str, int]) -> int:
    if input_["version"] != 1:
        return ROOM_EPOCH_BAD_VERSION
    if input_["room_mode"] < 1 or input_["room_mode"] > 4:
        return ROOM_EPOCH_BAD_ROOM_MODE
    if input_["current_epoch"] < 1 or input_["min_accepted_epoch"] < 1:
        return ROOM_EPOCH_BAD_EPOCH
    if input_["current_epoch"] < input_["min_accepted_epoch"]:
        return ROOM_EPOCH_BAD_EPOCH
    if input_["message_epoch"] < input_["min_accepted_epoch"]:
        return ROOM_EPOCH_STALE_EPOCH
    if input_["message_epoch"] < input_["current_epoch"]:
        return ROOM_EPOCH_STALE_EPOCH
    if input_["message_epoch"] > input_["current_epoch"]:
        return ROOM_EPOCH_FUTURE_EPOCH
    return ROOM_EPOCH_ACCEPT


def validate_device(input_: dict[str, int]) -> int:
    if input_["device_kind"] < 1 or input_["device_kind"] > 2:
        return ROOM_EPOCH_BAD_DEVICE_KIND
    if input_["device_state"] < 1 or input_["device_state"] > 3:
        return ROOM_EPOCH_BAD_DEVICE_STATE
    if input_["access_kind"] < 0 or input_["access_kind"] > 2:
        return ROOM_EPOCH_BAD_ACCESS_KIND
    if input_["device_state"] == 1:
        if input_["revoked_at_epoch"] != 0:
            return ROOM_EPOCH_BAD_REVOKED_AT_EPOCH
        return ROOM_EPOCH_ACCEPT
    if input_["revoked_at_epoch"] < 1:
        return ROOM_EPOCH_BAD_REVOKED_AT_EPOCH
    if input_["revoked_at_epoch"] > input_["current_epoch"]:
        return ROOM_EPOCH_BAD_REVOKED_AT_EPOCH
    if input_["device_state"] == 2:
        return ROOM_EPOCH_DEVICE_REMOVED
    return ROOM_EPOCH_DEVICE_COMPROMISED


def validate_highsec(input_: dict[str, int]) -> int:
    if input_["room_mode"] == 4 and input_["device_kind"] == 2:
        return ROOM_EPOCH_AI_DEVICE_BLOCKED_ROOM
    if input_["room_mode"] != 3:
        return ROOM_EPOCH_ACCEPT
    if input_["access_kind"] < 0 or input_["access_kind"] > 2:
        return ROOM_EPOCH_ACCEPT
    if input_["device_state"] == 1:
        return ROOM_EPOCH_ACCEPT
    if input_["revoked_at_epoch"] < 1:
        return ROOM_EPOCH_ACCEPT
    if input_["revoked_at_epoch"] > input_["current_epoch"]:
        return ROOM_EPOCH_ACCEPT
    if input_["current_epoch"] <= input_["revoked_at_epoch"]:
        return ROOM_EPOCH_HIGHSEC_EPOCH_ROTATION_REQUIRED
    return ROOM_EPOCH_ACCEPT


def first_reject(epoch_reason: int, device_reason: int, highsec_reason: int) -> int:
    if epoch_reason != ROOM_EPOCH_ACCEPT:
        return epoch_reason
    if highsec_reason == ROOM_EPOCH_HIGHSEC_EPOCH_ROTATION_REQUIRED:
        return highsec_reason
    if device_reason != ROOM_EPOCH_ACCEPT:
        return device_reason
    return highsec_reason


def audit_class_for_reason(reason_code: int) -> int:
    if reason_code == ROOM_EPOCH_ACCEPT:
        return ROOM_EPOCH_AUDIT_ACCEPTED_ROOM_EPOCH
    if reason_code in (ROOM_EPOCH_STALE_EPOCH, ROOM_EPOCH_FUTURE_EPOCH):
        return ROOM_EPOCH_AUDIT_ROOM_EPOCH_REPLAY_REJECT
    if reason_code in (ROOM_EPOCH_DEVICE_REMOVED, ROOM_EPOCH_DEVICE_COMPROMISED):
        return ROOM_EPOCH_AUDIT_ROOM_DEVICE_MEMBERSHIP_REJECT
    if reason_code == ROOM_EPOCH_HIGHSEC_EPOCH_ROTATION_REQUIRED:
        return ROOM_EPOCH_AUDIT_ROOM_HIGHSEC_ROTATION_REJECT
    if reason_code == ROOM_EPOCH_AI_DEVICE_BLOCKED_ROOM:
        return ROOM_EPOCH_AUDIT_ROOM_AI_POLICY_REJECT
    return ROOM_EPOCH_AUDIT_ROOM_EPOCH_POLICY_REJECT


def validate(input_: dict[str, int]) -> int:
    return first_reject(
        validate_epoch(input_),
        validate_device(input_),
        validate_highsec(input_),
    )


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "room_epoch"
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

    if count != 19:
        print(f"expected 19 vectors, checked {count}")
        return 1

    print("room epoch vectors: OK (19 checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
