// Mercury account_recovery policy.
//
// Helix mirrors mercury-core's evaluate_account_recovery(AccountRecoveryInput) ->
// AccountRecoveryDecision (core/rust/mercury-core/src/lib.rs:21949): the account-recovery decider, a
// 12-gate priority chain producing 13 reasons. The rich AccountRecoveryInput (15 fields incl
// threshold-quorum counters and entropy/length integers) is reduced to a scalar set; because no
// single Helix fn takes them all (the <=6-int-param codegen cap) it is STAGED into four stage-reason
// fns + a first-non-zero composer + an output derivation (the policy_pipeline / bootstrap pattern).
// Pure scalar; no crypto/JSON.
//
// Scalar reduction (faithful at the decision boundaries the real fn tests):
//   recovery_requested / high_security_account / device_approval_present / server_authenticated /
//   server_rate_limited / backup_encrypted / rotates_device_secret  <- the matching bools
//   method_code  <- AccountRecoveryMethod::code() 1 HighEntropyRecoveryKey / 2 ThresholdRecovery /
//                   3 HardwareDeviceTransfer / 4 LowEntropyPin / 5 CloudBackup (also the decision's
//                   passed-through method)
//   entropy_ok   <- (recovery_key_entropy_bits >= (high_security ? 192 : 128))
//   digest_ok    <- (recovery_key_digest_len == 32)
//   threshold_ok <- NOT(threshold_required<2 || threshold_shares<threshold_required ||
//                   threshold_approvals<threshold_required)   [only consulted for ThresholdRecovery]
//   plaintext_backup_ok <- (plaintext_backup_fields == 0)
//   audit_ok     <- (audit_digest_len == 32)
//
// Stages (verbatim priority): A pre/key (recovery/low-entropy-pin/entropy/digest -> 1/2/3/4) ->
// B method-specific (ThresholdRecovery quorum -> 5, HardwareDeviceTransfer device -> 6) -> C
// server/backup (7/8/9/10) -> D rotation/audit (11/12). reason = first non-zero of (A,B,C,D).
// Output: accepted/can_start_recovery/can_restore_device_secret on reason 0; requires_user_action
// {2,3,4,5,6,12}; requires_server_setup {7,8}; requires_key_rotation {11}; forbids_low_entropy_recovery
// and forbids_plaintext_backup always true; plaintext_bytes_exposed always false; the method passes
// through unchanged. core/rust/mercury-core/tests/account_recovery_vectors.rs reconstructs a REAL
// AccountRecoveryInput and pins to evaluate_account_recovery.

// stage A: identity / low-entropy-pin / recovery-key entropy + digest (gates 1-4).
@pure
fn mercury_ar_stage_a(
    recovery_requested: i32,
    method_code: i32,
    entropy_ok: i32,
    digest_ok: i32,
) -> i32 {
    if recovery_requested == 0 { 1 } else {
        if method_code == 4 { 2 } else {
            if entropy_ok == 0 { 3 } else {
                if digest_ok == 0 { 4 } else { 0 }
            }
        }
    }
}

// stage B: method-specific quorum/device gates (5 ThresholdRecovery, 6 HardwareDeviceTransfer).
@pure
fn mercury_ar_stage_b(method_code: i32, threshold_ok: i32, device_approval_present: i32) -> i32 {
    if method_code == 2 {
        if threshold_ok == 0 { 5 } else { 0 }
    } else {
        if method_code == 3 {
            if device_approval_present == 0 { 6 } else { 0 }
        } else { 0 }
    }
}

// stage C: server authentication / rate-limit / backup encryption / plaintext-backup (gates 7-10).
@pure
fn mercury_ar_stage_c(
    server_authenticated: i32,
    server_rate_limited: i32,
    backup_encrypted: i32,
    plaintext_backup_ok: i32,
) -> i32 {
    if server_authenticated == 0 { 7 } else {
        if server_rate_limited == 0 { 8 } else {
            if backup_encrypted == 0 { 9 } else {
                if plaintext_backup_ok == 0 { 10 } else { 0 }
            }
        }
    }
}

