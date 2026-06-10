#!/usr/bin/env python3
"""Mercury receive_decide policy mirror + vector checker (no external deps).

Mirrors mercury-core's `evaluate_client_receive(ClientReceiveInput) -> ClientReceiveDecision`
(core/rust/mercury-core/src/lib.rs:17813) -- the combinator that turns the inbound-receive
sub-decisions into the ClientReceiveReason + decision fields. It is an 11-gate priority chain
over ~12 input scalars, so (the pinned Helix -O1 backend caps at 6 int params per fn) the Helix
mirror is STAGED into three stage-reason functions + a first-non-zero composer + an output
derivation (the same composition pattern as policy_pipeline), each fn <=6 params:

  relay stage (gates 1-4): relay_accepted -> RELAY_DELIVERY_REJECTED(1);
    relay_delivered(next_state==Delivered) -> RELAY_DELIVERY_NOT_DELIVERED(2);
    ack_duplicate -> DUPLICATE_DELIVERY_ACK(4); ack_accepted -> DELIVERY_ACK_REJECTED(3); else 0
  content stage (gates 5-7): digest_ok(len==32) -> BAD_CIPHERTEXT_DIGEST_LENGTH(11);
    plaintext_identity_ok(==0) -> PLAINTEXT_IDENTITY_FORBIDDEN(12);
    replay_state 1 Duplicate->REPLAY_DUPLICATE(8) / 2 Stale->REPLAY_STALE(9) /
    3 FutureGap->ORDERING_GAP(10); else 0
  trust stage (gates 8-10): dt_can_send -> SENDER_DEVICE_TRUST_REJECTED(5);
    message_policy_accepted -> MESSAGE_POLICY_REJECTED(6);
    ciphertext_sealing_accepted -> LOCAL_STORE_SEALING_REJECTED(7); else 0
  reason = first non-zero of (relay, content, trust)  [the verbatim priority order]
  requires_client_retry: reasons 1,2,10 -> true; reason 3 -> ack_requires_client_retry (derived);
    else false. requires_user_action: reasons 5,6,7 -> true; reason 0 -> dt_requires_user_action
    (derived); else false. accept => accepted/can_decrypt/can_persist_ciphertext/can_expose_to_ui
    all true; every reject => all false (client_receive_reject).

core/rust/mercury-core/tests/receive_decide_vectors.rs reconstructs the REAL sub-decisions from
these scalars + calls the REAL evaluate_client_receive. replay_state: 0 NewInOrder / 1 Duplicate /
2 Stale / 3 FutureGap. Reason codes/labels = ClientReceiveReason.
"""

from __future__ import annotations

import json
from pathlib import Path

RECEIVE_REASONS = {
    0: "ACCEPTED",
    1: "RELAY_DELIVERY_REJECTED",
    2: "RELAY_DELIVERY_NOT_DELIVERED",
    3: "DELIVERY_ACK_REJECTED",
    4: "DUPLICATE_DELIVERY_ACK",
    5: "SENDER_DEVICE_TRUST_REJECTED",
    6: "MESSAGE_POLICY_REJECTED",
    7: "LOCAL_STORE_SEALING_REJECTED",
    8: "REPLAY_DUPLICATE",
    9: "REPLAY_STALE",
    10: "ORDERING_GAP",
    11: "BAD_CIPHERTEXT_DIGEST_LENGTH",
    12: "PLAINTEXT_IDENTITY_FORBIDDEN",
}

INPUT_FIELDS = [
    "relay_accepted",
    "relay_delivered",
    "ack_duplicate",
    "ack_accepted",
    "ack_requires_client_retry",
    "ciphertext_digest_ok",
    "plaintext_identity_ok",
    "replay_state",
    "dt_can_send",
    "message_policy_accepted",
    "ciphertext_sealing_accepted",
    "dt_requires_user_action",
]

DECISION_FIELDS = [
    "accepted",
    "can_decrypt",
    "can_persist_ciphertext",
    "can_expose_to_ui",
    "requires_client_retry",
    "requires_user_action",
    "reason_code",
    "reason_label",
]

# stage reasons the Helix asserts independently before the composition.
STAGE_FIELDS = ["relay_reason", "content_reason", "trust_reason"]

# pack: reason_code (high) then the 6 bools (accepted, can_decrypt, can_persist, can_expose, rcr, rua).
BOOL_FIELDS = [
    "accepted",
    "can_decrypt",
    "can_persist_ciphertext",
    "can_expose_to_ui",
    "requires_client_retry",
    "requires_user_action",
]


def relay_reason(i):
    if i["relay_accepted"] == 0:
        return 1
    if i["relay_delivered"] == 0:
        return 2
    if i["ack_duplicate"] == 1:
        return 4
    if i["ack_accepted"] == 0:
        return 3
    return 0


def content_reason(i):
    if i["ciphertext_digest_ok"] == 0:
        return 11
    if i["plaintext_identity_ok"] == 0:
        return 12
    if i["replay_state"] == 1:
        return 8
    if i["replay_state"] == 2:
        return 9
    if i["replay_state"] == 3:
        return 10
    return 0


