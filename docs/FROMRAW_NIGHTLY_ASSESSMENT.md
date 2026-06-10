# From-raw cross-check as a CI nightly — assessment

Can `tools/run_fromraw_helix_check.sh` (the FU-2 from-raw compiler cross-check) run as a GitHub
Actions nightly? Short answer: **not without vendoring a prebuilt binary blob, which is a real
decision — and the recommendation is to keep it a local cross-check rather than vendor.**

## What the from-raw check needs

`run_fromraw_helix_check.sh` builds the self-hosted from-raw Helix compiler **K1** from the raw
`seed.bin` + `k1src.hx` and compiles + runs the 12 Mercury policy tests under it (each exits 42). It
reads those inputs from **`C:/Projects/Helix/`** — a separate local repository that is
**not** a Mercury dependency (not a submodule, not a package, not vendored). The script is guarded to
skip cleanly when Helix is absent.

A GitHub Actions runner has no access to Helix. So a nightly cannot run this check unless
the from-raw inputs are **vendored into the Mercury repo**.

## The options

**Option 1 — vendor the from-raw seed + run K1 in a nightly.** Copy `seed.bin` (~62 KB, a prebuilt
x86-64 ELF) + `k1src.hx` (~300 KB) into Mercury under a clearly-fenced path with provenance
(sha256 + an origin note pointing at the Helix commit they were built from). A nightly
workflow builds K1 from the vendored seed in `/tmp` and runs the 12 policy tests.
- **Benefit:** continuous, automated independent-compiler-lineage agreement, in CI.
- **Cost (the real one):** it puts a **prebuilt, opaque binary blob** into a security-critical
  messenger repository. Anyone auditing Mercury now has to trust (or independently rebuild) that
  blob. "A 62 KB ELF that we promise was built from hex0→seed" is exactly the kind of artifact a
  high-assurance project should be reluctant to vendor — it inverts the from-raw effort's whole
  point (no unexplained binaries). Reproducing it from source in CI would mean vendoring the entire
  hex0→seed bootstrap ladder, which is a large, separate undertaking.

**Option 2 — keep it local, document the cadence.** Leave `run_fromraw_helix_check.sh` as a local,
manually-run cross-check (run it when the policies or the pinned `helixc` change), and record that
cadence. No binary blob enters the repo.
- **Benefit:** no provenance/trust cost; the repo stays free of unexplained binaries.
- **Cost:** the from-raw agreement is checked on a human cadence, not every night.

## Why this is not a coverage hole

The from-raw check is **defense-in-depth, not the sole gate.** The Mercury policies are already
gated in CI three independent ways that do *not* need Helix:

1. the pinned `helixc` submodule compiles + runs every policy test (exit 42) in the `helix` CI job;
2. the Rust pins (`*_vectors.rs`) assert each reduction against the REAL `evaluate_*`;
3. the exhaustive differential (`decider_exhaustive_diff.rs`) checks the deep deciders over their
   full reduced domain.

The from-raw K1 check adds a *second, independently-bootstrapped compiler lineage* agreeing on the
artifacts. Valuable as corroboration; not load-bearing for correctness.

## Recommendation

**Keep the from-raw check local (Option 2). Do not vendor the seed.** The trust cost of a prebuilt
binary blob in this repo outweighs the marginal benefit of nightly automation, given the three CI
gates above already cover the policies. Run `run_fromraw_helix_check.sh` manually when the policy set
or the pinned `helixc` changes.

If the project later decides the nightly automation is worth it, the vendoring should be its **own
clearly-scoped commit**: the seed + `k1src.hx` under a fenced path, a `PROVENANCE.md` with the
sha256 and the exact Helix commit, and a separate nightly workflow that builds K1 in `/tmp`
and runs the 12 policy tests. That is offered, not done here — it needs an explicit "yes, vendor the
blob" decision.
