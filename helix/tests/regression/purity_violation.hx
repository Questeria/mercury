// NEGATIVE regression fixture — this MUST FAIL to compile.
//
// A @pure decider that performs I/O (print_int). The pinned Helix compiler rejects this under
// --strict via the IR-level effect-check pass (trap 19001). run_helix_checks.sh compiles this and
// asserts the compile FAILS, locking the purity gate: if a future toolchain or flag change silently
// demotes purity enforcement (e.g. drops --strict, or moves to --check-only which misses builtin
// I/O), this fixture would start compiling clean and the gate would catch the regression.
//
// gen_proof_manifests.py relies on every real policy function having an empty effect set; this proves
// the compiler actually computes + rejects effects, so that assertion is meaningful.
@pure
fn impure_decider(x: i32) -> i32 {
    print_int(x);
    if x == 0 { 1 } else { 0 }
}

fn main() -> i32 { impure_decider(0) }
