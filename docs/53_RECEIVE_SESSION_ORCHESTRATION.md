# Receive Session Orchestration

Generated: 2026-05-28

## Status

Mercury now has a deterministic receive-side backend orchestration in `mercury-core`:

```text
PrototypeReceiveSession
PrototypeReceiveSessionInput
PrototypeReceiveSessionOutcome
PrototypeReceiveSessionReason
PrototypeReceiveSessionEvent
PrototypeReceiveSessionEventKind
```

This complements the existing outbound/backend session prototype. It does not decrypt or render message plaintext. It proves the receive order of operations before UI, sync, and platform notification work begins.

## Flow

```text
relay submit fixture data
  -> relay deliver once
  -> delivery acknowledgement decision
  -> client receive gate
  -> encrypted local-store write
```

The session only persists delivered ciphertext after:

- relay submission and delivery accept
- delivery acknowledgement accepts
- client receive accepts
- message policy, sender trust, replay state, and local-store sealing facts accept

No plaintext payload is introduced, and `plaintext_exposed = false` in every outcome.

## Event Transcript

Receive sessions now emit plaintext-free progress events:

```text
receive_started
relay_submit_evaluated
relay_delivery_evaluated
delivery_ack_evaluated
client_receive_evaluated
local_store_write_evaluated
receive_finished
```

Each event has stable kind/reason codes, labels, terminal state, acceptance state, and `plaintext_bytes_exposed = false`.

## Stop Points

Stable receive session reason labels:

```text
completed
relay_submit_rejected
relay_delivery_rejected
delivery_ack_rejected
client_receive_rejected
local_store_write_rejected
```

Rejected branches stop before later side effects:

- relay submit rejection stops before delivery, ack, receive, or local persistence
- delivery ack rejection stops before client receive and local persistence
- client receive rejection stops before local persistence
- store-write rejection leaves the local encrypted store unchanged

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_receive_session
cargo test -p mercury-bindings --test prototype_fixtures --test backend_commands --test platform_bridge --test ui_sim_cli
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers the complete accepted receive path, relay submit rejection, delivery acknowledgement rejection, ordering-gap retry, store-write rejection, stable reason codes/labels, and plaintext-free event transcripts.

## Simulator And Bridge Fixtures

Prototype fixture names:

```text
receive_session_happy_path
receive_session_ack_rejected
receive_session_ordering_gap
receive_session_store_write_rejected
```

Backend command names:

```text
run_receive_session_happy_path
run_receive_session_ack_rejected
run_receive_session_ordering_gap
run_receive_session_store_write_rejected
```

These let platform shells exercise inbound message readiness through the same bridge path as outbound/session and local-store readiness.

The fixture and command JSON now include the `events` transcript, so UI shells can render inbound progress without deriving it from the final outcome.

## Next Backend Step

Add a real inbound sync source around the receive-session prototype once the platform notification and transport contracts are ready to bind.
