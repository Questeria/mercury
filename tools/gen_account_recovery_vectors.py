#!/usr/bin/env python3
"""Generate the account_recovery golden vectors + Helix test + manifest from the spec.

Correct-by-construction from tools/check_account_recovery_vectors.py over its representative
covering set (all 13 reasons + the method passthrough + the priority boundaries). Because no single
Helix fn takes all 13 reduced inputs (the <=6-param codegen cap), the test asserts PER VECTOR the
four stage-reason fns over their raw inputs, the first-non-zero composer over the stage reasons, and
the output pack over the reason -- six expects, exit 42 on all pass.

  python3 tools/gen_account_recovery_vectors.py --write
  python3 tools/gen_account_recovery_vectors.py --check   # exit 1 on drift (CI)
"""
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_account_recovery_vectors as spec  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VEC_DIR = os.path.join(ROOT, "vectors", "account_recovery")
POLICY = os.path.join(ROOT, "helix", "policy", "account_recovery.hx")
TEST = os.path.join(ROOT, "helix", "tests", "account_recovery_test.hx")
MANIFEST = os.path.join(ROOT, "policy", "account_recovery_v1.json")
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
                "description": f"{v['reason_label']} / method={inp['method_code']} / pack={spec.pack(v)}",
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
        sys.exit("error: no fn main() in account_recovery.hx")
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
        "    // GENERATED from vectors/account_recovery/*.json by",
        "    // tools/gen_account_recovery_vectors.py --write. Do not edit by hand.",
    ]
    for v in vectors:
        i, e = v["input"], v["expected"]
        stage_a = (f"mercury_ar_stage_a({i['recovery_requested']}, {i['method_code']}, "
                   f"{i['entropy_ok']}, {i['digest_ok']})")
        stage_b = (f"mercury_ar_stage_b({i['method_code']}, {i['threshold_ok']}, "
                   f"{i['device_approval_present']})")
        stage_c = (f"mercury_ar_stage_c({i['server_authenticated']}, {i['server_rate_limited']}, "
                   f"{i['backup_encrypted']}, {i['plaintext_backup_ok']})")
        stage_d = (f"mercury_ar_stage_d({i['high_security_account']}, "
                   f"{i['rotates_device_secret']}, {i['audit_ok']})")
        reason = (f"mercury_ar_reason({e['stage_a_reason']}, {e['stage_b_reason']}, "
                  f"{e['stage_c_reason']}, {e['stage_d_reason']})")
        pack = f"mercury_ar_pack({e['reason_code']})"
        lines += [
            "",
            f"    // {v['name']}",
            f"    failed = failed + expect({stage_a}, {e['stage_a_reason']});",
            f"    failed = failed + expect({stage_b}, {e['stage_b_reason']});",
            f"    failed = failed + expect({stage_c}, {e['stage_c_reason']});",
            f"    failed = failed + expect({stage_d}, {e['stage_d_reason']});",
            f"    failed = failed + expect({reason}, {e['reason_code']});",
            f"    failed = failed + expect({pack}, {v['expected_pack']});",
        ]
    lines += ["", "    if failed == 0 { 42 } else { failed }", "}", ""]
    return "\n".join(lines)


def build_manifest() -> str:
    manifest = {
        "schema": "mercury.account_recovery_policy.v1",
        "description": (
            "Account-recovery decider: mirrors mercury-core evaluate_account_recovery "
            "(AccountRecoveryInput -> AccountRecoveryDecision), a 12-gate priority chain (13 reasons) "
            "staged into pre-key/method-quorum/server-backup/rotation-audit stage reasons + a "
            "first-non-zero composer + output derivation (<=6 int params per fn). The rich input is "
            "reduced to scalars (method_code + entropy/digest/threshold/audit predicates); the "
            "recovery method passes through to the decision. Pure scalar; no crypto/JSON."
        ),
        "generated_by": "tools/gen_account_recovery_vectors.py --write (do not hand-edit)",
        "input_fields": list(spec.INPUT_FIELDS),
        "decision_fields": list(spec.DECISION_FIELDS),
        "stage_fields": list(spec.STAGE_FIELDS),
        "pack_low_bits_msb_to_lsb": list(spec.BOOL_FIELDS),
        "pack_layout": (
            "reason_code*512 + accepted*256 + can_start_recovery*128 + can_restore_device_secret*64 "
            "+ requires_user_action*32 + requires_server_setup*16 + requires_key_rotation*8 + "
            "forbids_low_entropy_recovery*4 + forbids_plaintext_backup*2 + plaintext_bytes_exposed "
            "(method passes through, asserted separately not packed)"
        ),
        "reason_codes": {label: code for code, label in spec.ACCOUNT_RECOVERY_REASONS.items()},
        "vector_count": len(spec.enumerate_inputs()),
        "source_of_truth": (
            "core/rust/mercury-core/src/lib.rs evaluate_account_recovery (21949); pinned by "
            "core/rust/mercury-core/tests/account_recovery_vectors.rs. Exhaustive agreement: the FU-4 "
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
    ap = argparse.ArgumentParser(description="Generate account_recovery vectors + test + manifest.")
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
        print(f"wrote {len(files)} vectors + account_recovery_test.hx + manifest")
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
        print("\nRun: python3 tools/gen_account_recovery_vectors.py --write", file=sys.stderr)
        return 1
    print(f"account_recovery: vectors + test + manifest up to date ({len(files)} vectors)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
