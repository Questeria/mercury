# Security Research Cycle 10: MLS KeyPackage Admission

Generated: 2026-05-28

## Primary Source Refresh

- RFC 9420, Messaging Layer Security: <https://www.rfc-editor.org/rfc/rfc9420.html>

The relevant security takeaway is that adding a client to an MLS group depends on externally supplied KeyPackage material that carries protocol version, cipher suite, init key, leaf node, extensions, lifetime, and credentials. Mercury should treat this as an adversarial admission object, not as UI-owned invite metadata.

## Implementation Increment

Added an MLS KeyPackage admission gate that:

- rejects if group-chat readiness or membership-change capability is not accepted
- binds protocol version and suite to the current group policy
- rejects invalid leaf nodes, leaf signatures, KeyPackage signatures, credentials, missing capabilities, and unsupported credentials
- rejects bad, expired, future, or too-long lifetimes
- rejects non-KeyPackage leaf sources, unsupported extensions, init/encryption key reuse, malformed init/hash sizes, reused KeyPackage hashes, and plaintext identity fields
- emits stable reason codes and labels for UI/backend command routing
- exposes checked prototype fixtures and backend commands for accepted and blocked admission states

## Security Effect

This closes another pre-UI membership hole: a future desktop/mobile UI can show invite state, but it cannot enable add-member or Welcome-send paths unless backend admission says the KeyPackage is current, group-bound, suite-bound, signature/credential-valid, replay-fresh, and plaintext-free.

## Verification

Focused checks passed:

```powershell
cargo test -p mercury-core --test mls_key_package_admission
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Full repo preflight must remain the final merge gate for this cycle:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
