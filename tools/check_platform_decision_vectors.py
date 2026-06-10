#!/usr/bin/env python3
"""Mercury platform_decision policy mirror + vector checker (no external deps).

Mirrors mercury-core's `PlatformDecisionView` decision boundary: the FINAL 13-field
view produced for every (source, reason) pair, folding the reject-helper /
accepted-path field values together with the `from_bootstrap` / `from_outbound_send`
/ `from_client_receive` constructors.

This is a deterministic, pure, scalar mirror. Like every Helix policy mirror it does
NOT parse JSON envelopes, inspect ciphertext, verify signatures, or implement any
cryptography -- it only reproduces the platform-binding *capability projection* of a
decision that mercury-core already made.

Source of truth (verbatim-extracted; cite when auditing):
  core/rust/mercury-core/src/lib.rs
    struct PlatformDecisionView ............................ 30292-30307 (13 fields, order below)
    from_bootstrap / from_outbound_send / from_client_receive 30309-30362
    ClientBootstrapReason enum/code/label ................. 21417-21492 (21 reasons, 0..20)
    OutboundSendReason  enum/code/label .................. 10771-10801 (6 reasons, 0..5)
    ClientReceiveReason enum/code/label .................. 17708-17759 (13 reasons, 0..12)
    client_bootstrap_reject / accepted path .............. 21770-21787 / 21758-21767
    outbound_send_reject / accepted (+derived RUA) ....... 10885-10896 / 10868-10883
    client_receive_reject / accepted (+derived RUA) ...... 17882-17896 / 17871-17879
  Ground-truth Rust tests: mercury-core/tests/platform_binding_view.rs,
                           mercury-core/tests/platform_decision_vectors.rs (added with this policy).

Four booleans are NOT constant per reason -- they are copied from a deeper decision
input. A vector supplies that one bit as the single int `derived` (0/1); which view
field it feeds is fixed by DERIVED_AT. Every other field is a literal constant fixed
by (source, reason).
"""

from __future__ import annotations

import json
from pathlib import Path

# ---- sources -------------------------------------------------------------------

SRC_BOOTSTRAP = 0
SRC_OUTBOUND = 1
SRC_RECEIVE = 2

SOURCE_LABEL = {
    SRC_BOOTSTRAP: "client_bootstrap",
    SRC_OUTBOUND: "outbound_send",
    SRC_RECEIVE: "client_receive",
}

# ---- reason registries: code -> label (verbatim from the three enums) ----------

BOOTSTRAP_REASONS = {
    0: "ACCEPTED", 1: "MISSING_ACCOUNT_ID", 2: "MISSING_DEVICE_ID", 3: "RECOVERY_REQUIRED",
    4: "PLAINTEXT_CACHE_FORBIDDEN", 5: "LOCAL_DEVICE_TRUST_REJECTED",
    6: "KEY_TRANSPARENCY_NOT_READY", 7: "KEY_TRANSPARENCY_REJECTED",
    8: "ACCOUNT_SECRET_MISSING", 9: "ACCOUNT_SECRET_CORRUPT", 10: "DEVICE_SECRET_MISSING",
    11: "DEVICE_SECRET_CORRUPT", 12: "ROOM_STATE_MISSING", 13: "ROOM_STATE_CORRUPT",
    14: "PLAINTEXT_SECRET_FORBIDDEN", 15: "REPLAY_CHECKPOINT_MISSING",
    16: "REPLAY_CHECKPOINT_STALE", 17: "REPLAY_GAP_DETECTED", 18: "SYNC_INCOMPLETE",
    19: "SYNC_GAP_DETECTED", 20: "SYNC_FAILED",
}

OUTBOUND_REASONS = {
    0: "ACCEPTED", 1: "DEVICE_TRUST_REJECTED", 2: "ROOM_MEMBERSHIP_TRANSITION_REJECTED",
    3: "ROOM_MEMBERSHIP_EPOCH_NOT_ROTATED", 4: "MESSAGE_POLICY_REJECTED",
    5: "LOCAL_STORE_SEALING_REJECTED",
}

RECEIVE_REASONS = {
    0: "ACCEPTED", 1: "RELAY_DELIVERY_REJECTED", 2: "RELAY_DELIVERY_NOT_DELIVERED",
    3: "DELIVERY_ACK_REJECTED", 4: "DUPLICATE_DELIVERY_ACK", 5: "SENDER_DEVICE_TRUST_REJECTED",
    6: "MESSAGE_POLICY_REJECTED", 7: "LOCAL_STORE_SEALING_REJECTED", 8: "REPLAY_DUPLICATE",
    9: "REPLAY_STALE", 10: "ORDERING_GAP", 11: "BAD_CIPHERTEXT_DIGEST_LENGTH",
    12: "PLAINTEXT_IDENTITY_FORBIDDEN",
}

REASONS_BY_SOURCE = {
    SRC_BOOTSTRAP: BOOTSTRAP_REASONS,
    SRC_OUTBOUND: OUTBOUND_REASONS,
    SRC_RECEIVE: RECEIVE_REASONS,
}

