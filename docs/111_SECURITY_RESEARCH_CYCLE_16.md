# Security Research Cycle 16: Welcome Send Atomicity

Generated: 2026-05-28

## Sources Reviewed

- RFC 9420, KeyPackage reuse: <https://www.rfc-editor.org/rfc/rfc9420.html#section-16.8>
- RFC 9420, adding members / Welcome after Commit: <https://www.rfc-editor.org/rfc/rfc9420.html#section-12.4.4>
- RFC 9750, Delivery Service ordering and delivery: <https://www.rfc-editor.org/rfc/rfc9750.html#section-5.2>
- AWS transactional outbox guidance: <https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html>
- Microsoft transactional outbox guidance: <https://learn.microsoft.com/en-us/azure/architecture/databases/guide/transactional-out-box-cosmos>

## Finding

The previous increment made KeyPackage consumption one-time before Welcome sending, but there was still a crash window: a backend could consume a KeyPackage and fail before durably queuing the corresponding Welcome. A retry could then either lose the Welcome or try to reinterpret consumed state outside the original transaction.

MLS also makes Commit ordering security-relevant. A Welcome send must be bound to a winning accepted Commit and must not be sent inline before the durable store has committed the outbox row.

## Increment

Added an MLS Welcome send outbox boundary that:

- accepts only after KeyPackage consume-store and Commit admission accept
- persists digest-only Welcome send transaction records
- binds group id, KeyPackage hash, added-member ref, Welcome-send transaction digest, Commit hash, Welcome ciphertext hash, delivery route id, replay token, and TTL
- rejects malformed digest/route/token shapes and invalid timestamps
- rejects duplicate Welcome-send transaction digests
- rejects a second outbox row for the same KeyPackage hash
- rejects plaintext metadata
- exposes checked fixtures and backend commands for accepted, consume-rejected, duplicate-transaction, KeyPackage-already-queued, bad-shape, and plaintext-blocked states

## Security Impact

This closes the sender-side crash/race gap between one-time KeyPackage consumption and Welcome delivery. Future production code now has a checked adapter boundary that can be implemented as one transaction:

```text
accepted Commit + consumed KeyPackage + queued Welcome outbox row
```

Workers can then retry delivery from durable outbox rows without generating a second Welcome for the same KeyPackage or sending a Welcome for a losing Commit.

## Verification

Focused checks:

```powershell
cargo test -p mercury-core --test mls_welcome_send_outbox
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Full gate:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Research Target

Completed by `docs/112_MLS_MEMBERSHIP_TRANSACTION.md` and `docs/113_SECURITY_RESEARCH_CYCLE_17.md`: Mercury now has a checked membership transaction witness for the transaction primitive, cross-record bindings, unique constraints, crash-recovery reconciliation behavior, and idempotent worker semantics. The next research target is the concrete encrypted production database adapter.
