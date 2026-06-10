# MLS Welcome Send Outbox

Generated: 2026-05-28

## Status

Mercury now has a sender-side MLS Welcome-send outbox boundary in `mercury-core`:

```text
MlsWelcomeSendOutboxWrite
MlsWelcomeSendOutboxDecision
MlsWelcomeSendOutboxAdapter
put_mls_welcome_send_outbox_record(...)
```

The boundary queues a Welcome for delivery only after KeyPackage consumption and Commit admission both accept. It turns the existing `welcome_send_transaction_digest` from a shape-checked promise into an actual digest-only outbox record.

## Research Basis

RFC 9420 requires KeyPackages to be one-use because reuse can enable replay attacks. RFC 9420 and RFC 9750 also make Commit ordering and Delivery Service behavior security-relevant: a Welcome must be tied to a winning Commit, not an attempted or losing one. Transactional outbox guidance from AWS and Microsoft maps this to the storage layer: state changes and outgoing messages should be persisted atomically, and send workers should publish only committed outbox rows.

Sources:

- <https://www.rfc-editor.org/rfc/rfc9420.html#section-16.8>
- <https://www.rfc-editor.org/rfc/rfc9420.html#section-12.4.4>
- <https://www.rfc-editor.org/rfc/rfc9750.html#section-5.2>
- <https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html>
- <https://learn.microsoft.com/en-us/azure/architecture/databases/guide/transactional-out-box-cosmos>

## Accepted Output

Accepted output enables:

```text
accepted = true
can_enqueue_welcome_once = true
can_send_welcome_after_commit = true
consumes_key_package = true
binds_welcome_send_transaction = true
binds_commit = true
binds_delivery_route = true
prevents_duplicate_outbox = true
keeps_digest_only = true
plaintext_bytes_exposed = false
```

Rejected output never enables Welcome delivery.

## Checked Conditions

The send outbox requires:

- accepted MLS KeyPackage consume-store decision
- accepted MLS Commit admission decision
- `can_send_welcome_once = true`
- `can_apply_commit = true`
- `can_initialize_epoch = true`
- `can_continue_group = true`
- 32-byte group id
- 32-byte KeyPackage hash
- 32-byte added-member reference
- 32-byte Welcome-send transaction digest
- 32-byte Commit hash
- 32-byte Welcome ciphertext hash
- 32-byte delivery route id
- 32-byte replay token
- non-negative creation timestamp
- expiration timestamp greater than creation timestamp
- zero plaintext metadata fields
- no existing outbox record for the same Welcome-send transaction digest
- no existing outbox record for the same KeyPackage hash

## Persisted Record

The prototype adapter persists only:

```text
group_id
key_package_hash
added_member_ref
welcome_send_transaction_digest
commit_hash
welcome_ciphertext_hash
delivery_route_id
replay_token
created_at_s
expires_at_s
plaintext_bytes_exposed
```

It intentionally does not persist raw Welcome bytes, raw KeyPackage bytes, credential material, member profile metadata, delivery plaintext, or provider secret material.

## Checked Fixtures

Prototype fixtures:

```text
mls_welcome_send_outbox_ready
mls_welcome_send_outbox_consume_rejected
mls_welcome_send_outbox_duplicate_transaction_rejected
mls_welcome_send_outbox_key_package_queued
mls_welcome_send_outbox_bad_shape
mls_welcome_send_outbox_plaintext_rejected
```

Backend commands:

```text
run_mls_welcome_send_outbox_ready
run_mls_welcome_send_outbox_consume_rejected
run_mls_welcome_send_outbox_duplicate_transaction_rejected
run_mls_welcome_send_outbox_key_package_queued
run_mls_welcome_send_outbox_bad_shape
run_mls_welcome_send_outbox_plaintext_rejected
```

## UI Contract

UI and platform code must not send or mark a Welcome as queued unless all three conditions are true:

```text
key_package_admission.accepted = true
key_package_consume_store.accepted = true
welcome_send_outbox.accepted = true
membership_transaction.accepted = true
```

Treat `WELCOME_SEND_TRANSACTION_ALREADY_QUEUED` and `KEY_PACKAGE_ALREADY_QUEUED` as hard duplicate or crash-recovery states. Treat consume rejection, Commit admission rejection, bad digest shapes, and plaintext metadata rejection as backend/provider integration failures. Never recover by sending a Welcome inline outside the backend outbox.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_welcome_send_outbox
cargo test -p mercury-core --test mls_membership_transaction
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_welcome_send_outbox_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_membership_transaction_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_send_outbox_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_membership_transaction_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

The follow-on membership transaction witness now checks that Commit replay, KeyPackage consumption, Welcome outbox insertion, and a transaction marker are bound under one durable, serializable transaction. The next production step is implementing the actual encrypted database adapter behind that witness.
