# Security Research Cycle 23: Accepted-Only Sealed Audit Storage

Generated: 2026-05-28

## Sources Reviewed

- NIST SP 800-92, Guide to Computer Security Log Management: <https://csrc.nist.gov/pubs/sp/800/92/final>
- Trillian verifiable data structures: <https://transparency.dev/verifiable-data-structures/>
- Trillian transparent logging guide: <https://google.github.io/trillian/docs/TransparentLogging.html>
- Transparency Logs as a Verifiable Transport Layer: <https://transparency.dev/articles/logs-a-verifiable-transport-layer/>
- transparency-dev witness package overview: <https://pkg.go.dev/github.com/transparency-dev/witness>
- Schneier and Kelsey, Secure Audit Logs to Support Computer Forensics: <https://www.schneier.com/academic/archives/1999/05/secure_audit_logs_to.html>

## Finding

The event-chain gate was necessary but not sufficient. Research points to a second boundary: the store must preserve the same append-only, digest-only, checkpoint-bound properties and must not become a bypass path where unaccepted audit events or plaintext metadata are written "temporarily."

Key design conclusions:

- NIST treats log management as an infrastructure and process problem, including confidentiality, integrity, and availability of logs, not just log generation.
- Trillian-style verifiable logs rely on Merkle inclusion and consistency proofs, but clients must still retain checkpoints and verify later checkpoints extend earlier ones.
- Transparent log admission criteria should be explicit and machine-checkable, especially when logs carry arbitrary or sensitive application data.
- Witnesses defend against split views by storing prior checkpoints and countersigning only append-only evolutions.
- Forward-secure audit-log work argues that logs on compromised machines should limit the attacker's ability to read, modify, or destroy past entries undetectably.

## Increment

Added the sealed audit event store boundary:

- accepted-only `SealedAuditEventStoreAdapter`
- accepted wrapper type for production adapters
- in-memory prototype store
- duplicate event sequence rejection
- duplicate event hash rejection
- duplicate checkpoint id rejection
- rollback sequence rejection
- plaintext metadata rejection
- checkpoint binding checks
- transparency receipt binding checks
- append-only guard checks
- checked simulator fixtures and backend command envelopes

## Security Impact

Mercury can now model both halves of sealed audit persistence:

1. the event-chain gate determines whether a security-critical event is safe to append
2. the store boundary determines whether the accepted event can actually be persisted without replay, rollback, duplicate checkpoint, or plaintext leakage

This makes the future production adapter safer to implement because its write method receives an accepted wrapper instead of raw event data.

## Verification

Focused checks:

```powershell
cargo test -p mercury-core --test sealed_audit_event_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_event_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_event_store_ready
```

## Next Research Target

This target is implemented by `docs/127_SEALED_AUDIT_WITNESS_CHECKPOINT.md` and `docs/128_SECURITY_RESEARCH_CYCLE_24.md`.

The next research target is the production service adapter behind the witness/checkpoint gate: witness network configuration, checkpoint publisher durability, monitor private-retrieval protocol selection, and operator alerting/runbook behavior for split-view evidence.
