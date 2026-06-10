#!/usr/bin/env python3
"""Generate the bootstrap_decide golden vectors + Helix test + manifest from the spec.

Correct-by-construction from tools/check_bootstrap_decide_vectors.py over its representative
covering set (all 21 reasons + the priority boundaries + the derived KEY_TRANSPARENCY_NOT_READY
rua). Because no single Helix fn takes all 12 inputs (the <=6-param codegen limit), the test asserts
PER VECTOR the four stage-reason fns over their raw inputs, the first-non-zero composer over the
stage reasons, and the output pack over (reason, kt_requires_user_action) -- six expects, exit 42 on
all pass.

  python3 tools/gen_bootstrap_decide_vectors.py --write
  python3 tools/gen_bootstrap_decide_vectors.py --check   # exit 1 on drift (CI)
"""
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_bootstrap_decide_vectors as spec  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VEC_DIR = os.path.join(ROOT, "vectors", "bootstrap_decide")
POLICY = os.path.join(ROOT, "helix", "policy", "bootstrap_decide.hx")
TEST = os.path.join(ROOT, "helix", "tests", "bootstrap_decide_test.hx")
MANIFEST = os.path.join(ROOT, "policy", "bootstrap_decide_v1.json")
MAIN_MARKER = "fn main()"


def build_vectors() -> list[dict]:
    out = []
    seen: dict[str, int] = {}
    for inp in spec.enumerate_inputs():
        v = spec.decide(inp)
        label = v["reason_label"].lower()
        seen[label] = seen.get(label, 0) + 1
        out.append(
            {
                "name": f"{v['reason_code']:02d}_{label}__{seen[label]}",
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
        sys.exit("error: no fn main() in bootstrap_decide.hx")
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
        "    // GENERATED from vectors/bootstrap_decide/*.json by",
        "    // tools/gen_bootstrap_decide_vectors.py --write. Do not edit by hand.",
    ]
    for v in vectors:
        i, e = v["input"], v["expected"]
        pre = (f"mercury_bd_pre({i['account_id_ok']}, {i['device_id_ok']}, "
               f"{i['pending_recovery']}, {i['plaintext_cache_ok']}, {i['local_trust']})")
        kt = f"mercury_bd_kt({i['kt_code']})"
        secrets = (f"mercury_bd_secrets({i['account_secret']}, {i['device_secret']}, "
                   f"{i['room_state']})")
        rs = f"mercury_bd_rs({i['replay_checkpoint']}, {i['sync_state']})"
        reason = (f"mercury_bd_reason({e['pre_reason']}, {e['kt_reason']}, "
                  f"{e['secrets_reason']}, {e['rs_reason']})")
        pack = f"mercury_bd_pack({e['reason_code']}, {i['kt_requires_user_action']})"
        lines += [
            "",
            f"    // {v['name']}",
            f"    failed = failed + expect({pre}, {e['pre_reason']});",
            f"    failed = failed + expect({kt}, {e['kt_reason']});",
            f"    failed = failed + expect({secrets}, {e['secrets_reason']});",
            f"    failed = failed + expect({rs}, {e['rs_reason']});",
            f"    failed = failed + expect({reason}, {e['reason_code']});",
            f"    failed = failed + expect({pack}, {v['expected_pack']});",
        ]
    lines += ["", "    if failed == 0 { 42 } else { failed }", "}", ""]
    return "\n".join(lines)


def build_manifest() -> str:
    manifest = {
        "schema": "mercury.bootstrap_decide_policy.v1",
        "description": (
            "Client-bootstrap decider: mirrors mercury-core evaluate_client_bootstrap "
            "(ClientBootstrapInput -> ClientBootstrapDecision), the richest decider (21 reasons), a "
            "~13-gate priority chain staged into pre/kt/secrets/rs stage reasons + a first-non-zero "
            "composer + output derivation (<=6 int params per fn). The richer state enums are reduced "
            "to small branch-equivalent codes (kt_code, secret/replay/sync state). Pure scalar; no "
            "crypto/JSON."
        ),
        "generated_by": "tools/gen_bootstrap_decide_vectors.py --write (do not hand-edit)",
        "input_fields": list(spec.INPUT_FIELDS),
        "decision_fields": list(spec.DECISION_FIELDS),
        "stage_fields": list(spec.STAGE_FIELDS),
        "pack_low_bits_msb_to_lsb": list(spec.BOOL_FIELDS),
        "pack_layout": (
            "reason_code*128 + accepted*64 + can_start_sync*32 + can_decrypt_local_store*16 + "
            "can_open_message_ui*8 + requires_sync*4 + requires_recovery*2 + requires_user_action"
        ),
        "reason_codes": {label: code for code, label in spec.BOOTSTRAP_REASONS.items()},
        "vector_count": len(spec.enumerate_inputs()),
        "source_of_truth": (
            "core/rust/mercury-core/src/lib.rs evaluate_client_bootstrap (21528); pinned by "
            "core/rust/mercury-core/tests/bootstrap_decide_vectors.rs. Exhaustive agreement: the FU-4 exhaustive differential."
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
    ap = argparse.ArgumentParser(description="Generate bootstrap_decide vectors + test + manifest.")
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
        print(f"wrote {len(files)} vectors + bootstrap_decide_test.hx + manifest")
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
        print("\nRun: python3 tools/gen_bootstrap_decide_vectors.py --write", file=sys.stderr)
        return 1
    print(f"bootstrap_decide: vectors + test + manifest up to date ({len(files)} vectors)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
