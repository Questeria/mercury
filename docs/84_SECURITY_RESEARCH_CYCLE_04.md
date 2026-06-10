# Security Research Cycle 04

Generated: 2026-05-28

## Scope

This cycle refreshed Mercury's anonymous group membership and abuse-control posture against Privacy Pass, BBS, and Signal private-group sources.

## Primary Sources Checked

- IETF RFC 9576, Privacy Pass Architecture: https://www.rfc-editor.org/rfc/rfc9576
- IETF RFC 9578, Privacy Pass Issuance Protocols: https://www.ietf.org/ietf-ftp/rfc/rfc9578.pdf
- IRTF CFRG BBS Signatures draft: https://www.ietf.org/archive/id/draft-irtf-cfrg-bbs-signatures-06.html
- Chase, Perrin, Zaverucha, "The Signal Private Group System and Anonymous Credentials Supporting Efficient Verifiable Encryption": https://www.microsoft.com/en-us/research/publication/the-signal-private-group-system-and-anonymous-credentials-supporting-efficient-verifiable-encryption/

## Finding

Mercury's group relay envelope required a proof-like opaque byte length, but did not independently model the proof's security properties.

The missing properties were:

- accepted group context before proof use
- PQ-safe proof class for high-security rooms
- challenge digest and presentation nonce binding
- presentation header binding
- group epoch binding
- relay route binding
- replay/nullifier handling
- proof freshness
- plaintext member identity rejection
- anonymous rate-limit support

## Implemented Increment

`AnonymousGroupMembershipProofInput` now composes:

- accepted group readiness
- proof scheme class
- high-security/PQ posture
- issuer key id length
- challenge digest length
- presentation nonce length
- proof length
- presentation-header binding
- group-epoch binding
- route binding
- replay nullifier length and seen-state
- issued/expires/now timestamps
- plaintext member identifier count

`evaluate_anonymous_group_membership_proof(...)` rejects any missing proof binding before `can_authenticate_member`, `can_redeem_once`, or `can_rate_limit_anonymously` can become true.

`GroupRelayEnvelopeInput` now consumes `AnonymousGroupMembershipProofDecision`, so relay enqueue is rejected if the anonymous proof gate rejects.

The binding simulator now has checked prototype payloads for proof-ready, high-security PQ-required, replay rejected, route binding required, and plaintext member identity rejected states.

## Next Research Targets

- Study issuer key-consistency and transparency approaches for anonymous credential issuers. Follow-up completed in `docs/86_SECURITY_RESEARCH_CYCLE_05.md`.
- Review revocation/nullifier strategies that preserve unlinkability while blocking abuse.
- Add backend command envelopes for anonymous proof and group relay envelope readiness.
