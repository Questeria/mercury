# UI Simulation Harness

Generated: 2026-05-28

## Status

Mercury now has a small command-line simulator for UI integration:

```text
cargo run -p mercury-bindings --bin mercury-ui-sim -- --list
cargo run -p mercury-bindings --bin mercury-ui-sim -- --scenario <name>
cargo run -p mercury-bindings --bin mercury-ui-sim -- --sequence <name>
cargo run -p mercury-bindings --bin mercury-ui-sim -- --all
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype <name>
cargo run -p mercury-bindings --bin mercury-ui-sim -- --all-prototypes
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command <name>
cargo run -p mercury-bindings --bin mercury-ui-sim -- --bridge <operation> <name>
cargo run -p mercury-bindings --bin mercury-ui-sim -- --bridge-json <json>
```

The simulator emits `PlatformDecisionView`, prototype fixture, backend command, and platform bridge JSON. It is for frontend and platform development while real networking, crypto, and storage layers are still being built.

## Scenarios

Single scenarios are the checked fixture names documented in `docs/31_PLATFORM_BINDINGS_AND_FIXTURES.md`.

Examples:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --scenario bootstrap_sync_incomplete
cargo run -p mercury-bindings --bin mercury-ui-sim -- --scenario client_receive_ordering_gap
```

## Sequences

Current sequences:

- `startup_ready`
- `sync_then_ready`
- `recovery_then_ready`
- `send_receive_happy`
- `receive_gap_retry`
- `ai_unavailable`

Sequence output is a JSON array:

```json
[
  {
    "scenario": "bootstrap_accepted",
    "view": {
      "source": "client_bootstrap"
    }
  }
]
```

The example above is abbreviated. Real output includes the full platform decision view.

## Prototype Fixtures

Prototype fixtures are checked backend-shaped JSON states for local store, local crypto, relay, and AI participant behavior:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --list-prototypes
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype ai_participant_draft_accepted
```

They are documented in `docs/38_PROTOTYPE_FIXTURE_COVERAGE.md`.

## Backend Commands And Bridge

Backend command and bridge output can be exercised with:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --list-commands
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command local_ai_draft_assist
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_production_store_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_platform_local_store_adapter_desktop_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_receive_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --bridge backend_command run_session_happy_path
```

The bridge contract is documented in `docs/46_PLATFORM_BRIDGE_CONTRACT.md`.

## UI Use

Use the simulator to build UI state handling before real services exist:

- app startup allowed
- sync blocks message UI then resolves
- recovery blocks message UI then resolves
- send is accepted or rejected
- receive is accepted or blocked by retryable ordering gap
- AI grant/lifecycle unavailable states

Do not add simulator-only branches to UI logic. The UI should consume the same `PlatformDecisionView` shape that future real bindings will return.

## Verification

Run from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test -p mercury-bindings
cargo test --workspace
```

The CLI tests cover listing, single-scenario output, prototype output, backend command output, bridge output, sequence output, and unknown-name rejection.

## Next Step

The platform bridge contract is documented in `docs/46_PLATFORM_BRIDGE_CONTRACT.md`.
