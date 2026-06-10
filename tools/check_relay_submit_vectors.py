#!/usr/bin/env python3
"""Check Mercury relay submission vectors without external dependencies."""

from __future__ import annotations

import json
from pathlib import Path


RELAY_SUBMIT_ACCEPT = 0
RELAY_SUBMIT_BAD_VERSION = 1
RELAY_SUBMIT_BAD_SEND_GATE_REASON = 2
RELAY_SUBMIT_SEND_GATE_REJECTED = 3
RELAY_SUBMIT_BAD_ROUTE_ID_LEN = 4
RELAY_SUBMIT_BAD_REPLAY_TOKEN_LEN = 5
RELAY_SUBMIT_BAD_TTL = 6
RELAY_SUBMIT_TTL_TOO_LONG = 7
RELAY_SUBMIT_BAD_CIPHERTEXT_LEN = 8
RELAY_SUBMIT_CIPHERTEXT_TOO_LARGE = 9
RELAY_SUBMIT_BAD_SEALED_HEADER_LEN = 10
RELAY_SUBMIT_SEALED_HEADER_TOO_LARGE = 11
RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN = 12
RELAY_SUBMIT_BAD_PADDING_BUCKET = 13

RELAY_SUBMIT_AUDIT_ACCEPTED_RELAY_SUBMISSION = 1
RELAY_SUBMIT_AUDIT_RELAY_CONTRACT_REJECT = 2
RELAY_SUBMIT_AUDIT_CLIENT_SEND_REJECT = 3
RELAY_SUBMIT_AUDIT_RELAY_METADATA_REJECT = 4
RELAY_SUBMIT_AUDIT_RELAY_RETENTION_REJECT = 5
RELAY_SUBMIT_AUDIT_RELAY_SIZE_REJECT = 6


def validate_send_gate(input_: dict[str, int]) -> int:
    reason = input_["send_gate_reason"]
    if reason < 0 or reason > 5:
        return RELAY_SUBMIT_BAD_SEND_GATE_REASON
    if reason != 0:
        return RELAY_SUBMIT_SEND_GATE_REJECTED
    return RELAY_SUBMIT_ACCEPT


def validate_metadata(input_: dict[str, int]) -> int:
    route_id_len = input_["route_id_len"]
    if route_id_len < 16 or route_id_len > 128:
        return RELAY_SUBMIT_BAD_ROUTE_ID_LEN
    if input_["replay_token_len"] != 32:
        return RELAY_SUBMIT_BAD_REPLAY_TOKEN_LEN
    sealed_header_len = input_["sealed_header_len"]
    if sealed_header_len < 16:
        return RELAY_SUBMIT_BAD_SEALED_HEADER_LEN
    if sealed_header_len > 4096:
        return RELAY_SUBMIT_SEALED_HEADER_TOO_LARGE
    if input_["plaintext_identity_fields"] != 0:
        return RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN
    padding_bucket = input_["padding_bucket"]
    if padding_bucket < 1 or padding_bucket > 8:
        return RELAY_SUBMIT_BAD_PADDING_BUCKET
    return RELAY_SUBMIT_ACCEPT


def validate_lifetime(input_: dict[str, int]) -> int:
    ttl = input_["queue_ttl_s"]
    max_ttl = input_["max_queue_ttl_s"]
    if ttl < 1 or max_ttl < 1:
        return RELAY_SUBMIT_BAD_TTL
    if max_ttl > 604800 or ttl > max_ttl:
        return RELAY_SUBMIT_TTL_TOO_LONG
    return RELAY_SUBMIT_ACCEPT


def validate_ciphertext(input_: dict[str, int]) -> int:
    ciphertext_len = input_["ciphertext_len"]
    max_ciphertext_len = input_["max_ciphertext_len"]
    if ciphertext_len < 1 or max_ciphertext_len < 1:
        return RELAY_SUBMIT_BAD_CIPHERTEXT_LEN
    if max_ciphertext_len > 4194304 or ciphertext_len > max_ciphertext_len:
        return RELAY_SUBMIT_CIPHERTEXT_TOO_LARGE
    return RELAY_SUBMIT_ACCEPT


def first_reject(
    version_reason: int,
    send_gate_reason: int,
    metadata_reason: int,
    lifetime_reason: int,
    ciphertext_reason: int,
) -> int:
    if version_reason != RELAY_SUBMIT_ACCEPT:
        return version_reason
    if send_gate_reason != RELAY_SUBMIT_ACCEPT:
        return send_gate_reason
    if metadata_reason != RELAY_SUBMIT_ACCEPT:
        return metadata_reason
    if lifetime_reason != RELAY_SUBMIT_ACCEPT:
        return lifetime_reason
    return ciphertext_reason


def audit_class_for_reason(reason_code: int) -> int:
    if reason_code == RELAY_SUBMIT_ACCEPT:
        return RELAY_SUBMIT_AUDIT_ACCEPTED_RELAY_SUBMISSION
    if reason_code in (
        RELAY_SUBMIT_BAD_VERSION,
        RELAY_SUBMIT_BAD_SEND_GATE_REASON,
    ):
        return RELAY_SUBMIT_AUDIT_RELAY_CONTRACT_REJECT
    if reason_code == RELAY_SUBMIT_SEND_GATE_REJECTED:
        return RELAY_SUBMIT_AUDIT_CLIENT_SEND_REJECT
    if reason_code in (RELAY_SUBMIT_BAD_TTL, RELAY_SUBMIT_TTL_TOO_LONG):
        return RELAY_SUBMIT_AUDIT_RELAY_RETENTION_REJECT
    if reason_code in (
        RELAY_SUBMIT_BAD_CIPHERTEXT_LEN,
        RELAY_SUBMIT_CIPHERTEXT_TOO_LARGE,
    ):
        return RELAY_SUBMIT_AUDIT_RELAY_SIZE_REJECT
    return RELAY_SUBMIT_AUDIT_RELAY_METADATA_REJECT


def validate(input_: dict[str, int]) -> int:
    return first_reject(
        RELAY_SUBMIT_ACCEPT
        if input_["version"] == 1
        else RELAY_SUBMIT_BAD_VERSION,
        validate_send_gate(input_),
        validate_metadata(input_),
        validate_lifetime(input_),
        validate_ciphertext(input_),
    )


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "relay_submit"
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

    print("relay submit vectors: OK (15 checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
