# Rust Client Core Policy

Generated: 2026-05-27

## Status

Mercury now has its first client-core Rust layer:

- `core/rust/mercury-core/Cargo.toml`
- `core/rust/mercury-core/src/lib.rs`
- `core/rust/mercury-core/tests/core_policy_vectors.rs`
- `vectors/core_policy/*.json`

The new crate depends on `mercury-policy` and exposes typed fact structs plus one decision function:

```text
evaluate_policy(PolicyEvaluationInput) -> PolicyDecision
```

This layer is intentionally narrow. It does not perform networking, storage, cryptography, signature verification, AI execution, or message parsing. It turns already-parsed Mercury facts into component policy reasons and one final policy pipeline decision.

The follow-up label layer is documented in `docs/13_RUST_POLICY_LABELS.md`.

## Fact Types

The core layer currently groups facts into:

- `EnvelopeFacts`
- `RoomEpochFacts`
- `AiGrantFacts`
- `AiLifecycleFacts`
- `AiPolicyFacts`
- `PolicyEvaluationInput`
- `PolicyDecision`

Human actors may omit AI facts. AI actors must provide AI facts, otherwise the core layer reports an AI grant rejection through the policy pipeline. If a human actor carries AI facts, the core layer marks that as an AI component attached to a human decision.

## Verification

Run:

```powershell
cargo check --workspace
cargo test --workspace
```

On this Windows machine, `cargo test --workspace` remains blocked by the missing MSVC linker `link.exe`. GitHub Actions is configured to run workspace tests on Linux.

## Next Step

The next increment should add a compact serialized decision view for client UI and audit-log boundaries, derived from `PolicyDecision::labels()` instead of re-evaluating policy.
