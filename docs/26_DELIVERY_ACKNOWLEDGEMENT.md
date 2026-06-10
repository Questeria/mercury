# Delivery Acknowledgement

Generated: 2026-05-28

## Status

Mercury now has a typed delivery acknowledgement boundary in `mercury-core`.

```text
DeliveryAckInput
evaluate_delivery_ack(DeliveryAckInput) -> DeliveryAckDecision
```

This boundary sits after relay queue delivery. It defines when the relay may accept a client acknowledgement, retain hash-only audit evidence, and delete the remaining queue record.

## Inputs

Acknowledgement evaluation uses:

- queue item state
- duplicate acknowledgement flag
- delivery and acknowledgement timestamps
- maximum acknowledgement delay
- acknowledgement token length
- ciphertext digest length
- opaque delivery tag length
- plaintext identity field count

The acknowledgement token and ciphertext digest are both fixed at 32 bytes. The delivery tag is an opaque handle between 16 and 128 bytes. Plaintext account, device, conversation, room, epoch, and AI principal identifiers are represented by `plaintext_identity_fields`; the only accepted value is zero.

## Security Rules

The evaluator rejects:

- acknowledgements before delivery
- acknowledgements for expired or deleted queue items
- duplicate acknowledgements
- malformed time windows
- acknowledgements outside the configured delay window
- malformed acknowledgement tokens
- malformed ciphertext digests
- malformed delivery tags
- any plaintext identity fields

Accepted acknowledgements return:

- `retain_hash_audit = true`
- `delete_queue_record = true`
- `requires_client_retry = false`

Duplicate acknowledgements are idempotence-guarded: they do not rewrite audit state, do not delete again, and do not require client retry.

## Verification

The `delivery_ack` integration test covers:

- accepted acknowledgement behavior
- queue-state rejection
- duplicate acknowledgement handling
- time-window validation
- fixed token and digest lengths
- opaque delivery tag bounds
- plaintext identity rejection

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Next Step

The client receive gate is documented in `docs/27_CLIENT_RECEIVE_GATE.md`.
