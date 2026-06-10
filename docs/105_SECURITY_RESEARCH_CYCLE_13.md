# Security Research Cycle 13: MLS Commit Replay Store

Generated: 2026-05-28

## Primary Source Refresh

- RFC 9420, Messaging Layer Security: <https://www.rfc-editor.org/rfc/rfc9420.html>
- RFC 9750, MLS Architecture: <https://www.rfc-editor.org/rfc/rfc9750>

The relevant security takeaway is that a Commit is not ordinary message data. Once accepted, it changes epoch state, transcript state, ratchet-tree state, and membership state. Applying the same accepted Commit twice, or applying it after local recovery without a durable replay check, can produce exactly the kind of forked group state that MLS deployments must avoid.

## Implementation Increment

Added an MLS Commit replay-store boundary that:

- accepts only after MLS Commit admission accepts
- rejects rejected admission decisions before storage is touched
- rejects malformed group ids and Commit hashes
- rejects non-positive epochs and invalid application timestamps
- rejects plaintext metadata fields
- persists digest-only accepted Commit records
- rejects duplicate Commit hashes per group
- carries terminal local-member removal state without allowing continued group use
- exposes checked fixtures and backend commands for accepted, rejected, duplicate, removed-member, and plaintext-blocked states

## Security Effect

This closes the durable replay gap after Commit admission. A UI or platform adapter can now ask whether a Commit was admitted and whether its accepted digest was durably recorded before applying the Commit once. The replay store does not need plaintext Commit bytes and does not expose member metadata to UI state.

## Verification

Focused checks passed during development:

```powershell
cargo test -p mercury-core --test mls_commit_replay_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Full repo preflight remains the final merge gate:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Research Target

Move from prototype gates to production provider mapping: select and document the MLS implementation adapter, the supported classical and post-quantum ciphersuite classes, provider transcript evidence, and the storage interface that binds Commit admission plus replay persistence to local epoch advancement.
