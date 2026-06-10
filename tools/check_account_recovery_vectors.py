#!/usr/bin/env python3
"""Mercury account_recovery policy mirror + vector checker (no external deps).

Mirrors mercury-core's `evaluate_account_recovery(AccountRecoveryInput) -> AccountRecoveryDecision`
(core/rust/mercury-core/src/lib.rs:21949): the account-recovery decider, a 12-gate priority chain
producing 13 reasons. The rich AccountRecoveryInput (15 fields incl threshold quorum counters and
entropy/length integers) reduces to a scalar set; because no single Helix fn takes them all (the
<=6-int-param codegen cap) it is STAGED into four stage-reason fns + a first-non-zero composer + an
output derivation (the policy_pipeline / bootstrap pattern).

Scalar reduction of the real input (faithful at the decision boundaries the real fn tests):
  recovery_requested        <- bool
  method_code               <- AccountRecoveryMethod::code() 1 HighEntropyRecoveryKey / 2 ThresholdRecovery
                               / 3 HardwareDeviceTransfer / 4 LowEntropyPin / 5 CloudBackup (passed through
                               to the decision's method field)
  high_security_account     <- bool (sets the entropy threshold 192 vs 128 AND gate 11)
  entropy_ok                <- (recovery_key_entropy_bits >= (high_security ? 192 : 128))
  digest_ok                 <- (recovery_key_digest_len == 32)
  threshold_ok              <- NOT(threshold_required < 2 || threshold_shares < threshold_required ||
                               threshold_approvals < threshold_required)   [only consulted for ThresholdRecovery]
  device_approval_present   <- bool   [only consulted for HardwareDeviceTransfer]
  server_authenticated      <- bool
  server_rate_limited       <- bool
  backup_encrypted          <- bool
  plaintext_backup_ok       <- (plaintext_backup_fields == 0)
  rotates_device_secret     <- bool
  audit_ok                  <- (audit_digest_len == 32)

Stages (verbatim priority): A pre/key (recovery/method-low-entropy/entropy/digest -> 1/2/3/4) ->
B method-specific (ThresholdRecovery quorum -> 5, HardwareDeviceTransfer device -> 6) -> C server/backup
(7/8/9/10) -> D rotation/audit (11/12). reason = first non-zero of (A,B,C,D). Output: accepted /
can_start_recovery / can_restore_device_secret on reason 0; requires_user_action {2,3,4,5,6,12};
requires_server_setup {7,8}; requires_key_rotation {11}; forbids_low_entropy_recovery and
forbids_plaintext_backup always true; plaintext_bytes_exposed always false; method passes through.
core/rust/mercury-core/tests/account_recovery_vectors.rs reconstructs the REAL AccountRecoveryInput
and pins to evaluate_account_recovery. Exhaustive agreement: the FU-4 exhaustive differential.
"""

from __future__ import annotations

import json
from pathlib import Path

ACCOUNT_RECOVERY_REASONS = {
    0: "ACCEPTED",
    1: "RECOVERY_NOT_REQUESTED",
    2: "LOW_ENTROPY_PIN_FORBIDDEN",
    3: "RECOVERY_KEY_TOO_WEAK",
    4: "RECOVERY_KEY_DIGEST_MISSING",
    5: "THRESHOLD_QUORUM_INSUFFICIENT",
    6: "DEVICE_APPROVAL_MISSING",
    7: "SERVER_AUTHENTICATION_MISSING",
    8: "SERVER_RATE_LIMIT_MISSING",
    9: "BACKUP_ENCRYPTION_MISSING",
    10: "PLAINTEXT_BACKUP_FORBIDDEN",
    11: "KEY_ROTATION_MISSING",
    12: "AUDIT_DIGEST_MISSING",
}

# AccountRecoveryMethod::code() — passed through to the decision's `method` field.
METHOD_HIGH_ENTROPY_RECOVERY_KEY = 1
METHOD_THRESHOLD_RECOVERY = 2
METHOD_HARDWARE_DEVICE_TRANSFER = 3
METHOD_LOW_ENTROPY_PIN = 4
METHOD_CLOUD_BACKUP = 5

INPUT_FIELDS = [
    "recovery_requested",
    "method_code",
    "high_security_account",
    "entropy_ok",
    "digest_ok",
    "threshold_ok",
    "device_approval_present",
    "server_authenticated",
    "server_rate_limited",
    "backup_encrypted",
    "plaintext_backup_ok",
    "rotates_device_secret",
    "audit_ok",
]

DECISION_FIELDS = [
    "accepted",
    "can_start_recovery",
    "can_restore_device_secret",
    "requires_user_action",
    "requires_server_setup",
    "requires_key_rotation",
    "forbids_low_entropy_recovery",
    "forbids_plaintext_backup",
    "plaintext_bytes_exposed",
    "method_code",
    "reason_code",
    "reason_label",
]

STAGE_FIELDS = ["stage_a_reason", "stage_b_reason", "stage_c_reason", "stage_d_reason"]

# pack: reason_code (high) then the 9 decision bools in AccountRecoveryDecision field order
# (method is a passthrough enum, asserted separately, not packed).
BOOL_FIELDS = [
    "accepted",
    "can_start_recovery",
    "can_restore_device_secret",
    "requires_user_action",
    "requires_server_setup",
    "requires_key_rotation",
    "forbids_low_entropy_recovery",
    "forbids_plaintext_backup",
    "plaintext_bytes_exposed",
]

RUA_SET = {2, 3, 4, 5, 6, 12}
SERVER_SET = {7, 8}
ROTATION_SET = {11}


def stage_a(i):
    if i["recovery_requested"] == 0:
        return 1
    if i["method_code"] == METHOD_LOW_ENTROPY_PIN:
        return 2
    if i["entropy_ok"] == 0:
        return 3
    if i["digest_ok"] == 0:
        return 4
    return 0


