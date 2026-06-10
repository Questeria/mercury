# Security Research Cycle 07

Generated: 2026-05-28

## Scope

This cycle refreshed Mercury's command-bound anonymous group relay posture after the nullifier gate. The goal was to ensure UI/platform clients can exercise anonymous credential, proof, nullifier, and relay-envelope states through the same plaintext-free backend command path used by other checked security gates.

## Primary Sources Checked

- IETF RFC 9420, "The Messaging Layer Security (MLS) Protocol": https://www.rfc-editor.org/rfc/rfc9420
- IETF RFC 9750, "The Messaging Layer Security (MLS) Architecture": https://www.rfc-editor.org/rfc/rfc9750
- IETF RFC 9576, "The Privacy Pass Architecture": https://www.rfc-editor.org/rfc/rfc9576
- IETF RFC 9578, "Privacy Pass Issuance Protocols": https://www.rfc-editor.org/rfc/rfc9578
- IETF RFC 9497, "Oblivious Pseudorandom Functions (OPRFs) Using Prime-Order Groups": https://www.rfc-editor.org/rfc/rfc9497
- NIST CSRC, "Announcing Approval of Three Federal Information Processing Standards (FIPS) for Post-Quantum Cryptography": https://csrc.nist.gov/News/2024/postquantum-cryptography-fips-approved

## Finding

Mercury had checked anonymous issuer-trust, group-proof, nullifier, and group relay envelope fixtures, but those states were still prototype-fixture calls instead of command-gated backend operations.

That left a wiring gap: a future UI could inspect success and failure states, but it could not yet request them through the same command envelope that rejects bad command IDs, remote AI actors, local AI misuse, and plaintext command payloads.

## Implemented Increment

`PrototypeBackendCommandKind` now includes stable labels and codes for:

- anonymous credential issuer trust accepted, transparency required, revoked, and partitioning-metadata rejection
- anonymous credential issuer trust witness/auditor rejection
- anonymous group membership proof accepted, high-security PQ required, replay rejected, route binding required, and plaintext member identity rejection
- anonymous rate-limit nullifier accepted, replay rejected, limit exceeded, and non-opaque store rejection
- group relay envelope accepted, transcript sync required, transcript rekey required, missing delivery token, and plaintext metadata rejection
- anonymous nullifier store accepted persistence, duplicate/replay rejection, and plaintext metadata rejection

`BACKEND_COMMANDS` maps each command to the existing checked fixture result, so the simulator and platform bridge return:

```text
command
result
```

The command side proves the request passed Mercury's command authorization and plaintext-payload checks. The result side carries the checked issuer/proof/nullifier/relay decision.

## Security Impact

This does not implement production MLS, ARC, Privacy Pass, or post-quantum cryptography. It improves Mercury's security boundary by making anonymous group relay diagnostics command-gated and hard to accidentally bypass in UI/platform code.

The next production adapter must keep these properties:

- UI never decides issuer trust, proof validity, nullifier replay, or relay enqueue locally.
- Remote AI cannot run backend commands.
- Local AI remains limited to draft-assist command paths.
- Command payloads remain plaintext-free.
- Group relay enqueue remains blocked unless issuer trust, anonymous proof, anonymous nullifier, transcript, relay submission, delivery token, sender certificate, and sealed envelope checks accept.
- Nullifier storage remains accepted-only, duplicate-resistant, and digest-only for redemption and credential contexts.

## Verification Targets

```powershell
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test platform_bridge
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-core --test anonymous_nullifier_store
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Research Targets

- Production private-set/nullifier database designs for small deployments.
- Witness/auditor deployment for anonymous credential issuer key consistency.
- PQ-hybrid mapping for MLS group operations and anonymous proof providers after production cryptographic library selection.
