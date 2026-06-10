# Security Research Cycle 11: MLS Welcome Admission

Generated: 2026-05-28

## Primary Source Refresh

- RFC 9420, Messaging Layer Security: <https://www.rfc-editor.org/rfc/rfc9420.html>
- RFC 9750, MLS Architecture: <https://www.rfc-editor.org/rfc/rfc9750>

The relevant security takeaway is that receiving a Welcome is as dangerous as admitting a KeyPackage: a malicious server, stale Commit, malformed ratchet tree, wrong PSK state, bad GroupInfo signature, bad confirmation tag, or replayed Welcome could fork local group state before the UI ever sends a message.

## Implementation Increment

Added an MLS Welcome admission gate that:

- rejects missing matching encrypted group secrets and ciphersuite mismatches
- rejects group-secret, GroupInfo, PSK, and GroupInfo-signature failures
- requires locally unique group ids
- requires confidential ratchet tree availability, tree-hash match, valid parent hashes, valid leaves/unmerged leaves, and unique encryption keys
- requires the local leaf to match the admitted KeyPackage
- requires valid path secret, epoch secret, confirmed transcript hash, and confirmation tag
- rejects Welcomes tied to losing Commits, bad epochs, reinit-PSK epoch mismatch, malformed or replayed Welcome hashes, and plaintext group metadata
- exposes checked fixtures and backend commands for accepted and blocked Welcome states

## Security Effect

This closes another membership bootstrap hole: a UI can display invite status, but it cannot open a newly joined group unless backend-admitted Welcome evidence says the group secrets, GroupInfo, tree, transcript, replay state, and Commit ordering are coherent.

## Verification

Focused checks passed:

```powershell
cargo test -p mercury-core --test mls_welcome_admission
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Full repo preflight remains the final merge gate:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
