# Account Recovery Gate

Generated: 2026-05-28

## Status

Mercury now has a backend account-recovery gate in `mercury-core`:

```text
AccountRecoveryMethod
AccountRecoveryInput
AccountRecoveryDecision
AccountRecoveryReason
evaluate_account_recovery(...)
recover_account_with_adapter(...)
```

This is not the final recovery service. It is the security contract a future mobile, desktop, server, or threshold-recovery implementation must satisfy before the platform can start recovery or restore a device secret.

## Accepted Recovery

The accepted path is intentionally narrow:

- recovery must be explicitly requested
- low-entropy PIN recovery is always forbidden
- normal accounts require at least 128 bits of recovery-key entropy
- high-security accounts require at least 192 bits of recovery-key entropy
- recovery-key digest must be 32 bytes
- threshold recovery must have quorum shares and approvals
- hardware device transfer must have device approval
- server recovery path must be authenticated and rate limited
- backup material must be encrypted
- plaintext backup fields must be zero
- high-security accounts must rotate device secrets
- audit digest must be 32 bytes

Accepted output enables:

```text
can_start_recovery = true
can_restore_device_secret = true
```

Accepted output always keeps:

```text
forbids_low_entropy_recovery = true
forbids_plaintext_backup = true
plaintext_bytes_exposed = false
```

## Rejection Classes

Stable rejection labels:

```text
RECOVERY_NOT_REQUESTED
LOW_ENTROPY_PIN_FORBIDDEN
RECOVERY_KEY_TOO_WEAK
RECOVERY_KEY_DIGEST_MISSING
THRESHOLD_QUORUM_INSUFFICIENT
DEVICE_APPROVAL_MISSING
SERVER_AUTHENTICATION_MISSING
SERVER_RATE_LIMIT_MISSING
BACKUP_ENCRYPTION_MISSING
PLAINTEXT_BACKUP_FORBIDDEN
KEY_ROTATION_MISSING
AUDIT_DIGEST_MISSING
```

The decision separates:

- `requires_user_action`
- `requires_server_setup`
- `requires_key_rotation`

## Adapter Boundary

`AccountRecoveryServiceAdapter` is the production-service boundary. `recover_account_with_adapter(...)` evaluates `AccountRecoveryInput` first and calls `recover_accepted_account(...)` only after the decision accepts.

Future service implementations should bind this accepted-only adapter to:

- high-entropy recovery-key import
- threshold approval verification
- hardware device transfer
- encrypted backup retrieval
- server-side authentication and rate limiting
- device-secret rotation and audit persistence

Rejected recovery attempts do not call the adapter.

## Checked Fixtures

Prototype fixtures:

```text
account_recovery_high_entropy_ready
account_recovery_low_entropy_pin_forbidden
account_recovery_threshold_quorum_required
account_recovery_plaintext_backup_forbidden
account_recovery_key_rotation_required
```

These fixtures expose accepted high-entropy recovery, low-entropy PIN rejection, missing threshold quorum, plaintext backup rejection, and high-security key-rotation requirements through the simulator.

## Verification

Run:

```powershell
cargo test -p mercury-core --test account_recovery_gate
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype account_recovery_high_entropy_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused core test covers accepted recovery, accepted-only service adapter calls, missing request rejection, low-entropy PIN rejection, entropy and digest requirements, threshold quorum, hardware approval, server authentication/rate limiting, encrypted backup requirements, plaintext backup rejection, high-security key rotation, audit digest checks, and stable codes/labels.

## Follow-On Gate

`docs/120_SECURE_BACKUP_RESTORE_GATE.md` now extends this account-recovery gate to backup creation and restore. Production recovery work must satisfy both gates before it can restore device secrets, MLS group state, or archive data.
