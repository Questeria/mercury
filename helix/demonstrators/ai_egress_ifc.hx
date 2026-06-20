// DEMONSTRATOR — a compile-checked AI-data-egress confidentiality contract using Helix
// information-flow types. NOT a production policy: Mercury's runtime egress enforcement lives in
// Rust. This proves Helix's IFC can EXPRESS + COMPILE-CHECK the data-leak guardrail's core invariant.
//
// The invariant: context data labelled Confidential<T> can reach a remote AI provider (a plain-typed
// "public" sink) ONLY through an explicit, audit-greppable __declassify, and only inside a
// valid-grant branch. Any DIRECT flow of confidential data to the remote sink is a COMPILE ERROR —
// see the negative fixture helix/tests/regression/ai_egress_leak.hx, which run_helix_checks.sh
// asserts fails to compile.
//
// Honest scope: this proves DIRECT-flow non-leakage at the type level (no value path from a labelled
// source to the public sink without an explicit declassify); it does not model implicit/covert
// channels. Same framing as the guardrail wedge: provably ENFORCE + audit the egress points, not
// claim completeness.

// The public boundary: anything reaching a remote AI provider crosses this plain-i32 sink.
@pure
fn send_to_remote_provider(payload: i32) -> i32 { payload }

// Pure policy fact: remote egress is permitted only when the grant is ACCEPT (0) AND it explicitly
// allows a remote provider (remote_allowed == 1).
@pure
fn remote_egress_permitted(grant_reason: i32, remote_allowed: i32) -> bool {
    if grant_reason == 0 { remote_allowed == 1 } else { false }
}

// The egress decision. `context` is Confidential, so it can ONLY reach send_to_remote_provider via an
// explicit __declassify — reachable here solely inside the permitted branch. When egress is not
// permitted the data is never declassified (returns a blocked code -1).
fn ai_egress_decide(context: Confidential<i32>, grant_reason: i32, remote_allowed: i32) -> i32 {
    if remote_egress_permitted(grant_reason, remote_allowed) {
        send_to_remote_provider(__declassify(context))
    } else {
        -1
    }
}

fn main() -> i32 {
    let permitted = ai_egress_decide(__wrap_taint(7), 0, 1);   // accept + remote allowed -> 7
    let blocked = ai_egress_decide(__wrap_taint(7), 0, 0);     // remote not allowed   -> -1
    if permitted == 7 { if blocked == 0 - 1 { 42 } else { 1 } } else { 2 }
}
