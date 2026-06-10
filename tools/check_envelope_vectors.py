#!/usr/bin/env python3
"""Check Mercury Phase 1 envelope vectors without external dependencies."""

from __future__ import annotations

import json
from pathlib import Path


ACCEPT = 0
BAD_VERSION = 1
UNSUPPORTED_SUITE = 2
DOWNGRADED_SUITE = 3
BAD_CONVERSATION_ID_LEN = 4
BAD_SENDER_ACCOUNT_ID_LEN = 5
BAD_SENDER_DEVICE_ID_LEN = 6
BAD_EPOCH = 7
BAD_SEQUENCE = 8
BAD_MESSAGE_KIND = 9
PAYLOAD_TOO_LARGE = 10
UNKNOWN_CRITICAL_FLAG = 11
RESERVED_AI_KIND = 12

AUDIT_ACCEPTED_MESSAGE = 1
AUDIT_POLICY_REJECT = 2
AUDIT_DOWNGRADE_ATTEMPT = 3
AUDIT_SIZE_REJECT = 4


def supported_suite(suite_id: int) -> bool:
    return suite_id in (257, 513)


def valid_id_len(id_len: int) -> bool:
    return 8 <= id_len <= 128


def validate_identity(input_: dict[str, int]) -> int:
    if input_["version"] != 1:
        return BAD_VERSION
    if not supported_suite(input_["suite_id"]):
        return UNSUPPORTED_SUITE
    if input_["suite_id"] < input_["min_suite_id"]:
        return DOWNGRADED_SUITE
    if not valid_id_len(input_["conversation_id_len"]):
        return BAD_CONVERSATION_ID_LEN
    if not valid_id_len(input_["sender_account_id_len"]):
        return BAD_SENDER_ACCOUNT_ID_LEN
    if not valid_id_len(input_["sender_device_id_len"]):
        return BAD_SENDER_DEVICE_ID_LEN
    return ACCEPT


def validate_order(input_: dict[str, int]) -> int:
    if input_["epoch"] != input_["expected_epoch"]:
        return BAD_EPOCH
    if input_["sequence"] != input_["expected_sequence"]:
        return BAD_SEQUENCE
    return ACCEPT


def validate_content(input_: dict[str, int]) -> int:
    message_kind = input_["message_kind"]
    if message_kind not in (1, 2, 3):
        return BAD_MESSAGE_KIND
    if message_kind == 3:
        return RESERVED_AI_KIND
    if input_["payload_len"] < 0 or input_["max_payload_len"] < 0:
        return PAYLOAD_TOO_LARGE
    if input_["payload_len"] > input_["max_payload_len"]:
        return PAYLOAD_TOO_LARGE
    if input_["critical_flags"] < 0 or input_["critical_flags"] > 3:
        return UNKNOWN_CRITICAL_FLAG
    return ACCEPT


def first_reject(identity_reason: int, order_reason: int, content_reason: int) -> int:
    if identity_reason != ACCEPT:
        return identity_reason
    if order_reason != ACCEPT:
        return order_reason
    return content_reason


def audit_class_for_reason(reason_code: int) -> int:
    if reason_code == ACCEPT:
        return AUDIT_ACCEPTED_MESSAGE
    if reason_code == DOWNGRADED_SUITE:
        return AUDIT_DOWNGRADE_ATTEMPT
    if reason_code == PAYLOAD_TOO_LARGE:
        return AUDIT_SIZE_REJECT
    return AUDIT_POLICY_REJECT


def validate(input_: dict[str, int]) -> int:
    return first_reject(
        validate_identity(input_),
        validate_order(input_),
        validate_content(input_),
    )


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "envelope"
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

    if count != 12:
        print(f"expected 12 vectors, checked {count}")
        return 1

    print("envelope vectors: OK (12 checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

