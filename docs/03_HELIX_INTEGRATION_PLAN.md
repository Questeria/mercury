# Mercury Helix Integration Plan

## Position

Use Helix as Mercury's deterministic policy and verification language first. Do not use Helix as the production cryptography runtime yet.

The current practical path is two-tiered:

1. The pinned `third_party/helix` compiler submodule is Mercury's normal local/CI Helix gate.
2. `C:\Projects\Helix` is optional read-only context for the from-raw, self-hosted K1 cross-check. Mercury copies `seed.bin` and `k1src.hx` out to `/tmp`; it must not write into the Helix tree.

Production crypto/session execution remains in mature Rust libraries. Helix supplies deterministic policy source, proof artifacts, vector agreement, and provenance hashes.

## What Helix Should Own First

Helix is a good fit for logic that must be deterministic, reviewable, and testable:

- Message envelope validation.
- Protocol version and suite-id acceptance.
- Downgrade rejection.
- Sender/device id shape checks.
- Epoch, sequence, and message-kind policy.
- Replay and out-of-order windows.
- Skipped-message bounds and ratchet-counter sanity checks.
- Device add/remove/rotate/revoke lifecycle rules.
- MLS application-policy layer around allowed proposals and commits.
- AI grant scope validation.
- Audit-event classification.
- Provenance and `why was this accepted?` traces.

Helix can also use confidentiality/provenance wrapper types to catch accidental policy-layer misuse, while remembering that those wrappers do not replace runtime secret handling.

## What Helix Should Not Own Yet

Keep these in audited crypto/platform libraries:

- Curve25519, Ed25519, ML-KEM, ML-DSA, SLH-DSA.
- PQXDH, X3DH, Double Ratchet, Triple Ratchet, Sender Keys.
- MLS tree crypto and key schedule.
- AEAD, HPKE, HKDF, password hashing, random generation.
- Secure deletion, zeroization, key material lifecycle.
- Production networking, mobile keychain bindings, and hardware enclave calls.

## Proposed Repo Layout

```text
C:\Projects\Mercury\
  helix\
    policy\
      envelope.hx
      device_lifecycle.hx
      replay_window.hx
      ai_grants.hx
    tests\
      envelope_test.hx
      ai_grants_test.hx
  vectors\
    envelope\
      valid_minimal.json
      reject_downgrade.json
      reject_oversize.json
  core\
    rust\
      mercury-core\
  tools\
    run_helix_checks.ps1
```

This layout is now active. Mercury also generates a policy attestation manifest at
`helix/policy/attestation.json` plus a UI import at
`ui/app/src/mercury/policyAttestation.generated.ts`.

## Compiler And Attestation Commands

Pinned compiler lane, used locally and in CI:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
```

Linux/CI equivalent:

```bash
bash tools/run_helix_checks.sh third_party/helix
```

Attestation drift check:

```powershell
python .\tools\gen_policy_attestations.py --check
```

Regenerate after changing `helix/policy`, `helix/tests`, `vectors`, `policy`, or Rust mirror tests:

```powershell
python .\tools\gen_policy_attestations.py --write
```

Optional from-raw K1 cross-check, read-only on `C:\Projects\Helix`:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_native_helix_checks.ps1
```

## Host Integration Strategy

Phase 1: host-driven validation artifacts. Done.

- The pinned Helix lane runs checks, hashes, proof obligations, ELF emission, and exit-42 tests.
- Golden JSON vectors are validated by Helix tests and a host mirror.
- CI fails if Helix and host mirror disagree.
- `tools/gen_policy_attestations.py` pins source/test/vector/policy/Rust-mirror hashes.

Phase 2: Rust mirror and differential tests. Done for the current policy set.

- Rust core implements the same policy functions.
- Test harness compares Helix outputs to Rust outputs across fixed and generated cases.
- Helix artifacts become part of audit evidence.

Phase 3: UI and audit provenance. Active.

- The UI inspector renders the Helix policy name, source hash, vector hash, gate family, and Rust mirror runtime for each decision view.
- Audit docs and future release notes can cite `helix/policy/attestation.json`.

Phase 4: narrow FFI or shadow pilot. Deferred.

- Rust exposes pointer/length or scalar-only C ABI functions.
- Helix calls extern functions only for demonstrations or non-secret validation.
- No secrets returned to Helix across FFI.
- Refined/wrapper types stay out of FFI signatures.

Phase 5: deeper migration.

- Move more canonical serialization and protocol state into Helix when byte buffers, ownership, cross-platform targets, and linking are stronger.
- Revisit production use after self-hosted `kovc` parity improves.

## First Helix Module

Started with `helix/policy/envelope.hx`.

Inputs:

- Protocol version.
- Cipher suite id.
- Conversation id.
- Sender account id.
- Sender device id.
- Epoch.
- Sequence number.
- Message kind.
- Payload length.
- Flags.

Checks:

- Version is supported.
- Suite id is not deprecated or downgraded.
- Ids are non-empty and within canonical length bounds.
- Epoch and sequence are monotonic for the caller-provided state.
- Message kind is allowed in the current room mode.
- Payload length is under configured limit.
- Unknown critical flags reject.
- Unknown non-critical flags preserve but do not grant behavior.

Output:

- Accept or reject.
- Deterministic reason code.
- Audit classification.
- Proof-obligation/hash artifact from the compiler.

The current backend codegen path supports a small integer-parameter budget, so
the validator is staged into identity, ordering, and content functions rather
than one large all-fields function. See `docs/05_PHASE1_ENVELOPE_POLICY.md`.

## Milestones

M1: Create Mercury Helix skeleton and envelope validator. Done.

M2: Add golden vectors and Python check script. Done for the first scalar envelope policy.

M3: Add Rust mirror and differential tests.

M4: Add AI grant validator in Helix.

M5: Add replay-window and device-lifecycle policy modules.

M6: Add WSL/Linux execution lane for Helix-emitted tests.

M7: Add policy attestation artifacts and surface provenance in the UI inspector. Done.

M8: Evaluate C ABI or shadow-decider pilot only after the compiler/linking path is ready.

## Design Rule

Helix should make Mercury harder to accidentally weaken. It should not make Mercury depend on experimental implementations for the parts where mature audited crypto is available today.
