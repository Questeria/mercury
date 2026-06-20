#!/usr/bin/env python3
"""Generate EXHAUSTIVE Helix differential tests — compile-and-run the real Helix policy over the FULL
input domain and assert it matches the Python spec reference at EVERY point (exit 42).

Why: Mercury's normal Helix lane (gen_*_vectors.py / gen_helix_tests.py) checks the COMPILED .hx
against a curated COVERING set of vectors; full input-space agreement was previously only checked in
RUST (`decider_exhaustive_diff.rs`: a Rust PORT of the logic vs `evaluate_*`), so the COMPILED Helix
was never executed over the whole domain. This closes that gap for the small-domain spec-derived
policies: it enumerates the entire cartesian product via the policy's Python spec (the documented
mirror of the Rust `evaluate_*`), computes the expected decision at each point, and bakes the policy's
assertion(s) per point into a dedicated exhaustive `*.hx`. `run_helix_checks.sh` compiles it with the
pinned helixc and runs it (exit 42 == every point agrees), so a divergence ANYWHERE in the input
space — not just at the covering boundaries — fails CI.

HONEST SCOPE: this proves COMPILED-Helix == the Python SPEC over the full domain. The spec is the
source-of-truth mirror of the Rust `evaluate_*` (itself cross-checked by the Rust vector mirror +
decider_exhaustive_diff), so transitively it ties the compiled Helix to Rust over the whole space.
It covers only policies whose domain is small enough to bake directly (outbound_decide: 128 points;
inbound_sync: 256). Large-domain policies (receive ~20k, account_recovery ~20k, bootstrap ~400k) stay
on the covering-set lane + the Rust exhaustive diff; baking those would need a runtime-input `.hx`
harness (reading a packed vector blob), noted as future work in docs/HELIX_EVIDENCE_TRAIL.md.

  python tools/gen_exhaustive_helix_diff.py --write
  python tools/gen_exhaustive_helix_diff.py --check   # exit 1 on drift (CI)
"""

from __future__ import annotations

import argparse
import itertools
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_inbound_sync_vectors as inbound_spec  # noqa: E402
import check_outbound_decide_vectors as outbound_spec  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXHAUSTIVE_DIR = os.path.join(ROOT, "helix", "tests", "exhaustive")
MAIN_MARKER = "fn main()"


def _args(inp: dict, fields: list[str]) -> str:
    return ", ".join(str(inp[f]) for f in fields)


def outbound_assertions(inp: dict, spec) -> list[tuple[str, int]]:
    """outbound_decide exposes a single packed-decision fn mercury_od_pack(<all input fields>)."""
    return [(f"mercury_od_pack({_args(inp, list(spec.INPUT_FIELDS))})", spec.pack(spec.decide(inp)))]


# inbound_sync's Helix interface is split: mercury_is_reason(6 inputs) -> reason_code, and
# mercury_is_pack(reason_code, bootstrap_rua) -> packed decision. Mirror gen_inbound_sync_vectors.py.
_INBOUND_REASON_ARGS = [
    "plaintext_preview_ok",
    "bootstrap_can_start_sync",
    "source_state",
    "poll_batch_ok",
    "pending_delivery",
    "route_id_ok",
]


def inbound_assertions(inp: dict, spec) -> list[tuple[str, int]]:
    d = spec.decide(inp)
    reason = d["reason_code"]
    return [
        (f"mercury_is_reason({_args(inp, _INBOUND_REASON_ARGS)})", reason),
        (f"mercury_is_pack({reason}, {inp['bootstrap_rua']})", spec.pack(d)),
    ]


