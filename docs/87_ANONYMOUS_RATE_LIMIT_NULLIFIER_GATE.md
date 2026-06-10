# Anonymous Rate-Limit Nullifier Gate

Generated: 2026-05-28

## Status

Mercury now has a backend nullifier-window gate for anonymous abuse control:

```text
AnonymousRateLimitCredentialKind
AnonymousRateLimitNullifierInput
AnonymousRateLimitNullifierDecision
AnonymousRateLimitNullifierReason
evaluate_anonymous_rate_limit_nullifier(...)
AnonymousNullifierStoreWrite
AnonymousNullifierStoreDecision
AnonymousNullifierStoreReason
PrototypeAnonymousNullifierStore
put_anonymous_nullifier_record(...)
```

This is not a final ARC or Privacy Pass implementation. It is the policy-facing boundary a production anonymous credential provider and nullifier store must satisfy before Mercury records a nullifier or lets a group relay envelope enqueue.

## Accepted Nullifier Window

The accepted path requires:

- accepted `AnonymousGroupMembershipProofDecision`
- credential kind explicitly selected as one-time redemption or ARC-style window
- 32-byte nullifier
- nullifier not already spent
- available nullifier store
- opaque nullifier storage
- nullifier bound to relay route
- nullifier bound to group epoch
- 32-byte redemption context
- 32-byte credential context
- current time inside the rate-limit window
- positive presentation limit no greater than the configured maximum
- current presentation count below the limit
- one-time credentials limited to exactly one presentation
- zero plaintext rate-limit metadata fields

Accepted output enables:

```text
can_record_nullifier = true
can_redeem_this_window = true
can_rate_limit_without_identity = true
```

Accepted output always keeps:

```text
forbids_plaintext_rate_limit_metadata = true
plaintext_bytes_exposed = false
```

## Rejection Classes

Stable rejection labels:

```text
MEMBERSHIP_PROOF_REJECTED
BAD_NULLIFIER
NULLIFIER_ALREADY_SPENT
NULLIFIER_STORE_UNAVAILABLE
NULLIFIER_STORE_NOT_OPAQUE
CONTEXT_NOT_BOUND
BAD_REDEMPTION_CONTEXT
BAD_CREDENTIAL_CONTEXT
BAD_WINDOW
WINDOW_EXPIRED
BAD_PRESENTATION_LIMIT
PRESENTATION_LIMIT_EXCEEDED
ONE_TIME_REQUIRES_SINGLE_USE
PLAINTEXT_RATE_LIMIT_METADATA
```

The decision separates:

- `requires_sync`
- `requires_rekey`
- `requires_user_action`

## Relay Envelope Binding

`GroupRelayEnvelopeInput` now consumes an `AnonymousRateLimitNullifierDecision`.

Relay enqueue is rejected with `ANONYMOUS_RATE_LIMIT_REJECTED` unless the nullifier-window gate accepts. This keeps a valid anonymous group membership proof from becoming sufficient on its own when the replay/rate-limit store is unavailable, non-opaque, exhausted, expired, or not bound to the route and group epoch.

## Source Alignment

The gate follows these constraints from the research cycle:

- Privacy Pass one-time-use tokens require one token per redemption.
- ARC credentials allow a credential to produce a fixed number of unlinkable tokens for a public presentation context.
- ARC token challenges include redemption and credential contexts that bind presentations to origins/windows.
- Public-metadata and rate-limited-token work makes visible metadata explicit; Mercury therefore rejects plaintext or unbudgeted rate-limit metadata at the backend boundary.

The production adapter must still implement cryptographic ARC/Privacy Pass verification, nullifier derivation, durable nullifier storage, window rollover, abuse escalation, and anonymity-set monitoring with audited libraries.

## Nullifier Store Boundary

The prototype nullifier store records only accepted opaque nullifiers and digest-only contexts. It rejects duplicate/replayed nullifiers, bad digest shapes, exhausted presentation windows, and plaintext metadata before persistence.

## Verification

Run:

```powershell
cargo test -p mercury-core --test anonymous_rate_limit_nullifier
cargo test -p mercury-core --test anonymous_nullifier_store
cargo test -p mercury-core --test group_relay_envelope
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_rate_limit_nullifier_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_nullifier_store_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused nullifier test covers accepted ARC-style windows, proof rejection propagation, nullifier shape, replayed/spent nullifier rejection, missing/non-opaque nullifier stores, route and epoch binding, redemption and credential contexts, malformed/expired windows, presentation-limit bounds, exhausted limits, one-time credential enforcement, plaintext metadata rejection, and stable reason/kind labels.

Checked simulator fixtures now expose:

```text
anonymous_rate_limit_nullifier_ready
anonymous_rate_limit_nullifier_replay_rejected
anonymous_rate_limit_nullifier_limit_exceeded
anonymous_rate_limit_nullifier_opaque_store_required
anonymous_nullifier_store_ready
anonymous_nullifier_store_replay_rejected
anonymous_nullifier_store_plaintext_metadata_rejected
```

## Next Backend Step

Backend command envelopes now expose issuer trust, anonymous membership proof, anonymous rate-limit nullifier, anonymous nullifier store, and group relay envelope accepted and blocked states through the simulator and platform bridge.

Next backend work should study production private-set/nullifier database designs and witness/auditor deployment for anonymous credential issuer consistency in small Mercury deployments.
