# Sealed Audit Event Chain Gate

Generated: 2026-05-28

## Status

Mercury now has a sealed audit event-chain gate in `mercury-core`:

```text
SealedAuditEventKind
SealedAuditAnchorKind
SealedAuditEnvelopeSuite
SealedAuditEventChainInput
SealedAuditEventChainDecision
SealedAuditEventChainReason
evaluate_sealed_audit_event_chain(...)
```

This is not the production audit database or transparency service. It is the security contract that production audit storage, device history, recovery records, MLS group transitions, AI grant changes, and relay-source changes must satisfy before Mercury treats an audit event as appendable or verifiable.

## Accepted Event Contract

The accepted path requires:

- known security-critical event kind
- known local or transparency-backed anchor
- sealed authenticated-encryption envelope suite
- zero plaintext event fields
- zero plaintext payload bytes
- 32-byte event hash, record digest, Merkle leaf hash, and Merkle root hash
- event sequence equal to previous chain size
- previous event hash for non-genesis events
- monotonic counter presence and increase
- device, actor, and epoch digest binding
- room epoch digest binding for MLS Commit and backup-restore events
- explicit critical-event binding
- sealed event body with AAD binding event context
- signed checkpoint with timestamp and 64-byte-or-larger signature
- verified inclusion proof
- verified consistency proof
- transparency receipt for non-local Merkle anchors
- witness quorum and operator diversity for witnessed or public transparency anchors
- append-only, transactional storage
- rollback-resistant, sealed local store
- forward-secret audit key rotation
- deletion of previous key material

Accepted output enables:

```text
can_append_event = true
can_verify_inclusion = true
can_publish_transparency_receipt = true for transparency anchors
can_detect_rollback = true
tamper_evident = true
append_only = true
plaintext_bytes_exposed = false
```

## Reason Labels

Stable sealed-audit labels:

```text
ACCEPTED
EVENT_KIND_REJECTED
ANCHOR_REJECTED
ENVELOPE_SUITE_REJECTED
PLAINTEXT_EVENT_FORBIDDEN
DIGEST_SHAPE_REJECTED
SEQUENCE_REJECTED
PREVIOUS_HASH_MISSING
MONOTONIC_COUNTER_REJECTED
EVENT_BINDING_MISSING
SEAL_MISSING
CHECKPOINT_MISSING
CHECKPOINT_SIGNATURE_MISSING
MERKLE_PROOF_MISSING
TRANSPARENCY_RECEIPT_MISSING
WITNESS_QUORUM_MISSING
APPEND_ONLY_STORAGE_MISSING
ROLLBACK_PROTECTION_MISSING
FORWARD_SECRECY_MISSING
```

## Fixture Surface

Checked fixtures:

```text
sealed_audit_event_chain_ready
sealed_audit_event_chain_plaintext_rejected
sealed_audit_event_chain_rollback_rejected
sealed_audit_event_chain_witness_rejected
sealed_audit_event_chain_binding_rejected
```

Backend command envelopes:

```text
run_sealed_audit_event_chain_ready
run_sealed_audit_event_chain_plaintext_rejected
run_sealed_audit_event_chain_rollback_rejected
run_sealed_audit_event_chain_witness_rejected
run_sealed_audit_event_chain_binding_rejected
```

## Research Basis

- RFC 9162 defines Certificate Transparency v2 as an append-only Merkle log with inclusion and consistency proofs, and notes that the mechanism can transparently log binary data subject to inclusion criteria: https://www.rfc-editor.org/rfc/rfc9162.html
- Sigstore Rekor provides an immutable transparency log for signed metadata; auditors monitor append-only consistency and verifiers check inclusion: https://docs.sigstore.dev/logging/overview/
- Sigstore's security model signs Merkle tree heads with timestamps and requires monitoring for long-term trust: https://docs.sigstore.dev/about/security/
- RFC 9943 SCITT defines transparent signed statements, receipts, append-only logs, non-equivocation, and replayability for trustworthy supply-chain records: https://ftp.nic.ad.jp/rfc/authors/rfc9943.pdf
- Schneier and Kelsey describe forward-secure audit logging for untrusted machines, including limiting an attacker's ability to read, modify, or destroy prior log entries after compromise: https://www.schneier.com/academic/archives/1999/05/secure_audit_logs_to.html

## Verification

Run:

```powershell
cargo test -p mercury-core --test sealed_audit_event_chain
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_event_chain_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_event_chain_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

Build a production sealed-audit store adapter behind this gate. It should remain disabled until it can write only sealed digest-bound records, persist signed checkpoints, verify local inclusion and consistency proofs, retain witness receipts, rotate audit keys forward, and detect local rollback across desktop and mobile storage backends.
