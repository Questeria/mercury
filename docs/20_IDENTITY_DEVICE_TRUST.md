# Identity Device Trust

Generated: 2026-05-28

## Status

Mercury now has a typed identity and device trust boundary in `mercury-core`.

```text
DeviceTrustInput
evaluate_device_trust(DeviceTrustInput) -> DeviceTrustDecision
```

This boundary gives the future UI a decision to display, not raw ingredients to reinterpret. The frontend should not decide whether a key change is safe, whether key transparency is acceptable, or whether an AI device can be trusted in high-security mode.

## Inputs

Trust evaluation uses:

- `DeviceTrustPolicyMode`
- `ActorKind`
- `DeviceKind`
- `DeviceState`
- `ManualVerificationState`
- `KeyTransparencyState`
- `DeviceKeyChangeState`

The policy modes are:

- `Opportunistic`
- `Strict`
- `HighSecurity`

The split is deliberate. Opportunistic mode can allow trust-on-first-use sends when key transparency is consistent, but it does not mark the device as fully trusted. Strict and high-security modes require both consistent key transparency and manual verification.

## Decision Shape

`DeviceTrustDecision` returns:

- `trusted`
- `can_send`
- `requires_user_action`
- `reason`

This keeps UI states honest. A device can be sendable but not fully trusted in opportunistic mode, which should become a visible verification prompt rather than a silent green check.

## Security Rules

The evaluator rejects:

- human actors paired with AI devices
- AI actors paired with human devices
- removed devices
- compromised devices
- key transparency inconsistency
- key changes that have not been manually verified
- strict/high-security sends without key transparency consistency
- strict/high-security sends without manual verification

High-security AI devices use the same trust boundary: active AI device, matching actor/device shape, consistent key transparency, and manual verification.

## Verification

The `device_trust` integration test covers:

- strict verified human devices
- strict unverified rejection
- opportunistic trust-on-first-use
- key transparency inconsistency blocking every mode
- key-change verification requirements
- actor/device mismatch rejection
- compromised device rejection
- high-security verified AI device acceptance

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Key Transparency Follow-Up

The first key transparency proof boundary is documented in `docs/21_KEY_TRANSPARENCY_PROOF_BOUNDARY.md`.

## Next Step

The next increment should define the room membership transition boundary: adding/removing human and AI devices, epoch rotation requirements, and the decision shape that later UI screens will display.
