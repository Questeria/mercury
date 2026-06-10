# Sealed Audit Event Store Boundary

Generated: 2026-05-28

## Status

Mercury now has an accepted-only sealed audit event store boundary in `mercury-core`:

```text
SealedAuditEventStoreReason
SealedAuditEventStoreWrite
SealedAuditEventStoreRecord
SealedAuditEventStoreDecision
AcceptedSealedAuditEventStoreWrite
SealedAuditEventStoreAdapter
PrototypeSealedAuditEventStore
evaluate_sealed_audit_event_store_write(...)
put_sealed_audit_event_record(...)
```

This is the adapter contract behind `evaluate_sealed_audit_event_chain(...)`. Production storage should receive only `AcceptedSealedAuditEventStoreWrite`, so rejected chain events, plaintext metadata, duplicate event hashes, duplicate checkpoint ids, and rollback sequences cannot silently enter the audit database.

## Accepted Store Contract

The accepted write requires:

- an accepted sealed audit event-chain decision
- event sequence and chain decision alignment
- 32-byte event hash, previous event hash for non-genesis events, record digest, Merkle root hash, and checkpoint id
- 64-byte-or-larger checkpoint signature
- sealed payload length greater than zero
- event kind and anchor kind matching the accepted chain decision
- checkpoint binding to the chain
- zero plaintext metadata fields
- transparency receipt and receipt-to-checkpoint binding for transparency-backed anchors
- witness receipt for witnessed or public transparency anchors
- explicit append-only guard
- no duplicate event sequence
- no duplicate event hash
- no duplicate checkpoint id
- no rollback to a sequence below the highest stored event sequence

Accepted output sets:

```text
persisted_record = true
can_publish_receipt = true for transparency anchors
can_detect_replay = true
append_only = true
keeps_digest_only = true
keeps_plaintext_metadata = false
plaintext_bytes_exposed = false
```

## Reason Labels

Stable sealed-audit store labels:

```text
ACCEPTED
CHAIN_REJECTED
DUPLICATE_SEQUENCE
DUPLICATE_EVENT_HASH
DUPLICATE_CHECKPOINT
ROLLBACK_SEQUENCE
BAD_DIGEST_SHAPE
PLAINTEXT_METADATA_FORBIDDEN
CHECKPOINT_BINDING_MISSING
TRANSPARENCY_RECEIPT_MISSING
APPEND_ONLY_GUARD_MISSING
```

## Fixture Surface

Checked fixtures:

```text
sealed_audit_event_store_ready
sealed_audit_event_store_chain_rejected
sealed_audit_event_store_duplicate_rejected
sealed_audit_event_store_rollback_rejected
sealed_audit_event_store_plaintext_rejected
```

Backend command envelopes:

```text
run_sealed_audit_event_store_ready
run_sealed_audit_event_store_chain_rejected
run_sealed_audit_event_store_duplicate_rejected
run_sealed_audit_event_store_rollback_rejected
run_sealed_audit_event_store_plaintext_rejected
```

## Security Impact

This closes the gap between "the event would be safe to append" and "the event actually entered a store that preserves the safety properties." The prototype store is intentionally small, but it proves the production boundary:

- storage adapters should not re-evaluate policy after the fact
- rejected chain decisions do not mutate storage
- duplicate and rollback attempts are rejected before persistence
- checkpoint and receipt binding remain part of the write contract
- plaintext metadata is rejected and not retained

The remaining production work is to implement a durable encrypted append-only adapter, checkpoint signing key lifecycle, witness submission, monitor queries, and recovery/repair behavior behind this contract.

## Verification

Run:

```powershell
cargo test -p mercury-core --test sealed_audit_event_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_event_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_event_store_ready
```
