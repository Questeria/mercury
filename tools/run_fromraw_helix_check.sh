#!/usr/bin/env bash
#
# FU-2: cross-check every Mercury Helix policy test under the FROM-RAW, self-hosted Helix
# compiler, INDEPENDENT of the pinned third_party/helix Python helixc that
# tools/run_helix_checks.sh uses.
#
# Why this exists: run_helix_checks.sh proves the policies against ONE compiler (the pinned
# helixc). This runs the SAME .hx policy tests + the SAME golden vectors through a compiler of a
# completely different lineage -- Helix's raw-binary bootstrap: hex0 -> a 62 KB
# raw-binary seed (sha256-pinned, no Python) -> K1, the self-hosted Helix compiler. If the two
# independent compilers agree (every test's emitted ELF exits 42, the all-assertions-pass
# sentinel), then neither compiler's codegen is silently masking a policy bug. It is a
# differential check ON THE COMPILERS, complementing the Rust pins (which check the policy logic
# against mercury-core).
#
# Status: LOCAL / manual cross-check, NOT a CI gate. The from-raw compiler lives in the separate
# Helix repo, which is not a Mercury build dependency, so this is absent in CI and
# SKIPs cleanly (exit 0) when the toolchain is not present. The pinned-helixc proofs in
# tools/run_helix_checks.sh + .github/workflows/ci.yml gate merges regardless.
#
# READ-ONLY on Helix: the seed + k1src.hx are copied out and the entire K1 build and
# every policy compile run under /tmp; nothing is written into the Helix tree. K1 has a
# frozen I/O convention (reads /tmp/k1_in.hx, writes /tmp/k1_out.bin), so this uses those exact
# paths and must run alone (no other concurrent from-raw build sharing /tmp).
#
# A NOTE ON STAGES (honest): the raw 62 KB *seed* is the MINIMAL bootstrap compiler -- enough to
# compile the next stage -- and its codegen does NOT cover every construct (e.g. the `bool` type /
# `true`/`false` / `== false` that envelope.hx uses); feeding policy tests directly to the seed
# diverges on those. K1 -- the self-hosted compiler the seed produces -- is the feature-complete
# from-raw v1.0 Helix and compiles all of them correctly. This script therefore cross-checks under
# K1 (the real compiler), not the seed.
#
# Usage: tools/run_fromraw_helix_check.sh [HELIX_BOOTSTRAP_DIR]
#   default dir: $HELIX_BOOTSTRAP or
#                /mnt/c/Projects/Helix/stage0/helixc-bootstrap
set -u

KN="${1:-${HELIX_BOOTSTRAP:-/mnt/c/Projects/Helix/stage0/helixc-bootstrap}}"
MERCURY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TESTS_DIR="$MERCURY_ROOT/helix/tests"

if [ ! -f "$KN/seed.bin" ] || [ ! -f "$KN/k1src.hx" ]; then
  echo "SKIP: from-raw Helix toolchain not found at:"
  echo "        $KN  (need seed.bin + k1src.hx)"
  echo "      This optional differential cross-check needs the Helix raw-binary"
  echo "      bootstrap. The pinned-helixc proofs (tools/run_helix_checks.sh) gate the policies."
  exit 0
fi

# Stage the from-raw seed + self-host source into /tmp (read-only on Helix).
cp "$KN/seed.bin" /tmp/fr_seed.bin && chmod +x /tmp/fr_seed.bin
cp "$KN/k1src.hx" /tmp/fr_k1src.hx

echo "== build self-hosted from-raw K1 (raw seed compiles k1src.hx) =="
rm -f /tmp/fr_K1.bin
/tmp/fr_seed.bin /tmp/fr_k1src.hx /tmp/fr_K1.bin >/dev/null 2>&1 || true
if [ ! -s /tmp/fr_K1.bin ]; then
  echo "error: from-raw K1 build produced no binary" >&2
  exit 1
fi
chmod +x /tmp/fr_K1.bin
echo "   K1 = $(stat -c%s /tmp/fr_K1.bin) bytes"

pass=0
fail=0
for test in "$TESTS_DIR"/*_test.hx; do
  name="$(basename "$test" .hx)"
  cp "$test" /tmp/k1_in.hx          # K1's frozen input path
  rm -f /tmp/k1_out.bin
  timeout 120 /tmp/fr_K1.bin >/dev/null 2>&1 || true
  if [ ! -s /tmp/k1_out.bin ]; then
    printf '  %-28s K1-COMPILE-FAIL\n' "$name"
    fail=$((fail + 1))
    continue
  fi
  chmod +x /tmp/k1_out.bin
  timeout 30 /tmp/k1_out.bin
  rc=$?
  if [ "$rc" -eq 42 ]; then
    printf '  %-28s OK (exit 42)\n' "$name"
    pass=$((pass + 1))
  else
    printf '  %-28s FAIL (exit %s, expected 42)\n' "$name" "$rc"
    fail=$((fail + 1))
  fi
done

echo "---"
echo "from-raw K1 cross-check: $pass passed, $fail failed"
if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "All Mercury Helix policies agree under the pinned helixc AND the from-raw self-hosted K1."
