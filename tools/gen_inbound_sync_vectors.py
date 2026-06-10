#!/usr/bin/env python3
"""Generate the inbound_sync golden vectors + Helix test + manifest from the spec.

Correct-by-construction from tools/check_inbound_sync_vectors.py over its representative covering
set (all 9 reasons + the derived BOOTSTRAP_BLOCKED rua + the priority boundaries). inbound_sync is
a single-stage decider (its reason is computable in 6 int params, no staging), so the test asserts
PER VECTOR just two expects: the reason fn over the 6 reason-scalars, and the output pack over
(reason, bootstrap_rua) -- exit 42 on all pass.

  python3 tools/gen_inbound_sync_vectors.py --write
  python3 tools/gen_inbound_sync_vectors.py --check   # exit 1 on drift (CI)
"""
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_inbound_sync_vectors as spec  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VEC_DIR = os.path.join(ROOT, "vectors", "inbound_sync")
POLICY = os.path.join(ROOT, "helix", "policy", "inbound_sync.hx")
TEST = os.path.join(ROOT, "helix", "tests", "inbound_sync_test.hx")
MANIFEST = os.path.join(ROOT, "policy", "inbound_sync_v1.json")
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
                "name": f"{v['reason_code']}_{label}__{seen[label]}",
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
        sys.exit("error: no fn main() in inbound_sync.hx")
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
        "    // GENERATED from vectors/inbound_sync/*.json by",
        "    // tools/gen_inbound_sync_vectors.py --write. Do not edit by hand.",
    ]
    for v in vectors:
        i, e = v["input"], v["expected"]
        reason = (f"mercury_is_reason({i['plaintext_preview_ok']}, {i['bootstrap_can_start_sync']}, "
                  f"{i['source_state']}, {i['poll_batch_ok']}, {i['pending_delivery']}, "
                  f"{i['route_id_ok']})")
        pack = f"mercury_is_pack({e['reason_code']}, {i['bootstrap_rua']})"
        lines += [
            "",
            f"    // {v['name']}",
            f"    failed = failed + expect({reason}, {e['reason_code']});",
            f"    failed = failed + expect({pack}, {v['expected_pack']});",
        ]
    lines += ["", "    if failed == 0 { 42 } else { failed }", "}", ""]
    return "\n".join(lines)


def build_manifest() -> str:
    manifest = {
        "schema": "mercury.inbound_sync_policy.v1",
        "description": (
            "Client-sync decider: mirrors mercury-core evaluate_inbound_sync (InboundSyncInput -> "
            "InboundSyncDecision), a 9-reason priority chain whose reason is computable in 6 int "
            "params (no staging). The rich InboundSyncInput is reduced to scalars (bootstrap.can_start_sync "
            "+ requires_user_action, the InboundSyncSourceState code, and length/limit predicates). "
            "Pure scalar; no crypto/JSON."
        ),
        "generated_by": "tools/gen_inbound_sync_vectors.py --write (do not hand-edit)",
        "input_fields": list(spec.INPUT_FIELDS),
        "decision_fields": list(spec.DECISION_FIELDS),
        "pack_low_bits_msb_to_lsb": list(spec.BOOL_FIELDS),
        "pack_layout": (
            "reason_code*128 + accepted*64 + can_poll_relay*32 + can_run_receive_session*16 + "
            "can_update_replay_checkpoint*8 + requires_network_retry*4 + requires_user_action*2 + "
            "plaintext_bytes_exposed"
        ),
        "reason_codes": {label: code for code, label in spec.INBOUND_SYNC_REASONS.items()},
        "vector_count": len(spec.enumerate_inputs()),
        "source_of_truth": (
            "core/rust/mercury-core/src/lib.rs evaluate_inbound_sync (30212); pinned by "
            "core/rust/mercury-core/tests/inbound_sync_vectors.rs. Exhaustive agreement: the FU-4 "
            "exhaustive differential."
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
    ap = argparse.ArgumentParser(description="Generate inbound_sync vectors + test + manifest.")
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
        print(f"wrote {len(files)} vectors + inbound_sync_test.hx + manifest")
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
        print("\nRun: python3 tools/gen_inbound_sync_vectors.py --write", file=sys.stderr)
        return 1
    print(f"inbound_sync: vectors + test + manifest up to date ({len(files)} vectors)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
