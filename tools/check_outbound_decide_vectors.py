#!/usr/bin/env python3
"""Mercury outbound_decide policy mirror + vector checker (no external deps).

Mirrors mercury-core's `evaluate_outbound_send(OutboundSendInput) -> OutboundSendDecision`
(core/rust/mercury-core/src/lib.rs:10841) -- the COMBINATOR that turns four sub-decisions into
the outbound reason + the OutboundSendDecision fields. This is the layer BELOW the
platform_decision projection (which maps a reason -> the 13-field view); this maps the
sub-decision inputs -> the reason. Pure scalar: the combinator reads only these fields:

  sender_device_trust.can_send, .requires_user_action     (DeviceTrustDecision, lib.rs:1669)
  room_membership: Stable | Transition{accepted, epoch_rotated, requires_user_action}
  message_policy.accepted                                  (PolicyDecision, :30417)
  ciphertext_sealing.accepted                              (LocalStoreSealingDecision, :10148)

Priority order (verbatim): device-trust -> room transition (accepted, then epoch) -> message
policy -> ciphertext sealing -> accept. On ACCEPT, requires_user_action = device-trust rua OR
(Transition ? transition rua : false); every reject => true; reject => accepted/can_send/
can_persist_ciphertext all false; accept => all true.

REDUCTION (the pinned Helix -O1 backend supports <=6 int params per fn): the room_membership
sub-decision is encoded as one `room_state` scalar that captures its branch outcome --
  0 = Stable
  1 = Transition, accepted && epoch_rotated      (passes)
  2 = Transition, !accepted                       (-> ROOM_MEMBERSHIP_TRANSITION_REJECTED)
  3 = Transition, accepted && !epoch_rotated       (-> ROOM_MEMBERSHIP_EPOCH_NOT_ROTATED)
plus `room_requires_user_action` (only meaningful for state 1). This is a FAITHFUL encoding of
the intra-room priority (accepted before epoch_rotated). core/rust/mercury-core/tests/
outbound_decide_vectors.rs reconstructs a REAL RoomMembershipSendState from each state and calls
the REAL evaluate_outbound_send, confirming the reduction matches reality. Exhaustive input
agreement: the FU-4 exhaustive differential.
"""

from __future__ import annotations

import json
from pathlib import Path

# OutboundSendReason registry (same codes/labels as mercury-core OutboundSendReason).
OUTBOUND_REASONS = {
    0: "ACCEPTED",
    1: "DEVICE_TRUST_REJECTED",
    2: "ROOM_MEMBERSHIP_TRANSITION_REJECTED",
    3: "ROOM_MEMBERSHIP_EPOCH_NOT_ROTATED",
    4: "MESSAGE_POLICY_REJECTED",
    5: "LOCAL_STORE_SEALING_REJECTED",
}

# The 6 scalar inputs (room_state 0..3; the rest are 0/1 bools).
INPUT_FIELDS = [
    "dt_can_send",
    "dt_requires_user_action",
    "room_state",
    "room_requires_user_action",
    "message_policy_accepted",
    "ciphertext_sealing_accepted",
]

DECISION_FIELDS = [
    "accepted",
    "can_send",
    "can_persist_ciphertext",
    "requires_user_action",
    "reason_code",
    "reason_label",
]

# pack: reason_code in the high bits, the 4 decision bools in the low 4 (lossless).
BOOL_FIELDS = ["accepted", "can_send", "can_persist_ciphertext", "requires_user_action"]


def decide(i):
    """Reproduce evaluate_outbound_send over the 6 scalar inputs."""
    rs = i["room_state"]
    if i["dt_can_send"] == 0:
        reason = 1
    elif rs == 2:
        reason = 2
    elif rs == 3:
        reason = 3
    elif i["message_policy_accepted"] == 0:
        reason = 4
    elif i["ciphertext_sealing_accepted"] == 0:
        reason = 5
    else:
        reason = 0

    accepted = reason == 0
    if accepted:
        rua = i["dt_requires_user_action"] == 1 or (
            rs == 1 and i["room_requires_user_action"] == 1
        )
    else:
        rua = True

    return {
        "accepted": accepted,
        "can_send": accepted,
        "can_persist_ciphertext": accepted,
        "requires_user_action": rua,
        "reason_code": reason,
        "reason_label": OUTBOUND_REASONS[reason],
    }


