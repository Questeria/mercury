# From-raw Helix cross-check (FU-2)

A differential check on the Helix compilers: every Mercury Helix policy test is compiled and run
under a compiler of a completely independent lineage from the one CI uses, and the two must agree.

## What it checks, and why

`tools/run_helix_checks.sh` and CI prove the policies against one compiler: the pinned
`third_party/helix` Python `helixc`. That is the right merge gate, but it leaves one residual
question: could a quirk in that single compiler's codegen silently mask a policy bug?

`tools/run_fromraw_helix_check.sh` answers it by running the same `helix/tests/*_test.hx` policy
tests and the same golden vectors through a second compiler built from raw binary:

```text
hex0 -> seed.bin (62 KB, sha256-pinned, no Python) -> K1 (self-hosted Helix compiler)
```

This is the Helix raw-binary bootstrap. K1 shares no code with the pinned Python
`helixc`: different implementation, different codegen. A policy test passes when the ELF the
compiler emits exits 42, the all-assertions-pass sentinel used by the pinned-helixc runner.

This complements the Rust pins in `core/rust/mercury-core/tests/*_vectors.rs`, which check the
policy logic against the real `mercury-core` deciders. The from-raw check tests compilation of that
logic by an independent compiler lineage.

## Result

All 12 Mercury Helix policies cross-validate: emit an ELF that exits 42 under the from-raw
self-hosted K1, matching the pinned helixc exactly:

| policy | pinned helixc | from-raw K1 |
| --- | --- | --- |
| envelope | exit 42 | exit 42 |
| ai_grant | exit 42 | exit 42 |
| ai_grant_lifecycle | exit 42 | exit 42 |
| room_epoch | exit 42 | exit 42 |
| policy_pipeline | exit 42 | exit 42 |
| relay_submit | exit 42 | exit 42 |
| platform_decision | exit 42 | exit 42 |
| outbound_decide | exit 42 | exit 42 |
| receive_decide | exit 42 | exit 42 |
| bootstrap_decide | exit 42 | exit 42 |
| inbound_sync | exit 42 | exit 42 |
| account_recovery | exit 42 | exit 42 |

That includes all five standalone decision-boundary deciders (`outbound_decide`, `receive_decide`,
`bootstrap_decide`, `inbound_sync`, `account_recovery`) plus `platform_decision`. The full current
decision mirror compiles and runs identically under an independently bootstrapped from-raw-binary
compiler.

## An honest note on stages (seed vs K1)

The raw seed (`hex0 -> seed.bin`) is the minimal bootstrap compiler: it only has to compile the
next stage's source. Its codegen does not cover every Helix construct. Feeding the policy tests
directly to the seed used to show a seed-stage divergence in `envelope_test`, because
`envelope.hx` is the only policy that uses the `bool` return type with `true` / `false` literals
and `== false` comparisons.

Switching the same probe to run under K1 makes it pass. So the divergence is a property of the
minimal seed stage only, not of the from-raw Helix language. K1, the self-hosted compiler the seed
produces, is the actual from-raw Helix compiler used for this cross-check.

## Scope / boundaries

- Local / manual, not a CI gate. The from-raw compiler lives in the separate Helix repo,
  which is not a Mercury build dependency. The script skips cleanly when that toolchain is absent.
- Read-only on Helix. `seed.bin` and `k1src.hx` are copied out; the K1 build and every
  policy compile run entirely under `/tmp`. Nothing is written into the Helix tree.
- Frozen I/O. K1 reads `/tmp/k1_in.hx` and writes `/tmp/k1_out.bin`, so the script must run alone
  with no other concurrent from-raw build sharing `/tmp`.
- This pins compiler agreement on the current artifacts, not exhaustive input-space equivalence of
  the deciders. That is the FU-4 Rust exhaustive differential.

## How to run

```bash
# default toolchain path: /mnt/c/Projects/Helix/stage0/helixc-bootstrap
bash tools/run_fromraw_helix_check.sh

# or point at another checkout:
bash tools/run_fromraw_helix_check.sh /path/to/helixc-bootstrap
```

On Windows, use the wrapper:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_native_helix_checks.ps1
```
