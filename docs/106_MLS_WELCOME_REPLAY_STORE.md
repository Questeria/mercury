# MLS Welcome Replay Store

Generated: 2026-05-28

## Status

Mercury now has a backend MLS Welcome replay-store boundary in `mercury-core`:

```text
MlsWelcomeReplayStoreWrite
MlsWelcomeReplayStoreDecision
MlsWelcomeReplayStoreAdapter
put_mls_welcome_replay_record(...)
```

The boundary persists accepted Welcome records only after the MLS Welcome admission gate accepts. Unlike a simple digest cache, it also requires consumed KeyPackage identity, init-key deletion, tree/transcript binding, and transactional group-state commit evidence before a newly joined group can be initialized once.

## Research Basis

RFC 9420 defines Welcome processing as the path by which a new member obtains current group state and epoch secrets after a Commit. RFC 9420 and RFC 9750 both make KeyPackage reuse and init-key lifecycle important for replay safety. RFC 9750 also highlights fork and replay risks that applications must handle around delivery and recovery.

Sources:

- <https://www.rfc-editor.org/rfc/rfc9420.html#section-12.4.3.1>
- <https://www.rfc-editor.org/rfc/rfc9420.html#section-16.8>
- <https://datatracker.ietf.org/doc/html/rfc9750#section-5.1>
- <https://datatracker.ietf.org/doc/html/rfc9750#section-5.2>
- <https://datatracker.ietf.org/doc/html/rfc9750#section-8.6>

## Accepted Output

Accepted output enables:

```text
accepted = true
can_initialize_group_once = true
can_open_group = true
consumes_key_package = true
deletes_init_key = true
binds_tree_hash = true
binds_confirmed_transcript_hash = true
commits_group_state_transactionally = true
keeps_digest_only = true
plaintext_bytes_exposed = false
```

Rejected output never enables group initialization.

## Checked Conditions

The replay store requires:

- accepted MLS Welcome admission
- `can_join_group = true`
- `can_initialize_epoch = true`
- `can_open_group = true`
- `prevents_welcome_replay = true`
- 32-byte group id
- 32-byte Welcome hash
- 32-byte consumed KeyPackage ref
- 32-byte ratchet tree hash
- 32-byte confirmed transcript hash
- 32-byte group-state commit digest
- positive epoch
- non-negative join timestamp
- init key deleted
- group state transaction committed
- zero plaintext metadata fields
- no existing record for the same group id and Welcome hash
- no existing record for the same consumed KeyPackage ref

## Persisted Record

The prototype adapter persists only:

```text
group_id
welcome_hash
consumed_key_package_ref
tree_hash
confirmed_transcript_hash
group_state_commit_digest
epoch
joined_at_s
init_key_deleted
plaintext_bytes_exposed
```

It intentionally does not persist raw Welcome bytes, group secrets, path secrets, init private keys, credential material, ratchet-tree plaintext, or UI-visible group metadata.

## Checked Fixtures

Prototype fixtures:

```text
mls_welcome_replay_store_ready
mls_welcome_replay_store_admission_rejected
mls_welcome_replay_store_duplicate_rejected
mls_welcome_replay_store_key_package_reused
mls_welcome_replay_store_bad_shape
mls_welcome_replay_store_plaintext_rejected
```

Backend commands:

```text
run_mls_welcome_replay_store_ready
run_mls_welcome_replay_store_admission_rejected
run_mls_welcome_replay_store_duplicate_rejected
run_mls_welcome_replay_store_key_package_reused
run_mls_welcome_replay_store_bad_shape
run_mls_welcome_replay_store_plaintext_rejected
```

## UI Contract

UI and platform code must not initialize a joined group unless both conditions are true:

```text
welcome_admission.accepted = true
welcome_replay_store.accepted = true
```

Treat `WELCOME_ALREADY_RECORDED` as a hard duplicate/replay stop. Treat `KEY_PACKAGE_ALREADY_CONSUMED`, `INIT_KEY_NOT_DELETED`, and `GROUP_STATE_NOT_COMMITTED` as backend persistence or provider-lifecycle failures, not retryable UI states. Never recover by accepting server-supplied group state without provider verification and local transactional persistence.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_welcome_replay_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_welcome_replay_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_replay_store_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

Select a production MLS provider and storage adapter, then connect real accepted Welcome output, consumed KeyPackage state, init-key deletion, and durable group-state writes to this replay store before local group initialization.
