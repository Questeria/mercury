# Local Encrypted Store Prototype

Generated: 2026-05-28

## Status

Mercury now has a non-production encrypted local-store prototype in `mercury-core`:

```text
PrototypeEncryptedLocalStore
PrototypeLocalStoreRecord
PrototypeFileEncryptedLocalStore
```

The prototype implements `EncryptedLocalStoreAdapter`, so callers still pass through the existing `LocalStoreWriteRequest` and `put_local_store_record` policy gate. It stores only accepted typed payloads:

- sealed bytes for encrypted-only records
- hash digest bytes for hash-only records
- public metadata bytes for public metadata records

It has no plaintext payload path and no production cryptographic sealing implementation. The prototype is an integration surface for backend and platform work while production stores are still being designed.

## API Shape

```text
PrototypeEncryptedLocalStore::put_record(...)
PrototypeEncryptedLocalStore::get_record(...)
PrototypeEncryptedLocalStore::delete(...)
PrototypeEncryptedLocalStore::records()
PrototypeFileEncryptedLocalStore::put_record(...)
PrototypeFileEncryptedLocalStore::get_record(...)
PrototypeFileEncryptedLocalStore::delete(...)
PrototypeFileEncryptedLocalStore::record_path(...)
```

`put_record` returns the same `LocalStoreWriteDecision` used by the adapter boundary. Rejected writes do not mutate stored records.

`PrototypeFileEncryptedLocalStore` persists the same accepted record shape to disk under hex-encoded namespace and record IDs. It is still a prototype, but it exercises durable write/read/delete behavior without adding a plaintext storage path.

## Intended Use

Use this prototype for:

- client-core integration tests
- desktop/mobile binding development before OS-backed storage exists
- relay and receive-flow tests that need local persistence behavior
- UI simulations that need realistic store accept/reject outcomes without storing plaintext

Do not treat either prototype as production storage. The memory store is not durable. The file store is durable but still prototype-grade: it writes already-sealed/hash/public payload bytes, not final database pages, OS-keychain protected keys, or production cryptographic storage envelopes.

## Verification

The `prototype_encrypted_store` integration test covers:

- accepted sealed message ciphertext is stored and readable by locator
- rejected writes do not replace existing accepted records
- hash-only audit digest storage
- locator-scoped deletion

The `prototype_file_encrypted_store` integration test covers:

- accepted sealed message ciphertext persists across store reopen
- rejected plaintext records do not create files
- rejected policy writes do not replace existing durable records
- durable delete removes hash-only audit records

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test -p mercury-core prototype_store
cargo test -p mercury-core --test prototype_file_encrypted_store
cargo test --workspace
```

## Next Step

The durable store backend prototype is documented in `docs/45_DURABLE_STORE_BACKEND_PROTOTYPE.md`.