# The 13 view fields, in the struct's declaration/serde order.
VIEW_FIELDS = [
    "source", "accepted", "reason_code", "reason_label", "can_open_message_ui",
    "can_start_sync", "can_send", "can_receive", "can_persist_ciphertext",
    "requires_sync", "requires_recovery", "requires_client_retry", "requires_user_action",
]

# The 10 boolean fields, MSB->LSB, for the lossless `pack` projection (accepted=bit9 ..
# requires_user_action=bit0). The manifest pins this identical order for every mirror.
BOOL_FIELDS = [
    "accepted", "can_open_message_ui", "can_start_sync", "can_send", "can_receive",
    "can_persist_ciphertext", "requires_sync", "requires_recovery",
    "requires_client_retry", "requires_user_action",
]

# (source, reason_code) that copy ONE boolean from a deeper decision input instead of a
# constant; the vector supplies it as the single int `derived` (0/1). Marker "D" in the tables.
DERIVED_AT = {
    (SRC_BOOTSTRAP, 6): "requires_user_action",   # KEY_TRANSPARENCY_NOT_READY
    (SRC_OUTBOUND, 0): "requires_user_action",     # ACCEPTED
    (SRC_RECEIVE, 0): "requires_user_action",       # ACCEPTED
    (SRC_RECEIVE, 3): "requires_client_retry",      # DELIVERY_ACK_REJECTED
}


def pack(v):
    """Fold the 10 booleans of a view dict into one i32 (accepted = high bit)."""
    out = 0
    for name in BOOL_FIELDS:
        out = (out << 1) | (1 if v[name] else 0)
    return out


def _mk(source, code, label, accepted, *, can_open_message_ui, can_start_sync, can_send,
        can_receive, can_persist_ciphertext, requires_sync, requires_recovery,
        requires_client_retry, requires_user_action):
    return {
        "source": SOURCE_LABEL[source],
        "accepted": accepted,
        "reason_code": code,
        "reason_label": label,
        "can_open_message_ui": can_open_message_ui,
        "can_start_sync": can_start_sync,
        "can_send": can_send,
        "can_receive": can_receive,
        "can_persist_ciphertext": can_persist_ciphertext,
        "requires_sync": requires_sync,
        "requires_recovery": requires_recovery,
        "requires_client_retry": requires_client_retry,
        "requires_user_action": requires_user_action,
    }


# ---- per-source field tables (folded reject-helper/accepted-path + constructor) -
# "D" marks the single derived cell (substituted with the vector's `derived` input).

# bootstrap variable = (can_open_message_ui, can_start_sync, requires_sync, requires_recovery,
#   requires_user_action); constants can_send=F can_receive=F can_persist_ciphertext=F
#   requires_client_retry=F.
_BOOTSTRAP_T = {
    0: (True, True, False, False, False),
    1: (False, False, False, False, True),
    2: (False, False, False, False, True),
    3: (False, False, False, True, True),
    4: (False, False, False, False, True),
    5: (False, False, False, False, True),
    6: (False, True, True, False, "D"),
    7: (False, False, False, False, True),
    8: (False, False, False, True, True),
    9: (False, False, False, True, True),
    10: (False, False, False, True, True),
    11: (False, False, False, True, True),
    12: (False, True, True, False, False),
    13: (False, True, True, False, True),
    14: (False, False, False, False, True),
    15: (False, True, True, False, False),
    16: (False, True, True, False, False),
    17: (False, True, True, False, False),
    18: (False, True, True, False, False),
    19: (False, True, True, False, False),
    20: (False, True, True, False, True),
}

# outbound variable = (can_send, can_persist_ciphertext, requires_user_action); constants
#   can_open_message_ui=F can_start_sync=F can_receive=F requires_sync=F requires_recovery=F
#   requires_client_retry=F.
_OUTBOUND_T = {
    0: (True, True, "D"),
    1: (False, False, True),
    2: (False, False, True),
    3: (False, False, True),
    4: (False, False, True),
    5: (False, False, True),
}

# receive variable = (can_open_message_ui[=can_expose_to_ui], can_receive[=can_decrypt],
#   can_persist_ciphertext, requires_client_retry, requires_user_action); constants
#   can_start_sync=F can_send=F requires_sync=F requires_recovery=F.
_RECEIVE_T = {
    0: (True, True, True, False, "D"),
    1: (False, False, False, True, False),
    2: (False, False, False, True, False),
    3: (False, False, False, "D", False),
    4: (False, False, False, False, False),
    5: (False, False, False, False, True),
    6: (False, False, False, False, True),
    7: (False, False, False, False, True),
    8: (False, False, False, False, False),
    9: (False, False, False, False, False),
    10: (False, False, False, True, False),
    11: (False, False, False, False, False),
    12: (False, False, False, False, False),
}


def _sub(val, derived):
    return bool(derived) if val == "D" else val


