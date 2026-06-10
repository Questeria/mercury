# Security Research Cycle 22: Sealed Audit Event Chains

Generated: 2026-05-28

## Sources Reviewed

- RFC 9162, Certificate Transparency Version 2.0: <https://www.rfc-editor.org/rfc/rfc9162.html>
- Sigstore Rekor overview: <https://docs.sigstore.dev/logging/overview/>
- Sigstore security model: <https://docs.sigstore.dev/about/security/>
- RFC 9943, SCITT Architecture: <https://ftp.nic.ad.jp/rfc/authors/rfc9943.pdf>
- Schneier and Kelsey, Secure Audit Logs to Support Computer Forensics: <https://www.schneier.com/academic/archives/1999/05/secure_audit_logs_to.html>

## Finding

Secure messaging systems need auditability for security-critical state transitions, but ordinary logs become a liability because they often contain plaintext metadata and can be edited after device compromise. Mercury needs audit evidence that is useful for rollback detection, incident response, and cross-device consistency without becoming a second plaintext database.

The research points to a conservative Mercury contract:

- each critical event is sealed before storage
- event payloads and metadata are represented by digests, not plaintext
- the local event sequence is hash chained
- non-genesis events carry the previous event hash
- monotonic counters make local rollback harder to hide
- Merkle roots, inclusion proofs, and consistency proofs make append-only claims auditable
- signed checkpoints and timestamps make log heads non-repudiable
- transparency receipts and witness quorums reduce split-view and equivocation risk
- forward-secret key rotation limits what a later device compromise can reveal or alter

## Increment

Added `evaluate_sealed_audit_event_chain(...)` with checked fixtures and backend command envelopes. The new gate rejects:

- unknown audit event kind
- unknown audit anchor
- HMAC-only, plaintext, or unknown envelope suites
- plaintext event fields or payload bytes
- malformed event, record, Merkle leaf, or Merkle root digests
- sequence gaps or stale checkpoint sizes
- missing previous hash for non-genesis events
- missing or non-increasing monotonic counters
- missing device, actor, epoch, room-epoch, or critical-event binding
- unsealed events or AAD that does not bind event context
- missing signed checkpoints
- weak checkpoint signatures
- missing inclusion or consistency proof verification
- missing transparency receipt for transparency-backed anchors
- missing witness quorum or operator diversity
- missing append-only transactional storage guarantees
- missing rollback-resistant sealed local storage
- missing forward-secret rotation or previous-key deletion

## Security Impact

Mercury now has a backend boundary for digest-only, tamper-evident security audit events. Future production code can route device key changes, MLS Commits, account recovery, secure backup restore, AI grant changes, relay-source changes, and media-retention decisions through this gate before writing audit records or publishing transparency receipts.

This does not make audit logs magic. Clients still need production storage, checkpoint signing keys, witness operation, and monitoring. It does prevent the next implementation from silently turning auditability into a plaintext metadata leak or a rollback-prone local text log.

## Verification

Focused checks:

```powershell
cargo test -p mercury-core --test sealed_audit_event_chain
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_event_chain_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_event_chain_ready
```

## Next Research Target

The accepted-only sealed-audit store boundary was added in `docs/125_SEALED_AUDIT_EVENT_STORE.md` and researched in `docs/126_SECURITY_RESEARCH_CYCLE_23.md`.

Continue with production witness/checkpoint operations: checkpoint signing key lifecycle, witness gossip, public/private transparency deployment, split-view response, and privacy-preserving monitor queries.