def pack(v):
    out = v["reason_code"]
    for name in BOOL_FIELDS:
        out = (out << 1) | (1 if v[name] else 0)
    return out


def _inp(**kw):
    base = {f: 0 for f in INPUT_FIELDS}
    base.update(kw)
    return base


def enumerate_inputs():
    """Representative covering set: every reason, the priority boundaries, and the full ACCEPT
    requires_user_action matrix (device-trust x room-state x transition-rua). Exhaustive
    input-space agreement is the FU-4 exhaustive differential; this is the golden + Helix exit-42 corpus."""
    return [
        # reason 1 DEVICE_TRUST_REJECTED (highest priority; everything else irrelevant)
        _inp(dt_can_send=0),
        _inp(dt_can_send=0, room_state=1, message_policy_accepted=1, ciphertext_sealing_accepted=1),
        # reason 2 ROOM_MEMBERSHIP_TRANSITION_REJECTED (room reject even with good policy+sealing)
        _inp(dt_can_send=1, room_state=2, message_policy_accepted=1, ciphertext_sealing_accepted=1),
        # reason 3 ROOM_MEMBERSHIP_EPOCH_NOT_ROTATED
        _inp(dt_can_send=1, room_state=3),
        # reason 4 MESSAGE_POLICY_REJECTED -- both Stable and Transition-ok (good sealing, so distinct
        # from the policy-beats-sealing priority case below)
        _inp(dt_can_send=1, room_state=0, message_policy_accepted=0, ciphertext_sealing_accepted=1),
        _inp(dt_can_send=1, room_state=1, message_policy_accepted=0),
        # reason 5 LOCAL_STORE_SEALING_REJECTED
        _inp(dt_can_send=1, room_state=0, message_policy_accepted=1, ciphertext_sealing_accepted=0),
        # reason 0 ACCEPTED -- rua matrix over (dt_rua, room_state in {0,1}, room_rua)
        _inp(dt_can_send=1, room_state=0, message_policy_accepted=1, ciphertext_sealing_accepted=1),  # rua=0
        _inp(
            dt_can_send=1, dt_requires_user_action=1, room_state=0,
            message_policy_accepted=1, ciphertext_sealing_accepted=1,
        ),  # rua=1 via device trust
        _inp(
            dt_can_send=1, room_state=1, room_requires_user_action=0,
            message_policy_accepted=1, ciphertext_sealing_accepted=1,
        ),  # transition-ok, rua=0
        _inp(
            dt_can_send=1, room_state=1, room_requires_user_action=1,
            message_policy_accepted=1, ciphertext_sealing_accepted=1,
        ),  # transition-ok, rua=1 via transition
        _inp(
            dt_can_send=1, dt_requires_user_action=1, room_state=1, room_requires_user_action=1,
            message_policy_accepted=1, ciphertext_sealing_accepted=1,
        ),  # both rua sources
        # Stable ignores room_rua: room_state=0 + room_requires_user_action=1 -> rua stays 0
        _inp(
            dt_can_send=1, room_state=0, room_requires_user_action=1,
            message_policy_accepted=1, ciphertext_sealing_accepted=1,
        ),
        # priority: device-trust beats a bad transition (reason 1 not 2)
        _inp(dt_can_send=0, room_state=2),
        # priority: transition reject beats a bad policy (reason 2 not 4)
        _inp(dt_can_send=1, room_state=2, message_policy_accepted=0),
        # priority: policy reject beats a bad sealing (reason 4 not 5)
        _inp(dt_can_send=1, room_state=0, message_policy_accepted=0, ciphertext_sealing_accepted=0),
    ]


def main():
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "outbound_decide"
    count = 0
    for path in sorted(vector_dir.glob("*.json")):
        vector = json.loads(path.read_text(encoding="utf-8"))
        got = decide(vector["input"])
        expected = vector["expected"]
        for field in DECISION_FIELDS:
            if got[field] != expected.get(field):
                print(f"{path.name}: field '{field}' mismatch, got {got[field]!r}, "
                      f"expected {expected.get(field)!r}")
                return 1
        if "expected_pack" in vector and pack(got) != vector["expected_pack"]:
            print(f"{path.name}: pack mismatch, got {pack(got)}, expected {vector['expected_pack']}")
            return 1
        count += 1
    if count == 0:
        print("error: no outbound_decide vectors found")
        return 1
    print(f"outbound_decide vectors: OK ({count} checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
