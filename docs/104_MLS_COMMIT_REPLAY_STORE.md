# MLS Commit Replay Store

Generated: 2026-05-28

## Status

Mercury now has a backend MLS Commit replay-store boundary in `mercury-core`:

```text
MlsCommitReplayStoreWrite
MlsCommitReplayStoreDecision
MlsCommitReplayStoreAdapter
put_mls_commit_replay_record(...)
```

The boundary persists accepted Commit hashes only after the MLS Commit admission gate accepts. It gives production storage one narrow responsibility: remember opaque accepted Commit digests per group so the same Commit cannot be applied twice.

## Research Basis

RFC 9420 treats Commit processing as the epoch transition point for an MLS group. A processed Commit changes group epoch, tree, transcript state, and application membership assumptions.

RFC 9750 describes MLS deployment risks around Commit ordering, fork recovery, and replay-sensitive delivery behavior. Mercury models accepted Commit persistence as an application-layer anti-replay guard that must succeed before local group state is advanced.

Sources:

- <https://www.rfc-editor.org/rfc/rfc9420.html>
- <https://www.rfc-editor.org/rfc/rfc9750>

## Accepted Output

Accepted output enables:

```text
accepted = true
can_apply_commit_once = true
keeps_digest_only = true
plaintext_bytes_exposed = false
```

If the admitted Commit removes the local member, the stored record is terminal:

```text
local_member_removed = true
can_continue_group = false
```

Rejected output never enables one-time Commit application.

## Checked Conditions

The replay store requires:

- accepted MLS Commit admission
- `can_apply_commit = true`
- `prevents_commit_replay = true`
- 32-byte group id
- 32-byte Commit hash
- positive epoch
- non-negative application timestamp
- zero plaintext metadata fields
- no existing record for the same group id and Commit hash

## Persisted Record

The prototype adapter persists only:

```text
group_id
commit_hash
epoch
applied_at_s
local_member_removed
plaintext_bytes_exposed
```

It intentionally does not persist raw Commit bytes, proposal contents, member names, credential material, tree material, or plaintext metadata.

## Checked Fixtures

Prototype fixtures:

```text
mls_commit_replay_store_ready
mls_commit_replay_store_admission_rejected
mls_commit_replay_store_duplicate_rejected
mls_commit_replay_store_local_member_removed
mls_commit_replay_store_plaintext_rejected
```

Backend commands:

```text
run_mls_commit_replay_store_ready
run_mls_commit_replay_store_admission_rejected
run_mls_commit_replay_store_duplicate_rejected
run_mls_commit_replay_store_local_member_removed
run_mls_commit_replay_store_plaintext_rejected
```

## UI Contract

UI and platform code must not apply a Commit unless both conditions are true:

```text
commit_admission.accepted = true
commit_replay_store.accepted = true
```

Treat `COMMIT_ALREADY_RECORDED` as a hard duplicate/replay stop. Treat `LOCAL_MEMBER_REMOVED` admission with accepted replay persistence as a terminal removed-member state. Do not attempt to repair replay-store failures by trusting server-supplied state or replaying plaintext Commit data through UI state.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_commit_replay_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_commit_replay_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_commit_replay_store_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

Select a production MLS provider and storage adapter, then connect real accepted Commit output to this digest-only replay store before local epoch advancement.