// stage D: high-security key rotation (11) then audit digest (12). Gate 11 fires only for a
// high-security account that does not rotate; the audit gate is checked after in all other cases.
@pure
fn mercury_ar_stage_d(high_security_account: i32, rotates_device_secret: i32, audit_ok: i32) -> i32 {
    if high_security_account == 1 {
        if rotates_device_secret == 0 { 11 } else {
            if audit_ok == 0 { 12 } else { 0 }
        }
    } else {
        if audit_ok == 0 { 12 } else { 0 }
    }
}

// first non-zero of (A, B, C, D) -- the verbatim priority order.
@pure
fn mercury_ar_reason(a: i32, b: i32, c: i32, d: i32) -> i32 {
    if a == 0 {
        if b == 0 {
            if c == 0 { d } else { c }
        } else { b }
    } else { a }
}

// accepted (== can_start_recovery == can_restore_device_secret) = reason is ACCEPTED.
@pure
fn mercury_ar_acc(reason: i32) -> i32 {
    if reason == 0 { 1 } else { 0 }
}

// requires_user_action: {2,3,4,5,6,12}.
@pure
fn mercury_ar_rua(reason: i32) -> i32 {
    if reason == 2 { 1 } else {
        if reason == 3 { 1 } else {
            if reason == 4 { 1 } else {
                if reason == 5 { 1 } else {
                    if reason == 6 { 1 } else {
                        if reason == 12 { 1 } else { 0 }
                    }
                }
            }
        }
    }
}

// requires_server_setup: {7,8}.
@pure
fn mercury_ar_server(reason: i32) -> i32 {
    if reason == 7 { 1 } else {
        if reason == 8 { 1 } else { 0 }
    }
}

// requires_key_rotation: {11}.
@pure
fn mercury_ar_rotation(reason: i32) -> i32 {
    if reason == 11 { 1 } else { 0 }
}

// lossless pack of the AccountRecoveryDecision: reason in the high bits, then the 9 decision bools in
// field order (accepted, can_start_recovery, can_restore_device_secret, requires_user_action,
// requires_server_setup, requires_key_rotation, forbids_low_entropy_recovery, forbids_plaintext_backup,
// plaintext_bytes_exposed). forbids_low_entropy_recovery and forbids_plaintext_backup are always 1
// (the constant +4 +2); plaintext_bytes_exposed always 0. The method is a passthrough, asserted
// separately. Mirrors check_account_recovery_vectors.py::pack.
@pure
fn mercury_ar_pack(reason: i32) -> i32 {
    reason * 512
    + mercury_ar_acc(reason) * 256
    + mercury_ar_acc(reason) * 128
    + mercury_ar_acc(reason) * 64
    + mercury_ar_rua(reason) * 32
    + mercury_ar_server(reason) * 16
    + mercury_ar_rotation(reason) * 8
    + 4
    + 2
}

@pure
fn expect(actual: i32, expected: i32) -> i32 {
    if actual == expected { 0 } else { 1 }
}

