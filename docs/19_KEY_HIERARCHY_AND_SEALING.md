# Key Hierarchy And Sealing

Generated: 2026-05-28

## Status

Mercury now has a typed local-store key hierarchy and sealing contract in `mercury-core`.

```text
LocalStoreKeyDescriptor
LocalStoreKeyBinding
LocalStoreSealRequest::evaluate() -> LocalStoreSealingDecision
build_sealed_local_store_write_request(...) -> Result<LocalStoreWriteRequest, LocalStoreSealingDecision>
```

This is not a cryptographic implementation. It is the pre-crypto contract that future audited sealing code must satisfy before bytes are allowed to become `LocalStorePayload::sealed(...)`.

## Key Scopes

The contract binds local-store keys to the same scopes used by the persistence policy:

- account root
- device local
- conversation
- room epoch
- media
- audit

Every `LocalStoreKeyDescriptor` includes:

- `scope`
- `suite`
- `generation`
- `binding`

The binding pins the key to account, device, conversation, and room-epoch dimensions as appropriate. For example, a room-epoch message ciphertext must use a room-epoch key with a positive account id length, conversation id length, and room epoch.

## Sealing Rules

The sealing gate rejects:

- empty locators
- records that are never-store, hash-only, or public-metadata class
- keys with the wrong scope for the record kind
- malformed key bindings
- nonpositive key generations
- nonce lengths that do not match the sealing suite
- nonpositive plaintext lengths
- missing or rejected policy decisions for policy-gated records

The current suite marker is `MercuryLocalStoreV1`, with a 24-byte nonce and 16-byte authentication tag contract. The actual audited algorithm and library are still a future implementation decision.

## Store Flow

```text
caller builds LocalStoreSealRequest
  -> request validates key scope, binding, generation, nonce, plaintext length, and policy
  -> future crypto code seals the bytes
  -> build_sealed_local_store_write_request creates LocalStoreWriteRequest
  -> encrypted-store adapter evaluates and writes accepted records only
```

This keeps three concerns separate:

- policy decides whether the record is allowed
- sealing decides whether the key and context are correct
- storage accepts only already-approved sealed/hash/public payload classes

## Verification

The `local_store_sealing` integration test covers:

- room-epoch message ciphertext sealing
- wrong key scope rejection
- malformed room-epoch binding rejection
- plaintext and hash-only records rejected from sealing
- nonce length, key generation, and rejected-policy checks
- valid seal request conversion into a sealed write request
- conversation secret sealing without requiring message policy

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Identity Device Trust Follow-Up

The identity and device trust boundary is documented in `docs/20_IDENTITY_DEVICE_TRUST.md`.

## Next Step

The next increment should define the first key transparency proof boundary: inclusion/consistency proof status, proof freshness, and how those facts feed `KeyTransparencyState`.
