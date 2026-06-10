# Client State Builder

Generated: 2026-05-27

## Status

Mercury now has a checked local-client construction layer around `ClientMessagePolicyInput`:

```text
ClientMessagePolicyInput::try_from_local_state(
    ClientRoomState,
    ClientSenderState,
    OutboundMessageDraft,
) -> Result<ClientMessagePolicyInput, ClientStateError>
```

This does not replace the lower-level raw fact structs or policy evaluator. It gives normal client code a safer path for assembling policy inputs before the Helix-backed policy pipeline runs.

## Builder Types

The new layer groups state into:

- `ClientRoomState`
- `ClientSenderState`
- `OutboundMessageDraft`
- `ClientStateError`

Convenience constructors cover the common paths:

- `ClientRoomState::new(...)`
- `ClientSenderState::human(...)`
- `ClientSenderState::local_ai(...)`
- `ClientSenderState::remote_ai(...)`
- `OutboundMessageDraft::new(...)`

## Pre-Policy Guardrails

The checked builder rejects inconsistent local state before policy evaluation:

- human actor paired with an AI device
- AI actor paired with a human device
- human actor carrying AI policy facts
- AI actor missing AI policy facts
- active device with a nonzero revoked epoch
- removed or compromised device with no revoked epoch
- local/remote AI actor kind mismatched with AI mode
- AI grant room mode mismatched with the room snapshot
- AI lifecycle room mode mismatched with the room snapshot
- AI lifecycle access kind mismatched with the draft access kind

These are local consistency checks, not cryptographic verification and not a substitute for policy evaluation. A correctly assembled input can still be rejected by the policy pipeline.

## Verification

The `client_state_builder` integration test covers valid human construction plus the main shape and AI consistency errors.

Run in CI:

```powershell
cargo test --workspace
```

Local Windows note: if `link.exe` is not on the normal PowerShell PATH, run the test command from the Visual Studio Build Tools developer environment.

## Local Store Follow-Up

The first local encrypted-store boundary is documented in `docs/17_LOCAL_STORE_BOUNDARY.md`.

## Next Step

The next increment should add a concrete encrypted-store trait or adapter boundary for mobile and desktop clients, using the local-store classifier before records reach platform storage.