def stage_b(i):
    if i["method_code"] == METHOD_THRESHOLD_RECOVERY and i["threshold_ok"] == 0:
        return 5
    if i["method_code"] == METHOD_HARDWARE_DEVICE_TRANSFER and i["device_approval_present"] == 0:
        return 6
    return 0


def stage_c(i):
    if i["server_authenticated"] == 0:
        return 7
    if i["server_rate_limited"] == 0:
        return 8
    if i["backup_encrypted"] == 0:
        return 9
    if i["plaintext_backup_ok"] == 0:
        return 10
    return 0


def stage_d(i):
    if i["high_security_account"] == 1 and i["rotates_device_secret"] == 0:
        return 11
    if i["audit_ok"] == 0:
        return 12
    return 0


def compose(a, b, c, d):
    for r in (a, b, c, d):
        if r != 0:
            return r
    return 0


def decide(i):
    a, b, c, d = stage_a(i), stage_b(i), stage_c(i), stage_d(i)
    reason = compose(a, b, c, d)
    acc = reason == 0
    return {
        "stage_a_reason": a,
        "stage_b_reason": b,
        "stage_c_reason": c,
        "stage_d_reason": d,
        "accepted": acc,
        "can_start_recovery": acc,
        "can_restore_device_secret": acc,
        "requires_user_action": reason in RUA_SET,
        "requires_server_setup": reason in SERVER_SET,
        "requires_key_rotation": reason in ROTATION_SET,
        "forbids_low_entropy_recovery": True,
        "forbids_plaintext_backup": True,
        "plaintext_bytes_exposed": False,
        "method_code": i["method_code"],
        "reason_code": reason,
        "reason_label": ACCOUNT_RECOVERY_REASONS[reason],
    }


def pack(v):
    out = v["reason_code"]
    for name in BOOL_FIELDS:
        out = (out << 1) | (1 if v[name] else 0)
    return out


def _inp(**kw):
    base = {f: 0 for f in INPUT_FIELDS}
    # happy path -> Accepted (HighEntropyRecoveryKey skips the method-specific gates)
    base.update(
        recovery_requested=1,
        method_code=METHOD_HIGH_ENTROPY_RECOVERY_KEY,
        high_security_account=0,
        entropy_ok=1,
        digest_ok=1,
        threshold_ok=1,
        device_approval_present=1,
        server_authenticated=1,
        server_rate_limited=1,
        backup_encrypted=1,
        plaintext_backup_ok=1,
        rotates_device_secret=0,
        audit_ok=1,
    )
    base.update(kw)
    return base


def enumerate_inputs():
    """Representative covering set: all 13 reasons + the method passthrough + the priority
    boundaries. Exhaustive input-space agreement is the FU-4 exhaustive differential."""
    return [
        _inp(),                                                         # 0 accept (HighEntropyRecoveryKey)
        _inp(method_code=METHOD_THRESHOLD_RECOVERY, threshold_ok=1),     # 0 accept (Threshold, quorum ok)
        _inp(method_code=METHOD_CLOUD_BACKUP),                          # 0 accept (CloudBackup passthrough)
        _inp(recovery_requested=0),                                     # 1
        _inp(method_code=METHOD_LOW_ENTROPY_PIN),                       # 2
        _inp(entropy_ok=0),                                            # 3
        _inp(digest_ok=0),                                            # 4
        _inp(method_code=METHOD_THRESHOLD_RECOVERY, threshold_ok=0),    # 5
        _inp(method_code=METHOD_HARDWARE_DEVICE_TRANSFER, device_approval_present=0),  # 6
        _inp(server_authenticated=0),                                  # 7
        _inp(server_rate_limited=0),                                   # 8
        _inp(backup_encrypted=0),                                     # 9
        _inp(plaintext_backup_ok=0),                                  # 10
        _inp(high_security_account=1, rotates_device_secret=0),        # 11
        _inp(high_security_account=1, rotates_device_secret=1, audit_ok=0),  # 12 (rotation ok, audit bad)
        _inp(audit_ok=0),                                             # 12 (non-high-security)
        # threshold_ok ignored unless ThresholdRecovery -> still accept
        _inp(method_code=METHOD_HIGH_ENTROPY_RECOVERY_KEY, threshold_ok=0),  # 0
        # device_approval ignored unless HardwareDeviceTransfer -> still accept
        _inp(method_code=METHOD_HIGH_ENTROPY_RECOVERY_KEY, device_approval_present=0),  # 0
        # priority: recovery_requested beats everything
        _inp(recovery_requested=0, method_code=METHOD_LOW_ENTROPY_PIN, entropy_ok=0, server_authenticated=0),  # 1
        # low-entropy-pin beats entropy/digest
        _inp(method_code=METHOD_LOW_ENTROPY_PIN, entropy_ok=0, digest_ok=0),  # 2
        # stage A beats stage C
        _inp(digest_ok=0, server_authenticated=0),                    # 4
        # stage B beats stage C
        _inp(method_code=METHOD_THRESHOLD_RECOVERY, threshold_ok=0, server_authenticated=0),  # 5
        # stage C beats stage D
        _inp(backup_encrypted=0, high_security_account=1, rotates_device_secret=0),  # 9
        # within D: rotation (11) before audit (12)
        _inp(high_security_account=1, rotates_device_secret=0, audit_ok=0),  # 11
    ]


def main():
    repo_root = Path(__file__).resolve().parents[1]
    vector_dir = repo_root / "vectors" / "account_recovery"
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
        print("error: no account_recovery vectors found")
        return 1
    print(f"account_recovery vectors: OK ({count} checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
