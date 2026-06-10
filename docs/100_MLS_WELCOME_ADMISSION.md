# MLS Welcome Admission Gate

Generated: 2026-05-28

## Status

Mercury now has a backend MLS Welcome admission gate in `mercury-core`:

```text
MlsWelcomeAdmissionInput
MlsWelcomeAdmissionDecision
MlsWelcomeAdmissionReason
evaluate_mls_welcome_admission(...)
```

The gate is the receiving-side partner to the KeyPackage admission gate. It blocks a client from joining or opening a new MLS group from a Welcome unless a production provider has already verified the Welcome, GroupInfo, ratchet tree, transcript confirmation, replay state, and Commit ordering facts.

## Research Basis

RFC 9420 defines Welcome processing around encrypted group secrets, encrypted GroupInfo, GroupInfo signatures, PSKs, ratchet tree verification, transcript confirmation, and epoch-secret derivation.

RFC 9750 describes how an MLS architecture has to deal with Commit ordering and fork prevention. Mercury models this as `commit_won_tie_break`; a Welcome tied to a losing Commit cannot initialize local group state.

Sources:

- <https://www.rfc-editor.org/rfc/rfc9420.html>
- <https://www.rfc-editor.org/rfc/rfc9750>

## Accepted Output

Accepted output enables:

```text
can_join_group = true
can_initialize_epoch = true
can_open_group = true
prevents_welcome_replay = true
forbids_plaintext_group_metadata = true
plaintext_bytes_exposed = false
```

Rejected output never enables group join, epoch initialization, or group open.

## Checked Conditions

The gate requires:

- accepted KeyPackage admission
- matching Welcome, KeyPackage, and GroupInfo ciphersuite classes
- matching encrypted group secrets for the local KeyPackage
- decrypted group secrets and encrypted GroupInfo
- available PSKs, with at most one resumption PSK
- valid GroupInfo signature
- locally unique group id
- confidentially available ratchet tree material
- matching ratchet tree hash
- valid parent hashes, leaves, unmerged leaves, and unique encryption keys
- local leaf present and matching the admitted KeyPackage
- valid path secret and epoch secret derivation
- 32-byte confirmed transcript hash
- valid confirmation tag
- winning Commit tie-break result
- positive group epoch
- reinit PSK epoch rules
- 32-byte Welcome hash and no prior processing
- zero plaintext group metadata fields

## Checked Fixtures

Prototype fixtures:

```text
mls_welcome_admission_ready
mls_welcome_admission_secrets_missing
mls_welcome_admission_tree_rejected
mls_welcome_admission_confirmation_rejected
mls_welcome_admission_tie_break_rejected
mls_welcome_admission_replay_rejected
mls_welcome_admission_plaintext_rejected
```

Backend commands:

```text
run_mls_welcome_admission_ready
run_mls_welcome_admission_secrets_missing
run_mls_welcome_admission_tree_rejected
run_mls_welcome_admission_confirmation_rejected
run_mls_welcome_admission_tie_break_rejected
run_mls_welcome_admission_replay_rejected
run_mls_welcome_admission_plaintext_rejected
```

## UI Contract

UI and platform code must not open a newly invited group unless:

```text
accepted = true
can_join_group = true
can_initialize_epoch = true
can_open_group = true
```

Route `requires_sync`, `requires_mls_setup`, `requires_tree_fetch`, and `requires_user_action` from backend decisions. Never recover by fetching plaintext group/member metadata, accepting a server-supplied tree without provider verification, or opening a group from a Welcome tied to a losing Commit.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_welcome_admission
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_welcome_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_admission_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

The follow-on Welcome replay-store boundary now persists accepted Welcome hashes with consumed KeyPackage refs, init-key deletion, tree/transcript hash binding, and transactional group-state commit evidence. When the production MLS provider is selected, wire real Welcome verification output into this gate and connect accepted Welcome admission plus replay-store persistence to production group-state initialization.
