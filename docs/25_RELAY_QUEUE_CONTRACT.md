# Relay Queue Contract

Generated: 2026-05-28

## Status

Mercury now has a typed relay queue state-machine boundary in `mercury-core`.

```text
RelayQueueInput
evaluate_relay_queue(RelayQueueInput) -> RelayQueueDecision
```

This boundary sits after the Helix-backed relay submission policy. It does not store bytes itself; it defines whether the server-side queue may enqueue, deliver, expire, or delete an already validated encrypted queue item.

## Queue States

The first states are:

- `Absent`
- `Pending`
- `Delivered`
- `Expired`
- `Deleted`

The first operations are:

- `Enqueue`
- `Deliver`
- `Expire`
- `Delete`

Accepted delivery and expiry both require ciphertext deletion. This keeps the relay from becoming a durable message archive.

## Security Rules

The evaluator rejects:

- malformed time windows
- enqueue attempts with rejected relay submissions
- enqueue attempts with duplicate replay tokens
- enqueue attempts over an existing pending, delivered, expired, or deleted item
- enqueue attempts at or after expiry
- delivery, expiry, or deletion of absent items
- delivery after expiry
- delivery from delivered, expired, or deleted states
- expiry before the deadline
- expiry from delivered, expired, or deleted states
- deletion of already deleted items

Accepted enqueue persists the pending encrypted item. Accepted delivery, expiry, or deletion returns `delete_item = true` and `persist_item = false`.

## Verification

The `relay_queue` integration test covers:

- accepted enqueue
- rejected relay submission
- duplicate replay token rejection
- delivery deleting ciphertext
- delivery rejection for expired or terminal items
- expiry only after deadline
- delete idempotence guard
- malformed time-window rejection

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Next Step

The delivery acknowledgement boundary is documented in `docs/26_DELIVERY_ACKNOWLEDGEMENT.md`.
