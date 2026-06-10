# AI Grant Policy

Generated: 2026-05-27

## Status

Mercury now has its first executable AI-access policy:

- `policy/ai_grant_policy_v1.json`
- `helix/policy/ai_grant.hx`
- `helix/tests/ai_grant_test.hx`
- `vectors/ai_grant/*.json`
- `tools/check_ai_grant_vectors.py`

The policy is deliberately scalar-only. It validates whether an AI principal may receive selected context or act in a room. It does not run AI, inspect messages, verify signatures, call tools, or perform encryption.

## Phase 1 Rules

- Only AI principals can receive AI grants.
- AI-blocked rooms reject all AI grants.
- Grants require a positive TTL and are capped at 900 seconds.
- At least one approver is required.
- Remote provider AI is rejected in sensitive and high-security rooms.
- Full-room-history reads are rejected.
- Autonomous sending is rejected.
- Open-world external tools are rejected.
- Prompt storage and training are rejected.
- High-security rooms require local AI, two approvers, and no tools wider than read-only local access.

## Staged API

The Helix API follows the existing envelope-policy pattern and keeps each function within the current compiler backend's small integer-parameter budget:

```text
mercury_ai_grant_validate_subject_v1(version, principal_kind, room_mode, ai_mode, ttl_s, approver_count)
mercury_ai_grant_validate_scopes_v1(read_scope, write_scope, tool_scope, retention_mode, training_allowed, prompt_store_allowed)
mercury_ai_grant_validate_highsec_v1(room_mode, principal_kind, ai_mode, write_scope, tool_scope, approver_count)
mercury_ai_grant_first_reject(subject_reason, scope_reason, highsec_reason)
mercury_ai_grant_audit_class_for_reason(reason_code)
```

## Verification

Run:

```powershell
python .\tools\check_ai_grant_vectors.py
python .\tools\check_policy_contract.py
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
```

Verified locally:

- AI grant vector checker: 20 vectors checked.
- Policy contract checker covers AI grant manifest drift across Helix, Rust, Python, and vectors.
- Helix AI grant policy parse/typecheck/totality: OK.
- Helix AI grant test ELF codegen: OK.
- WSL runtime execution: `ai_grant_test.bin` exits `42`.

## Lifecycle Follow-Up

The follow-up revocation/expiry state transition policy now lives in:

- `docs/09_AI_GRANT_LIFECYCLE_POLICY.md`
- `policy/ai_grant_lifecycle_policy_v1.json`
- `helix/policy/ai_grant_lifecycle.hx`
