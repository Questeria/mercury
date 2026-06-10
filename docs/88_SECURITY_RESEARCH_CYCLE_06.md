# Security Research Cycle 06

Generated: 2026-05-28

## Scope

This cycle refreshed Mercury's anonymous rate-limit, nullifier, and bounded-redemption posture.

## Primary Sources Checked

- IETF Privacy Pass, "Privacy Pass Issuance Protocol for Anonymous Rate-Limited Credentials": https://ietf-wg-privacypass.github.io/draft-arc/draft-ietf-privacypass-arc-protocol.html
- IETF Privacy Pass, "Rate-Limited Tokens": https://datatracker.ietf.org/doc/html/draft-ietf-privacypass-rate-limit-tokens
- IETF Privacy Pass, "Privacy Pass Issuance Protocols with Public Metadata": https://datatracker.ietf.org/doc/html/draft-ietf-privacypass-public-metadata-issuance
- IETF RFC 9576, "The Privacy Pass Architecture": https://www.rfc-editor.org/rfc/rfc9576

## Finding

Mercury's anonymous proof gate could say `can_rate_limit_anonymously = true`, but there was no separate state machine for nullifier storage, bounded presentation windows, ARC-style repeated presentations, or one-time redemption enforcement.

The missing properties were:

- accepted proof before nullifier use
- nullifier shape
- spent-nullifier rejection
- nullifier store availability
- opaque nullifier storage
- route and group-epoch binding
- redemption context and credential context shape
- current-time window validation
- presentation limit bounds
- exhausted-limit rejection
- one-time credential single-use enforcement
- plaintext rate-limit metadata rejection

## Implemented Increment

`AnonymousRateLimitNullifierInput` now composes:

- accepted anonymous membership proof
- credential kind
- nullifier length and spent state
- nullifier store availability and opacity
- route binding and group-epoch binding
- redemption context length
- credential context length
- rate-limit window
- presentation count, presentation limit, and maximum presentation limit
- plaintext rate-limit metadata count

`evaluate_anonymous_rate_limit_nullifier(...)` rejects any missing replay/rate-limit property before `can_record_nullifier`, `can_redeem_this_window`, or `can_rate_limit_without_identity` can become true.

`GroupRelayEnvelopeInput` now consumes `AnonymousRateLimitNullifierDecision`, so relay enqueue is rejected if anonymous rate limiting is unsafe even when the membership proof itself accepts.

The binding simulator now has checked prototype payloads for nullifier-ready, replay rejected, limit exceeded, and opaque-store-required states. Group relay envelope fixtures also expose the nested anonymous rate-limit decision.

## Next Research Targets

- Review Privacy Pass/ARC production storage guidance and private-set/nullifier database designs.
- Backend command envelope follow-up completed in `docs/89_SECURITY_RESEARCH_CYCLE_07.md`.
- Study witness/auditor deployment for key transparency and issuer key consistency in small Mercury deployments.
