use mercury_core::{
    AccountRecoveryInput, AccountRecoveryMethod, SecureBackupRestoreEnvelopeSuite,
    SecureBackupRestoreInput, SecureBackupRestoreReason, SecureBackupRestoreScope,
    SecureBackupRestoreTransport, evaluate_account_recovery, evaluate_secure_backup_restore,
};

#[test]
fn secure_backup_restore_accepts_epoch_bound_mls_archive() {
    let decision = evaluate_secure_backup_restore(valid_input());

    assert!(decision.accepted);
    assert_eq!(decision.reason, SecureBackupRestoreReason::Accepted);
    assert_eq!(decision.scope, SecureBackupRestoreScope::AccountAndMlsState);
    assert_eq!(
        decision.transport,
        SecureBackupRestoreTransport::CloudObjectStore
    );
    assert_eq!(
        decision.envelope_suite,
        SecureBackupRestoreEnvelopeSuite::XChaCha20Poly1305Blake3
    );
    assert!(decision.can_create_backup);
    assert!(decision.can_restore_device);
    assert!(decision.can_restore_mls_state);
    assert!(decision.can_use_cloud_storage);
    assert!(!decision.requires_user_action);
    assert!(!decision.requires_server_setup);
    assert!(!decision.requires_device_rekey);
    assert!(!decision.requires_group_rekey);
    assert!(!decision.requires_backup_reconfiguration);
    assert!(decision.forbids_plaintext_export);
    assert!(decision.forbids_os_plaintext_backup);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn secure_backup_restore_requires_accepted_account_recovery() {
    let mut recovery = valid_recovery_input();
    recovery.method = AccountRecoveryMethod::LowEntropyPin;
    recovery.recovery_key_entropy_bits = 32;

    let mut input = valid_input();
    input.account_recovery = evaluate_account_recovery(recovery);

    let decision = evaluate_secure_backup_restore(input);
    assert_rejected(decision, SecureBackupRestoreReason::AccountRecoveryRejected);
    assert!(decision.requires_user_action);
}

#[test]
fn secure_backup_restore_rejects_unknown_scope_transport_and_nonportable_envelope() {
    let mut scope = valid_input();
    scope.scope = SecureBackupRestoreScope::Unknown;
    assert_rejected(
        evaluate_secure_backup_restore(scope),
        SecureBackupRestoreReason::ScopeRejected,
    );

    let mut transport = valid_input();
    transport.transport = SecureBackupRestoreTransport::Unknown;
    assert_rejected(
        evaluate_secure_backup_restore(transport),
        SecureBackupRestoreReason::TransportRejected,
    );

    let mut envelope = valid_input();
    envelope.envelope_suite = SecureBackupRestoreEnvelopeSuite::PlatformSealedOnly;
    let envelope_decision = evaluate_secure_backup_restore(envelope);
    assert_rejected(
        envelope_decision,
        SecureBackupRestoreReason::EnvelopeSuiteRejected,
    );
    assert!(envelope_decision.requires_backup_reconfiguration);
}

#[test]
fn secure_backup_restore_requires_strong_backup_key_digest_and_kdf() {
    let mut weak_key = valid_input();
    weak_key.backup_key_entropy_bits = 128;
    let weak_decision = evaluate_secure_backup_restore(weak_key);
    assert_rejected(weak_decision, SecureBackupRestoreReason::BackupKeyTooWeak);
    assert!(weak_decision.requires_user_action);

    let mut high_security_weak = valid_input();
    high_security_weak.high_security_account = true;
    high_security_weak.backup_key_entropy_bits = 192;
    high_security_weak.kdf_memory_cost_mib = 128;
    assert_rejected(
        evaluate_secure_backup_restore(high_security_weak),
        SecureBackupRestoreReason::BackupKeyTooWeak,
    );

    let mut missing_digest = valid_input();
    missing_digest.backup_key_digest_len = 16;
    assert_rejected(
        evaluate_secure_backup_restore(missing_digest),
        SecureBackupRestoreReason::BackupKeyDigestMissing,
    );

    let mut weak_kdf = valid_input();
    weak_kdf.kdf_memory_cost_mib = 32;
    let weak_kdf_decision = evaluate_secure_backup_restore(weak_kdf);
    assert_rejected(
        weak_kdf_decision,
        SecureBackupRestoreReason::KdfPolicyMissing,
    );
    assert!(weak_kdf_decision.requires_backup_reconfiguration);
}

#[test]
fn secure_backup_restore_checks_transport_specific_recovery_proofs() {
    let mut hardware = valid_input();
    hardware.transport = SecureBackupRestoreTransport::HardwareDeviceTransfer;
    hardware.device_approval_present = false;
    let hardware_decision = evaluate_secure_backup_restore(hardware);
    assert_rejected(
        hardware_decision,
        SecureBackupRestoreReason::DeviceApprovalMissing,
    );
    assert!(hardware_decision.requires_user_action);

    let mut threshold = valid_input();
    threshold.transport = SecureBackupRestoreTransport::ThresholdRecoveryService;
    threshold.threshold_approvals = 1;
    let threshold_decision = evaluate_secure_backup_restore(threshold);
    assert_rejected(
        threshold_decision,
        SecureBackupRestoreReason::ThresholdQuorumMissing,
    );
    assert!(threshold_decision.requires_user_action);

    let mut server = valid_input();
    server.server_authenticated = false;
    let server_decision = evaluate_secure_backup_restore(server);
    assert_rejected(
        server_decision,
        SecureBackupRestoreReason::ServerAuthenticationMissing,
    );
    assert!(server_decision.requires_server_setup);

    let mut rate_limit = valid_input();
    rate_limit.server_rate_limited = false;
    assert_rejected(
        evaluate_secure_backup_restore(rate_limit),
        SecureBackupRestoreReason::ServerRateLimitMissing,
    );

    let mut opaque = valid_input();
    opaque.opaque_account_identifier = false;
    let opaque_decision = evaluate_secure_backup_restore(opaque);
    assert_rejected(
        opaque_decision,
        SecureBackupRestoreReason::OpaqueIdentifierMissing,
    );
    assert!(opaque_decision.requires_backup_reconfiguration);
}

#[test]
fn secure_backup_restore_forbids_plaintext_backup_paths() {
    let mut unencrypted = valid_input();
    unencrypted.backup_encrypted = false;
    assert_rejected(
        evaluate_secure_backup_restore(unencrypted),
        SecureBackupRestoreReason::BackupEncryptionMissing,
    );

    let mut plaintext = valid_input();
    plaintext.plaintext_export_fields = 1;
    let plaintext_decision = evaluate_secure_backup_restore(plaintext);
    assert_rejected(
        plaintext_decision,
        SecureBackupRestoreReason::PlaintextExportForbidden,
    );
    assert!(plaintext_decision.requires_backup_reconfiguration);
    assert!(!plaintext_decision.plaintext_bytes_exposed);

    let mut os_backup = valid_input();
    os_backup.os_plaintext_backup_excluded = false;
    let os_decision = evaluate_secure_backup_restore(os_backup);
    assert_rejected(
        os_decision,
        SecureBackupRestoreReason::OsBackupPolicyRejected,
    );
    assert!(os_decision.requires_backup_reconfiguration);
}

#[test]
fn secure_backup_restore_requires_mls_state_sealing_epoch_binding_and_rekey() {
    let mut not_sealed = valid_input();
    not_sealed.mls_state_sealed = false;
    let not_sealed_decision = evaluate_secure_backup_restore(not_sealed);
    assert_rejected(
        not_sealed_decision,
        SecureBackupRestoreReason::MlsStateNotSealed,
    );
    assert!(not_sealed_decision.requires_group_rekey);

    let mut unbound_epoch = valid_input();
    unbound_epoch.mls_epoch_bound = false;
    assert_rejected(
        evaluate_secure_backup_restore(unbound_epoch),
        SecureBackupRestoreReason::MlsEpochBindingMissing,
    );

    let mut device_rekey = valid_input();
    device_rekey.restore_rotates_device_secret = false;
    let device_rekey_decision = evaluate_secure_backup_restore(device_rekey);
    assert_rejected(
        device_rekey_decision,
        SecureBackupRestoreReason::RestoreRekeyMissing,
    );
    assert!(device_rekey_decision.requires_device_rekey);

    let mut group_rekey = valid_input();
    group_rekey.restore_rekeys_groups = false;
    let group_rekey_decision = evaluate_secure_backup_restore(group_rekey);
    assert_rejected(
        group_rekey_decision,
        SecureBackupRestoreReason::RestoreRekeyMissing,
    );
    assert!(group_rekey_decision.requires_group_rekey);
}

#[test]
fn secure_backup_restore_requires_tamper_replay_audit_and_bounded_retention() {
    let mut tamper = valid_input();
    tamper.archive_manifest_authenticated = false;
    assert_rejected(
        evaluate_secure_backup_restore(tamper),
        SecureBackupRestoreReason::TamperEvidenceMissing,
    );

    let mut replay = valid_input();
    replay.replay_nonce_len = 16;
    assert_rejected(
        evaluate_secure_backup_restore(replay),
        SecureBackupRestoreReason::ReplayProtectionMissing,
    );

    let mut audit = valid_input();
    audit.audit_digest_len = 16;
    assert_rejected(
        evaluate_secure_backup_restore(audit),
        SecureBackupRestoreReason::AuditDigestMissing,
    );

    let mut retention = valid_input();
    retention.retention_days = 90;
    retention.max_retention_days = 45;
    let retention_decision = evaluate_secure_backup_restore(retention);
    assert_rejected(
        retention_decision,
        SecureBackupRestoreReason::RetentionPolicyRejected,
    );
    assert!(retention_decision.requires_backup_reconfiguration);
}

#[test]
fn secure_backup_restore_codes_and_labels_are_stable() {
    let scopes = [
        (
            SecureBackupRestoreScope::AccountSecretsOnly,
            1,
            "account_secrets_only",
        ),
        (
            SecureBackupRestoreScope::AccountAndMlsState,
            2,
            "account_and_mls_state",
        ),
        (SecureBackupRestoreScope::MediaArchive, 3, "media_archive"),
        (
            SecureBackupRestoreScope::FullDeviceTransfer,
            4,
            "full_device_transfer",
        ),
        (SecureBackupRestoreScope::Unknown, 5, "unknown"),
    ];
    for (scope, code, label) in scopes {
        assert_eq!(scope.code(), code);
        assert_eq!(scope.label(), label);
    }

    let transports = [
        (
            SecureBackupRestoreTransport::UserRecoveryKeyArchive,
            1,
            "user_recovery_key_archive",
        ),
        (
            SecureBackupRestoreTransport::ThresholdRecoveryService,
            2,
            "threshold_recovery_service",
        ),
        (
            SecureBackupRestoreTransport::HardwareDeviceTransfer,
            3,
            "hardware_device_transfer",
        ),
        (
            SecureBackupRestoreTransport::CloudObjectStore,
            4,
            "cloud_object_store",
        ),
        (SecureBackupRestoreTransport::Unknown, 5, "unknown"),
    ];
    for (transport, code, label) in transports {
        assert_eq!(transport.code(), code);
        assert_eq!(transport.label(), label);
    }

    let suites = [
        (
            SecureBackupRestoreEnvelopeSuite::XChaCha20Poly1305Blake3,
            1,
            "xchacha20_poly1305_blake3",
        ),
        (
            SecureBackupRestoreEnvelopeSuite::Aes256GcmHkdfSha256,
            2,
            "aes256_gcm_hkdf_sha256",
        ),
        (
            SecureBackupRestoreEnvelopeSuite::PlatformSealedOnly,
            3,
            "platform_sealed_only",
        ),
        (SecureBackupRestoreEnvelopeSuite::Unknown, 4, "unknown"),
    ];
    for (suite, code, label) in suites {
        assert_eq!(suite.code(), code);
        assert_eq!(suite.label(), label);
    }

    let reasons = [
        (SecureBackupRestoreReason::Accepted, 0, "ACCEPTED"),
        (
            SecureBackupRestoreReason::AccountRecoveryRejected,
            1,
            "ACCOUNT_RECOVERY_REJECTED",
        ),
        (
            SecureBackupRestoreReason::ScopeRejected,
            2,
            "SCOPE_REJECTED",
        ),
        (
            SecureBackupRestoreReason::TransportRejected,
            3,
            "TRANSPORT_REJECTED",
        ),
        (
            SecureBackupRestoreReason::EnvelopeSuiteRejected,
            4,
            "ENVELOPE_SUITE_REJECTED",
        ),
        (
            SecureBackupRestoreReason::BackupKeyTooWeak,
            5,
            "BACKUP_KEY_TOO_WEAK",
        ),
        (
            SecureBackupRestoreReason::BackupKeyDigestMissing,
            6,
            "BACKUP_KEY_DIGEST_MISSING",
        ),
        (
            SecureBackupRestoreReason::KdfPolicyMissing,
            7,
            "KDF_POLICY_MISSING",
        ),
        (
            SecureBackupRestoreReason::DeviceApprovalMissing,
            8,
            "DEVICE_APPROVAL_MISSING",
        ),
        (
            SecureBackupRestoreReason::ThresholdQuorumMissing,
            9,
            "THRESHOLD_QUORUM_MISSING",
        ),
        (
            SecureBackupRestoreReason::ServerAuthenticationMissing,
            10,
            "SERVER_AUTHENTICATION_MISSING",
        ),
        (
            SecureBackupRestoreReason::ServerRateLimitMissing,
            11,
            "SERVER_RATE_LIMIT_MISSING",
        ),
        (
            SecureBackupRestoreReason::OpaqueIdentifierMissing,
            12,
            "OPAQUE_IDENTIFIER_MISSING",
        ),
        (
            SecureBackupRestoreReason::BackupEncryptionMissing,
            13,
            "BACKUP_ENCRYPTION_MISSING",
        ),
        (
            SecureBackupRestoreReason::PlaintextExportForbidden,
            14,
            "PLAINTEXT_EXPORT_FORBIDDEN",
        ),
        (
            SecureBackupRestoreReason::OsBackupPolicyRejected,
            15,
            "OS_BACKUP_POLICY_REJECTED",
        ),
        (
            SecureBackupRestoreReason::MlsStateNotSealed,
            16,
            "MLS_STATE_NOT_SEALED",
        ),
        (
            SecureBackupRestoreReason::MlsEpochBindingMissing,
            17,
            "MLS_EPOCH_BINDING_MISSING",
        ),
        (
            SecureBackupRestoreReason::RestoreRekeyMissing,
            18,
            "RESTORE_REKEY_MISSING",
        ),
        (
            SecureBackupRestoreReason::TamperEvidenceMissing,
            19,
            "TAMPER_EVIDENCE_MISSING",
        ),
        (
            SecureBackupRestoreReason::ReplayProtectionMissing,
            20,
            "REPLAY_PROTECTION_MISSING",
        ),
        (
            SecureBackupRestoreReason::AuditDigestMissing,
            21,
            "AUDIT_DIGEST_MISSING",
        ),
        (
            SecureBackupRestoreReason::RetentionPolicyRejected,
            22,
            "RETENTION_POLICY_REJECTED",
        ),
    ];
    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> SecureBackupRestoreInput {
    SecureBackupRestoreInput {
        account_recovery: evaluate_account_recovery(valid_recovery_input()),
        scope: SecureBackupRestoreScope::AccountAndMlsState,
        transport: SecureBackupRestoreTransport::CloudObjectStore,
        envelope_suite: SecureBackupRestoreEnvelopeSuite::XChaCha20Poly1305Blake3,
        high_security_account: false,
        backup_key_entropy_bits: 192,
        backup_key_digest_len: 32,
        kdf_memory_cost_mib: 64,
        kdf_iterations: 3,
        device_approval_present: true,
        threshold_shares: 5,
        threshold_required: 3,
        threshold_approvals: 3,
        server_authenticated: true,
        server_rate_limited: true,
        opaque_account_identifier: true,
        backup_encrypted: true,
        plaintext_export_fields: 0,
        os_plaintext_backup_excluded: true,
        mls_state_included: true,
        mls_state_sealed: true,
        mls_epoch_bound: true,
        restore_rotates_device_secret: true,
        restore_rekeys_groups: true,
        archive_manifest_authenticated: true,
        replay_nonce_len: 24,
        audit_digest_len: 32,
        retention_days: 30,
        max_retention_days: 45,
    }
}

fn valid_recovery_input() -> AccountRecoveryInput {
    AccountRecoveryInput {
        recovery_requested: true,
        method: AccountRecoveryMethod::CloudBackup,
        high_security_account: false,
        recovery_key_entropy_bits: 192,
        recovery_key_digest_len: 32,
        threshold_shares: 5,
        threshold_required: 3,
        threshold_approvals: 3,
        device_approval_present: true,
        server_authenticated: true,
        server_rate_limited: true,
        backup_encrypted: true,
        plaintext_backup_fields: 0,
        rotates_device_secret: true,
        audit_digest_len: 32,
    }
}

fn assert_rejected(
    decision: mercury_core::SecureBackupRestoreDecision,
    reason: SecureBackupRestoreReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_create_backup);
    assert!(!decision.can_restore_device);
    assert!(!decision.can_restore_mls_state);
    assert!(!decision.can_use_cloud_storage);
    assert!(decision.forbids_plaintext_export);
    assert!(decision.forbids_os_plaintext_backup);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
