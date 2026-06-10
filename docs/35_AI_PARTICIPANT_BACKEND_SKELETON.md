# AI Participant Backend Skeleton

Generated: 2026-05-28

## Status

Mercury now has a non-model AI participant backend skeleton in `mercury-core`:

```text
AiParticipantAction
AiParticipantRequest
AiParticipantDecision
PrototypeAiParticipantBackend
PrototypeAiActionAuditRecord
```

This skeleton does not call an AI model. It decides whether a proposed AI action is allowed by visible participant state, visible grant state, AI grant policy, AI lifecycle policy, action scopes, and hash-only audit metadata.

## Security Rules

The backend rejects:

- hidden AI participants
- hidden grants
- plaintext identity fields
- malformed hash-audit digest lengths
- rejected AI grant policy
- rejected AI lifecycle policy
- selected-context actions with no selected context
- reads, drafts, sends, or tools outside the grant scope
- autonomous sending
- prompt storage
- training on room context
- memory writes

Accepted decisions never set `can_store_prompt` or `can_train`. The prototype audit record stores action, accepted/rejected state, reason, and digest lengths only; it does not store prompt or transcript plaintext.

## Action Surface

Current actions:

- `ReadSelectedContext`
- `DraftReply`
- `SendMessageWithConfirmation`
- `AutonomousSend`
- `UseReadOnlyLocalTool`
- `UseRoomSearchSelectedTool`
- `UseOpenWorldExternalTool`
- `StorePrompt`
- `TrainOnContext`
- `WriteMemory`

## Intended Use

Use this backend skeleton to wire AI participant flows before model execution exists. The UI and platform layer can ask core whether an AI action is allowed, then render the decision or require confirmation without duplicating grant logic.

## Verification

The `prototype_ai_participant` integration test covers:

- accepted visible local draft actions
- visible participant and visible grant enforcement
- plaintext bridge and bad audit digest rejection
- grant policy and lifecycle rejection
- selected context, read/write, tool, autonomous send, retention, training, and memory boundaries

Run locally from a Visual Studio Build Tools developer environment on Windows:

```powershell
cargo test -p mercury-core prototype_ai
cargo test --workspace
```

## Next Step

The production AI connector gate is documented in `docs/73_PRODUCTION_AI_CONNECTOR_GATE.md`. Future model execution adapters should pass that gate after the participant decision accepts and before any selected context is sent to a model runtime.
