# UI Integration Report

Generated: 2026-05-28

## Purpose

This report is for the next AI agent integrating Mercury's first mobile and desktop UI. It summarizes what the UI should consume, what screens/states it needs, and what must remain delegated to `mercury-core`.

This is not a visual design brief. It intentionally avoids themes, creative direction, styling, layout aesthetics, animation, brand tone, and color guidance.

## Current Integration Boundary

The UI should integrate against `PlatformDecisionView` from `core/rust/mercury-core`.

```text
PlatformDecisionView
PlatformDecisionView::from_bootstrap(...)
PlatformDecisionView::from_outbound_send(...)
PlatformDecisionView::from_client_receive(...)
PlatformDecisionView::from_policy(...)
```

Treat this view as the UI security boundary. UI code should present decisions and enable/disable actions from capability fields. It should not reimplement trust, policy, relay, replay, store, AI grant, or sync logic.

Every platform decision view carries:

- `source`
- `accepted`
- `reason_code`
- `reason_label`
- `can_open_message_ui`
- `can_start_sync`
- `can_send`
- `can_receive`
- `can_persist_ciphertext`
- `requires_sync`
- `requires_recovery`
- `requires_client_retry`
- `requires_user_action`

Current sources:

- `client_bootstrap`
- `outbound_send`
- `client_receive`
- `policy_pipeline`

## Required First UI Surfaces

### Bootstrap And Lock Surface

Purpose: prevent message UI and local decrypt until core startup accepts.

Inputs:

- `PlatformDecisionView::from_bootstrap(...)`

Required behavior:

- Open message UI only when `can_open_message_ui = true`.
- Allow background sync only when `can_start_sync = true`.
- Keep local decrypt closed when `can_open_message_ui = false`.
- Show recovery path when `requires_recovery = true`.
- Show sync path when `requires_sync = true`.
- Show user action path when `requires_user_action = true`.

Do not:

- Show message previews before bootstrap accepts.
- Decrypt local message timelines before bootstrap accepts.
- Treat `can_start_sync = true` as permission to open message UI.

### Conversation List Shell

Purpose: list rooms only after bootstrap readiness permits the message surface.

Inputs:

- accepted bootstrap view
- future room list binding data
- future room membership/trust summaries

Required behavior:

- Render only after bootstrap `can_open_message_ui = true`.
- Surface room-level sync/recovery states when provided by the binding.
- Do not show plaintext previews until receive policy has accepted each message.
- Do not infer trust state from names, avatars, local cache, or server metadata.

### Message Thread Shell

Purpose: display messages only after receive decisions accept.

Inputs:

- `PlatformDecisionView::from_client_receive(...)`
- future decrypted message payload supplied only after core/binding authorization

Required behavior:

- Render a received message only when `can_receive = true` and `can_open_message_ui = true`.
- Trigger retry/sync behavior when `requires_client_retry = true`.
- Keep duplicate, stale, malformed, or out-of-order messages hidden from plaintext display.
- Show verification or trust prompt when `requires_user_action = true`.

Do not:

- Display a ciphertext, placeholder plaintext, or notification preview as if accepted.
- Resolve replay/order gaps in UI logic.
- Re-run policy from raw message fields.

### Composer And Send Controls

Purpose: allow sending only when the outbound gate accepts.

Inputs:

- `PlatformDecisionView::from_outbound_send(...)`
- future send draft state

Required behavior:

- Enable final send only when `can_send = true`.
- Persist outgoing ciphertext only when `can_persist_ciphertext = true`.
- Require user confirmation or verification when `requires_user_action = true`.
- Surface `reason_label` for rejected sends in diagnostic or developer UI paths.

Do not:

- Enable send based only on text-box contents or network availability.
- Persist ciphertext when outbound-send rejects.
- Hide trust-on-first-use user action behind an automatic send.

### Trust And Verification Surface

Purpose: make device/key transparency/user-action states explicit.

Inputs:

- device trust decisions
- key transparency decisions
- platform views carrying `requires_user_action`
- future safety-number/QR binding data

Required behavior:

- Provide a user path for manual verification.
- Make device changes and AI device presence visible.
- Treat `requires_user_action = true` as a blocking or prominent state, depending on the underlying action.
- Distinguish fully trusted, sendable-but-not-fully-trusted, rejected, stale, missing proof, and inconsistent states when the binding exposes them.

Do not:

- Mark a device as fully trusted from UI-only actions.
- Suppress key-change or key-transparency warnings.
- Treat AI devices as hidden infrastructure.

### AI Participant And Grant Surface

Purpose: show AI access as explicit, scoped, and revocable.

Inputs:

- future AI grant binding data
- AI policy decisions
- AI lifecycle decisions
- room membership state

Required behavior:

- Show AI participants as visible room participants or explicitly scoped context recipients.
- Show grant scope, duration, local/remote mode, tool access, and send permissions when available.
- Require grant acceptance before AI context is made available.
- Show revoke/expiry states.
- Do not let remote AI access sensitive/high-security rooms unless core policy later allows it.

Do not:

- Treat AI as a hidden server-side feature.
- Send room plaintext to an AI provider without an explicit accepted grant path.
- Store AI prompt or transcript plaintext in durable local storage.

### Recovery And Sync Surface

Purpose: repair incomplete state without exposing message UI prematurely.

Inputs:

- bootstrap platform view
- future sync progress binding data
- future recovery status binding data

Required behavior:

- Start sync only when `can_start_sync = true`.
- Keep message UI closed until bootstrap accepts.
- Show recovery path when account/device secrets are missing or corrupt.
- Show sync gap/remediation path for missing room state, replay checkpoint gaps, or sync failures.

