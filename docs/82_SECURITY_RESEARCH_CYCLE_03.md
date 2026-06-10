# Security Research Cycle 03

Generated: 2026-05-28

## Scope

This cycle refreshed Mercury's group relay submission posture against MLS provider docs, forward-secrecy notes, sealed-sender design, and metadata-hiding group messaging research.

## Primary Sources Checked

- mls-rs group module docs: https://docs.rs/mls-rs/latest/mls_rs/group/index.html
- OpenMLS forward secrecy notes: https://book.openmls.tech/forward_secrecy.html
- Signal sealed sender technology preview: https://signal.org/blog/sealed-sender/
- Hashimoto, Katsumata, Prest, "How to Hide MetaData in MLS-Like Secure Group Messaging": https://eprint.iacr.org/2022/1533

## Finding

Mercury had an MLS transcript gate and an opaque relay-submission policy, but no single backend contract for the handoff between them.

The gap matters because a group message can satisfy generic relay metadata bounds while still failing a stronger group privacy rule, such as:

- transcript state was not accepted for relay submit
- delivery token was missing or not bound to the opaque route
- sender certificate was not sealed inside an outer envelope
- anonymous group membership proof was missing
- relay-visible fields exposed sender or group metadata

## Implemented Increment

`GroupRelayEnvelopeInput` now composes:

- accepted group message transcript decision
- accepted relay submission decision
- delivery-token length and route binding
- sealed sender certificate flag
- anonymous group membership proof length
- sealed envelope length
- plaintext sender metadata count
- plaintext group metadata count

`evaluate_group_relay_envelope(...)` rejects any missing sealed-sender or metadata-hiding precondition before `can_enqueue_relay` can become true.

The binding simulator now has checked prototype payloads for accepted relay enqueue, transcript sync required, transcript rekey required, missing delivery token, and plaintext metadata rejection states.

## Next Research Targets

- Study production sealed-sender envelope constructions and abuse-control tradeoffs for unknown senders.
- Review anonymous credential and group membership proof schemes suitable for small high-security groups. Follow-up started in `docs/84_SECURITY_RESEARCH_CYCLE_04.md`.
- Add backend command envelopes for the group relay envelope gate.