def trust_reason(i):
    if i["dt_can_send"] == 0:
        return 5
    if i["message_policy_accepted"] == 0:
        return 6
    if i["ciphertext_sealing_accepted"] == 0:
        return 7
    return 0


def compose(relay_r, content_r, trust_r):
    if relay_r != 0:
        return relay_r
    if content_r != 0:
        return content_r
    return trust_r


def derive_rcr(reason, ack_rcr):
    if reason in (1, 2, 10):
        return 1
    if reason == 3:
        return ack_rcr
    return 0


def derive_rua(reason, dt_rua):
    if reason in (5, 6, 7):
        return 1
    if reason == 0:
        return dt_rua
    return 0


def decide(i):
    rr = relay_reason(i)
    cr = content_reason(i)
    tr = trust_reason(i)
    reason = compose(rr, cr, tr)
    accepted = reason == 0
    rcr = derive_rcr(reason, i["ack_requires_client_retry"]) == 1
    rua = derive_rua(reason, i["dt_requires_user_action"]) == 1
    return {
        "relay_reason": rr,
        "content_reason": cr,
        "trust_reason": tr,
        "accepted": accepted,
        "can_decrypt": accepted,
        "can_persist_ciphertext": accepted,
        "can_expose_to_ui": accepted,
        "requires_client_retry": rcr,
        "requires_user_action": rua,
        "reason_code": reason,
        "reason_label": RECEIVE_REASONS[reason],
    }


def pack(v):
    out = v["reason_code"]
    for name in BOOL_FIELDS:
        out = (out << 1) | (1 if v[name] else 0)
    return out


def _inp(**kw):
    base = {f: 0 for f in INPUT_FIELDS}
    # the "happy" defaults that reach ACCEPT, then overrides trigger each reason
    base.update(
        relay_accepted=1, relay_delivered=1, ack_duplicate=0, ack_accepted=1,
        ciphertext_digest_ok=1, plaintext_identity_ok=1, replay_state=0,
        dt_can_send=1, message_policy_accepted=1, ciphertext_sealing_accepted=1,
    )
    base.update(kw)
    return base


def enumerate_inputs():
    """Representative covering set: every reachable reason (all 13), both derived cases
    (DELIVERY_ACK_REJECTED rcr, ACCEPTED rua), and the cross-stage priority order. Exhaustive
    input-space agreement is the FU-4 exhaustive differential."""
    return [
        # reason 1 RELAY_DELIVERY_REJECTED
        _inp(relay_accepted=0),
        # reason 2 RELAY_DELIVERY_NOT_DELIVERED
        _inp(relay_delivered=0),
        # reason 4 DUPLICATE_DELIVERY_ACK (before ack_accepted check)
        _inp(ack_duplicate=1),
        # reason 3 DELIVERY_ACK_REJECTED, rcr derived 0 / 1
        _inp(ack_accepted=0, ack_requires_client_retry=0),
        _inp(ack_accepted=0, ack_requires_client_retry=1),
        # reason 11 BAD_CIPHERTEXT_DIGEST_LENGTH
        _inp(ciphertext_digest_ok=0),
        # reason 12 PLAINTEXT_IDENTITY_FORBIDDEN
        _inp(plaintext_identity_ok=0),
        # reason 8/9/10 replay
        _inp(replay_state=1),
        _inp(replay_state=2),
        _inp(replay_state=3),
        # reason 5/6/7 trust stage
        _inp(dt_can_send=0),
        _inp(message_policy_accepted=0),
        _inp(ciphertext_sealing_accepted=0),
        # reason 0 ACCEPTED, rua derived 0 / 1
        _inp(),
        _inp(dt_requires_user_action=1),
        # priority: relay beats content beats trust
        _inp(relay_accepted=0, ciphertext_digest_ok=0, dt_can_send=0),  # -> 1
        _inp(ciphertext_digest_ok=0, dt_can_send=0),                    # -> 11
        _inp(replay_state=3, dt_can_send=0),                            # -> 10 (content beats trust)
        # priority within relay: delivered before duplicate before ack
        _inp(relay_delivered=0, ack_duplicate=1, ack_accepted=0),      # -> 2
    ]


def main():
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "receive_decide"
    count = 0
    for path in sorted(vector_dir.glob("*.json")):
        vector = json.loads(path.read_text(encoding="utf-8"))
        got = decide(vector["input"])
        expected = vector["expected"]
        for field in DECISION_FIELDS + STAGE_FIELDS:
            if got[field] != expected.get(field):
                print(f"{path.name}: field '{field}' mismatch, got {got[field]!r}, "
                      f"expected {expected.get(field)!r}")
                return 1
        if "expected_pack" in vector and pack(got) != vector["expected_pack"]:
            print(f"{path.name}: pack mismatch, got {pack(got)}, expected {vector['expected_pack']}")
            return 1
        count += 1
    if count == 0:
        print("error: no receive_decide vectors found")
        return 1
    print(f"receive_decide vectors: OK ({count} checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
