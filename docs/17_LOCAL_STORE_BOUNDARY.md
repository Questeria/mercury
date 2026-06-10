# Local Store Boundary

Generated: 2026-05-27

## Status

Mercury now has a first typed boundary for local persistence decisions in `mercury-core`.

```text
LocalStoreRecordKind::policy() -> LocalStoreRecordPolicy
LocalStoreWriteIntent::evaluate() -> LocalStoreWriteDecision
evaluate_local_store_write(LocalStoreWriteIntent) -> LocalStoreWriteDecision
```

This is not a database, key manager, or encryption implementation. It is the guardrail layer that client storage code should call before anything is written to disk.

## Record Categories

The boundary classifies records by:

- `LocalStoreKeyScope`
- `LocalStorePlaintextClass`
- `LocalStoreRetentionClass`
- `LocalStorePolicyRequirement`

Current record kinds include account/device/conversation secrets, room snapshots, message envelopes, message ciphertext, media ciphertext, policy audit hashes, AI grant state, AI prompt/transcript plaintext, and AI transcript digests.

## Security Rules

The initial rules are deliberately strict:

- account, device, conversation, room, message, media, and AI grant state records require encryption at rest
- message envelopes, message ciphertext, media ciphertext, and AI transcript digests require an accepted message policy decision
- policy audit records may record accepted or rejected decisions, but only as hash-only audit records
- message plaintext is never a writable local-store record
- media plaintext is never a writable local-store record
- AI prompt plaintext is never a writable local-store record
- AI transcript plaintext is never a writable local-store record

That last group matters for Mercury's AI design. An AI participant must not create a second plaintext archive of user conversations just because the AI workflow needs context. Durable AI memory should be expressed as scoped grants, encrypted state, user-visible summaries, or hash/digest audit material, not hidden prompt and transcript files.

## Verification

The `local_store_policy` integration test covers:

- plaintext message records are always rejected
- ciphertext message records require encryption and accepted policy
- audit hashes can record rejected policy decisions
- AI prompt/transcript plaintext is blocked
- AI transcript digests are hash-only audit records
- account secrets require encryption without requiring a message policy decision

Run in CI:

```powershell
cargo test --workspace
```

Local Windows note: if `link.exe` is not on the normal PowerShell PATH, run the test command from the Visual Studio Build Tools developer environment.

## Encrypted Store Adapter Follow-Up

The concrete encrypted-store adapter boundary is documented in `docs/18_ENCRYPTED_STORE_ADAPTER.md`.

## Next Step

The local-store unlock gate is documented in `docs/47_LOCAL_STORE_UNLOCK_GATE.md`.
