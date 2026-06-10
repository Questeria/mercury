# Policy Pipeline

Generated: 2026-05-27

## Status

Mercury now has a deterministic policy pipeline:

- `policy/policy_pipeline_v1.json`
- `helix/policy/policy_pipeline.hx`
- `helix/tests/policy_pipeline_test.hx`
- `vectors/policy_pipeline/*.json`
- `tools/check_policy_pipeline_vectors.py`

The pipeline composes existing policy decisions into one final reason code. It does not inspect raw messages, run cryptography, call AI, or perform server work. It accepts scalar reason codes from:

- envelope validation
- room epoch and device membership validation
- AI grant validation
- AI grant lifecycle validation

## Decision Order

The pipeline rejects in a stable order:

1. malformed pipeline input or out-of-range component reason codes
2. envelope rejection
3. room epoch or device membership rejection
4. AI component attached to a human actor
5. AI grant rejection
6. AI lifecycle rejection

AI lifecycle post-revoke access and high-security rotation requirements are preserved as distinct pipeline reasons instead of being collapsed into a generic AI lifecycle rejection.

## Staged API

```text
mercury_pipeline_validate_inputs_v1(version, actor_kind, envelope_reason, room_epoch_reason, ai_grant_reason, ai_lifecycle_reason)
mercury_pipeline_validate_actor_components(actor_kind, ai_grant_reason, ai_lifecycle_reason)
mercury_pipeline_component_first_reject(envelope_reason, room_epoch_reason, actor_component_reason, ai_grant_reason, ai_lifecycle_reason)
mercury_pipeline_decide_v1(version, actor_kind, envelope_reason, room_epoch_reason, ai_grant_reason, ai_lifecycle_reason)
mercury_pipeline_audit_class_for_reason(reason_code)
```

## Verification

Run:

```powershell
python .\tools\check_policy_pipeline_vectors.py
python .\tools\check_policy_contract.py
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
```

The pipeline contract is mirrored across Helix, Rust, Python, and JSON manifests. GitHub Actions runs the Rust and Python sides; Helix remains local until Mercury can pin or install the compiler in CI.

## Rust Core Follow-Up

The follow-up Rust client-core policy layer now lives in:

- `docs/12_RUST_CLIENT_CORE_POLICY.md`
- `core/rust/mercury-core`
- `vectors/core_policy`
