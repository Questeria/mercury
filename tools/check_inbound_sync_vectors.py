#!/usr/bin/env python3
"""Mercury inbound_sync policy mirror + vector checker (no external deps).

Mirrors mercury-core's `evaluate_inbound_sync(InboundSyncInput) -> InboundSyncDecision`
(core/rust/mercury-core/src/lib.rs:30212): the client-sync decider that turns the bootstrap
decision + transport source state + poll/route shape into an InboundSyncReason + decision fields.
A 7-gate priority chain over a small scalar reduction; the reason is computable in exactly 6 int
params (at the pinned Helix -O1 cap), so NO staging is needed -- one reason fn + an output
derivation (the same shape as the accept/reject deciders).

Scalar reduction of the real InboundSyncInput:
  plaintext_preview_ok     <- (plaintext_notification_preview_len == 0)
  bootstrap_can_start_sync <- bootstrap.can_start_sync   (the real ClientBootstrapDecision field)
  bootstrap_rua            <- bootstrap.requires_user_action (derived into reason 2's output only)
  source_state             <- InboundSyncSourceState: 0 Ready / 1 Offline / 2 AuthRejected / 3 BackoffRequired
  poll_batch_ok            <- (1 <= poll_batch_limit <= 100)
  pending_delivery         <- bool
  route_id_ok              <- (route_id_len == 32)

Priority (verbatim): plaintext-preview(8) -> bootstrap-blocked(2) -> source_state(3/4/5) ->
poll-batch(6) -> route-id when pending(7) -> not-pending Idle(1) -> DeliveryReady(0).
Output: accepted/can_poll_relay/can_update_replay_checkpoint on reasons {0,1}; can_run_receive_session
only on 0; requires_network_retry on {3,5}; requires_user_action 8 & 4 true, reason 2 -> bootstrap_rua
(derived), else false; plaintext_bytes_exposed always false. pack is lossless.
core/rust/mercury-core/tests/inbound_sync_vectors.rs reconstructs the REAL InboundSyncInput (incl a
real ClientBootstrapDecision carrying can_start_sync/requires_user_action) and pins to the REAL fn.
"""

from __future__ import annotations

import json
from pathlib import Path

INBOUND_SYNC_REASONS = {
    0: "DELIVERY_READY",
    1: "IDLE",
    2: "BOOTSTRAP_BLOCKED",
    3: "TRANSPORT_OFFLINE",
    4: "TRANSPORT_AUTH_REJECTED",
    5: "BACKOFF_REQUIRED",
    6: "BAD_POLL_BATCH_LIMIT",
    7: "BAD_ROUTE_ID_LENGTH",
    8: "PLAINTEXT_NOTIFICATION_PREVIEW_FORBIDDEN",
}

INPUT_FIELDS = [
    "plaintext_preview_ok",
    "bootstrap_can_start_sync",
    "bootstrap_rua",
    "source_state",
    "poll_batch_ok",
    "pending_delivery",
    "route_id_ok",
]

DECISION_FIELDS = [
    "accepted",
    "can_poll_relay",
    "can_run_receive_session",
    "can_update_replay_checkpoint",
    "requires_network_retry",
    "requires_user_action",
    "plaintext_bytes_exposed",
    "reason_code",
    "reason_label",
]

# pack: reason_code (high) then the 7 decision bools in InboundSyncDecision field order.
BOOL_FIELDS = [
    "accepted",
    "can_poll_relay",
    "can_run_receive_session",
    "can_update_replay_checkpoint",
    "requires_network_retry",
    "requires_user_action",
    "plaintext_bytes_exposed",
]

# per-reason output sets.
ACCEPT_SET = {0, 1}          # accepted / can_poll_relay / can_update_replay_checkpoint
NETWORK_RETRY_SET = {3, 5}   # requires_network_retry


def reason_of(i):
    if i["plaintext_preview_ok"] == 0:
        return 8
    if i["bootstrap_can_start_sync"] == 0:
        return 2
    if i["source_state"] == 1:
        return 3
    if i["source_state"] == 2:
        return 4
    if i["source_state"] == 3:
        return 5
    if i["poll_batch_ok"] == 0:
        return 6
    if i["pending_delivery"] == 1 and i["route_id_ok"] == 0:
        return 7
    if i["pending_delivery"] == 0:
        return 1
    return 0


def decide(i):
    reason = reason_of(i)
    accept = reason in ACCEPT_SET
    if reason == 8 or reason == 4:
        rua = True
    elif reason == 2:
        rua = i["bootstrap_rua"] == 1
    else:
        rua = False
    return {
        "accepted": accept,
        "can_poll_relay": accept,
        "can_run_receive_session": reason == 0,
        "can_update_replay_checkpoint": accept,
        "requires_network_retry": reason in NETWORK_RETRY_SET,
        "requires_user_action": rua,
        "plaintext_bytes_exposed": False,
        "reason_code": reason,
        "reason_label": INBOUND_SYNC_REASONS[reason],
    }


def pack(v):
    out = v["reason_code"]
    for name in BOOL_FIELDS:
        out = (out << 1) | (1 if v[name] else 0)
    return out


def _inp(**kw):
    base = {f: 0 for f in INPUT_FIELDS}
    # happy path -> DeliveryReady (reason 0)
    base.update(
        plaintext_preview_ok=1,
        bootstrap_can_start_sync=1,
        source_state=0,
        poll_batch_ok=1,
        pending_delivery=1,
        route_id_ok=1,
    )
    base.update(kw)
    return base


def enumerate_inputs():
    """Representative covering set: all 9 reasons + the derived BOOTSTRAP_BLOCKED rua + the priority
    boundaries. Exhaustive input-space agreement is the FU-4 exhaustive differential."""
    return [
        _inp(),                                              # 0 DeliveryReady
        _inp(pending_delivery=0),                            # 1 Idle
        _inp(bootstrap_can_start_sync=0),                    # 2 (rua=0)
        _inp(bootstrap_can_start_sync=0, bootstrap_rua=1),   # 2 (rua=1 derived)
        _inp(source_state=1),                                # 3 TransportOffline
        _inp(source_state=2),                                # 4 TransportAuthRejected
        _inp(source_state=3),                                # 5 BackoffRequired
        _inp(poll_batch_ok=0),                               # 6 BadPollBatchLimit
        _inp(pending_delivery=1, route_id_ok=0),             # 7 BadRouteIdLength
        # route_id_ok irrelevant when not pending -> Idle, not reason 7
        _inp(pending_delivery=0, route_id_ok=0),             # 1 Idle (route ignored)
        _inp(plaintext_preview_ok=0),                        # 8 PlaintextNotificationPreviewForbidden
        # priority: plaintext-preview beats everything
        _inp(plaintext_preview_ok=0, bootstrap_can_start_sync=0, source_state=2, poll_batch_ok=0),  # 8
        # bootstrap-blocked beats source/poll/route
        _inp(bootstrap_can_start_sync=0, source_state=1, poll_batch_ok=0),                          # 2
        # source_state beats poll/route
        _inp(source_state=2, poll_batch_ok=0, route_id_ok=0),                                       # 4
        # poll-batch beats route
        _inp(poll_batch_ok=0, route_id_ok=0),                                                       # 6
    ]


def main():
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "inbound_sync"
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
        print("error: no inbound_sync vectors found")
        return 1
    print(f"inbound_sync vectors: OK ({count} checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
