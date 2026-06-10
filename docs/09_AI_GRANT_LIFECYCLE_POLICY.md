# AI Grant Lifecycle Policy

Generated: 2026-05-27

## Status

Mercury now has a second executable AI-access policy slice:

- `policy/ai_grant_lifecycle_policy_v1.json`
- `helix/policy/ai_grant_lifecycle.hx`
- `helix/tests/ai_grant_lifecycle_test.hx`
- `vectors/ai_grant_lifecycle/*.json`
- `tools/check_ai_grant_lifecycle_vectors.py`

The policy validates whether an existing AI grant is still usable. It covers active, expired, and revoked grants, plus the high-security requirement that a revoked grant must be paired with an epoch-rotation marker before ordinary post-revoke access reasons are reported.

## Phase 1 Rules

- Only lifecycle version `1` is accepted.
- Grant state must be `active` or `revoked`.
- Active grants must not carry a revoke reason.
- Revoked grants must carry an explicit revoke reason.
- Expiry is derived from `now_s >= expires_at_s`.
- Revoked grants are rejected.
- Read and write attempts after revocation are rejected with specific reasons.
- High-security revoked grants report `HIGHSEC_EPOCH_ROTATION_REQUIRED` until `epoch_rotated == 1`.
- Once the high-security epoch rotation marker is present, the revoked grant still cannot read or write.

## Staged API

The Helix API keeps functions small and scalar:

```text
mercury_lifecycle_validate_state_v1(version, grant_state, revoke_reason, now_s, expires_at_s, room_mode)
mercury_lifecycle_validate_access_v1(lifecycle_reason, access_kind, room_mode, epoch_rotated)
mercury_lifecycle_first_reject(state_reason, access_reason)
mercury_lifecycle_audit_class_for_reason(reason_code)
```

`access_kind = 0` is a state-only check. It lets test vectors distinguish a revoked grant state from a read or write attempt after revocation.

## Verification

Run:

```powershell
python .\tools\check_ai_grant_lifecycle_vectors.py
python .\tools\check_policy_contract.py
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
```

The lifecycle vector contract is mirrored across Helix, Rust, Python, and JSON manifests. GitHub Actions runs the Rust and Python sides; Helix remains local until Mercury can pin or install the compiler in CI.

## Room Epoch Follow-Up

The follow-up room epoch and device membership policy now lives in:

- `docs/10_ROOM_EPOCH_POLICY.md`
- `policy/room_epoch_policy_v1.json`
- `helix/policy/room_epoch.hx`
