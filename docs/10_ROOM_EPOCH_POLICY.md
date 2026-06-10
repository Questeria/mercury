# Room Epoch Policy

Generated: 2026-05-27

## Status

Mercury now has an executable room epoch and device membership policy:

- `policy/room_epoch_policy_v1.json`
- `helix/policy/room_epoch.hx`
- `helix/tests/room_epoch_test.hx`
- `vectors/room_epoch/*.json`
- `tools/check_room_epoch_vectors.py`

This policy is the first bridge from AI grant lifecycle state toward encrypted room membership. It does not perform cryptography, derive keys, parse MLS messages, or mutate membership. It validates scalar facts that a future crypto layer must provide.

## Phase 1 Rules

- Only room epoch policy version `1` is accepted.
- Room modes must be standard, sensitive, high-security, or AI-blocked.
- Message epoch must equal the current room epoch.
- Messages below the minimum accepted epoch are stale.
- Active devices must not carry a revoked-at epoch.
- Removed and compromised devices must carry a valid revoked-at epoch.
- Removed and compromised devices are rejected.
- In high-security rooms, removed or compromised devices require a later room epoch before ordinary membership rejection is reported.
- AI devices are rejected in AI-blocked rooms.

## Staged API

The Helix API keeps each validator scalar and small:

```text
mercury_room_epoch_validate_epoch_v1(version, room_mode, current_epoch, message_epoch, min_accepted_epoch)
mercury_room_epoch_validate_device_v1(device_kind, device_state, revoked_at_epoch, current_epoch, access_kind)
mercury_room_epoch_validate_highsec_v1(room_mode, device_kind, device_state, revoked_at_epoch, current_epoch, access_kind)
mercury_room_epoch_first_reject(epoch_reason, device_reason, highsec_reason)
mercury_room_epoch_audit_class_for_reason(reason_code)
```

The high-security epoch-rotation marker is derived from room state: `current_epoch > revoked_at_epoch` means the room has rotated past the revoked device.

## Verification

Run:

```powershell
python .\tools\check_room_epoch_vectors.py
python .\tools\check_policy_contract.py
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
```

The room epoch vector contract is mirrored across Helix, Rust, Python, and JSON manifests. GitHub Actions runs the Rust and Python sides; Helix remains local until Mercury can pin or install the compiler in CI.

## Pipeline Follow-Up

The follow-up deterministic policy pipeline now lives in:

- `docs/11_POLICY_PIPELINE.md`
- `policy/policy_pipeline_v1.json`
- `helix/policy/policy_pipeline.hx`