def view(inp):
    """Compute the 13-field PlatformDecisionView for input {source, reason_code, derived?}."""
    source = inp["source"]
    code = inp["reason_code"]
    derived = bool(inp.get("derived", 0))

    if source == SRC_BOOTSTRAP:
        if code not in _BOOTSTRAP_T:
            raise ValueError(f"unknown bootstrap reason_code {code}")
        cmui, css, rsy, rrc, rua = _BOOTSTRAP_T[code]
        return _mk(
            source, code, BOOTSTRAP_REASONS[code], code == 0,
            can_open_message_ui=cmui, can_start_sync=css,
            can_send=False, can_receive=False, can_persist_ciphertext=False,
            requires_sync=rsy, requires_recovery=rrc,
            requires_client_retry=False, requires_user_action=_sub(rua, derived),
        )

    if source == SRC_OUTBOUND:
        if code not in _OUTBOUND_T:
            raise ValueError(f"unknown outbound reason_code {code}")
        cs, cpc, rua = _OUTBOUND_T[code]
        return _mk(
            source, code, OUTBOUND_REASONS[code], code == 0,
            can_open_message_ui=False, can_start_sync=False,
            can_send=cs, can_receive=False, can_persist_ciphertext=cpc,
            requires_sync=False, requires_recovery=False,
            requires_client_retry=False, requires_user_action=_sub(rua, derived),
        )

    if source == SRC_RECEIVE:
        if code not in _RECEIVE_T:
            raise ValueError(f"unknown receive reason_code {code}")
        cmui, crx, cpc, rcr, rua = _RECEIVE_T[code]
        return _mk(
            source, code, RECEIVE_REASONS[code], code == 0,
            can_open_message_ui=cmui, can_start_sync=False,
            can_send=False, can_receive=crx, can_persist_ciphertext=cpc,
            requires_sync=False, requires_recovery=False,
            requires_client_retry=_sub(rcr, derived), requires_user_action=_sub(rua, derived),
        )

    raise ValueError(f"unknown source {source}")


def enumerate_inputs():
    """Yield input dicts covering the FULL (source x reason x derived-bit) space.

    Every reason appears once; the four DERIVED_AT reasons appear twice (derived 0 and 1).
    Deterministic + ordered. Used by tools/gen_platform_decision_vectors.py (Tier 2).
    """
    for source in (SRC_BOOTSTRAP, SRC_OUTBOUND, SRC_RECEIVE):
        for code in sorted(REASONS_BY_SOURCE[source]):
            if (source, code) in DERIVED_AT:
                for val in (0, 1):
                    yield {"source": source, "reason_code": code, "derived": val}
            else:
                yield {"source": source, "reason_code": code}


def _check_manifest(repo_root):
    """Cross-check policy/platform_decision_v1.json against this module's registries, so a
    manifest typo (its reason codes / pack order / derived map) is caught in CI."""
    m = json.loads((repo_root / "policy" / "platform_decision_v1.json").read_text("utf-8"))
    want_sources = {label: sid for sid, label in SOURCE_LABEL.items()}
    want_reasons = {
        SOURCE_LABEL[s]: {lbl: code for code, lbl in REASONS_BY_SOURCE[s].items()}
        for s in (SRC_BOOTSTRAP, SRC_OUTBOUND, SRC_RECEIVE)
    }
    want_derived = {
        f"{SOURCE_LABEL[s]}.{REASONS_BY_SOURCE[s][c]}": field
        for (s, c), field in DERIVED_AT.items()
    }
    checks = [
        (m.get("sources") == want_sources, "sources"),
        (m.get("reason_codes") == want_reasons, "reason_codes"),
        (m.get("pack_bit_order_msb_to_lsb") == BOOL_FIELDS, "pack_bit_order_msb_to_lsb"),
        (m.get("view_fields") == VIEW_FIELDS, "view_fields"),
        (m.get("derived_fields") == want_derived, "derived_fields"),
        (m.get("vector_count") == 44, "vector_count"),
    ]
    for ok, name in checks:
        if not ok:
            return f"manifest field '{name}' disagrees with the spec registries"
    return None


def main():
    repo_root = Path(__file__).resolve().parents[1]
    manifest_err = _check_manifest(repo_root)
    if manifest_err is not None:
        print(f"manifest check FAILED: {manifest_err}")
        return 1
    vector_dir = repo_root / "vectors" / "platform_decision"
    count = 0

    for path in sorted(vector_dir.glob("*.json")):
        vector = json.loads(path.read_text(encoding="utf-8"))
        got = view(vector["input"])
        expected = vector["expected"]
        for field in VIEW_FIELDS:
            if got[field] != expected.get(field):
                print(f"{path.name}: field '{field}' mismatch, got {got[field]!r}, "
                      f"expected {expected.get(field)!r}")
                return 1
        if "expected_pack" in vector and pack(got) != vector["expected_pack"]:
            print(f"{path.name}: pack mismatch, got {pack(got)}, "
                  f"expected {vector['expected_pack']}")
            return 1
        count += 1

    if count == 0:
        print("error: no platform_decision vectors found")
        return 1

    print(f"platform_decision vectors: OK ({count} checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
