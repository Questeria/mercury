# Anonymous Nullifier Store

Generated: 2026-05-28

## Status

Mercury now has a prototype backend boundary for anonymous nullifier persistence:

```text
AnonymousNullifierStoreWrite
AnonymousNullifierStoreDecision
AnonymousNullifierStoreReason
AnonymousNullifierStoreAdapter
PrototypeAnonymousNullifierStore
put_anonymous_nullifier_record(...)
```

This is not a production private-set database. It is the accepted-only adapter contract a production nullifier store must satisfy before Mercury records anonymous rate-limit/nullifier state.

## Accepted Store Write

Accepted writes require:

- accepted `AnonymousRateLimitNullifierDecision`
- 32-byte opaque nullifier
- 32-byte redemption context digest
- 32-byte credential context digest
- valid window bounds
- presentation count below the presentation limit
- zero plaintext metadata fields
- no existing store record for the same nullifier

Accepted output persists a record and keeps:

```text
keeps_context_digest_only = true
plaintext_bytes_exposed = false
```

## Rejection Classes

Stable rejection labels:

```text
NULLIFIER_GATE_REJECTED
BAD_NULLIFIER
BAD_REDEMPTION_CONTEXT_DIGEST
BAD_CREDENTIAL_CONTEXT_DIGEST
BAD_WINDOW
PRESENTATION_LIMIT_EXCEEDED
PLAINTEXT_METADATA_FORBIDDEN
NULLIFIER_ALREADY_RECORDED
```

## Fixtures And Commands

Checked prototype fixtures:

```text
anonymous_nullifier_store_ready
anonymous_nullifier_store_replay_rejected
anonymous_nullifier_store_plaintext_metadata_rejected
```

Backend command envelopes:

```text
run_anonymous_nullifier_store_ready
run_anonymous_nullifier_store_replay_rejected
run_anonymous_nullifier_store_plaintext_metadata_rejected
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test anonymous_nullifier_store
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_nullifier_store_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

Study production private-set/nullifier database designs and map this adapter contract to a durable store that does not expose member identity, group metadata, raw redemption contexts, or plaintext abuse-control metadata.
