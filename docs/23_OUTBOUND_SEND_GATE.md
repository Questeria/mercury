# Outbound Send Gate

Generated: 2026-05-28

## Status

Mercury now has a typed outbound send gate in `mercury-core`.

```text
OutboundSendInput
evaluate_outbound_send(OutboundSendInput) -> OutboundSendDecision
```

This boundary does not send packets, encrypt payloads, or write storage. It is the last deterministic client decision before an encrypted message is allowed to leave the client core and before its ciphertext is allowed to be persisted.

## Inputs

The gate composes four existing decisions:

- sender device trust from `evaluate_device_trust(...)`
- room membership state, either stable or a just-applied membership transition
- message policy from `ClientMessagePolicyInput::evaluate()` or `evaluate_policy(...)`
- local-store ciphertext sealing from `LocalStoreSealRequest::evaluate()`

`RoomMembershipSendState::Stable` covers ordinary sends where no membership transition is pending. `RoomMembershipSendState::Transition(...)` covers sends immediately after adding, removing, or compromising a device, and requires the transition to have rotated the room epoch.

## Decision Shape

`OutboundSendDecision` returns:

- `accepted`
- `can_send`
- `can_persist_ciphertext`
- `requires_user_action`
- `reason`

Accepted decisions may still carry `requires_user_action` when the sender device is sendable but not fully trusted, such as trust-on-first-use. Strict and high-security trust policy should prevent that state before this gate.

## Security Rules

The evaluator rejects:

- sender devices that cannot send
- rejected room membership transitions
- membership transitions that do not rotate the room epoch
- rejected message policy decisions
- rejected local-store sealing decisions

Rejected sends also set `can_persist_ciphertext` to false. The client should not persist ciphertext for a message that the final send gate refused.

## Verification

The `outbound_send_gate` integration test covers:

- fully accepted send and ciphertext persistence
- trust-on-first-use user-action propagation
- device trust rejection
- room membership transition rejection
- accepted transition without epoch rotation rejection
- message policy rejection before store commit
- local-store sealing rejection

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Next Step

The relay submission policy is documented in `docs/24_RELAY_SUBMISSION_POLICY.md`.
