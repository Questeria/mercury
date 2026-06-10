#!/usr/bin/env python3
"""Generate the platform_decision golden vectors + Helix test from the spec (Tier 2).

The vectors are CORRECT-BY-CONSTRUCTION from the single spec mirror
(tools/check_platform_decision_vectors.py::view/pack) over the FULL enumerated
(source x reason x derived-bit) space -- 44 cases, no hand-typed expecteds. The Helix
test (helix/tests/platform_decision_test.hx) shares the policy's exact @pure prelude
(so policy and test cannot drift) and asserts mercury_pd_pack == expected_pack for
every vector; exit 42 == all pass.

  python3 tools/gen_platform_decision_vectors.py --write
  python3 tools/gen_platform_decision_vectors.py --check   # exit 1 on drift (CI)
"""
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_platform_decision_vectors as spec  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VEC_DIR = os.path.join(ROOT, "vectors", "platform_decision")
POLICY = os.path.join(ROOT, "helix", "policy", "platform_decision.hx")
TEST = os.path.join(ROOT, "helix", "tests", "platform_decision_test.hx")
MANIFEST = os.path.join(ROOT, "policy", "platform_decision_v1.json")
MAIN_MARKER = "fn main()"


def vector_name(inp: dict) -> str:
    src = spec.SOURCE_LABEL[inp["source"]]
    label = spec.REASONS_BY_SOURCE[inp["source"]][inp["reason_code"]].lower()
    name = f"{src}__{label}"
    if "derived" in inp:
        name += f"__derived{inp['derived']}"
    return name


def build_vectors() -> list[dict]:
    out = []
    for inp in spec.enumerate_inputs():
        v = spec.view(inp)
        derived_note = f" / derived={inp['derived']}" if "derived" in inp else ""
        out.append(
            {
                "name": vector_name(inp),
                "description": f"{v['source']} / {v['reason_label']}{derived_note}",
                "input": inp,
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
        sys.exit("error: no fn main() in platform_decision.hx")
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
        "    // GENERATED from vectors/platform_decision/*.json by",
        "    // tools/gen_platform_decision_vectors.py --write. Do not edit by hand.",
    ]
    for v in vectors:
        i = v["input"]
        d = i.get("derived", 0)
        lines += [
            "",
            f"    // {v['name']}",
            f"    failed = failed + expect("
            f"mercury_pd_pack({i['source']}, {i['reason_code']}, {d}), {v['expected_pack']});",
        ]
    lines += ["", "    if failed == 0 { 42 } else { failed }", "}", ""]
    return "\n".join(lines)


def build_manifest() -> str:
    """The policy manifest, emitted from the spec registries (FU-3: no hand maintenance).
    Data (sources / reason codes / pack order / derived map / count) is computed; only the
    prose (schema/description/provenance) is constant."""
    reason_codes = {
        spec.SOURCE_LABEL[s]: {lbl: code for code, lbl in spec.REASONS_BY_SOURCE[s].items()}
        for s in (spec.SRC_BOOTSTRAP, spec.SRC_OUTBOUND, spec.SRC_RECEIVE)
    }
    derived_fields = {
        f"{spec.SOURCE_LABEL[s]}.{spec.REASONS_BY_SOURCE[s][c]}": field
        for (s, c), field in spec.DERIVED_AT.items()
    }
    manifest = {
        "schema": "mercury.platform_decision_policy.v1",
        "description": (
            "Platform-binding capability projection: for a decision mercury-core ALREADY made "
            "(source + reason_code, plus one derived bit for four pairs), the 13-field "
            "PlatformDecisionView the platform may rely on. Mirrors PlatformDecisionView::"
            "from_bootstrap/from_outbound_send/from_client_receive. Pure scalar projection: no "
            "JSON parse, no ciphertext, no cryptography."
        ),
        "generated_by": "tools/gen_platform_decision_vectors.py --write (do not hand-edit)",
        "sources": {label: sid for sid, label in spec.SOURCE_LABEL.items()},
        "view_fields": list(spec.VIEW_FIELDS),
        "pack_bit_order_msb_to_lsb": list(spec.BOOL_FIELDS),
        "derived_fields": derived_fields,
        "reason_codes": reason_codes,
        "vector_count": sum(1 for _ in spec.enumerate_inputs()),
        "vectors_generated_by": (
            "tools/gen_platform_decision_vectors.py (exhaustive over source x reason x "
            "derived-bit, expected computed by tools/check_platform_decision_vectors.py::view; "
            "--check gates drift)"
        ),
        "source_of_truth": (
            "core/rust/mercury-core/src/lib.rs PlatformDecisionView::from_* (30309-30362) + "
            "reason enums; pinned by core/rust/mercury-core/tests/platform_decision_vectors.rs"
        ),
    }
    return json.dumps(manifest, indent=2, ensure_ascii=False) + "\n"


def render() -> tuple[dict[str, str], str]:
    vectors = build_vectors()
    files = {
        os.path.join(VEC_DIR, v["name"] + ".json"):
        json.dumps(v, indent=2, ensure_ascii=False) + "\n"
        for v in vectors
    }
    return files, build_test(vectors)


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate platform_decision vectors + Helix test.")
    grp = ap.add_mutually_exclusive_group(required=True)
    grp.add_argument("--write", action="store_true", help="write vectors + test.hx")
    grp.add_argument("--check", action="store_true", help="fail (exit 1) on drift")
    args = ap.parse_args()

    files, test = render()

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
            fh.write(build_manifest())
        print(f"wrote {len(files)} vectors + platform_decision_test.hx + manifest")
        return 0

    drift = []
    for path, content in files.items():
        if not os.path.exists(path) or open(path, encoding="utf-8").read() != content:
            drift.append(os.path.relpath(path, ROOT))
    if not os.path.exists(TEST) or open(TEST, encoding="utf-8").read() != test:
        drift.append(os.path.relpath(TEST, ROOT))
    if not os.path.exists(MANIFEST) or open(MANIFEST, encoding="utf-8").read() != build_manifest():
        drift.append(os.path.relpath(MANIFEST, ROOT))
    if drift:
        print("DRIFT:", *drift, sep="\n  ", file=sys.stderr)
        print("\nRun: python3 tools/gen_platform_decision_vectors.py --write", file=sys.stderr)
        return 1
    print(f"platform_decision: vectors + test up to date ({len(files)} vectors)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
