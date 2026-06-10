//! Differential anchor for the Helix `account_recovery` policy.
//!
//! For every account_recovery golden vector this reconstructs the REAL AccountRecoveryInput from the
//! 13 scalar inputs (a real AccountRecoveryMethod, plus entropy/digest/threshold/audit integers set
//! at the decision boundaries the real fn tests), calls the REAL `evaluate_account_recovery`, and
//! asserts the produced AccountRecoveryDecision (9 bools + the passed-through method + reason
//! code/label) and its packed projection equal the vector. This pins the 12-gate priority chain, the
//! staged reductions, and the method passthrough to mercury-core. Exhaustive input agreement: the
//! FU-4 exhaustive differential.

use mercury_core::{
    AccountRecoveryDecision, AccountRecoveryInput, AccountRecoveryMethod, evaluate_account_recovery,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    input: InputJson,
    expected: ExpectedDecision,
    expected_pack: i32,
}

#[derive(Debug, Deserialize)]
struct InputJson {
    recovery_requested: i32,
    method_code: i32,
    high_security_account: i32,
    entropy_ok: i32,
    digest_ok: i32,
    threshold_ok: i32,
    device_approval_present: i32,
    server_authenticated: i32,
    server_rate_limited: i32,
    backup_encrypted: i32,
    plaintext_backup_ok: i32,
    rotates_device_secret: i32,
    audit_ok: i32,
}

#[derive(Debug, Deserialize)]
struct ExpectedDecision {
    accepted: bool,
    can_start_recovery: bool,
    can_restore_device_secret: bool,
    requires_user_action: bool,
    requires_server_setup: bool,
    requires_key_rotation: bool,
    forbids_low_entropy_recovery: bool,
    forbids_plaintext_backup: bool,
    plaintext_bytes_exposed: bool,
    method_code: i32,
    reason_code: i32,
    reason_label: String,
}

fn method(code: i32) -> AccountRecoveryMethod {
    match code {
        1 => AccountRecoveryMethod::HighEntropyRecoveryKey,
        2 => AccountRecoveryMethod::ThresholdRecovery,
        3 => AccountRecoveryMethod::HardwareDeviceTransfer,
        4 => AccountRecoveryMethod::LowEntropyPin,
        5 => AccountRecoveryMethod::CloudBackup,
        other => panic!("unknown method_code {other}"),
    }
}

fn real_decision(i: &InputJson) -> AccountRecoveryDecision {
    let high_security = i.high_security_account == 1;
    let min_entropy = if high_security { 192 } else { 128 };
    let input = AccountRecoveryInput {
        recovery_requested: i.recovery_requested == 1,
        method: method(i.method_code),
        high_security_account: high_security,
        // entropy_ok = (bits >= min): set the boundary value either side.
        recovery_key_entropy_bits: if i.entropy_ok == 1 {
            min_entropy
        } else {
            min_entropy - 1
        },
        recovery_key_digest_len: if i.digest_ok == 1 { 32 } else { 31 },
        // threshold_ok = NOT(required<2 || shares<required || approvals<required): keep required=2,
        // approvals=2, and drop shares below required when threshold_ok is false.
        threshold_shares: if i.threshold_ok == 1 { 2 } else { 1 },
        threshold_required: 2,
        threshold_approvals: 2,
        device_approval_present: i.device_approval_present == 1,
        server_authenticated: i.server_authenticated == 1,
        server_rate_limited: i.server_rate_limited == 1,
        backup_encrypted: i.backup_encrypted == 1,
        plaintext_backup_fields: if i.plaintext_backup_ok == 1 { 0 } else { 1 },
        rotates_device_secret: i.rotates_device_secret == 1,
        audit_digest_len: if i.audit_ok == 1 { 32 } else { 31 },
    };
    evaluate_account_recovery(input)
}

/// reason_code (high) then accepted, can_start_recovery, can_restore_device_secret,
/// requires_user_action, requires_server_setup, requires_key_rotation, forbids_low_entropy_recovery,
/// forbids_plaintext_backup, plaintext_bytes_exposed (method is a passthrough, asserted separately).
fn pack(d: &AccountRecoveryDecision) -> i32 {
    let bits = [
        d.accepted,
        d.can_start_recovery,
        d.can_restore_device_secret,
        d.requires_user_action,
        d.requires_server_setup,
        d.requires_key_rotation,
        d.forbids_low_entropy_recovery,
        d.forbids_plaintext_backup,
        d.plaintext_bytes_exposed,
    ];
    let mut out = d.reason.code();
    for b in bits {
        out = (out << 1) | (b as i32);
    }
    out
}

#[test]
fn account_recovery_vectors_match_real_combinator() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../vectors/account_recovery");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    let mut count = 0usize;
    for entry in entries {
        let path = entry.expect("vector entry").path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        let v: Vector = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));

        let got = real_decision(&v.input);
        let e = &v.expected;
        let at = format!("vector {} ({})", v.name, path.display());

        assert_eq!(got.accepted, e.accepted, "accepted {at}");
        assert_eq!(
            got.can_start_recovery, e.can_start_recovery,
            "can_start_recovery {at}"
        );
        assert_eq!(
            got.can_restore_device_secret, e.can_restore_device_secret,
            "can_restore_device_secret {at}"
        );
        assert_eq!(
            got.requires_user_action, e.requires_user_action,
            "requires_user_action {at}"
        );
        assert_eq!(
            got.requires_server_setup, e.requires_server_setup,
            "requires_server_setup {at}"
        );
        assert_eq!(
            got.requires_key_rotation, e.requires_key_rotation,
            "requires_key_rotation {at}"
        );
        assert_eq!(
            got.forbids_low_entropy_recovery, e.forbids_low_entropy_recovery,
            "forbids_low_entropy_recovery {at}"
        );
        assert_eq!(
            got.forbids_plaintext_backup, e.forbids_plaintext_backup,
            "forbids_plaintext_backup {at}"
        );
        assert_eq!(
            got.plaintext_bytes_exposed, e.plaintext_bytes_exposed,
            "plaintext_bytes_exposed {at}"
        );
        assert_eq!(got.method.code(), e.method_code, "method passthrough {at}");
        assert_eq!(got.reason.code(), e.reason_code, "reason_code {at}");
        assert_eq!(
            got.reason.label(),
            e.reason_label.as_str(),
            "reason_label {at}"
        );
        assert_eq!(pack(&got), v.expected_pack, "pack {at}");

        count += 1;
    }

    assert_eq!(count, 24, "expected all 24 account_recovery vectors to run");
}
