# MLS Commit Admission Gate

Generated: 2026-05-28

## Status

Mercury now has a backend MLS Commit admission gate in `mercury-core`:

```text
MlsCommitAdmissionInput
MlsCommitAdmissionDecision
MlsCommitAdmissionReason
evaluate_mls_commit_admission(...)
```

The gate blocks group epoch advancement unless a production MLS provider has already verified current-epoch binding, sender authentication, proposal validity, update-path and tree integrity, transcript confirmation, replay state, deterministic ordering, and plaintext-free Commit metadata.

## Research Basis

RFC 9420 requires Commit processing to use the current GroupContext epoch, validate committed proposals, apply proposal lists, validate required update paths, derive the new epoch secret, and verify the confirmation tag for the new epoch.

RFC 9750 describes delivery-service ordering and deterministic tie-breaking for concurrent Commits. Mercury models this as `commit_won_tie_break`; a losing Commit cannot advance local group state.

Sources:

- <https://www.rfc-editor.org/rfc/rfc9420.html>
- <https://www.rfc-editor.org/rfc/rfc9750>

## Accepted Output

Accepted output enables:

```text
can_apply_commit = true
can_initialize_epoch = true
can_continue_group = true
prevents_commit_replay = true
forbids_plaintext_commit_metadata = true
plaintext_bytes_exposed = false
```

If the accepted Commit removes the local member, the decision is terminal:

```text
reason_label = LOCAL_MEMBER_REMOVED
can_apply_commit = true
can_initialize_epoch = false
can_continue_group = false
local_member_removed = true
requires_user_action = true
```

Rejected output never enables Commit application, epoch initialization, or continued group use.

## Checked Conditions

The gate requires:

- accepted group chat readiness
- positive current epoch and Commit epoch equal to the current epoch
- member sender for regular Commits
- new-member sender type and ExternalInit for external Commits
- valid Commit signature and membership tag evidence
- valid proposal list
- available referenced proposals
- application policy acceptance of proposals
- no duplicate update/remove proposal targets
- no committer self-update or committer self-remove proposal
- required update path present
- valid update-path leaf and source
- valid update-path parent hashes
- decryptable update-path secrets
- matching ratchet-tree hash
- provisional group context binding
- epoch-secret derivation
- 32-byte confirmed transcript hash
- valid confirmation tag
- winning deterministic Commit tie-break result
- 32-byte Commit hash and no prior processing
- zero plaintext Commit metadata fields

## Checked Fixtures

Prototype fixtures:

```text
mls_commit_admission_ready
mls_commit_admission_bad_epoch
mls_commit_admission_auth_rejected
mls_commit_admission_path_rejected
mls_commit_admission_tie_break_rejected
mls_commit_admission_replay_rejected
mls_commit_admission_plaintext_rejected
```

Backend commands:

```text
run_mls_commit_admission_ready
run_mls_commit_admission_bad_epoch
run_mls_commit_admission_auth_rejected
run_mls_commit_admission_path_rejected
run_mls_commit_admission_tie_break_rejected
run_mls_commit_admission_replay_rejected
run_mls_commit_admission_plaintext_rejected
```

## UI Contract

UI and platform code must not apply a Commit unless:

```text
accepted = true
can_apply_commit = true
```

Continue showing the group as usable only when `can_continue_group = true`. Route `local_member_removed = true` to a terminal removed-member state, not a retry. Route `requires_sync`, `requires_mls_setup`, `requires_tree_repair`, `requires_rekey`, and `requires_user_action` from backend decisions.

Never recover by accepting a server-supplied tree without provider verification, applying a losing Commit, accepting replayed Commit hashes, or carrying plaintext Commit/member metadata into UI state.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_commit_admission
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_commit_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_commit_admission_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

The follow-on Commit replay-store boundary now persists accepted Commit hashes in an opaque digest-only store. When the production MLS provider is selected, wire real Commit verification output into this gate and connect accepted Commit admission plus replay-store persistence to production group-state epoch advancement.
