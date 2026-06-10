#!/usr/bin/env python3
"""Generate the receive_decide golden vectors + Helix test + manifest from the spec.

Correct-by-construction from tools/check_receive_decide_vectors.py over its representative
covering set (all 13 reasons + the priority boundaries + the two derived cases). Because no
single Helix fn takes all 12 inputs (the <=6-param codegen limit), the test asserts PER VECTOR
the three stage-reason fns over their raw inputs, the first-non-zero composer over the stage
reasons, and the output pack over (reason, ack_rcr, dt_rua) -- five expects, exit 42 on all pass.

  python3 tools/gen_receive_decide_vectors.py --write
  python3 tools/gen_receive_decide_vectors.py --check   # exit 1 on drift (CI)
"""
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_receive_decide_vectors as spec  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VEC_DIR = os.path.join(ROOT, "vectors", "receive_decide")
POLICY = os.path.join(ROOT, "helix", "policy", "receive_decide.hx")
TEST = os.path.join(ROOT, "helix", "tests", "receive_decide_test.hx")
MANIFEST = os.path.join(ROOT, "policy", "receive_decide_v1.json")
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
        sys.exit("error: no fn main() in receive_decide.hx")
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
        "    // GENERATED from vectors/receive_decide/*.json by",
        "    // tools/gen_receive_decide_vectors.py --write. Do not edit by hand.",
    ]
    for v in vectors:
        i, e = v["input"], v["expected"]
        relay = f"mercury_rd_relay({i['relay_accepted']}, {i['relay_delivered']}, {i['ack_duplicate']}, {i['ack_accepted']})"
        content = f"mercury_rd_content({i['ciphertext_digest_ok']}, {i['plaintext_identity_ok']}, {i['replay_state']})"
        trust = f"mercury_rd_trust({i['dt_can_send']}, {i['message_policy_accepted']}, {i['ciphertext_sealing_accepted']})"
        reason = f"mercury_rd_reason({e['relay_reason']}, {e['content_reason']}, {e['trust_reason']})"
        pack = f"mercury_rd_pack({e['reason_code']}, {i['ack_requires_client_retry']}, {i['dt_requires_user_action']})"
        lines += [
            "",
            f"    // {v['name']}",
            f"    failed = failed + expect({relay}, {e['relay_reason']});",
            f"    failed = failed + expect({content}, {e['content_reason']});",
            f"    failed = failed + expect({trust}, {e['trust_reason']});",
            f"    failed = failed + expect({reason}, {e['reason_code']});",
            f"    failed = failed + expect({pack}, {v['expected_pack']});",
        ]
    lines += ["", "    if failed == 0 { 42 } else { failed }", "}", ""]
    return "\n".join(lines)


def build_manifest() -> str:
    manifest = {
        "schema": "mercury.receive_decide_policy.v1",
        "description": (
            "Client-receive decider: mirrors mercury-core evaluate_client_receive "
            "(ClientReceiveInput -> ClientReceiveDecision), an 11-gate priority chain, staged into "
            "relay/content/trust stage reasons + a first-non-zero composer + output derivation "
            "(<=6 int params per fn). Pure scalar; no crypto/JSON/ciphertext."
        ),
        "generated_by": "tools/gen_receive_decide_vectors.py --write (do not hand-edit)",
        "input_fields": list(spec.INPUT_FIELDS),
        "decision_fields": list(spec.DECISION_FIELDS),
        "stage_fields": list(spec.STAGE_FIELDS),
        "pack_low_bits_msb_to_lsb": list(spec.BOOL_FIELDS),
        "pack_layout": "reason_code*64 + accepted*32 + can_decrypt*16 + can_persist_ciphertext*8 + can_expose_to_ui*4 + requires_client_retry*2 + requires_user_action",
        "reason_codes": {label: code for code, label in spec.RECEIVE_REASONS.items()},
        "vector_count": len(spec.enumerate_inputs()),
        "source_of_truth": (
            "core/rust/mercury-core/src/lib.rs evaluate_client_receive (17813); pinned by "
            "core/rust/mercury-core/tests/receive_decide_vectors.rs. Exhaustive agreement: the FU-4 exhaustive differential."
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
    ap = argparse.ArgumentParser(description="Generate receive_decide vectors + test + manifest.")
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
        print(f"wrote {len(files)} vectors + receive_decide_test.hx + manifest")
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
        print("\nRun: python3 tools/gen_receive_decide_vectors.py --write", file=sys.stderr)
        return 1
    print(f"receive_decide: vectors + test + manifest up to date ({len(files)} vectors)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
