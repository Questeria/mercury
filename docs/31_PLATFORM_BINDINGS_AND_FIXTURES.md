# Platform Bindings And Fixtures

Generated: 2026-05-28

## Status

Mercury now has a small non-visual platform binding crate:

```text
core/rust/mercury-bindings
```

The crate exposes wrapper functions around the core platform decision view:

```text
mercury_bootstrap_status(...) -> PlatformDecisionView
mercury_prepare_send(...) -> PlatformDecisionView
mercury_accept_received_ciphertext(...) -> PlatformDecisionView
mercury_policy_status(...) -> PlatformDecisionView
```

It also exposes checked UI fixture scenarios:

```text
platform_fixture_view(...)
platform_fixture_json(...)
platform_fixture_by_name(...)
PLATFORM_FIXTURES
```

## Fixture Payloads

The checked-in payloads live in:

```text
fixtures/platform/*.json
```

Current fixtures:

- `bootstrap_accepted`
- `bootstrap_sync_incomplete`
- `bootstrap_recovery_required`
- `outbound_send_accepted`
- `outbound_send_message_policy_rejected`
- `client_receive_accepted`
- `client_receive_ordering_gap`
- `client_receive_sender_trust_action`
- `policy_ai_grant_rejected`
- `policy_ai_lifecycle_expired`

The fixture test serializes the Rust binding view and compares it with the JSON files. If a core decision shape changes, fixture drift is caught by Rust tests.

## UI Use

The UI can use these fixtures immediately while real platform bridge work is still being wired. Treat them as contract-shaped states, not final app data.

Recommended first UI states:

- app unlock/startup allowed
- sync required before message UI
- recovery required before message UI
- send allowed
- send blocked by message policy
- receive allowed
- receive blocked by ordering gap
- receive allowed but verification/user action visible
- AI grant rejected
- AI lifecycle rejected or expired

## Verification

Run from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test -p mercury-bindings
cargo test --workspace
```

## Next Step

The UI simulation harness is documented in `docs/32_UI_SIMULATION_HARNESS.md`. The next parallel increment should add the first local encrypted-store prototype behind the existing storage adapter contracts.
