# Encrypted Store Adapter

Generated: 2026-05-28

## Status

Mercury now has a concrete write adapter boundary for local encrypted storage in `mercury-core`.

```text
LocalStoreWriteRequest::evaluate() -> LocalStoreWriteDecision
put_local_store_record(&mut store, request) -> Result<LocalStoreWriteDecision, Store::Error>
EncryptedLocalStoreAdapter::put_accepted_record(AcceptedLocalStoreWrite)
```

The adapter shape is intentionally narrow. Storage implementations do not receive a normal write request. They receive `AcceptedLocalStoreWrite`, which can only be constructed inside `mercury-core` after local-store policy evaluation succeeds.

## Payload Classes

The adapter request accepts only typed payload classes:

- `LocalStorePayload::sealed(...)`
- `LocalStorePayload::hash_digest(...)`
- `LocalStorePayload::public_metadata(...)`

There is no plaintext payload variant. This does not prove that bytes were correctly sealed by a future key manager, but it keeps plaintext out of the durable-storage API shape and gives tests a stable place to enforce policy.

## Write Flow

```text
caller builds LocalStoreWriteRequest
  -> request evaluates LocalStoreRecordKind policy
  -> request validates payload class
  -> rejected decisions return without calling the store
  -> accepted decisions are wrapped as AcceptedLocalStoreWrite
  -> adapter receives only accepted writes
```

The first adapter test implementation is an in-memory verifier, not a production store. Production mobile and desktop adapters should bind this trait to platform-specific encrypted storage:

- iOS: Keychain and Secure Enclave backed key wrapping where available
- Android: Keystore backed wrapping keys where available
- Desktop: OS keychain or hardware-backed key protection when available

## Security Rules

The adapter adds a second gate on top of `LocalStoreRecordKind::policy()`:

- encrypted-only records must use sealed payloads
- hash-only records must use hash digest payloads
- public metadata records must use public metadata payloads
- rejected policy decisions do not call the storage adapter
- forbidden plaintext record kinds do not call the storage adapter

## Verification

The `encrypted_store_adapter` integration test covers:

- accepted sealed message ciphertext reaches the store
- rejected message policy never calls the store
- plaintext message record kinds never call the store
- hash-only audit records reject sealed payloads
- hash-only audit records accept hash digest payloads for rejected decisions

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Key Hierarchy Follow-Up

The key hierarchy and sealing contract are documented in `docs/19_KEY_HIERARCHY_AND_SEALING.md`.

## Next Step

The next increment should define the identity and device trust boundary: account identity keys, device keys, verification state, and key-change decisions before any UI trust indicators are built.
