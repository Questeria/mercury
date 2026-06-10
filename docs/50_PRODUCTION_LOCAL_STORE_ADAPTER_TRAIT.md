# Production Local Store Adapter Trait

Generated: 2026-05-28

## Status

Mercury now has a production local-store adapter trait shape in `mercury-core`:

```text
ProductionLocalStoreAdapter
open_production_local_store(...)
replay_production_local_store_wal(...)
put_production_local_store_record(...)
read_production_local_store_record(...)
LocalStoreReadRecord
```

This is the backend boundary a real encrypted database implementation should satisfy after the keychain/keystore, unlock, and production-open gates accept.

## Open Flow

`open_production_local_store(...)` evaluates `LocalStoreProductionOpenInput` and only calls `ProductionLocalStoreAdapter::open_database(...)` when the decision is accepted.

Rejected open decisions return to the caller without opening the adapter. That keeps app-lock, recovery, migration, destructive-repair, and crash-recovery branches outside the database read path.

## WAL Replay Flow

`replay_production_local_store_wal(...)` only calls `ProductionLocalStoreAdapter::replay_wal(...)` when the prior production-open decision exposes:

```text
can_replay_wal = true
```

Accepted clean opens do not replay the WAL, and ordinary rejected opens do not replay it. The WAL path is explicit because replay is a state-changing recovery operation.

## Record Flow

Production record writes still use the existing local-store policy gate:

```text
LocalStoreWriteRequest
  -> evaluate_local_store_write_request(...)
  -> AcceptedLocalStoreWrite
  -> ProductionLocalStoreAdapter::put_accepted_record(...)
```

Rejected plaintext, rejected policy, and payload-class mismatches never call the adapter write path.

Reads return `LocalStoreReadRecord`, an owned sealed/hash/public record shape. There is still no plaintext payload variant.

## Prototype Coverage

`PrototypeEncryptedLocalStore` and `PrototypeFileEncryptedLocalStore` implement `ProductionLocalStoreAdapter` as conformance harnesses. They are still prototypes, not production database implementations.

## Verification

Run:

```powershell
cargo test -p mercury-core --test local_store_production_adapter
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers accepted-only open, no open on rejected decisions, explicit WAL replay, no WAL replay on clean open, accepted sealed writes, rejected plaintext writes, production reads, and prototype adapter conformance.

## Next Backend Step

The production-store session prototype is documented in `docs/51_PRODUCTION_STORE_SESSION_PROTOTYPE.md`. A future production adapter must also satisfy the local database security profile in `docs/114_LOCAL_STORE_DATABASE_SECURITY.md` and the adapter selection/provenance gate in `docs/116_LOCAL_STORE_DATABASE_ADAPTER_SELECTION.md`.
