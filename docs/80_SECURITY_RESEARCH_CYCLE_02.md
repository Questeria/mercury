# Security Research Cycle 02

Generated: 2026-05-28

## Scope

This cycle refreshed Mercury's MLS application-message handling against current primary sources and converted the finding into one backend hardening increment.

## Primary Sources Checked

- IETF RFC 9420, The Messaging Layer Security (MLS) Protocol: https://www.rfc-editor.org/rfc/rfc9420.html
- IETF RFC 9750, The Messaging Layer Security (MLS) Architecture: https://www.rfc-editor.org/rfc/rfc9750.html

## Finding

Mercury had group readiness, epoch policy, outbound send gating, and local room-epoch sealing, but no single contract for a group application message after MLS protection and before local persistence or relay submit.

RFC 9420 makes the relevant binding shape explicit:

- group context includes group id, epoch, tree hash, and confirmed transcript hash
- application messages use sender ratchets with generations
- sender data is separately encrypted
- application encryption uses a four-byte reuse guard

RFC 9750 also emphasizes deleting used message keys promptly; otherwise MLS forward secrecy is weakened.

## Implemented Increment

`GroupMessageTranscriptInput` now composes:

- accepted group readiness
- accepted outbound send readiness
- group id length
- message/local epoch match
- sender leaf index and generation
- group context and confirmed transcript hash digest lengths
- sealed sender data and sealed application payload flags
- reuse guard length
- local room-epoch store sealing
- used generation deletion

`evaluate_group_message_transcript(...)` rejects any missing binding before `can_persist_ciphertext` or `can_submit_to_relay` can become true.

The binding simulator now has checked prototype payloads for accepted transcript send, transcript sync required, rekey required, and local-store epoch binding rejection states.

## Next Research Targets

- Review MLS provider APIs for access to group context digest, sender generation, reuse guard handling, and key-deletion guarantees.
- Add backend commands for group transcript send states once the UI/platform bridge needs command envelopes instead of prototype fixtures.
- Study sealed-sender and metadata-hiding designs for relay submission after group transcript acceptance. Follow-up started in `docs/82_SECURITY_RESEARCH_CYCLE_03.md`.
