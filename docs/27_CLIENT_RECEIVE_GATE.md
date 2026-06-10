# Client Receive Gate

Generated: 2026-05-28

## Status

Mercury now has a typed client receive gate in `mercury-core`.

```text
ClientReceiveInput
evaluate_client_receive(ClientReceiveInput) -> ClientReceiveDecision
```

This boundary sits between relay delivery and any UI, notification, plaintext rendering, or local message timeline insertion. It does not decrypt ciphertext, verify signatures, or write storage. It defines the deterministic decision all of those future steps must respect.

## Inputs

The gate composes:

- relay queue delivery decision
- delivery acknowledgement decision
- sender device trust decision
- message policy decision
- local-store ciphertext sealing decision
- replay and ordering state
- ciphertext digest length
- plaintext identity field count

The receive path intentionally mirrors the send path: a message cannot be exposed unless it was delivered by the relay state machine, acknowledged as a new receipt, accepted by message policy, accepted for sealed local persistence, and recognized as a new in-order ciphertext.

## Decision Shape

`ClientReceiveDecision` returns:

- `accepted`
- `can_decrypt`
- `can_persist_ciphertext`
- `can_expose_to_ui`
- `requires_client_retry`
- `requires_user_action`
- `reason`

Accepted decisions may still carry `requires_user_action` when the sender device is sendable but not fully trusted, such as trust-on-first-use. Strict and high-security receive policy should produce a rejected sender-device trust decision before this gate.

## Security Rules

The evaluator rejects:

- relay deliveries that were not accepted
- relay deliveries that did not transition the item to delivered
- duplicate or rejected acknowledgements
- malformed ciphertext digest lengths
- plaintext identity fields
- duplicate, stale, or out-of-order replay state
- sender devices that are not allowed to send
- rejected message policy decisions
- rejected local-store sealing decisions

Accepted receives set:

- `can_decrypt = true`
- `can_persist_ciphertext = true`
- `can_expose_to_ui = true`

Every rejection keeps all three capabilities false. Duplicate and stale deliveries are not retried or exposed. Ordering gaps require client retry or sync before any UI sees the message.

## Verification

The `client_receive_gate` integration test covers:

- accepted receive behavior
- trust-on-first-use user-action propagation
- relay delivery acceptance and delivered-state requirements
- new accepted acknowledgement requirements
- duplicate, stale, and future-gap replay handling
- sender trust, message policy, and local-store sealing rejection
- ciphertext digest and plaintext identity rejection

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Next Step

The client sync and bootstrap boundary is documented in `docs/28_CLIENT_BOOTSTRAP_SYNC.md`.
