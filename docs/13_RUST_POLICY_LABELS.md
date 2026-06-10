# Rust Policy Labels

Generated: 2026-05-27

## Status

Mercury client-core decisions now expose stable labels and source attribution without changing the underlying numeric policy contract.

The Rust API keeps the wire-friendly fields:

```text
accepted: bool
reason_code: i32
audit_class: i32
components: ComponentReasons
```

It now also exposes no-allocation label helpers:

```text
PolicyDecision::pipeline_reason() -> PipelineReason
PolicyDecision::pipeline_audit_class() -> PipelineAuditClass
PolicyDecision::reason_label() -> CodeLabel
PolicyDecision::audit_class_label() -> CodeLabel
PolicyDecision::primary_source() -> PolicySource
PolicyDecision::primary_component() -> Option<CodeLabel>
PolicyDecision::component_labels() -> ComponentCodeLabels
PolicyDecision::labels() -> DecisionLabels
PolicyDecision::view() -> DecisionView
```

## Security Purpose

Policy labels prevent clients, logs, and audit displays from inventing their own meaning for integer codes. Each label is bound to a policy namespace:

- `policy_pipeline`
- `envelope`
- `room_epoch`
- `ai_grant`
- `ai_grant_lifecycle`

This keeps the compact numeric contract suitable for mobile and cross-language bindings while giving user interfaces and audit logs stable names such as `ENVELOPE_REJECT`, `PAYLOAD_TOO_LARGE`, or `AI_HIGHSEC_ROTATION_REQUIRED`.

## Source Attribution

`PolicyDecision::primary_source()` identifies the policy namespace responsible for the final decision:

- pipeline contract and composition failures stay attributed to `policy_pipeline`
- message envelope failures attribute to `envelope`
- room epoch and membership failures attribute to `room_epoch`
- AI grant failures attribute to `ai_grant`
- AI lifecycle, post-revoke, and high-security rotation failures attribute to `ai_grant_lifecycle`

`PolicyDecision::primary_component()` gives the most relevant component reason when a final pipeline rejection was caused by a component policy.

## Verification

The core policy vector test now asserts stable label behavior for all nine client-core vectors in addition to numeric decision codes.

Run:

```powershell
cargo check --workspace
cargo test --workspace
```

On the current Windows machine, test linking is still blocked by the missing MSVC `link.exe`; GitHub Actions remains the intended workspace test runner.

## Decision View Follow-Up

The compact serialized decision view is documented in `docs/14_RUST_DECISION_VIEW.md`.

## Next Step

The next client-core increment should define the first typed Mercury message and room-state inputs that feed `PolicyEvaluationInput`, still without networking or cryptography.
