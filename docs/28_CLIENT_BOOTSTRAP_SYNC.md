# Client Bootstrap Sync

Generated: 2026-05-28

## Status

Mercury now has a typed client bootstrap and sync readiness boundary in `mercury-core`.

```text
ClientBootstrapInput
evaluate_client_bootstrap(ClientBootstrapInput) -> ClientBootstrapDecision
```

This boundary is the pre-UI startup gate. A future mobile or desktop shell may start background sync when recoverable state is incomplete, but it must not open the message UI or decrypt the local store until this decision accepts.

## Inputs

The gate checks:

- account id presence
- device id presence
- local device trust
- key transparency readiness
- sealed account secret state
- sealed device secret state
- sealed room state
- replay checkpoint state
- sync catch-up state
- pending recovery flag
- plaintext cache record count

The secret-state inputs are deliberately coarse. Platform clients can map OS keychain, keystore, database, or recovery results into these states without letting the UI reinterpret raw security facts.

## Decision Shape

`ClientBootstrapDecision` returns:

- `accepted`
- `can_start_sync`
- `can_decrypt_local_store`
- `can_open_message_ui`
- `requires_sync`
- `requires_recovery`
- `requires_user_action`
- `reason`

Recoverable sync gaps can set `can_start_sync = true` while keeping local decryption and message UI closed. Fatal identity, trust, plaintext-cache, or recovery failures keep all capability flags closed.

## Security Rules

The evaluator rejects:

- missing account or device identity
- pending recovery work
- any plaintext cache record
- local devices that are not fully trusted
- missing, stale, unchecked, or inconsistent key transparency
- missing or corrupt account/device secrets
- plaintext account, device, or room secrets
- missing or corrupt room state
- missing, stale, or gapped replay checkpoints
- offline, catching-up, failed, or gapped sync state

Accepted bootstrap decisions set:

- `can_start_sync = true`
- `can_decrypt_local_store = true`
- `can_open_message_ui = true`

This gives frontend clients a simple rule: no message list, notification preview, decrypted timeline, or AI context surface is allowed until bootstrap accepts.

## Verification

The `client_bootstrap_sync` integration test covers:

- accepted startup
- missing account and device ids
- recovery and plaintext-cache refusal
- local device trust rejection
- key transparency not-ready and inconsistent states
- missing, corrupt, and plaintext secret states
- room-state and replay-checkpoint sync requirements
- sync incomplete, gap, and failed states

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Next Step

The account recovery gate that handles `requires_recovery` repair paths is documented in `docs/74_ACCOUNT_RECOVERY_GATE.md`.