fn main() -> i32 {
    let mut failed: i32 = 0;

    // GENERATED from vectors/account_recovery/*.json by
    // tools/gen_account_recovery_vectors.py --write. Do not edit by hand.

    // 00_accepted__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 0), 0);
    failed = failed + expect(mercury_ar_pack(0), 454);

    // 00_accepted__2
    failed = failed + expect(mercury_ar_stage_a(1, 2, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(2, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 0), 0);
    failed = failed + expect(mercury_ar_pack(0), 454);

    // 00_accepted__3
    failed = failed + expect(mercury_ar_stage_a(1, 5, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(5, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 0), 0);
    failed = failed + expect(mercury_ar_pack(0), 454);

    // 01_recovery_not_requested__1
    failed = failed + expect(mercury_ar_stage_a(0, 1, 1, 1), 1);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(1, 0, 0, 0), 1);
    failed = failed + expect(mercury_ar_pack(1), 518);

    // 02_low_entropy_pin_forbidden__1
    failed = failed + expect(mercury_ar_stage_a(1, 4, 1, 1), 2);
    failed = failed + expect(mercury_ar_stage_b(4, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(2, 0, 0, 0), 2);
    failed = failed + expect(mercury_ar_pack(2), 1062);

    // 03_recovery_key_too_weak__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 0, 1), 3);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(3, 0, 0, 0), 3);
    failed = failed + expect(mercury_ar_pack(3), 1574);

    // 04_recovery_key_digest_missing__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 0), 4);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(4, 0, 0, 0), 4);
    failed = failed + expect(mercury_ar_pack(4), 2086);

    // 05_threshold_quorum_insufficient__1
    failed = failed + expect(mercury_ar_stage_a(1, 2, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(2, 0, 1), 5);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 5, 0, 0), 5);
    failed = failed + expect(mercury_ar_pack(5), 2598);

    // 06_device_approval_missing__1
    failed = failed + expect(mercury_ar_stage_a(1, 3, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(3, 1, 0), 6);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 6, 0, 0), 6);
    failed = failed + expect(mercury_ar_pack(6), 3110);

    // 07_server_authentication_missing__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(0, 1, 1, 1), 7);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 7, 0), 7);
    failed = failed + expect(mercury_ar_pack(7), 3606);

    // 08_server_rate_limit_missing__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 0, 1, 1), 8);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 8, 0), 8);
    failed = failed + expect(mercury_ar_pack(8), 4118);

    // 09_backup_encryption_missing__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 0, 1), 9);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 9, 0), 9);
    failed = failed + expect(mercury_ar_pack(9), 4614);

    // 10_plaintext_backup_forbidden__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 0), 10);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 10, 0), 10);
    failed = failed + expect(mercury_ar_pack(10), 5126);

    // 11_key_rotation_missing__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(1, 0, 1), 11);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 11), 11);
    failed = failed + expect(mercury_ar_pack(11), 5646);

    // 12_audit_digest_missing__1
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(1, 1, 0), 12);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 12), 12);
    failed = failed + expect(mercury_ar_pack(12), 6182);

    // 12_audit_digest_missing__2
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 0), 12);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 12), 12);
    failed = failed + expect(mercury_ar_pack(12), 6182);

    // 00_accepted__4
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 0, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 0), 0);
    failed = failed + expect(mercury_ar_pack(0), 454);

    // 00_accepted__5
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 0), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 0), 0);
    failed = failed + expect(mercury_ar_pack(0), 454);

    // 01_recovery_not_requested__2
    failed = failed + expect(mercury_ar_stage_a(0, 4, 0, 1), 1);
    failed = failed + expect(mercury_ar_stage_b(4, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(0, 1, 1, 1), 7);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(1, 0, 7, 0), 1);
    failed = failed + expect(mercury_ar_pack(1), 518);

    // 02_low_entropy_pin_forbidden__2
    failed = failed + expect(mercury_ar_stage_a(1, 4, 0, 0), 2);
    failed = failed + expect(mercury_ar_stage_b(4, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(2, 0, 0, 0), 2);
    failed = failed + expect(mercury_ar_pack(2), 1062);

    // 04_recovery_key_digest_missing__2
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 0), 4);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(0, 1, 1, 1), 7);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(4, 0, 7, 0), 4);
    failed = failed + expect(mercury_ar_pack(4), 2086);

    // 05_threshold_quorum_insufficient__2
    failed = failed + expect(mercury_ar_stage_a(1, 2, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(2, 0, 1), 5);
    failed = failed + expect(mercury_ar_stage_c(0, 1, 1, 1), 7);
    failed = failed + expect(mercury_ar_stage_d(0, 0, 1), 0);
    failed = failed + expect(mercury_ar_reason(0, 5, 7, 0), 5);
    failed = failed + expect(mercury_ar_pack(5), 2598);

    // 09_backup_encryption_missing__2
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 0, 1), 9);
    failed = failed + expect(mercury_ar_stage_d(1, 0, 1), 11);
    failed = failed + expect(mercury_ar_reason(0, 0, 9, 11), 9);
    failed = failed + expect(mercury_ar_pack(9), 4614);

    // 11_key_rotation_missing__2
    failed = failed + expect(mercury_ar_stage_a(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_b(1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_c(1, 1, 1, 1), 0);
    failed = failed + expect(mercury_ar_stage_d(1, 0, 0), 11);
    failed = failed + expect(mercury_ar_reason(0, 0, 0, 11), 11);
    failed = failed + expect(mercury_ar_pack(11), 5646);

    if failed == 0 { 42 } else { failed }
}
