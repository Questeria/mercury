#!/usr/bin/env python3
"""Mercury bootstrap_decide policy mirror + vector checker (no external deps).

Mirrors mercury-core's `evaluate_client_bootstrap(ClientBootstrapInput) -> ClientBootstrapDecision`
(core/rust/mercury-core/src/lib.rs:21528) -- the richest decider (21 reasons), a ~13-gate priority
chain over 12 scalar inputs. Because the pinned Helix -O1 backend caps at 6 int params per fn, it
is STAGED (the same composition pattern as policy_pipeline / receive_decide), each fn <=5 params:

  pre  (gates: account_id, device_id, pending_recovery, plaintext_cache, local_trust)
       -> 1 MISSING_ACCOUNT_ID / 2 MISSING_DEVICE_ID / 3 RECOVERY_REQUIRED /
          4 PLAINTEXT_CACHE_FORBIDDEN / 5 LOCAL_DEVICE_TRUST_REJECTED / 0
  kt   (key_transparency.state reduced to kt_code: 0 Consistent / 1 Inconsistent / 2 NotReady
        [= NotChecked|MissingProof|StaleProof]) -> 7 KEY_TRANSPARENCY_REJECTED / 6 KEY_TRANSPARENCY_NOT_READY / 0
  secrets (account_secret, device_secret, room_state, each 0 PresentSealed / 1 Missing / 2 Corrupt /
        3 PlaintextPresent) -> first-failing: account {8,9,14} / device {10,11,14} / room {12,13,14}
  rs   (replay_checkpoint 0 Ready/1 Missing/2 Stale/3 GapDetected -> 15/16/17;
        sync_state 0 CaughtUp/1 Offline/2 CatchingUp/3 GapDetected/4 Failed -> 18/18/19/20)
  reason = first non-zero of (pre, kt, secrets, rs)  [verbatim priority]

Output ClientBootstrapDecision (7 fields): accepted/can_decrypt_local_store/can_open_message_ui =
(reason==0); can_start_sync, requires_sync, requires_recovery per reason; requires_user_action per
reason with reason 6 -> key_transparency.requires_user_action (derived). The per-reason output
bools are IDENTICAL to the platform_decision bootstrap table (this decider's output feeds
from_bootstrap). core/rust/mercury-core/tests/bootstrap_decide_vectors.rs reconstructs the REAL
ClientBootstrapInput from these scalars + calls the REAL evaluate_client_bootstrap. kt_code 2 picks
any NotReady state; faithful because the real body treats NotChecked/MissingProof/StaleProof alike.
"""

from __future__ import annotations

import json
from pathlib import Path

BOOTSTRAP_REASONS = {
    0: "ACCEPTED", 1: "MISSING_ACCOUNT_ID", 2: "MISSING_DEVICE_ID", 3: "RECOVERY_REQUIRED",
    4: "PLAINTEXT_CACHE_FORBIDDEN", 5: "LOCAL_DEVICE_TRUST_REJECTED",
    6: "KEY_TRANSPARENCY_NOT_READY", 7: "KEY_TRANSPARENCY_REJECTED",
    8: "ACCOUNT_SECRET_MISSING", 9: "ACCOUNT_SECRET_CORRUPT", 10: "DEVICE_SECRET_MISSING",
    11: "DEVICE_SECRET_CORRUPT", 12: "ROOM_STATE_MISSING", 13: "ROOM_STATE_CORRUPT",
    14: "PLAINTEXT_SECRET_FORBIDDEN", 15: "REPLAY_CHECKPOINT_MISSING", 16: "REPLAY_CHECKPOINT_STALE",
    17: "REPLAY_GAP_DETECTED", 18: "SYNC_INCOMPLETE", 19: "SYNC_GAP_DETECTED", 20: "SYNC_FAILED",
}

INPUT_FIELDS = [
    "account_id_ok",
    "device_id_ok",
    "pending_recovery",
    "plaintext_cache_ok",
    "local_trust",
    "kt_code",
    "kt_requires_user_action",
    "account_secret",
    "device_secret",
    "room_state",
    "replay_checkpoint",
    "sync_state",
]

DECISION_FIELDS = [
    "accepted",
    "can_start_sync",
    "can_decrypt_local_store",
    "can_open_message_ui",
    "requires_sync",
    "requires_recovery",
    "requires_user_action",
    "reason_code",
    "reason_label",
]

STAGE_FIELDS = ["pre_reason", "kt_reason", "secrets_reason", "rs_reason"]

# pack: reason_code (high) then the 7 decision bools (in this order).
BOOL_FIELDS = [
    "accepted",
    "can_start_sync",
    "can_decrypt_local_store",
    "can_open_message_ui",
    "requires_sync",
    "requires_recovery",
    "requires_user_action",
]

# per-reason output sets (identical to the platform_decision bootstrap projection).
CSS_SET = {0, 6, 12, 13, 15, 16, 17, 18, 19, 20}
RSY_SET = {6, 12, 13, 15, 16, 17, 18, 19, 20}
RRC_SET = {3, 8, 9, 10, 11}
RUA_TRUE_SET = {1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 13, 14, 20}