Do not:

- Treat partial sync as a readable timeline.
- Let the UI choose between accepting stale replay state and fetching missing state.
- Present recovery as optional when `requires_recovery = true`.

## Initial State Mapping

Use these mappings as the first UI behavior contract:

```text
source = client_bootstrap
accepted = true
can_open_message_ui = true
```

Result: app shell may enter conversation list and thread surfaces.

```text
source = client_bootstrap
accepted = false
can_start_sync = true
requires_sync = true
```

Result: show sync/remediation surface; keep message UI and local decrypt closed.

```text
source = client_bootstrap
accepted = false
requires_recovery = true
```

Result: show recovery surface; keep message UI and local decrypt closed.

```text
source = outbound_send
accepted = true
can_send = true
can_persist_ciphertext = true
```

Result: final send action may proceed.

```text
source = outbound_send
accepted = false
can_send = false
```

Result: disable final send and present the reason path.

```text
source = client_receive
accepted = true
can_receive = true
can_open_message_ui = true
```

Result: received message may be handed to the rendering layer.

```text
source = client_receive
accepted = false
requires_client_retry = true
```

Result: do not render plaintext; run retry/sync flow.

## Sample Payloads

Bootstrap accepted:

```json
{
  "source": "client_bootstrap",
  "accepted": true,
  "reason_code": 0,
  "reason_label": "ACCEPTED",
  "can_open_message_ui": true,
  "can_start_sync": true,
  "can_send": false,
  "can_receive": false,
  "can_persist_ciphertext": false,
  "requires_sync": false,
  "requires_recovery": false,
  "requires_client_retry": false,
  "requires_user_action": false
}
```

Bootstrap sync incomplete:

```json
{
  "source": "client_bootstrap",
  "accepted": false,
  "reason_code": 18,
  "reason_label": "SYNC_INCOMPLETE",
  "can_open_message_ui": false,
  "can_start_sync": true,
  "can_send": false,
  "can_receive": false,
  "can_persist_ciphertext": false,
  "requires_sync": true,
  "requires_recovery": false,
  "requires_client_retry": false,
  "requires_user_action": false
}
```

Receive ordering gap:

```json
{
  "source": "client_receive",
  "accepted": false,
  "reason_code": 10,
  "reason_label": "ORDERING_GAP",
  "can_open_message_ui": false,
  "can_start_sync": false,
  "can_send": false,
  "can_receive": false,
  "can_persist_ciphertext": false,
  "requires_sync": false,
  "requires_recovery": false,
  "requires_client_retry": true,
  "requires_user_action": false
}
```

Outbound send accepted:

```json
{
  "source": "outbound_send",
  "accepted": true,
  "reason_code": 0,
  "reason_label": "ACCEPTED",
  "can_open_message_ui": false,
  "can_start_sync": false,
  "can_send": true,
  "can_receive": false,
  "can_persist_ciphertext": true,
  "requires_sync": false,
  "requires_recovery": false,
  "requires_client_retry": false,
  "requires_user_action": false
}
```

## Integration Rules

- UI code must not call lower-level policy functions directly when a platform view is available.
- UI code must not map raw numeric reason codes itself; use `reason_label`.
- UI code must not convert rejected states into accepted states for convenience.
- UI code must not store plaintext message, media, prompt, or AI transcript data in durable local storage.
- UI code must not show message notification previews unless receive and bootstrap gates allow the message surface.
- UI code must preserve AI participant visibility and grant boundaries.
- UI code must treat high-security mode as stricter than standard mode when future binding data exposes that state.

## Expected Binding Functions

The next implementation agent should expose small platform functions with names similar to:

```text
mercury_bootstrap_status(...) -> PlatformDecisionView
mercury_prepare_send(...) -> PlatformDecisionView
mercury_accept_received_ciphertext(...) -> PlatformDecisionView
mercury_policy_status(...) -> PlatformDecisionView
```

Exact FFI shape is not fixed yet. Keep the first binding narrow and generated from Rust core decisions.

## Minimum UI Test Scenarios

The first UI integration should include tests for:

- bootstrap accepted opens the message shell
- bootstrap sync incomplete keeps message shell closed
- bootstrap recovery required keeps message shell closed
- outbound send accepted enables final send
- outbound send rejected disables final send
- receive accepted renders message
- receive ordering gap triggers retry/sync and renders no plaintext
- receive rejected by sender trust shows user-action path
- AI grant absent keeps AI controls inactive
- AI grant revoked or expired removes AI access

## Files To Read First

- `docs/29_PLATFORM_BINDING_CONTRACT.md`
- `docs/28_CLIENT_BOOTSTRAP_SYNC.md`
- `docs/27_CLIENT_RECEIVE_GATE.md`
- `docs/23_OUTBOUND_SEND_GATE.md`
- `docs/20_IDENTITY_DEVICE_TRUST.md`
- `docs/02_AI_PARTICIPANT_MODEL.md`
- `core/rust/mercury-core/tests/platform_binding_view.rs`
- `core/rust/mercury-core/tests/client_bootstrap_sync.rs`
- `core/rust/mercury-core/tests/client_receive_gate.rs`
- `core/rust/mercury-core/tests/outbound_send_gate.rs`

## Open Engineering Items For UI Integration

- Choose the actual UI framework and platform shell.
- Add the first platform binding crate or bridge layer.
- Add fixture payloads for `PlatformDecisionView`.
- Decide how local development will simulate bootstrap, send, receive, sync, recovery, trust, and AI grant states.
- Keep all prototype simulation state behind the same platform view shape so it can be replaced by real bindings later.
