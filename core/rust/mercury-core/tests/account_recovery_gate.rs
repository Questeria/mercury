use std::convert::Infallible;

use mercury_core::{
    AcceptedAccountRecovery, AccountRecoveryInput, AccountRecoveryMethod, AccountRecoveryReason,
    AccountRecoveryServiceAdapter, evaluate_account_recovery, recover_account_with_adapter,
};

#[test]
fn account_recovery_accepts_high_entropy_encrypted_restore() {
    let decision = evaluate_account_recovery(valid_input());

    assert!(decision.accepted);
    assert_eq!(decision.reason, AccountRecoveryReason::Accepted);
    assert_eq!(
        decision.method,
        AccountRecoveryMethod::HighEntropyRecoveryKey
    );
    assert!(decision.can_start_recovery);
    assert!(decision.can_restore_device_secret);
    assert!(!decision.requires_user_action);
    assert!(!decision.requires_server_setup);
    assert!(!decision.requires_key_rotation);
    assert!(decision.forbids_low_entropy_recovery);
    assert!(decision.forbids_plaintext_backup);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn account_recovery_rejects_missing_request_and_low_entropy_pin() {
    let mut not_requested = valid_input();
    not_requested.recovery_requested = false;
    assert_rejected(
        evaluate_account_recovery(not_requested),
        AccountRecoveryReason::RecoveryNotRequested,
    );

    let mut pin = valid_input();
    pin.method = AccountRecoveryMethod::LowEntropyPin;
    pin.recovery_key_entropy_bits = 32;
    let pin_decision = evaluate_account_recovery(pin);
    assert_rejected(pin_decision, AccountRecoveryReason::LowEntropyPinForbidden);
    assert!(pin_decision.requires_user_action);
}

#[test]
fn account_recovery_requires_strong_key_material_and_digest_audit() {
    let mut weak = valid_input();
    weak.recovery_key_entropy_bits = 96;
    let weak_decision = evaluate_account_recovery(weak);
    assert_rejected(weak_decision, AccountRecoveryReason::RecoveryKeyTooWeak);
    assert!(weak_decision.requires_user_action);

    let mut high_security_weak = valid_input();
    high_security_weak.high_security_account = true;
    high_security_weak.recovery_key_entropy_bits = 128;
    high_security_weak.rotates_device_secret = true;
    assert_rejected(
        evaluate_account_recovery(high_security_weak),
        AccountRecoveryReason::RecoveryKeyTooWeak,
    );

    let mut missing_key_digest = valid_input();
    missing_key_digest.recovery_key_digest_len = 0;
    assert_rejected(
        evaluate_account_recovery(missing_key_digest),
        AccountRecoveryReason::RecoveryKeyDigestMissing,
    );

    let mut missing_audit = valid_input();
    missing_audit.audit_digest_len = 0;
    assert_rejected(
        evaluate_account_recovery(missing_audit),
        AccountRecoveryReason::AuditDigestMissing,
    );
}

#[test]
fn account_recovery_requires_threshold_quorum_or_device_approval() {
    let mut threshold = valid_input();
    threshold.method = AccountRecoveryMethod::ThresholdRecovery;
    threshold.threshold_shares = 3;
    threshold.threshold_required = 2;
    threshold.threshold_approvals = 1;
    let threshold_decision = evaluate_account_recovery(threshold);
    assert_rejected(
        threshold_decision,
        AccountRecoveryReason::ThresholdQuorumInsufficient,
    );
    assert!(threshold_decision.requires_user_action);

    let mut approved_threshold = threshold;
    approved_threshold.threshold_approvals = 2;
    assert!(evaluate_account_recovery(approved_threshold).accepted);

    let mut hardware_transfer = valid_input();
    hardware_transfer.method = AccountRecoveryMethod::HardwareDeviceTransfer;
    hardware_transfer.device_approval_present = false;
    assert_rejected(
        evaluate_account_recovery(hardware_transfer),
        AccountRecoveryReason::DeviceApprovalMissing,
    );
}

#[test]
fn account_recovery_requires_authenticated_rate_limited_encrypted_backup() {
    let mut unauthenticated = valid_input();
    unauthenticated.server_authenticated = false;
    let unauthenticated_decision = evaluate_account_recovery(unauthenticated);
    assert_rejected(
        unauthenticated_decision,
        AccountRecoveryReason::ServerAuthenticationMissing,
    );
    assert!(unauthenticated_decision.requires_server_setup);

    let mut unbounded_server = valid_input();
    unbounded_server.server_rate_limited = false;
    let unbounded_server_decision = evaluate_account_recovery(unbounded_server);
    assert_rejected(
        unbounded_server_decision,
        AccountRecoveryReason::ServerRateLimitMissing,
    );
    assert!(unbounded_server_decision.requires_server_setup);

    let mut unencrypted = valid_input();
    unencrypted.backup_encrypted = false;
    assert_rejected(
        evaluate_account_recovery(unencrypted),
        AccountRecoveryReason::BackupEncryptionMissing,
    );

    let mut plaintext = valid_input();
    plaintext.plaintext_backup_fields = 1;
    let plaintext_decision = evaluate_account_recovery(plaintext);
    assert_rejected(
        plaintext_decision,
        AccountRecoveryReason::PlaintextBackupForbidden,
    );
    assert!(!plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn account_recovery_requires_key_rotation_for_high_security_accounts() {
    let mut high_security = valid_input();
    high_security.high_security_account = true;
    high_security.recovery_key_entropy_bits = 192;
    high_security.rotates_device_secret = false;
    let high_security_decision = evaluate_account_recovery(high_security);
    assert_rejected(
        high_security_decision,
        AccountRecoveryReason::KeyRotationMissing,
    );
    assert!(high_security_decision.requires_key_rotation);
}

#[test]
fn account_recovery_methods_and_reasons_have_stable_codes_and_labels() {
    let methods = [
        (
            AccountRecoveryMethod::HighEntropyRecoveryKey,
            1,
            "high_entropy_recovery_key",
        ),
        (
            AccountRecoveryMethod::ThresholdRecovery,
            2,
            "threshold_recovery",
        ),
        (
            AccountRecoveryMethod::HardwareDeviceTransfer,
            3,
            "hardware_device_transfer",
        ),
        (AccountRecoveryMethod::LowEntropyPin, 4, "low_entropy_pin"),
        (AccountRecoveryMethod::CloudBackup, 5, "cloud_backup"),
    ];

    for (method, code, label) in methods {
        assert_eq!(method.code(), code);
        assert_eq!(method.label(), label);
    }

    let reasons = [
        (AccountRecoveryReason::Accepted, 0, "ACCEPTED"),
        (
            AccountRecoveryReason::RecoveryNotRequested,
            1,
            "RECOVERY_NOT_REQUESTED",
        ),
        (
            AccountRecoveryReason::LowEntropyPinForbidden,
            2,
            "LOW_ENTROPY_PIN_FORBIDDEN",
        ),
        (
            AccountRecoveryReason::RecoveryKeyTooWeak,
            3,
            "RECOVERY_KEY_TOO_WEAK",
        ),
        (
            AccountRecoveryReason::RecoveryKeyDigestMissing,
            4,
            "RECOVERY_KEY_DIGEST_MISSING",
        ),
        (
            AccountRecoveryReason::ThresholdQuorumInsufficient,
            5,
            "THRESHOLD_QUORUM_INSUFFICIENT",
        ),
        (
            AccountRecoveryReason::DeviceApprovalMissing,
            6,
            "DEVICE_APPROVAL_MISSING",
        ),
        (
            AccountRecoveryReason::ServerAuthenticationMissing,
            7,
            "SERVER_AUTHENTICATION_MISSING",
        ),
        (
            AccountRecoveryReason::ServerRateLimitMissing,
            8,
            "SERVER_RATE_LIMIT_MISSING",
        ),
        (
            AccountRecoveryReason::BackupEncryptionMissing,
            9,
            "BACKUP_ENCRYPTION_MISSING",
        ),
        (
            AccountRecoveryReason::PlaintextBackupForbidden,
            10,
            "PLAINTEXT_BACKUP_FORBIDDEN",
        ),
        (
            AccountRecoveryReason::KeyRotationMissing,
            11,
            "KEY_ROTATION_MISSING",
        ),
        (
            AccountRecoveryReason::AuditDigestMissing,
            12,
            "AUDIT_DIGEST_MISSING",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

#[test]
fn account_recovery_calls_service_adapter_only_after_accepted_gate() {
    let mut adapter = RecordingRecoveryAdapter::default();
    let accepted = recover_account_with_adapter(&mut adapter, valid_input())
        .expect("recording adapter is infallible");

    assert!(accepted.accepted);
    assert_eq!(adapter.recover_calls, 1);
    assert_eq!(
        adapter.last_method,
        Some(AccountRecoveryMethod::HighEntropyRecoveryKey)
    );

    let mut rejected_input = valid_input();
    rejected_input.method = AccountRecoveryMethod::LowEntropyPin;
    rejected_input.recovery_key_entropy_bits = 32;
    let rejected = recover_account_with_adapter(&mut adapter, rejected_input)
        .expect("recording adapter is infallible");

    assert_rejected(rejected, AccountRecoveryReason::LowEntropyPinForbidden);
    assert_eq!(adapter.recover_calls, 1);
}

fn valid_input() -> AccountRecoveryInput {
    AccountRecoveryInput {
        recovery_requested: true,
        method: AccountRecoveryMethod::HighEntropyRecoveryKey,
        high_security_account: false,
        recovery_key_entropy_bits: 128,
        recovery_key_digest_len: 32,
        threshold_shares: 3,
        threshold_required: 2,
        threshold_approvals: 2,
        device_approval_present: true,
        server_authenticated: true,
        server_rate_limited: true,
        backup_encrypted: true,
        plaintext_backup_fields: 0,
        rotates_device_secret: false,
        audit_digest_len: 32,
    }
}

fn assert_rejected(decision: mercury_core::AccountRecoveryDecision, reason: AccountRecoveryReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_start_recovery);
    assert!(!decision.can_restore_device_secret);
    assert!(decision.forbids_low_entropy_recovery);
    assert!(decision.forbids_plaintext_backup);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}

#[derive(Default)]
struct RecordingRecoveryAdapter {
    recover_calls: usize,
    last_method: Option<AccountRecoveryMethod>,
}

impl AccountRecoveryServiceAdapter for RecordingRecoveryAdapter {
    type Error = Infallible;

    fn recover_accepted_account(
        &mut self,
        accepted: AcceptedAccountRecovery,
    ) -> Result<(), Self::Error> {
        self.recover_calls += 1;
        self.last_method = Some(accepted.decision().method);
        Ok(())
    }
}
