# Key Transparency Proof Boundary

Generated: 2026-05-28

## Status

Mercury now has a typed key transparency proof boundary in `mercury-core`.

```text
KeyTransparencyProofInput
evaluate_key_transparency(KeyTransparencyProofInput) -> KeyTransparencyDecision
```

This boundary does not implement a transparency log or verify Merkle proofs. It defines the policy-facing contract for already-verified proof facts, so device trust does not consume hand-written booleans.

## Inputs

The proof boundary tracks:

- inclusion proof status
- consistency proof status
- key-history proof status
- witness quorum status
- whether witness quorum is required
- previous and current tree size
- proof age
- maximum accepted proof age

Proof statuses are:

- `Verified`
- `Missing`
- `Invalid`

Witness statuses are:

- `NotRequired`
- `QuorumSatisfied`
- `QuorumMissing`
- `Invalid`

## State Mapping

The evaluator maps proof facts into `KeyTransparencyState`:

- all required proofs verified, fresh, monotonic, and witnessed when required -> `Consistent`
- missing inclusion, consistency, key-history, or witness proof -> `MissingProof`
- stale proof age -> `StaleProof`
- invalid proof, invalid witness, bad freshness window, or log rollback -> `Inconsistent`

`DeviceTrustInput` consumes this state. Strict and high-security device trust continue to require `KeyTransparencyState::Consistent`.

## Security Rules

The evaluator rejects or downgrades:

- nonpositive max freshness windows
- negative proof age
- current tree size lower than previous tree size
- stale proofs
- missing inclusion proof
- invalid inclusion proof
- missing consistency proof
- invalid consistency proof
- missing key-history proof
- invalid key-history proof
- missing required witness quorum
- invalid witness proof

## Verification

The `key_transparency` integration test covers:

- fresh verified proofs
- stale proof mapping
- missing inclusion proof mapping
- invalid consistency proof mapping
- log rollback mapping
- required witness quorum checks
- proof decision feeding strict device trust
- stale proof blocking strict device trust

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Room Membership Follow-Up

The room membership transition boundary is documented in `docs/22_ROOM_MEMBERSHIP_TRANSITIONS.md`.

## Next Step

The outbound send gate is documented in `docs/23_OUTBOUND_SEND_GATE.md`.
