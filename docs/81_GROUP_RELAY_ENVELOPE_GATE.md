# Group Relay Envelope Gate

Generated: 2026-05-28

## Status

Mercury now has a backend group relay envelope gate in `mercury-core`:

```text
GroupRelayEnvelopeInput
GroupRelayEnvelopeDecision
GroupRelayEnvelopeReason
evaluate_group_relay_envelope(...)
```

This is not a final sealed-sender or metadata-hiding MLS implementation. It is the pre-adapter contract a future MLS provider, sealed envelope builder, and relay client must satisfy before a group application message can be enqueued to the relay.

## Accepted Envelope

The accepted path requires:

- accepted `GroupMessageTranscriptDecision`
- `can_submit_to_relay = true`
- accepted `RelaySubmissionDecision`
- 12-byte delivery token
- delivery token bound to the opaque route
- sealed sender certificate
- accepted `AnonymousGroupMembershipProofDecision`
- accepted nested anonymous credential issuer trust through the proof decision
- anonymous group membership proof of at least 64 bytes
- accepted `AnonymousRateLimitNullifierDecision`
- positive sealed envelope length
- zero plaintext sender metadata fields
- zero plaintext group metadata fields

Accepted output enables:

```text
can_enqueue_relay = true
```

Accepted output always keeps:

```text
forbids_plaintext_sender = true
forbids_plaintext_group = true
plaintext_bytes_exposed = false
```

## Rejection Classes

Stable rejection labels:

```text
TRANSCRIPT_REJECTED
RELAY_SUBMISSION_REJECTED
MISSING_DELIVERY_TOKEN
DELIVERY_TOKEN_NOT_ROUTE_BOUND
SENDER_CERTIFICATE_NOT_SEALED
ANONYMOUS_MEMBERSHIP_PROOF_MISSING
SEALED_ENVELOPE_MISSING
PLAINTEXT_SENDER_METADATA
PLAINTEXT_GROUP_METADATA
ANONYMOUS_MEMBERSHIP_PROOF_REJECTED
ANONYMOUS_RATE_LIMIT_REJECTED
```

The decision separates:

- `requires_sync`
- `requires_rekey`
- `requires_user_action`

Transcript rejections propagate the transcript gate's sync, rekey, and user-action flags so UI and platform layers do not guess at remediation.

Anonymous membership proof rejections propagate the proof gate's sync, rekey, and user-action flags for the same reason.

Anonymous rate-limit rejections propagate the nullifier-window gate's sync, rekey, and user-action flags for the same reason.

## Source Alignment

The gate follows these metadata-minimization constraints from the research cycle:

- Signal's sealed-sender design removes sender identity from the relay-visible envelope, uses a recipient delivery token for abuse control, and seals the sender certificate plus message ciphertext inside another encrypted envelope.
- Metadata-hiding MLS-like group messaging research uses anonymous group membership authentication so the server can learn that a sender is legitimate without learning which group member sent the message.
- `docs/83_ANONYMOUS_GROUP_MEMBERSHIP_PROOF_GATE.md` defines the proof gate that must accept before this relay envelope can enqueue.
- `docs/85_ANONYMOUS_CREDENTIAL_ISSUER_TRUST_GATE.md` defines the issuer key consistency and revocation gate consumed by the proof gate.
- `docs/87_ANONYMOUS_RATE_LIMIT_NULLIFIER_GATE.md` defines the nullifier-window gate that must accept before this relay envelope can enqueue.
- OpenMLS and MLS provider docs emphasize deleting application-message key material after encryption/decryption; this gate depends on the existing transcript gate's used-generation deletion before relay enqueue.

The production adapter must still implement the cryptographic envelope, delivery-token derivation, sender certificate validation, anonymous membership proof, relay abuse controls, and traffic-correlation defenses with audited libraries.

## Verification

Run:

```powershell
cargo test -p mercury-core --test group_message_transcript
cargo test -p mercury-core --test group_relay_envelope
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype group_relay_envelope_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused relay-envelope test covers accepted metadata-hidden group submit, transcript rejection with sync/rekey propagation, relay submission rejection, missing or unbound delivery tokens, unsealed sender certificates, missing or rejected anonymous membership proof, anonymous rate-limit rejection, missing sealed envelope, plaintext sender metadata, plaintext group metadata, and stable reason labels.

Checked simulator fixtures now expose:

```text
group_relay_envelope_ready
group_relay_envelope_transcript_sync_required
group_relay_envelope_transcript_rekey_required
group_relay_envelope_missing_delivery_token
group_relay_envelope_plaintext_metadata_rejected
```

## Next Backend Step

Group relay envelope command wiring is now exposed through:

```text
run_group_relay_envelope_ready
run_group_relay_envelope_transcript_sync_required
run_group_relay_envelope_transcript_rekey_required
run_group_relay_envelope_missing_delivery_token
run_group_relay_envelope_plaintext_metadata_rejected
```

Next backend work should replace the prototype relay-envelope and anonymous credential providers with production adapters while preserving the checked decision contract.
