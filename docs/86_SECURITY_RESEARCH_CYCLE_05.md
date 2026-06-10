# Security Research Cycle 05

Generated: 2026-05-28

## Scope

This cycle refreshed Mercury's anonymous credential issuer-key consistency, revocation, and partitioning-metadata posture.

## Primary Sources Checked

- IETF Privacy Pass, "Key Consistency and Discovery": https://datatracker.ietf.org/doc/html/draft-ietf-privacypass-key-consistency
- IETF Privacy Pass, "Privacy Pass Issuance Protocols with Public Metadata": https://datatracker.ietf.org/doc/draft-ietf-privacypass-public-metadata-issuance/
- IETF Privacy Pass, "Privacy Pass Issuance Protocol for Anonymous Rate-Limited Credentials": https://ietf-wg-privacypass.github.io/draft-arc/draft-ietf-privacypass-arc-protocol.html
- USENIX Security 2015, "CONIKS: Bringing Key Transparency to End Users": https://www.usenix.org/conference/usenixsecurity15/technical-sessions/presentation/melara

## Finding

Mercury's anonymous proof gate required an issuer key id length, but did not independently model whether that issuer key was consistent, fresh, non-revoked, or safe against user partitioning.

The missing properties were:

- issuer key transparency
- issuer-directory inclusion
- issuer key binding to the token challenge
- bounded active issuer key set
- issuer-directory freshness
- key validity window
- revocation freshness
- revoked-key rejection
- opaque partitioning metadata rejection

## Implemented Increment

`AnonymousCredentialIssuerTrustInput` now composes:

- `KeyTransparencyDecision`
- issuer key id length
- issuer directory inclusion status
- issuer key challenge binding
- active issuer key count and maximum allowed active key count
- directory age and maximum accepted age
- key validity window
- revocation freshness
- revoked-key state
- opaque partitioning metadata bit count

`evaluate_anonymous_credential_issuer_trust(...)` rejects any missing issuer trust property before token issuance, token verification, or anonymous membership proof use can become true.

`AnonymousGroupMembershipProofInput` now consumes `AnonymousCredentialIssuerTrustDecision`, so group proof acceptance is rejected if the issuer key is stale, revoked, not transparent, or privacy-partitioning.

The binding simulator now has checked prototype payloads for issuer-trust ready, key-transparency required, revoked issuer key, and partitioning-metadata rejected states. Anonymous group proof fixtures also expose the nested issuer-trust decision.

## Next Research Targets

- Study ARC and rate-limited token nullifier designs for bounded anonymous abuse control. Follow-up completed in `docs/88_SECURITY_RESEARCH_CYCLE_06.md`.
- Add a nullifier-window backend gate that distinguishes replay rejection, rate-limit exhaustion, window rollover, and unlinkability-preserving storage requirements. Follow-up completed in `docs/87_ANONYMOUS_RATE_LIMIT_NULLIFIER_GATE.md`.
- Review practical key-transparency witness/auditor deployment options suitable for small-scale Mercury servers. Follow-up implemented in `docs/91_ANONYMOUS_ISSUER_WITNESS_AUDIT.md`.
