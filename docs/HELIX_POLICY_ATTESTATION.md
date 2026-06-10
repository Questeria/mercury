# Helix Policy Attestation

Mercury uses Helix as a deterministic policy and verification layer. Rust remains the production
runtime for cryptography, ratchets, key storage, networking, and platform bindings.

## Artifacts

The generated attestation artifacts are:

```text
helix/policy/attestation.json
ui/app/src/mercury/policyAttestation.generated.ts
```

`attestation.json` is the audit-facing manifest. It records, for every Helix policy:

- Helix policy source path and SHA-256.
- Helix test source path and SHA-256.
- Policy JSON path and SHA-256.
- Vector directory, vector file count, and vector corpus SHA-256.
- Rust mirror tests that must agree with the Helix policy.
- Required gates: vector checker, Rust mirror or exhaustive test, Helix hash, proof obligations,
  and Helix ELF exit 42.
- Optional gate: from-raw K1 ELF exit 42.
- Production runtime boundary: `rust_mirror`.

The generated TypeScript file gives the desktop UI a compact view of the same data. The inspector
renders the policy name, source hash, vector hash, gate family, and Rust runtime boundary for each
decision card.

## Commands

Check for drift:

```powershell
python .\tools\gen_policy_attestations.py --check
```

Regenerate after touching policy sources, tests, vectors, policy JSON, or Rust mirror tests:

```powershell
python .\tools\gen_policy_attestations.py --write
```

Run the normal pinned Helix lane:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
```

Run the optional from-raw K1 lane. This treats `C:\Projects\Helix` as read-only and
copies bootstrap inputs into `/tmp` before compiling:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_native_helix_checks.ps1
```

## Boundary

The attestation is evidence that Mercury's deterministic policy layer, Rust mirrors, vectors, and
Helix compiler outputs agree. It is not a claim that Helix implements production cryptographic
primitives. Those stay in reviewed Rust libraries and platform APIs.
