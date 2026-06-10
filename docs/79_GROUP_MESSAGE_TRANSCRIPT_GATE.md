# Group Message Transcript Gate

Generated: 2026-05-28

## Status

Mercury now has a backend group message transcript gate in `mercury-core`:

```text
GroupMessageTranscriptInput
GroupMessageTranscriptDecision
GroupMessageTranscriptReason
evaluate_group_message_transcript(...)
```

This is not an MLS implementation. It is the pre-crypto contract a future MLS provider, local store, and relay adapter must satisfy before a group application message can be persisted as ciphertext or submitted to a relay.

## Accepted Transcript

The accepted path requires:

- accepted `GroupChatDecision`
- accepted `OutboundSendDecision`
- `can_send = true`
- `can_persist_ciphertext = true`
- positive group id length
- message epoch and local epoch match and are greater than zero
- sender leaf index is nonnegative
- sender generation is nonnegative
- group context digest is at least 32 bytes
- confirmed transcript hash is at least 32 bytes
- sender data is sealed
- application payload is sealed
- MLS reuse guard length is 4 bytes
- local-store seal request accepts
- local-store record kind is `MessageCiphertext`
- local-store key scope is `RoomEpoch`
- local-store room epoch matches the message epoch
- local-store conversation/group binding length matches the group id length
- used sender generation key has been deleted

Accepted output enables:

```text
can_persist_ciphertext = true
can_submit_to_relay = true
```

Accepted output always keeps:

```text
forbids_plaintext = true
plaintext_bytes_exposed = false
```

## Rejection Classes

Stable rejection labels:

```text
GROUP_CHAT_REJECTED
OUTBOUND_SEND_REJECTED
BAD_GROUP_IDENTIFIER
EPOCH_MISMATCH
BAD_SENDER_LEAF_INDEX
BAD_SENDER_GENERATION
TRANSCRIPT_CONTEXT_MISSING
SENDER_DATA_NOT_SEALED
APPLICATION_PAYLOAD_NOT_SEALED
REUSE_GUARD_MISSING
LOCAL_STORE_SEALING_REJECTED
LOCAL_STORE_EPOCH_BINDING_MISMATCH
USED_GENERATION_NOT_DELETED
```

The decision separates:

- `requires_sync`
- `requires_rekey`
- `requires_user_action`

## Source Alignment

The gate follows these MLS constraints from the standards track:

- MLS group context binds `group_id`, `epoch`, tree hash, and confirmed transcript hash.
- MLS application-message protection uses sender ratchets with per-sender generations.
- MLS sender data is separately encrypted.
- MLS application encryption uses a four-byte reuse guard.
- MLS forward secrecy depends on deleting used key material promptly.

The production provider must still implement RFC 9420 message framing, HPKE, AEAD, ratchets, transcript hashing, and key schedule with audited libraries.

## Verification

Run:

```powershell
cargo test -p mercury-core --test group_message_transcript
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype group_message_transcript_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused core test covers accepted MLS application-message context, rejected group readiness, rejected outbound send, malformed group id, epoch mismatch, bad sender leaf index, bad generation, missing transcript context, unsealed sender data, unsealed application payload, missing reuse guard, local-store sealing rejection, local-store epoch binding mismatch, and used-generation deletion.

Checked simulator fixtures now expose:

```text
group_message_transcript_ready
group_message_transcript_sync_required
group_message_transcript_rekey_required
group_message_transcript_store_binding_rejected
```

## Next Backend Step

Connect this gate to a future production MLS provider adapter, keep relay enqueue behind `GroupRelayEnvelopeDecision`, and then expose backend commands for accepted send, transcript sync required, rekey required, and local-store epoch mismatch states.
