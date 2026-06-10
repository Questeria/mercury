# Room Membership Transitions

Generated: 2026-05-28

## Status

Mercury now has a typed room membership transition boundary in `mercury-core`.

```text
RoomMembershipTransitionInput
evaluate_room_membership_transition(RoomMembershipTransitionInput) -> RoomMembershipTransitionDecision
```

This boundary does not mutate MLS state or implement group cryptography. It defines the policy decision that future room-management code and UI screens must respect before applying a membership change.

## Transition Kinds

The first transition kinds are:

- `AddDevice`
- `RemoveDevice`
- `MarkDeviceCompromised`

Every accepted transition rotates the room epoch. This keeps membership changes tied to the same epoch model already used by room message policy.

## Inputs

Transition evaluation uses:

- room mode
- current epoch
- proposed epoch
- target actor kind
- target device kind
- target device state
- target device trust decision
- optional AI policy decision

The target device trust decision comes from `evaluate_device_trust(...)`. AI grant decisions come from the existing policy pipeline.

## Security Rules

The evaluator rejects:

- invalid epochs
- proposed epochs that do not advance
- actor/device shape mismatches
- add-device transitions where the target state is not active
- remove-device transitions where the target state is not removed
- compromised-device transitions where the target state is not compromised
- standard/sensitive human device additions without sendable trust
- high-security human device additions without full trust
- AI device additions in AI-blocked rooms
- AI device additions without full trust
- AI device additions without an accepted AI grant decision
- human device additions carrying an AI grant decision

## Verification

The `room_membership_transition` integration test covers:

- standard human device addition with trust-on-first-use
- high-security human additions requiring full trust
- AI-blocked room rejection
- AI grant required/rejected/accepted cases
- mandatory epoch advancement
- removed-device target state checks
- compromised-device target state checks
- actor/device mismatch rejection

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test --workspace
```

## Next Step

The outbound send gate is documented in `docs/23_OUTBOUND_SEND_GATE.md`.
