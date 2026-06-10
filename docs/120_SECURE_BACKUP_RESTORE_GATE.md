# Secure Backup Restore Gate

Generated: 2026-05-28

## Status

Mercury now has a secure backup/restore gate in `mercury-core`:

```text
SecureBackupRestoreScope
SecureBackupRestoreTransport
SecureBackupRestoreEnvelopeSuite
SecureBackupRestoreInput
SecureBackupRestoreDecision
SecureBackupRestoreReason
evaluate_secure_backup_restore(...)
```

This is not the production backup implementation. It is the security contract a future mobile, desktop, cloud-archive, or device-transfer implementation must satisfy before Mercury creates or restores an archive containing account secrets, media history, or MLS group state.

## Accepted Backup Contract

The accepted path requires:

- accepted `AccountRecoveryDecision`
- known backup scope and transport
- portable authenticated-encryption archive suite
- 192-bit backup-key entropy for normal accounts
- 256-bit backup-key entropy for high-security accounts
- 32-byte backup-key digest
- Argon2-style KDF policy of at least 64 MiB and 3 iterations
- at least 128 MiB KDF memory for high-security accounts
- device approval for hardware transfer
- threshold quorum for threshold recovery service
- server authentication, server rate limiting, and opaque account identifiers for server-backed transports
- encrypted backup material
- zero plaintext export fields
- OS plaintext/automatic backup exclusion
- sealed MLS state when MLS state is included
- MLS epoch binding for restored group state
- device-secret rotation on restore
- group rekeying when MLS state is restored
- authenticated archive manifest
- replay nonce length of at least 24 bytes
- 32-byte audit digest
- bounded retention policy

Accepted output enables:

```text
can_create_backup = true
can_restore_device = true
can_restore_mls_state = true when MLS state is in scope
forbids_plaintext_export = true
forbids_os_plaintext_backup = true
plaintext_bytes_exposed = false
```

## Reason Labels

Stable secure-backup labels:

```text
ACCEPTED
ACCOUNT_RECOVERY_REJECTED
SCOPE_REJECTED
TRANSPORT_REJECTED
ENVELOPE_SUITE_REJECTED
BACKUP_KEY_TOO_WEAK
BACKUP_KEY_DIGEST_MISSING
KDF_POLICY_MISSING
DEVICE_APPROVAL_MISSING
THRESHOLD_QUORUM_MISSING
SERVER_AUTHENTICATION_MISSING
SERVER_RATE_LIMIT_MISSING
OPAQUE_IDENTIFIER_MISSING
BACKUP_ENCRYPTION_MISSING
PLAINTEXT_EXPORT_FORBIDDEN
OS_BACKUP_POLICY_REJECTED
MLS_STATE_NOT_SEALED
MLS_EPOCH_BINDING_MISSING
RESTORE_REKEY_MISSING
TAMPER_EVIDENCE_MISSING
REPLAY_PROTECTION_MISSING
AUDIT_DIGEST_MISSING
RETENTION_POLICY_REJECTED
```

## Fixture Surface

Checked fixtures:

```text
secure_backup_restore_ready
secure_backup_restore_recovery_rejected
secure_backup_restore_plaintext_rejected
secure_backup_restore_mls_rekey_rejected
secure_backup_restore_cloud_policy_rejected
```

Backend command envelopes:

```text
run_secure_backup_restore_ready
run_secure_backup_restore_recovery_rejected
run_secure_backup_restore_plaintext_rejected
run_secure_backup_restore_mls_rekey_rejected
run_secure_backup_restore_cloud_policy_rejected
```

## Research Basis

- Signal Secure Backups are opt-in, end-to-end encrypted, use a device-generated recovery key, and store archives without direct linkage to a Signal account: https://signal.org/blog/introducing-secure-backups/
- Signal Secure Value Recovery describes Argon2 hardening, offline-guess prevention, rate-limited recovery attempts, remote attestation, and replicated enclave state: https://signal.org/blog/secure-value-recovery/
- Apple iCloud Keychain escrow uses HSM clusters, SRP verification, strict attempt limits, and destructive lockout after repeated failures: https://support.apple.com/en-euro/guide/security/sec3e341e75d/web
- Android Auto Backup can include app data unless backup rules and encryption requirements are explicitly configured, so Mercury must treat OS backups as a security boundary: https://developer.android.com/identity/data/autobackup
- NIST SP 800-57 Part 1 Rev. 5 frames backup, archive, key recovery, split knowledge, audit, and key-management controls as part of cryptographic key lifecycle management: https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final
- RFC 9420 defines MLS group epochs, exporters, transcript binding, forward secrecy, and post-compromise security considerations that restored MLS state must not bypass: https://www.rfc-editor.org/info/rfc9420/
- Kintsugi proposes decentralized, threshold-assisted E2EE key recovery that protects against offline guessing without specialized secure hardware; Mercury should treat decentralized threshold recovery as a future research-backed option, not an immediate dependency: https://arxiv.org/abs/2507.21122

## Verification

Run:

```powershell
cargo test -p mercury-core --test secure_backup_restore
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype secure_backup_restore_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_secure_backup_restore_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

Implement a production backup package builder behind this gate. It should stay disabled until the platform can prove archive encryption, manifest authentication, replay protection, OS backup exclusion, account recovery acceptance, MLS epoch binding, restore rekeying, and audit persistence for every supported platform.
