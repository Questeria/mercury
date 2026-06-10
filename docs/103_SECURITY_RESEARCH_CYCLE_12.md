# Security Research Cycle 12: MLS Commit Admission

Generated: 2026-05-28

## Primary Source Refresh

- RFC 9420, Messaging Layer Security: <https://www.rfc-editor.org/rfc/rfc9420.html>
- RFC 9750, MLS Architecture: <https://www.rfc-editor.org/rfc/rfc9750>

The relevant security takeaway is that Commit processing is the sensitive MLS epoch-advance point. A malformed proposal list, wrong epoch, invalid sender/authentication state, bad update path, bad transcript confirmation, replayed Commit, or losing concurrent Commit can fork a group or apply membership changes the UI should never trust.

## Implementation Increment

Added an MLS Commit admission gate that:

- rejects Commits not bound to the current group epoch
- rejects non-member senders for regular Commits and malformed external Commit sender state
- rejects invalid Commit signature or membership-tag evidence
- rejects invalid proposal lists, missing referenced proposals, application-policy failures, duplicate proposal targets, committer self-update, and committer self-remove
- rejects missing required update paths, invalid update-path leaves, invalid parent hashes, undecryptable path secrets, ratchet-tree hash mismatches, and provisional context mismatches
- rejects epoch-secret, transcript-hash, and confirmation-tag failures
- rejects losing deterministic tie-breaks, malformed or replayed Commit hashes, and plaintext Commit metadata
- exposes checked fixtures and backend commands for accepted and blocked Commit states
- models local-member removal as an accepted terminal state that can apply the Commit but cannot continue the group

## Security Effect

This closes the group-state epoch-advance gap after KeyPackage and Welcome admission. UI and platform code can inspect Commit readiness, but cannot move local membership, tree state, send state, or group epoch forward unless backend-admitted Commit evidence says the Commit is current, authenticated, proposal-valid, transcript-confirmed, ordered, replay-fresh, and plaintext-free.

## Verification

Focused checks passed:

```powershell
cargo test -p mercury-core --test mls_commit_admission
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Full repo preflight remains the final merge gate:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
