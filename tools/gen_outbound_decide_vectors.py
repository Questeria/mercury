#!/usr/bin/env python3
"""Generate the outbound_decide golden vectors + Helix test + manifest from the spec.

The vectors are correct-by-construction from tools/check_outbound_decide_vectors.py
(decide/pack) over its representative covering set (every reason + the priority boundaries +
the ACCEPT requires_user_action matrix). Exhaustive input-space agreement with the real
evaluate_outbound_send is the FU-4 exhaustive differential. The Helix test shares the policy's
@pure prelude and asserts mercury_od_pack == expected_pack per vector (exit 42).

  python3 tools/gen_outbound_decide_vectors.py --write
  python3 tools/gen_outbound_decide_vectors.py --check   # exit 1 on drift (CI)
"""
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_outbound_decide_vectors as spec  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VEC_DIR = os.path.join(ROOT, "vectors", "outbound_decide")
POLICY = os.path.join(ROOT, "helix", "policy", "outbound_decide.hx")
TEST = os.path.join(ROOT, "helix", "tests", "outbound_decide_test.hx")
MANIFEST = os.path.join(ROOT, "policy", "outbound_decide_v1.json")
MAIN_MARKER = "fn main()"


def build_vectors() -> list[dict]:
    out = []
    seen: dict[str, int] = {}
    for inp in spec.enumerate_inputs():
        v = spec.decide(inp)
        label = v["reason_label"].lower()
        seen[label] = seen.get(label, 0) + 1
        name = f"{v['reason_code']}_{label}__{seen[label]}"
        out.append(
            {
                "name": name,
                "description": f"{v['reason_label']} / pack={spec.pack(v)}",
                "input": dict(inp),
                "expected": v,
                "expected_pack": spec.pack(v),
            }
        )
    return out


def policy_prelude() -> str:
    with open(POLICY, encoding="utf-8") as fh:
        text = fh.read()
    idx = text.find(MAIN_MARKER)
    if idx == -1:
        sys.exit("error: no fn main() in outbound_decide.hx")
    return text[:idx].rstrip()


def build_test(vectors: list[dict]) -> str:
    lines = [
        policy_prelude(),
        "",
        "@pure",
        "fn expect(actual: i32, expected: i32) -> i32 {",
        "    if actual == expected { 0 } else { 1 }",
        "}",
        "",
        "fn main() -> i32 {",
        "    let mut failed: i32 = 0;",
        "",
        "    // GENERATED from vectors/outbound_decide/*.json by",
        "    // tools/gen_outbound_decide_vectors.py --write. Do not edit by hand.",
    ]
    for v in vectors:
        args = ", ".join(str(v["input"][f]) for f in spec.INPUT_FIELDS)
        lines += [
            "",
            f"    // {v['name']}",
            f"    failed = failed + expect(mercury_od_pack({args}), {v['expected_pack']});",
        ]
    lines += ["", "    if failed == 0 { 42 } else { failed }", "}", ""]
    return "\n".join(lines)


def build_manifest() -> str:
    manifest = {
        "schema": "mercury.outbound_decide_policy.v1",
        "description": (
            "Outbound-send decider: mirrors mercury-core evaluate_outbound_send "
            "(OutboundSendInput -> OutboundSendDecision), the combinator that turns four "
            "sub-decisions into the outbound reason. Pure scalar; no crypto/JSON/ciphertext."
        ),
        "generated_by": "tools/gen_outbound_decide_vectors.py --write (do not hand-edit)",
        "input_fields": list(spec.INPUT_FIELDS),
        "decision_fields": list(spec.DECISION_FIELDS),
        "pack_low_bits_msb_to_lsb": list(spec.BOOL_FIELDS),
        "pack_layout": "reason_code * 16 + accepted*8 + can_send*4 + can_persist_ciphertext*2 + requires_user_action",
        "reason_codes": {label: code for code, label in spec.OUTBOUND_REASONS.items()},
        "vector_count": len(spec.enumerate_inputs()),
        "source_of_truth": (
            "core/rust/mercury-core/src/lib.rs evaluate_outbound_send (10841); pinned by "
            "core/rust/mercury-core/tests/outbound_decide_vectors.rs (constructs the real "
            "sub-decisions + calls evaluate_outbound_send). Exhaustive agreement: the FU-4 exhaustive differential."
        ),
    }
    return json.dumps(manifest, indent=2, ensure_ascii=False) + "\n"


def render() -> tuple[dict[str, str], str, str]:
    vectors = build_vectors()
    files = {
        os.path.join(VEC_DIR, v["name"] + ".json"):
        json.dumps(v, indent=2, ensure_ascii=False) + "\n"
        for v in vectors
    }
    return files, build_test(vectors), build_manifest()


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate outbound_decide vectors + test + manifest.")
    grp = ap.add_mutually_exclusive_group(required=True)
    grp.add_argument("--write", action="store_true")
    grp.add_argument("--check", action="store_true")
    args = ap.parse_args()

    files, test, manifest = render()

    if args.write:
        os.makedirs(VEC_DIR, exist_ok=True)
        keep = {os.path.basename(p) for p in files}
        for existing in os.listdir(VEC_DIR):
            if existing.endswith(".json") and existing not in keep:
                os.remove(os.path.join(VEC_DIR, existing))
        for path, content in files.items():
            with open(path, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(content)
        with open(TEST, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(test)
        with open(MANIFEST, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(manifest)
        print(f"wrote {len(files)} vectors + outbound_decide_test.hx + manifest")
        return 0

    drift = []
    for path, content in files.items():
        if not os.path.exists(path) or open(path, encoding="utf-8").read() != content:
            drift.append(os.path.relpath(path, ROOT))
    if not os.path.exists(TEST) or open(TEST, encoding="utf-8").read() != test:
        drift.append(os.path.relpath(TEST, ROOT))
    if not os.path.exists(MANIFEST) or open(MANIFEST, encoding="utf-8").read() != manifest:
        drift.append(os.path.relpath(MANIFEST, ROOT))
    if drift:
        print("DRIFT:", *drift, sep="\n  ", file=sys.stderr)
        print("\nRun: python3 tools/gen_outbound_decide_vectors.py --write", file=sys.stderr)
        return 1
    print(f"outbound_decide: vectors + test + manifest up to date ({len(files)} vectors)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
