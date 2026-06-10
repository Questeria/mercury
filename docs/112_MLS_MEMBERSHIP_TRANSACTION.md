# MLS Membership Transaction Witness

Generated: 2026-05-28

## Status

Mercury now has a backend MLS membership-change transaction witness in `mercury-core`:

```text
MlsMembershipTransactionWrite
MlsMembershipTransactionDecision
MlsMembershipTransactionAdapter
put_mls_membership_transaction_record(...)
```

The witness sits after three accepted persistence boundaries:

```text
accepted Commit replay store
accepted KeyPackage consume store
accepted Welcome send outbox
```

It records a digest-only transaction marker only when those records are cross-bound and the future production store promises one atomic, durable, serializable write with idempotent outbox delivery and crash recovery.

## Research Basis

RFC 9420 makes Commit processing and new-member Welcome delivery security-sensitive, and requires KeyPackage reuse to be tightly controlled. RFC 9750 makes Delivery Service ordering, tie-breaking, replay, and fork handling part of a safe MLS deployment. Transactional-outbox guidance from AWS and Microsoft maps the dual-write problem to storage: state change and outgoing message enqueue must commit or roll back together. SQLite's atomic-commit and WAL documentation gives Mercury a concrete embedded-store target for crash consistency.

Sources:

- <https://www.rfc-editor.org/rfc/rfc9420.html#section-12.4.4>
- <https://www.rfc-editor.org/rfc/rfc9420.html#section-16.8>
- <https://www.rfc-editor.org/rfc/rfc9750.html#section-5.2>
- <https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html>
- <https://learn.microsoft.com/en-us/azure/architecture/databases/guide/transactional-out-box-cosmos>
- <https://www.sqlite.org/atomiccommit.html>
- <https://www.sqlite.org/wal.html>

## Accepted Output

Accepted output enables:

```text
accepted = true
can_commit_membership_change_once = true
can_advance_epoch = true
can_send_welcome_from_outbox = true
binds_commit_key_package_welcome = true
uses_single_storage_transaction = true
uses_serializable_isolation = true
has_durable_commit = true
enforces_unique_constraints = true
has_idempotent_worker = true
has_crash_recovery = true
keeps_digest_only = true
plaintext_bytes_exposed = false
```

Rejected output never enables epoch advancement or Welcome delivery.

## Checked Conditions

The transaction witness requires:

- accepted Commit replay-store persistence
- accepted KeyPackage consume-store persistence
- accepted Welcome send outbox persistence
- no local-member-removal terminal Commit state
- matching group id across Commit replay, KeyPackage consumption, and outbox records
- matching Commit hash between Commit replay and outbox records
- matching KeyPackage hash between consumption and outbox records
- matching Welcome-send transaction digest between consumption and outbox records
- 32-byte group id, Commit hash, KeyPackage hash, Welcome-send digest, and membership-transaction digest
- non-negative creation timestamp
- one storage transaction for Commit replay, KeyPackage consumption, Welcome outbox, and transaction marker
- serializable isolation or an equivalent single-writer linearization guarantee
- durable commit before reporting success
- unique constraints for Commit hash, KeyPackage hash, and Welcome-send transaction digest
- idempotent outbox worker behavior
- crash-recovery reconciliation for queued-but-unsent Welcome rows
- zero plaintext metadata fields
- no existing membership transaction marker for the same digest

## Persisted Record

The prototype adapter persists only:

```text
group_id
commit_hash
key_package_hash
welcome_send_transaction_digest
membership_transaction_digest
created_at_s
plaintext_bytes_exposed
```

It intentionally does not persist raw Commit bytes, raw KeyPackage bytes, raw Welcome ciphertext, credentials, member profile metadata, delivery plaintext, MLS secrets, or provider secret material.

## Checked Fixtures

Prototype fixtures:

```text
mls_membership_transaction_ready
mls_membership_transaction_binding_rejected
mls_membership_transaction_storage_rejected
mls_membership_transaction_duplicate_rejected
mls_membership_transaction_plaintext_rejected
```

Backend commands:

```text
run_mls_membership_transaction_ready
run_mls_membership_transaction_binding_rejected
run_mls_membership_transaction_storage_rejected
run_mls_membership_transaction_duplicate_rejected
run_mls_membership_transaction_plaintext_rejected
```

## UI Contract

UI and platform code must not mark an MLS add-member operation as committed unless:

```text
commit_replay_store.accepted = true
key_package_consume_store.accepted = true
welcome_send_outbox.accepted = true
membership_transaction.accepted = true
```

Treat `BINDING_MISMATCH`, `ATOMIC_TRANSACTION_MISSING`, `SERIALIZABLE_ISOLATION_MISSING`, `DURABLE_COMMIT_MISSING`, `UNIQUE_CONSTRAINTS_MISSING`, `IDEMPOTENT_WORKER_MISSING`, `CRASH_RECOVERY_MISSING`, and `TRANSACTION_ALREADY_RECORDED` as backend storage hard stops. Never recover by applying the Commit locally, sending a Welcome inline, or trusting UI-side cached membership state.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_membership_transaction
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_membership_transaction_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_membership_transaction_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

Pick the production local database transaction primitive, likely an embedded encrypted SQLite path or a platform database with equivalent guarantees, then implement the adapter so Commit replay, KeyPackage consumption, Welcome outbox insertion, and transaction-marker persistence happen under one durable transaction.
