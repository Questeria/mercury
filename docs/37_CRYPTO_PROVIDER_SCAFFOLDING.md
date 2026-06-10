# Crypto Provider Scaffolding

Generated: 2026-05-28

## Status

Mercury now has local-store crypto integration scaffolding in `mercury-core`:

```text
LocalStoreCryptoProvider
seal_local_store_plaintext(...)
open_local_store_record(...)
LocalStoreSealOutput
LocalStoreOpenRequest
LocalStoreOpenDecision
PrototypeLocalStoreCryptoProvider
```

The provider trait is intentionally behind the existing local-store sealing contract. A provider only receives an `AcceptedLocalStoreSeal` or `AcceptedLocalStoreOpen` after Mercury core validates locator, record kind, key scope, key binding, generation, nonce length, plaintext length, and policy decision.

## Prototype Provider

`PrototypeLocalStoreCryptoProvider` exists only for deterministic tests. It is not a production cipher and must not be used as one. Its job is to prove that:

- rejected seal requests do not call a crypto provider
- accepted seal requests produce nonce, sealed bytes, and tag-length metadata
- sealed outputs can pass into the existing encrypted-store adapter
- open requests validate nonce, sealed length, tag length, and expected plaintext length
- open requests return plaintext through the provider boundary only after validation

## Security Posture

This increment does not choose a production cryptographic library. Production Mercury should bind `LocalStoreCryptoProvider` to an audited AEAD or HPKE-style construction appropriate to the key scope, with misuse-resistant nonce generation and platform key protection.

## Verification

The `local_store_crypto_provider` integration test covers:

- valid seal and store flow
- plaintext length mismatch rejection before provider calls
- rejected policy and unsealable record rejection before provider calls
- valid open flow
- bad nonce and bad tag rejection before provider calls
- store adapter still rejecting sealed payloads for hash-only records

Run locally:

```powershell
cargo test -p mercury-core local_store_crypto
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Step

Prototype fixture coverage is documented in `docs/38_PROTOTYPE_FIXTURE_COVERAGE.md`. The next parallel increment should add backend session orchestration that connects startup, local-store crypto, relay delivery, and AI participant decisions into one deterministic non-UI session flow.
