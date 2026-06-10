# Rust Decision View

Generated: 2026-05-27

## Status

Mercury client-core decisions now have a compact view for UI, client bindings, and audit-log boundaries:

```text
PolicyDecision::view() -> DecisionView
DecisionView::from_decision(PolicyDecision) -> DecisionView
```

`DecisionView` is derived from the already-evaluated `PolicyDecision`. It does not run policy again.

## Shape

The view exposes:

- `accepted`
- `reason`
- `audit_class`
- `primary_source`
- `primary_component`
- `components`

Every labeled reason uses the same compact shape:

```text
source: &'static str
code: i32
label: &'static str
```

This keeps integer codes available for storage, matching, and compact transport while giving clients stable source-qualified labels for display and audit.

## Serialization

`mercury-core` now has an optional `serde` feature:

```powershell
cargo test -p mercury-core --features serde decision_view_serializes_compact_fields
```

The feature is opt-in so the default core stays small. Serialization is manually implemented for the view structs instead of using derive macros.

## Verification

The core policy vector test asserts that `PolicyDecision::view()` matches the label layer for all nine client-core vectors. CI also runs the feature-gated JSON serialization test on Linux.

Local Windows note: test linking remains blocked here by the missing MSVC `link.exe`. The serde feature also pulls in serde build scripts and is blocked locally for the same reason, so local validation uses default `cargo check --workspace` plus the Python and Helix policy checks.

## Client Input Follow-Up

The first typed client message and room-state input boundary is documented in `docs/15_CLIENT_MESSAGE_POLICY_INPUT.md`.

## Next Step

The next increment should define a small local-client state module around this boundary: typed constructors or builders that prevent inconsistent room/device/AI state combinations before policy evaluation.
