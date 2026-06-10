# Security Research Cycle 17: MLS Membership Transaction Atomicity

Generated: 2026-05-28

## Sources Reviewed

- RFC 9420, adding members and Welcome handling: <https://www.rfc-editor.org/rfc/rfc9420.html#section-12.4.4>
- RFC 9420, KeyPackage reuse: <https://www.rfc-editor.org/rfc/rfc9420.html#section-16.8>
- RFC 9750, Delivery Service delivery and ordering: <https://www.rfc-editor.org/rfc/rfc9750.html#section-5.2>
- AWS transactional outbox guidance: <https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html>
- Microsoft transactional outbox guidance: <https://learn.microsoft.com/en-us/azure/architecture/databases/guide/transactional-out-box-cosmos>
- SQLite atomic commit: <https://www.sqlite.org/atomiccommit.html>
- SQLite write-ahead logging: <https://www.sqlite.org/wal.html>

## Finding

The Welcome send outbox closed the direct gap between KeyPackage consumption and queued Welcome delivery, but production storage still needed one higher-level witness: all membership-change records must be part of the same durable transaction. If Commit replay persistence, KeyPackage consumption, Welcome outbox insertion, and the transaction marker can drift apart, a crash or retry could fork local epoch state, lose a Welcome, or create an ambiguous add-member outcome.

The storage contract also needs to say more than "there is an outbox." It needs unique constraints, serializable linearization, durable commit, idempotent workers, and startup recovery that reconciles committed-but-unsent Welcome rows.

## Increment

Added an MLS membership transaction witness that:

- accepts only after Commit replay-store, KeyPackage consume-store, and Welcome send-outbox decisions accept
- binds group id, Commit hash, KeyPackage hash, and Welcome-send transaction digest across all three component records
- requires one durable, serializable storage transaction
- requires unique constraints for Commit hashes, KeyPackage hashes, and Welcome-send transaction digests
- requires idempotent outbox worker behavior
- requires crash-recovery reconciliation for pending Welcome rows
- persists only a digest-only transaction marker
- rejects duplicate transaction markers and plaintext metadata
- exposes checked fixtures and backend commands for accepted, binding-rejected, storage-rejected, duplicate, and plaintext-blocked states

## Security Impact

This turns the membership-add flow into a testable production storage invariant:

```text
accepted Commit replay + consumed KeyPackage + queued Welcome + transaction marker
```

All four facts must commit together or none should be considered committed. The witness prevents UI/platform code from treating partially persisted MLS membership state as safe, and it gives the future production adapter exact constraints to satisfy.

## Verification

Focused checks:

```powershell
cargo test -p mercury-core --test mls_membership_transaction
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

Completed in `docs/115_SECURITY_RESEARCH_CYCLE_18.md` and `docs/114_LOCAL_STORE_DATABASE_SECURITY.md`. The next research target is concrete adapter selection: SQLCipher packaging/licensing, Rust binding options, FIPS-mode tradeoffs, platform key wrapping, and crash-recovery tests that preserve MLS transaction constraints.
