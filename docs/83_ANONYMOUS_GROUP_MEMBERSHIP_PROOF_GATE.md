# Anonymous Group Membership Proof Gate

Generated: 2026-05-28

## Status

Mercury now has a backend anonymous group membership proof gate in `mercury-core`:

```text
AnonymousGroupMembershipProofScheme
AnonymousGroupMembershipProofInput
AnonymousGroupMembershipProofDecision
AnonymousGroupMembershipProofReason
evaluate_anonymous_group_membership_proof(...)
```

This is not a final anonymous credential implementation. It is the pre-adapter contract a future Privacy Pass, BBS, KVAC, or PQ metadata-hiding group proof provider must satisfy before a group relay envelope can authenticate membership anonymously.

## Accepted Proof

The accepted path requires:

- accepted `GroupChatDecision`
- high-security rooms require a PQ-safe proof scheme class
- accepted anonymous credential issuer-trust decision
- 32-byte issuer key id
- 32-byte challenge digest
- 32-byte presentation nonce
- proof length of at least 64 bytes
- presentation header bound to challenge/audience/time
- proof bound to group epoch
- proof bound to relay route
- 32-byte replay nullifier
- replay nullifier not already seen
- current time within issued/expires window
- zero plaintext member identifier fields

Accepted output enables:

```text
can_authenticate_member = true
can_redeem_once = true
can_rate_limit_anonymously = true
```

Accepted output always keeps:

```text
forbids_plaintext_member_identity = true
plaintext_bytes_exposed = false
```

## Rejection Classes

Stable rejection labels:

```text
GROUP_REJECTED
ISSUER_TRUST_REJECTED
BAD_ISSUER_KEY
BAD_CHALLENGE_DIGEST
BAD_PRESENTATION_NONCE
PROOF_MISSING
PRESENTATION_HEADER_NOT_BOUND
GROUP_EPOCH_NOT_BOUND
ROUTE_NOT_BOUND
REPLAY_NULLIFIER_MISSING
REPLAY_NULLIFIER_ALREADY_SEEN
PROOF_EXPIRED
PLAINTEXT_MEMBER_IDENTITY
HIGH_SECURITY_REQUIRES_PQ_PROOF
```

The decision separates:

- `requires_sync`
- `requires_rekey`
- `requires_user_action`

## Relay Envelope Binding

`AnonymousGroupMembershipProofInput` now consumes an `AnonymousCredentialIssuerTrustDecision`, and `GroupRelayEnvelopeInput` consumes the resulting `AnonymousGroupMembershipProofDecision`. Relay enqueue stays rejected unless the issuer-trust gate and anonymous proof gate both accept, even if the envelope still carries a proof-like byte length.

This keeps the relay boundary from treating arbitrary opaque bytes as proof of group membership.

## Source Alignment

The gate follows these constraints from the research cycle:

- Privacy Pass tokens are one-time redemptions linked to challenges and support abuse control without revealing a stable client identity.
- Privacy Pass issuance protocols bind token inputs to nonces and challenge digests.
- BBS presentations support unlinkable proof-of-possession and can bind presentation headers such as nonce, audience, or time validity.
- Signal's private group system uses anonymous credentials so members can authenticate to a server without revealing which encrypted membership entry they correspond to.

The issuer-trust gate is documented in `docs/85_ANONYMOUS_CREDENTIAL_ISSUER_TRUST_GATE.md`.

The production adapter must still implement the selected cryptographic scheme, proof verification, nullifier storage, revocation policy, and PQ-safe high-security mode with audited libraries.

## Verification

Run:

```powershell
cargo test -p mercury-core --test anonymous_credential_issuer_trust
cargo test -p mercury-core --test anonymous_group_membership_proof
cargo test -p mercury-core --test group_relay_envelope
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_group_membership_proof_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused proof test covers accepted bound presentations, rejected group state, issuer-trust rejection, high-security PQ proof requirements, issuer key shape, challenge digest shape, presentation nonce shape, missing proof, unbound presentation header, unbound group epoch, unbound route, missing/replayed nullifier, expired proof, plaintext member identity rejection, and stable reason/scheme labels.

Checked simulator fixtures now expose:

```text
anonymous_group_membership_proof_ready
anonymous_group_membership_proof_high_security_pq_required
anonymous_group_membership_proof_replay_rejected
anonymous_group_membership_proof_route_binding_required
anonymous_group_membership_proof_plaintext_identity_rejected
```

## Next Backend Step

The anonymous rate-limit nullifier gate is documented in `docs/87_ANONYMOUS_RATE_LIMIT_NULLIFIER_GATE.md`.
