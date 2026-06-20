# Helix policy evidence trail — what is attested, and what is not

Mercury's 12 policies (`helix/policy/*.hx`) are deterministic decision logic compiled by the **pinned
Helix compiler** (`third_party/helix`). Two layers of attestation make the policy layer auditable.
This document states precisely what each layer proves — and, in keeping with Mercury's honesty
posture, what it does **not**.

## Layer 1 — source attestation (`tools/gen_policy_attestations.py` → `helix/policy/attestation.json`)

Content hashes of each policy's Helix source, its test, the shared vector corpus, the policy JSON
spec, and the list of Rust mirror tests. `--check` fails CI on any drift. This proves *"these exact
source bytes / vectors / mirrors are what was reviewed"* — a tamper-evident content record. It does
**not** say anything about what the source *means* once compiled.

## Layer 2 — proof manifest (`tools/gen_proof_manifests.py` → `helix/policy/manifests/*.manifest.json`)

The new semantic layer. For each policy it typechecks the source with the pinned compiler (the same
`--no-stdlib` mode CI uses) and emits a canonical, tamper-evident manifest
(`helixc.backend.proof_manifest`, Stage 122) recording, **per function**, the compiler-verified
effect set, purity flag, and parameter count — bound to the normalized **source hash** and the
**pinned compiler version**. It then **fails closed** if any policy function carries an effect.

So Layer 2 upgrades the claim from *"these source bytes"* to:

> *"these source bytes typecheck, under compiler version `vX`, to N functions that are ALL
> compiler-verified side-effect-free (no I/O, FFI, mutation, arena, or trap)."*

That is the property a security-policy decision must have — deterministic and side-effect-free — now
**machine-checked and recorded**, not assumed. Each manifest's hash is folded into
`attestation.json` (and surfaced to the app as `proofManifestShort`), so it is drift-gated by the
same `--check`, and `run_helix_checks.sh` regenerates + verifies it before the attestation.

## What this is NOT (honest scope)

- **Not authenticity — yet.** The manifest is currently **UNSIGNED**: its canonical SHA-256 detects
  accidental edits (tamper-*evident*), but a motivated tamperer can recompute it after editing the
  body. The pinned helixc substrate ships only the unsigned digest. **Ed25519 release-signing** over
  the canonical bytes is a deliberate next step (the `cryptography`/`pynacl` libraries are available,
  so Mercury owns that ~30-line addition); it needs a protected release key, which is a key-management
  decision, not a code change. Until then: integrity + a re-derivable attestation, not anti-forgery.
- **Not a proof of correctness.** It attests *effects/purity from the type system*, not that the
  policy computes the right decision. Functional correctness is the job of the **vector differential**
  (`run_helix_checks.sh` compiles each test to an ELF and requires exit 42 over golden vectors, cross-
  checked by the Rust and Python mirrors). The manifest and the differential are complementary.
  - **Exhaustive lane (`tools/gen_exhaustive_helix_diff.py`).** For small-domain spec-derived
    policies the differential is now *exhaustive*: it bakes EVERY point of the input domain
    (outbound_decide: all 128) and asserts the COMPILED Helix agrees with the Python spec (the mirror
    of the Rust `evaluate_*`) at each one — closing the gap where only a Rust *port*
    (`decider_exhaustive_diff.rs`) was previously checked over the full space. Large-domain policies
    stay on the covering-set lane + the Rust exhaustive diff; baking those needs a runtime-input
    `.hx` harness (reading a packed vector blob), which is the next extension.
- **Not bound to the ELF.** The manifest binds the normalized **source** (reproducible across the
  Windows dev box and the Linux CI runner). It does **not** bind the compiled Linux ELF, whose
  cross-platform byte-determinism is unverified; the ELF is separately compiled + exit-42-run by CI.
  ELF-hash binding is a possible enhancement pending a determinism check.
- **Not yet verified in-app.** The app's `proofManifestShort` badge currently *displays* the manifest
  hash; it does not yet re-derive + verify the manifest against the running policy. Replacing the
  cosmetic source-hash badge with an in-app `verify_manifest_hash` is a follow-up.

## IFC data-egress contract (demonstrator — `helix/demonstrators/ai_egress_ifc.hx`)

A separate capability: Helix's **information-flow types** can express and *compile-enforce* the
data-leak guardrail's core invariant. Context data labelled `Confidential<T>` can reach a remote AI
provider (a plain-typed "public" sink) **only** through an explicit, audit-greppable `__declassify`,
and only inside a valid-grant branch. The demonstrator compiles + runs (the gated release works);
the negative fixture `helix/tests/regression/ai_egress_leak.hx` — confidential data flowing directly
to the public sink with no declassify — **fails to compile** (`arg expects i32, got
Confidential<i32>`), and `run_helix_checks.sh` asserts that failure.

This is a **demonstrator**, not production enforcement (Mercury's runtime egress is enforced in
Rust). It proves DIRECT-flow non-leakage at the type level — no value path from a labelled source to
the public sink without an explicit declassify. It does **not** model implicit/covert channels; same
honest framing as the guardrail wedge (provably enforce + audit the egress points, not claim
completeness).

## Roadmap (each a separate, honest increment)

1. Capture + hash the `--emit-proof-obligations` artifact (today emitted then discarded) into the
   manifest set, so the typecheck/totality receipt is recorded too.
2. Ed25519-sign the manifests with a Mercury release key (authenticity).
3. Verify the manifest in-app/audit instead of displaying a cosmetic hash.
4. ELF-hash binding once cross-platform determinism is established.
