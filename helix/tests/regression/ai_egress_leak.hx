// NEGATIVE fixture — MUST FAIL to compile.
//
// A Confidential<i32> flowing DIRECTLY into the remote-provider sink (a plain-i32 public boundary)
// with NO __declassify and no grant gate. Helix's information-flow types reject this as a type
// error; run_helix_checks.sh asserts the compile FAILS. This locks the data-egress confidentiality
// contract demonstrated in helix/demonstrators/ai_egress_ifc.hx — if IFC enforcement ever silently
// regressed, this fixture would start compiling and the gate would catch it.
@pure
fn send_to_remote_provider(payload: i32) -> i32 { payload }

fn leak(context: Confidential<i32>) -> i32 {
    send_to_remote_provider(context)
}

fn main() -> i32 { leak(__wrap_taint(7)) }