def pre_reason(i):
    if i["account_id_ok"] == 0:
        return 1
    if i["device_id_ok"] == 0:
        return 2
    if i["pending_recovery"] == 1:
        return 3
    if i["plaintext_cache_ok"] == 0:
        return 4
    if i["local_trust"] == 0:
        return 5
    return 0


def kt_reason(i):
    if i["kt_code"] == 1:
        return 7
    if i["kt_code"] == 2:
        return 6
    return 0


def _secret(state, missing, corrupt):
    return {1: missing, 2: corrupt, 3: 14}.get(state, 0)


def secrets_reason(i):
    for r in (
        _secret(i["account_secret"], 8, 9),
        _secret(i["device_secret"], 10, 11),
        _secret(i["room_state"], 12, 13),
    ):
        if r != 0:
            return r
    return 0


def rs_reason(i):
    replay = {1: 15, 2: 16, 3: 17}.get(i["replay_checkpoint"], 0)
    if replay != 0:
        return replay
    return {1: 18, 2: 18, 3: 19, 4: 20}.get(i["sync_state"], 0)


def compose(pre_r, kt_r, secrets_r, rs_r):
    for r in (pre_r, kt_r, secrets_r, rs_r):
        if r != 0:
            return r
    return 0


def decide(i):
    pre_r, kt_r, sec_r, rs_r = pre_reason(i), kt_reason(i), secrets_reason(i), rs_reason(i)
    reason = compose(pre_r, kt_r, sec_r, rs_r)
    acc = reason == 0
    if reason == 6:
        rua = i["kt_requires_user_action"] == 1
    else:
        rua = reason in RUA_TRUE_SET
    return {
        "pre_reason": pre_r,
        "kt_reason": kt_r,
        "secrets_reason": sec_r,
        "rs_reason": rs_r,
        "accepted": acc,
        "can_start_sync": reason in CSS_SET,
        "can_decrypt_local_store": acc,
        "can_open_message_ui": acc,
        "requires_sync": reason in RSY_SET,
        "requires_recovery": reason in RRC_SET,
        "requires_user_action": rua,
        "reason_code": reason,
        "reason_label": BOOTSTRAP_REASONS[reason],
    }


def pack(v):
    out = v["reason_code"]
    for name in BOOL_FIELDS:
        out = (out << 1) | (1 if v[name] else 0)
    return out


def _inp(**kw):
    base = {f: 0 for f in INPUT_FIELDS}
    base.update(account_id_ok=1, device_id_ok=1, plaintext_cache_ok=1, local_trust=1)  # happy path
    base.update(kw)
    return base


def enumerate_inputs():
    """Representative covering set: all 21 reasons + the derived KT_NOT_READY rua + cross-stage
    and within-stage priority. Exhaustive input-space agreement is the FU-4 exhaustive differential."""
    return [
        _inp(account_id_ok=0),                              # 1
        _inp(device_id_ok=0),                               # 2
        _inp(pending_recovery=1),                           # 3
        _inp(plaintext_cache_ok=0),                         # 4
        _inp(local_trust=0),                                # 5
        _inp(kt_code=2),                                    # 6 (rua=0)
        _inp(kt_code=2, kt_requires_user_action=1),         # 6 (rua=1)
        _inp(kt_code=1),                                    # 7
        _inp(account_secret=1),                             # 8
        _inp(account_secret=2),                             # 9
        _inp(device_secret=1),                              # 10
        _inp(device_secret=2),                              # 11
        _inp(room_state=1),                                 # 12
        _inp(room_state=2),                                 # 13
        _inp(account_secret=3),                             # 14 (via account)
        _inp(device_secret=3),                              # 14 (via device)
        _inp(room_state=3),                                 # 14 (via room)
        _inp(replay_checkpoint=1),                          # 15
        _inp(replay_checkpoint=2),                          # 16
        _inp(replay_checkpoint=3),                          # 17
        _inp(sync_state=1),                                 # 18 (Offline)
        _inp(sync_state=2),                                 # 18 (CatchingUp)
        _inp(sync_state=3),                                 # 19
        _inp(sync_state=4),                                 # 20
        _inp(),                                             # 0 accept
        # priority: pre beats kt beats secrets beats rs
        _inp(account_id_ok=0, kt_code=1, account_secret=1, sync_state=4),  # -> 1
        _inp(kt_code=2, account_secret=1, replay_checkpoint=1),            # -> 6
        _inp(account_secret=1, room_state=1, replay_checkpoint=1),         # -> 8 (account before room)
        _inp(replay_checkpoint=1, sync_state=4),                          # -> 15 (replay before sync)
    ]


def main():
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "bootstrap_decide"
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
        print("error: no bootstrap_decide vectors found")
        return 1
    print(f"bootstrap_decide vectors: OK ({count} checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
