# Relay Server Skeleton

Generated: 2026-05-28

## Status

Mercury now has a non-production in-memory relay/server skeleton in `mercury-core`:

```text
PrototypeRelayServer
PrototypeRelaySubmitRequest
PrototypeRelaySubmitOutcome
PrototypeRelayDelivery
PrototypeRelayQueueItem
```

The skeleton composes the existing relay submission policy and relay queue state machine. It is deliberately small and opaque: it stores route IDs, replay tokens, sealed headers, and ciphertext bytes as opaque byte vectors only, and it never accepts plaintext identity metadata.

## Flow

```text
submit request
  -> derive RelaySubmissionInput from opaque byte lengths
  -> evaluate_relay_submission(...)
  -> evaluate_relay_queue(Enqueue)
  -> accepted submissions are queued as Pending
  -> rejected submissions do not mutate server state

deliver route
  -> evaluate_relay_queue(Deliver)
  -> accepted delivery returns ciphertext once
  -> stored ciphertext and sealed header are cleared

expire/delete route
  -> evaluate_relay_queue(Expire/Delete)
  -> accepted terminal transitions clear stored payload bytes
```

Replay tokens are remembered after accepted enqueue, so duplicate replay tokens are rejected even if the route ID differs.

## Non-Goals

This is not a network server, durable database, rate limiter, federation layer, or production queue. It is a deterministic server-side integration harness for the already-defined relay contracts.

## Verification

The `prototype_relay_server` integration test covers:

- policy-approved opaque submissions enqueue
- rejected send-gate and plaintext-identity submissions do not queue
- duplicate replay-token rejection
- delivery returns ciphertext once and clears stored payload bytes
- expiry and deletion clear payload bytes while retaining metadata/tombstone state

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test -p mercury-core prototype_relay
cargo test --workspace
```

## Next Step

The AI participant backend skeleton is documented in `docs/35_AI_PARTICIPANT_BACKEND_SKELETON.md`. The next parallel increment should add security/test infrastructure around the new prototypes, including a repo-local preflight script that exercises all current non-UI gates.
