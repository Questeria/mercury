# Client Message Policy Input

Generated: 2026-05-27

## Status

Mercury now has its first typed client-side input boundary for policy evaluation:

```text
ClientMessagePolicyInput::into_policy_input() -> PolicyEvaluationInput
ClientMessagePolicyInput::evaluate() -> PolicyDecision
```

This is still below networking, storage, parsing, signatures, and encryption. It gives mobile and desktop clients a safer way to assemble the facts that are already required by the Helix-backed policy pipeline.

## Typed Inputs

The new boundary introduces typed enums for stable Mercury concepts:

- `ProtocolSuite`
- `MessageKind`
- `RoomMode`
- `DeviceKind`
- `DeviceState`
- `AccessKind`

It also groups client state into:

- `ClientMessageEnvelope`
- `ConversationPolicyState`
- `RoomStateSnapshot`
- `SenderDeviceState`
- `ClientMessagePolicyInput`

`ClientMessagePolicyInput` derives `EnvelopeFacts` and `RoomEpochFacts` from these structs, then reuses the existing `evaluate_policy` function. There is no second decision path.

## Security Purpose

Raw `i32` facts remain available for vector tests and low-level mirrors, but normal clients should prefer `ClientMessagePolicyInput`. That keeps supported suites, room modes, message kinds, and access kinds explicit at the API boundary and reduces the chance that a UI or client binding accidentally feeds mismatched policy fields.

The room-epoch policy derives `message_epoch` directly from the message envelope epoch, which avoids asking a caller to provide the same security-sensitive epoch twice.

## Verification

The new integration test covers:

- valid human application messages
- stale room epochs that pass envelope ordering but fail room policy
- AI devices blocked by room policy

Run in CI:

```powershell
cargo test --workspace
```

Local Windows note: integration-test linking remains blocked here by the missing MSVC `link.exe`, so local verification still uses `cargo check --workspace`, Python vector checks, and the Helix runner.

## Client State Builder Follow-Up

The checked local-client state builder is documented in `docs/16_CLIENT_STATE_BUILDER.md`.

## Next Step

The next increment should introduce the first local encrypted-store boundary design: key ownership, record categories, and policy-decided data that must never be written in plaintext.
