# Live-relay Helix shadow-decider — design (offered, not built)

This is a **design offer**, not an implementation. It sketches how a compiled Helix policy could
run as a *shadow* beside `mercury-core` in the live system to catch deploy/regression divergence in
production. Building it needs a runtime-architecture decision (below), so nothing here is wired in.

## Where we are today

The Helix policies already exist and are tied to `mercury-core` three ways, all offline / CI-time:

1. **Rust pins** (`core/rust/mercury-core/tests/*_vectors.rs`) — each golden vector's reduced
   scalars are reconstructed into the REAL input struct, the REAL `evaluate_*` is called, and the
   result is asserted against the vector.
2. **Exhaustive differential** (`core/rust/mercury-core/tests/decider_exhaustive_diff.rs`, FU-4) —
   the staged Helix reductions are re-derived in Rust and checked against the REAL `evaluate_*`
   over the *entire* reduced input domain (every variant combination).
3. **From-raw cross-check** (`tools/run_fromraw_helix_check.sh`, FU-2) — the policy ELFs are
   compiled + run under an independently-bootstrapped from-raw Helix compiler.

What none of these do is run *in production*. A shadow decider would close that last gap: prove,
on live traffic, that the deployed `mercury-core` binary still agrees with the proven policy.

## The shadow idea

```
                 reduce()            ┌──────────────────────────┐
  live inputs ───────────────────────▶ scalar codes            │
      │                              │  (dt_can_send, room_state,│
      │                              │   kt_code, …)            │
      │                              └──────────┬───────────────┘
      │ (rich structs)                          │ (scalars)
      ▼                                         ▼
┌──────────────────┐                  ┌──────────────────────┐
│ mercury-core     │                  │ Helix policy ELF      │
│ evaluate_*()     │── decision A ──▶ │ mercury_*_pack()      │── packed B
└──────────────────┘      │           └──────────────────────┘     │
        (authoritative)   └────────────── compare ─────────────────┘
                                          │
                              match → ok ; mismatch → ALERT (never gate)
```

- `mercury-core` stays **authoritative** — the shadow never changes a decision, only observes.
- A thin Rust boundary `reduce()`s the same rich inputs into the scalar codes the policy consumes
  (the identical reduction already written for the deciders), invokes the Helix policy, and
  compares the policy's packed decision to `mercury-core`'s.
- On mismatch it emits an alert / metric (and, in a sealed-audit-friendly form, a record). It does
  **not** block the message path.

## What it would and would not catch

Catches:
- A regression or mis-deploy where the running `mercury-core` no longer matches the proven policy
  for some reduced input class — i.e. defense-in-depth against "the binary in prod drifted from the
  artifact we verified."
- Distribution surprises: inputs that, post-reduction, hit a decision the offline corpus weighted
  differently (useful as a coverage signal).

Does **not** catch (honest boundaries):
- Bugs in the `reduce()` step itself — the reduction is Rust at the boundary, *shared* by both
  sides of the comparison, so a wrong reduction is invisible to the shadow. (The reduction's
  faithfulness is what the FU-4 exhaustive differential + the Rust pins establish offline.)
- Anything outside the reduced scalar domain — the rich-typed fields the reduction intentionally
  drops (lengths beyond the ≤0 test, ignored sub-decision fields, crypto) are not re-checked here.
- It is an **assurance** mechanism, not a security boundary: `mercury-core` remains the only
  decider that gates traffic.

## The open architectural decisions (why it's an offer)

1. **ELF I/O ABI.** Today the policy ELFs are compiled by the test harness to return a single
   `i32` exit code (42 = all-vectors-pass). A shadow needs *input → packed output* per call. That
   is a new ABI: either (a) a stdin/stdout byte protocol (scalars in, packed `i32` out), (b) a C
   ABI entry point the Rust side calls via FFI, or (c) regenerate the policy as a callable library
   rather than a `main()`-returns-exit-code test. Each is a real design choice with different
   perf/packaging/trust trade-offs.
2. **Process & perf model.** In-process FFI (lowest latency, but loads foreign code into the relay
   address space — weigh against `#![forbid(unsafe_code)]` and the threat model), vs a sandboxed
   subprocess / Wasm sandbox (isolation, higher per-call cost — so sample rather than shadow every
   message). Pick a latency budget and a sampling rate.
3. **Versioning & shipping.** Which policy ELF ships with which `mercury-core` build, how the
   `policy/<p>_v*.json` manifest hash is checked at load, and how a shadow mismatch is triaged
   (alert routing, sealed-audit record, rollback policy).
4. **Scope.** Start with one decider (e.g. `outbound_decide`, the smallest reduced domain) behind a
   sampling flag, default-off, alert-only — then widen.

## Recommendation

Defer until there is appetite for (1)–(3). The offline guarantees (pins + exhaustive differential
+ from-raw cross-check) already establish that the policies equal `mercury-core` over the full
reduced domain; the shadow's marginal value is *production drift detection*, which is worth wiring
only alongside a decision on the ELF ABI and the relay's process/perf model. This document is the
offer; implementing it is a separate, opt-in piece of work.
