# Anonymous Credential Issuer Trust Gate

Generated: 2026-05-28

## Status

Mercury now has a backend issuer-trust gate for anonymous credential issuers:

```text
AnonymousCredentialIssuerTrustInput
AnonymousCredentialIssuerTrustDecision
AnonymousCredentialIssuerTrustReason
AnonymousIssuerWitnessAuditInput
AnonymousIssuerWitnessAuditDecision
AnonymousIssuerWitnessAuditReason
evaluate_anonymous_credential_issuer_trust(...)
evaluate_anonymous_issuer_witness_audit(...)
```

This is not a transparency log or cryptographic anonymous credential implementation. It is the policy-facing boundary a production Privacy Pass, BBS, KVAC, or ARC-style provider must satisfy before Mercury accepts an issuer key for anonymous group membership proofs.

## Accepted Issuer Key

The accepted path requires:

- consistent key transparency for the issuer key directory
- accepted issuer witness/auditor audit for the issuer key set
- 32-byte issuer key id
- verified issuer-directory inclusion
- issuer key bound to the credential challenge
- bounded active issuer key set
- fresh issuer directory
- current time inside the issuer key validity window
- fresh revocation status
- non-revoked issuer key
- zero opaque partitioning metadata bits

Accepted output enables:

```text
can_issue_or_verify_tokens = true
can_use_for_anonymous_membership_proof = true
protects_anonymity_set = true
```

## Rejection Classes

Stable rejection labels:

```text
KEY_TRANSPARENCY_REQUIRED
ISSUER_WITNESS_AUDIT_REJECTED
BAD_ISSUER_KEY_ID
ISSUER_DIRECTORY_MISSING
DIRECTORY_STALE
ACTIVE_KEY_SET_PARTITIONING_RISK
KEY_NOT_YET_VALID
KEY_EXPIRED
REVOCATION_STATUS_STALE
KEY_REVOKED
CHALLENGE_KEY_BINDING_MISSING
PARTITIONING_METADATA_PRESENT
```

The decision separates:

- `requires_sync`
- `requires_rekey`
- `requires_user_action`

## Anonymous Proof Binding

`AnonymousGroupMembershipProofInput` now consumes an `AnonymousCredentialIssuerTrustDecision`.

Anonymous proof acceptance is rejected with `ISSUER_TRUST_REJECTED` unless the issuer-trust gate has accepted. The proof decision propagates issuer sync, rekey, and user-action flags so callers do not treat an opaque proof as sufficient when the issuer key set is stale, revoked, partitioned, not transparent, or rejected by witness/auditor checks.

## Source Alignment

The gate follows these constraints from the research cycle:

- Privacy Pass key consistency work warns that client privacy depends on many clients using the same authenticated public key, because per-client or small-set keys can be used to target users.
- Privacy Pass issuance protocols bind tokens to nonce, challenge digest, and token key id.
- Privacy Pass public metadata drafts make metadata explicit and visible to participants, which means Mercury must reject unbudgeted opaque partitioning metadata until the anonymity impact is modeled.
- CONIKS-style key transparency gives clients a way to check consistency and non-equivocation for public key bindings without trusting a single server view.

The production adapter must still implement cryptographic key discovery, log inclusion/consistency proof verification, revocation distribution, issuer key rotation, nullifier persistence, and anonymity-set monitoring with audited libraries.

## Verification

Run:

```powershell
cargo test -p mercury-core --test anonymous_credential_issuer_trust
cargo test -p mercury-core --test anonymous_issuer_witness_audit
cargo test -p mercury-core --test anonymous_group_membership_proof
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_credential_issuer_trust_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused issuer-trust test covers accepted issuer keys, key-transparency rejection, witness/auditor rejection, directory-inclusion rejection, key-id shape, stale directory state, active-key-set partitioning risk, key validity windows, stale revocation status, revoked issuer keys, challenge-key binding, partitioning metadata, and stable reason labels.

Checked simulator fixtures now expose:

```text
anonymous_credential_issuer_trust_ready
anonymous_credential_issuer_trust_transparency_required
anonymous_credential_issuer_trust_revoked
anonymous_credential_issuer_trust_partitioning_metadata_rejected
anonymous_credential_issuer_trust_witness_audit_rejected
```

## Next Backend Step

Add a nullifier-window decision for anonymous rate limiting so Mercury can distinguish one-time redemption storage from bounded ARC-style repeat presentations without weakening unlinkability. Follow-up completed in `docs/87_ANONYMOUS_RATE_LIMIT_NULLIFIER_GATE.md`.
