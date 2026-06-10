# MLS KeyPackage Consume Store

Generated: 2026-05-28

## Status

Mercury now has a sender-side MLS KeyPackage consumption boundary in `mercury-core`:

```text
MlsKeyPackageConsumeStoreWrite
MlsKeyPackageConsumeStoreDecision
MlsKeyPackageConsumeStoreAdapter
put_mls_key_package_consumption_record(...)
```

The boundary persists accepted KeyPackage consumption only after KeyPackage admission accepts. It is intentionally keyed by KeyPackage hash globally, not by group, so a KeyPackage cannot be consumed for one group and then reused for another.

## Research Basis

RFC 9420 says KeyPackages are intended for one use and should be removed from publication after use because reuse can lead to replay attacks. RFC 9750 places one-use responsibility on delivery-service behavior and recommends avoiding last-resort KeyPackage reuse, rotating last-resort keys after use, and deleting the `init_key` private component after Welcome processing.

Sources:

- <https://www.rfc-editor.org/rfc/rfc9420.html#section-16.8>
- <https://datatracker.ietf.org/doc/html/rfc9750#section-5.1>

## Accepted Output

Accepted output enables:

```text
accepted = true
can_consume_key_package_once = true
can_send_welcome_once = true
prevents_key_package_reuse = true
binds_added_member_ref = true
binds_welcome_send_transaction = true
keeps_digest_only = true
plaintext_bytes_exposed = false
```

Rejected output never enables Welcome sending.

## Checked Conditions

The consume store requires:

- accepted MLS KeyPackage admission
- `can_add_member = true`
- `can_send_welcome = true`
- `prevents_key_reuse = true`
- 32-byte group id
- 32-byte KeyPackage hash
- 32-byte added-member reference
- 32-byte Welcome-send transaction digest
- non-negative consumption timestamp
- zero plaintext metadata fields
- no existing record for the same KeyPackage hash, even across different groups

## Persisted Record

The prototype adapter persists only:

```text
group_id
key_package_hash
added_member_ref
welcome_send_transaction_digest
consumed_at_s
plaintext_bytes_exposed
```

It intentionally does not persist raw KeyPackage bytes, credential material, init private keys, Welcome plaintext, member profile metadata, or provider secret material.

## Checked Fixtures

Prototype fixtures:

```text
mls_key_package_consume_store_ready
mls_key_package_consume_store_admission_rejected
mls_key_package_consume_store_duplicate_rejected
mls_key_package_consume_store_bad_shape
mls_key_package_consume_store_plaintext_rejected
```

Backend commands:

```text
run_mls_key_package_consume_store_ready
run_mls_key_package_consume_store_admission_rejected
run_mls_key_package_consume_store_duplicate_rejected
run_mls_key_package_consume_store_bad_shape
run_mls_key_package_consume_store_plaintext_rejected
```

## UI Contract

UI and platform code must not send a Welcome unless both conditions are true:

```text
key_package_admission.accepted = true
key_package_consume_store.accepted = true
```

Treat `KEY_PACKAGE_ALREADY_CONSUMED` as a hard replay or race stop. Treat `KEY_PACKAGE_ADMISSION_REJECTED`, bad digest shapes, and plaintext metadata rejection as backend/provider integration failures, not retryable UI states. Never recover by requesting server plaintext identity data or by reusing a KeyPackage in another group.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_key_package_consume_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_key_package_consume_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_key_package_consume_store_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

The follow-on `MlsWelcomeSendOutboxAdapter` now persists the corresponding Welcome-send outbox record after accepted KeyPackage consumption and accepted Commit admission. Production MLS provider integration should connect real KeyPackage fetch/admission, atomic consume-store check-and-put, Commit linearization, and durable Welcome outbox writes in one storage transaction.
