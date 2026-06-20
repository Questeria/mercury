#!/usr/bin/env python3
"""Generate Mercury Helix policy PROOF MANIFESTS — a tamper-evident, per-function SEMANTIC
attestation that complements the source-hash attestation in gen_policy_attestations.py.

For each of the 12 policies this typechecks the Helix source with the PINNED compiler (the same
`--no-stdlib` mode CI uses) and records, per function, the compiler-VERIFIED effect set + purity,
bound to the normalized source hash and the pinned compiler version, under a canonical tamper-evident
SHA-256 (helixc.backend.proof_manifest, Stage 122). It then ASSERTS the load-bearing property that
every policy function is side-effect-free (effects == []), failing closed otherwise.

This upgrades the evidence trail from "these source bytes" (gen_policy_attestations) to "these source
bytes typecheck, under compiler version V, to N functions that are ALL compiler-verified
side-effect-free" — a semantic receipt, not just a content hash.

HONEST SCOPE (carried verbatim into docs/HELIX_EVIDENCE_TRAIL.md):
  * UNSIGNED. The manifest is tamper-EVIDENT (a canonical SHA-256 that detects accidental edits), not
    AUTHENTIC. The helixc substrate ships only the unsigned digest; Ed25519 release-signing over the
    canonical bytes is a deliberate next step that needs a protected release key.
  * Binds the normalized SOURCE (reproducible across Windows/Linux), NOT the Linux ELF (whose
    cross-platform byte-determinism is unverified). The ELF is separately compiled + exit-42-run by
    run_helix_checks.sh.
  * Attests effects/purity from the TYPE SYSTEM. It is NOT a proof of functional correctness — that
    is the vector differential's job (run_helix_checks.sh + the Rust/Python mirrors).
"""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
from pathlib import Path

from gen_policy_attestations import (
    POLICIES,
    ROOT,
    _normalize_newlines,
    check_file,
    rel,
    write_if_changed,
)

# The pinned compiler must be importable as the `helixc` package.
sys.path.insert(0, str(ROOT / "third_party" / "helix"))

from helixc.backend.proof_manifest import (  # noqa: E402
    SignatureFormat,
    as_sha256_hex,
    emit_manifest,
    serialize_manifest,
    verify_manifest_hash,
)
from helixc.frontend.parser import parse  # noqa: E402
from helixc.frontend.typecheck import TypeChecker  # noqa: E402

MANIFEST_DIR = ROOT / "helix" / "policy" / "manifests"


def helix_version() -> str:
    """A reproducible identifier for the pinned compiler (absent from the proof-obligation artifact's
    own metadata, so we pin it here). `git describe` on the submodule, falling back to the recorded
    gitlink SHA so the value is stable without a working `describe`."""
    helix_root = ROOT / "third_party" / "helix"
    try:
        out = subprocess.run(
            ["git", "-C", str(helix_root), "describe", "--tags", "--always", "--dirty"],
            capture_output=True,
            text=True,
            check=True,
        )
        return out.stdout.strip()
    except Exception:
        try:
            sha = subprocess.run(
                ["git", "-C", str(ROOT), "rev-parse", "HEAD:third_party/helix"],
                capture_output=True,
                text=True,
                check=True,
            ).stdout.strip()
            return f"gitlink-{sha}"
        except Exception:
            return "unknown"


def build_manifest_json(name: str, version: str) -> str:
    """Typecheck a policy and emit its canonical proof-manifest JSON. Fails closed if any function
    carries an effect (the deterministic / side-effect-free guarantee a policy decision must have)."""
    policy_path = ROOT / "helix" / "policy" / f"{name}.hx"
    raw = _normalize_newlines(policy_path.read_bytes())
    src = raw.decode("utf-8")

    prog = parse(src, include_stdlib=False)
    tc = TypeChecker(prog)
    tc.check()

    source_sha = as_sha256_hex(hashlib.sha256(raw).hexdigest())
    manifest = emit_manifest(
        tc.functions,
        artifact_path=rel(policy_path),
        artifact_sha256=source_sha,
        helix_version=version,
        signature_format=SignatureFormat.DEFERRED,
    )
    if not verify_manifest_hash(manifest):
        raise SystemExit(f"{name}: emitted manifest fails its own canonical hash check")

    data = manifest.to_dict()
    # FAIL-CLOSED gate: every policy function must be compiler-verified side-effect-free. A function
    # that prints / does FFI / mutates / traps would carry a non-empty effect set here and abort.
    with_effects = sorted(f["name"] for f in data["functions"] if f.get("effects"))
    if with_effects:
        raise SystemExit(
            f"{name}: policy functions must be side-effect-free, but these carry effects: {with_effects}"
        )
    return serialize_manifest(manifest, indent=2) + "\n"


def manifest_path(name: str) -> Path:
    return MANIFEST_DIR / f"{name}.manifest.json"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="write the per-policy manifests")
    parser.add_argument("--check", action="store_true", help="fail if any manifest is stale")
    args = parser.parse_args()

    version = helix_version()
    built = {name: build_manifest_json(name, version) for name in (p["name"] for p in POLICIES)}

    if args.write:
        for name, content in built.items():
            write_if_changed(manifest_path(name), content)
            print(f"wrote {rel(manifest_path(name))}")
        return 0

    if args.check:
        ok = True
        for name, content in built.items():
            ok = check_file(manifest_path(name), content) and ok
        if not ok:
            print("Run: python tools/gen_proof_manifests.py --write")
            return 1
        print(f"helix proof manifests: OK ({len(built)} policies, compiler {version})")
        return 0

    # Default: print the ai_grant manifest as a sample.
    print(built["ai_grant"], end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