# Per-policy config: the spec module (decide/pack + INPUT_FIELDS), the policy .hx whose @pure prelude
# we reuse, the per-field integer domains whose FULL cartesian product is the input space (must match
# the spec's decide() + the Rust exhaustive enumeration), and an assertion builder. Only small-domain
# policies are bakeable.
POLICIES = {
    "outbound_decide": {
        "spec": outbound_spec,
        "policy_hx": "helix/policy/outbound_decide.hx",
        "evaluate": "evaluate_outbound_send",
        "assertions": outbound_assertions,
        "domains": {
            "dt_can_send": range(2),
            "dt_requires_user_action": range(2),
            "room_state": range(4),
            "room_requires_user_action": range(2),
            "message_policy_accepted": range(2),
            "ciphertext_sealing_accepted": range(2),
        },
    },
    "inbound_sync": {
        "spec": inbound_spec,
        "policy_hx": "helix/policy/inbound_sync.hx",
        "evaluate": "evaluate_inbound_sync",
        "assertions": inbound_assertions,
        "domains": {
            "plaintext_preview_ok": range(2),
            "bootstrap_can_start_sync": range(2),
            "bootstrap_rua": range(2),
            "source_state": range(4),
            "poll_batch_ok": range(2),
            "pending_delivery": range(2),
            "route_id_ok": range(2),
        },
    },
}


def policy_prelude(policy_hx: str) -> str:
    with open(os.path.join(ROOT, policy_hx), encoding="utf-8") as fh:
        text = fh.read()
    idx = text.find(MAIN_MARKER)
    if idx == -1:
        sys.exit(f"error: no fn main() in {policy_hx}")
    return text[:idx].rstrip()


def enumerate_full_domain(spec, domains):
    fields = list(spec.INPUT_FIELDS)
    for combo in itertools.product(*(domains[f] for f in fields)):
        yield dict(zip(fields, combo))


def build_test(name: str, cfg: dict) -> tuple[str, int, int]:
    spec = cfg["spec"]
    points = list(enumerate_full_domain(spec, cfg["domains"]))
    lines = [
        policy_prelude(cfg["policy_hx"]),
        "",
        "@pure",
        "fn expect(actual: i32, expected: i32) -> i32 {",
        "    if actual == expected { 0 } else { 1 }",
        "}",
        "",
        "fn main() -> i32 {",
        "    let mut failed: i32 = 0;",
        "",
        "    // GENERATED by tools/gen_exhaustive_helix_diff.py --write. Do not edit by hand.",
        f"    // EXHAUSTIVE: all {len(points)} points of the {name} input domain. Each expected value",
        f"    // is from the Python spec {spec.__name__} — the mirror of {cfg['evaluate']}.",
        "    // exit 42 == the COMPILED Helix policy agrees with the spec at EVERY input.",
    ]
    n_assert = 0
    for inp in points:
        for expr, expected in cfg["assertions"](inp, spec):
            lines.append(f"    failed = failed + expect({expr}, {expected});")
            n_assert += 1
    lines += ["", "    if failed == 0 { 42 } else { failed }", "}", ""]
    return "\n".join(lines), len(points), n_assert


def test_path(name: str) -> str:
    return os.path.join(EXHAUSTIVE_DIR, f"{name}_exhaustive_test.hx")


def main() -> int:
    ap = argparse.ArgumentParser()
    grp = ap.add_mutually_exclusive_group(required=True)
    grp.add_argument("--write", action="store_true")
    grp.add_argument("--check", action="store_true")
    args = ap.parse_args()

    os.makedirs(EXHAUSTIVE_DIR, exist_ok=True)
    drift = []
    for name, cfg in POLICIES.items():
        content, points, asserts = build_test(name, cfg)
        path = test_path(name)
        if args.write:
            with open(path, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(content)
            print(f"wrote {os.path.relpath(path, ROOT)} ({points} points, {asserts} assertions)")
        elif not os.path.exists(path) or open(path, encoding="utf-8").read() != content:
            drift.append(os.path.relpath(path, ROOT))

    if args.check and drift:
        print("DRIFT:", *drift, sep="\n  ", file=sys.stderr)
        print("Run: python tools/gen_exhaustive_helix_diff.py --write", file=sys.stderr)
        return 1
    if args.check:
        print(f"exhaustive helix diff: OK ({len(POLICIES)} policies)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
