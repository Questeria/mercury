#!/usr/bin/env bash
#
# Linux/CI runner for the Helix policy proofs (Track 0a). Equivalent to
# tools/run_helix_checks.ps1, but runs the emitted ELF NATIVELY (no WSL) and so
# is what GitHub Actions executes. For each of the twelve policies it:
#   1. typechecks the policy + emits a per-function hash (--check-only --hash)
#   2. emits proof obligations (--emit-proof-obligations --strict)
#   3. typechecks the test, compiles it to an ELF (-O1), runs it (exit 42 == ok)
#
# Usage:  tools/run_helix_checks.sh [HELIX_ROOT]
#   HELIX_ROOT defaults to third_party/helix (the pinned submodule) or the
#   HELIX_ROOT env var. It must be importable as the `helixc` package root.
set -euo pipefail

HELIX_ROOT="${1:-${HELIX_ROOT:-third_party/helix}}"
MERCURY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$MERCURY_ROOT/build/helix"

cd "$MERCURY_ROOT"

# Resolve HELIX_ROOT to an absolute path for PYTHONPATH.
HELIX_ROOT="$(cd "$HELIX_ROOT" && pwd)"
if [ ! -d "$HELIX_ROOT/helixc" ]; then
  echo "error: helixc package not found under HELIX_ROOT=$HELIX_ROOT" >&2
  echo "       (did you check out submodules? git submodule update --init --recursive)" >&2
  exit 1
fi
export PYTHONPATH="$HELIX_ROOT${PYTHONPATH:+:$PYTHONPATH}"

PY="${PYTHON:-python3}"
mkdir -p "$OUT_DIR"

policies=(envelope ai_grant ai_grant_lifecycle room_epoch policy_pipeline relay_submit platform_decision outbound_decide receive_decide bootstrap_decide inbound_sync account_recovery)

# Per-policy proof manifests (compiler-verified per-function effect/purity attestation) MUST be
# current first — the attestation manifest folds each one's hash in, so a stale manifest would
# otherwise surface only as attestation drift. This also fails closed if any policy function is not
# side-effect-free.
"$PY" tools/gen_proof_manifests.py --check
"$PY" tools/gen_policy_attestations.py --check

for p in "${policies[@]}"; do
  policy="helix/policy/$p.hx"
  test="helix/tests/${p}_test.hx"
  bin="$OUT_DIR/${p}_test.bin"
  echo "== $p =="
  "$PY" -m helixc.check "$policy" --no-stdlib --check-only --strict --hash
  "$PY" -m helixc.check "$policy" --no-stdlib --emit-proof-obligations --strict
  "$PY" -m helixc.check "$test" --no-stdlib --check-only --strict --hash
  "$PY" -m helixc.check "$test" --no-stdlib -O1 -o "$bin"
  chmod +x "$bin"
  set +e
  "$bin"
  code=$?
  set -e
  if [ "$code" -ne 42 ]; then
    echo "error: Helix runtime test '$p' expected exit 42, got $code" >&2
    exit 1
  fi
  echo "runtime: OK ($p exit 42)"
done

# Exhaustive Helix differential: compile + run the policy over its FULL input domain (not just the
# covering vectors), asserting the COMPILED Helix agrees with the Python spec at every point. This
# closes the gap where only a Rust PORT (decider_exhaustive_diff.rs) was checked exhaustively — the
# compiled .hx was previously run only over the curated sample.
"$PY" tools/gen_exhaustive_helix_diff.py --check
for ex in helix/tests/exhaustive/*_exhaustive_test.hx; do
  [ -e "$ex" ] || continue
  exname="$(basename "$ex" .hx)"
  exbin="$OUT_DIR/${exname}.bin"
  echo "== exhaustive: $exname =="
  "$PY" -m helixc.check "$ex" --no-stdlib --check-only --strict --hash
  "$PY" -m helixc.check "$ex" --no-stdlib -O1 -o "$exbin"
  chmod +x "$exbin"
  set +e
  "$exbin"
  code=$?
  set -e
  if [ "$code" -ne 42 ]; then
    echo "error: exhaustive Helix diff '$exname' expected exit 42 (compiled Helix == spec over the full domain), got $code" >&2
    exit 1
  fi
  echo "exhaustive: OK ($exname exit 42)"
done

# Negative regression: the purity gate MUST reject an impure @pure decider. A CLEAN compile here means
# effect-check enforcement silently regressed (e.g. --strict dropped, or the IR-level pass bypassed) —
# which would also hollow out gen_proof_manifests.py's "every policy function is side-effect-free"
# assertion. Lock it by asserting this fixture FAILS to compile.
echo "== purity gate (negative regression) =="
set +e
"$PY" -m helixc.check helix/tests/regression/purity_violation.hx --no-stdlib -O1 --strict \
  -o "$OUT_DIR/_purity_violation.bin" >/dev/null 2>&1
pv_code=$?
set -e
if [ "$pv_code" -eq 0 ]; then
  echo "error: purity gate REGRESSED — an impure @pure decider compiled clean (expected trap 19001)" >&2
  exit 1
fi
echo "purity gate: OK (impure @pure decider correctly rejected, exit $pv_code)"

# IFC data-egress contract (demonstrator): Helix information-flow types enforce the data-leak
# guardrail invariant. The demonstrator MUST compile + run (exit 42 == the gated declassify works);
# the leak fixture (confidential data -> the public remote sink with NO declassify) MUST FAIL to
# compile. If IFC enforcement ever regresses, the leak fixture compiles clean and this catches it.
echo "== IFC data-egress contract =="
"$PY" -m helixc.check helix/demonstrators/ai_egress_ifc.hx --no-stdlib --check-only --strict --hash
"$PY" -m helixc.check helix/demonstrators/ai_egress_ifc.hx --no-stdlib -O1 -o "$OUT_DIR/ai_egress_ifc.bin"
chmod +x "$OUT_DIR/ai_egress_ifc.bin"
set +e
"$OUT_DIR/ai_egress_ifc.bin"
ifc_code=$?
"$PY" -m helixc.check helix/tests/regression/ai_egress_leak.hx --no-stdlib --check-only --strict >/dev/null 2>&1
leak_code=$?
set -e
if [ "$ifc_code" -ne 42 ]; then
  echo "error: IFC demonstrator expected exit 42 (gated declassify), got $ifc_code" >&2
  exit 1
fi
if [ "$leak_code" -eq 0 ]; then
  echo "error: IFC contract REGRESSED — confidential data reached the public sink with no declassify (leak fixture compiled clean)" >&2
  exit 1
fi
echo "IFC data-egress contract: OK (demonstrator exit 42; leak rejected, exit $leak_code)"

echo "All Helix policies + tests passed (typecheck + hash + proof obligations + proof manifests + ELF execution + exhaustive differential + purity gate + IFC contract)."
