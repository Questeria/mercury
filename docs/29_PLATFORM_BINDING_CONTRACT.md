# Platform Binding Contract

Generated: 2026-05-28

## Status

Mercury now has a first platform binding view in `mercury-core`.

```text
PlatformDecisionView
PlatformDecisionView::from_bootstrap(...)
PlatformDecisionView::from_outbound_send(...)
PlatformDecisionView::from_client_receive(...)
PlatformDecisionView::from_policy(...)
```

This is the stable decision shape mobile and desktop clients should consume before UI work begins. It does not replace the richer Rust decisions. It prevents platform bindings from copying security logic into Swift, Kotlin, TypeScript, C#, or UI code.

## Shape

Every platform decision view carries:

- `source`
- `accepted`
- `reason_code`
- `reason_label`
- `can_open_message_ui`
- `can_start_sync`
- `can_send`
- `can_receive`
- `can_persist_ciphertext`
- `requires_sync`
- `requires_recovery`
- `requires_client_retry`
- `requires_user_action`

The view is intentionally capability-oriented. UI code should use these booleans, not reinterpret lower-level policy, relay, store, trust, or sync reasons.

## Sources

Current sources are:

- `client_bootstrap`
- `outbound_send`
- `client_receive`
- `policy_pipeline`

`policy_pipeline` views reuse the existing policy label layer. Bootstrap, outbound-send, and receive views expose their own stable integer codes and labels.

## Serialization

`PlatformDecisionView` supports the same optional `serde` feature as `DecisionView`.

```powershell
cargo test -p mercury-core --features serde platform_decision_view_serializes_binding_fields
```

The default core remains dependency-light; serialization is opt-in for bindings that need JSON or bridge payloads.

## UI Rule

Before any UI screen is built, platform code should treat `PlatformDecisionView` as the boundary:

- message UI opens only when bootstrap says `can_open_message_ui = true`
- send buttons enable only when outbound-send says `can_send = true`
- received messages render only when receive says `can_open_message_ui = true`
- sync, recovery, retry, and verification prompts come from the `requires_*` fields

This makes the UI a presenter of core decisions, not another security-policy implementation.

## Verification

The `platform_binding_view` integration test covers:

- bootstrap sync-incomplete projection
- outbound-send capability projection
- client-receive retry projection
- policy label reuse

The feature-gated serde test covers serialized binding fields.

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
cargo test -p mercury-core --features serde platform_decision_view_serializes_binding_fields
```

## Next Step

The UI integration handoff report is documented in `docs/30_UI_INTEGRATION_REPORT.md`.
