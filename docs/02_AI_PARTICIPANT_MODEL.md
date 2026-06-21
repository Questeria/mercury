# Mercury AI Participant Model

## Position

AI in Mercury is a first-class participant, not a server-side feature. An AI can have an account, devices, keys, visible membership, scoped grants, and auditable actions. It never receives plaintext from an encrypted room unless a human participant explicitly grants that access.

The core rule:

> AI is either a visible encrypted device in a room or the recipient of a deliberately scoped encrypted context envelope.

## Operating Modes

### Local AI

Default private mode for sensitive chats.

- Runs on the user's device.
- Receives selected local plaintext only after local policy checks.
- May draft, summarize, search selected context, or help compose messages.
- Can be blocked from sending without user confirmation.
- No remote AI provider receives plaintext.

### Remote Enclave AI

Opt-in for heavier tasks.

- Client encrypts selected context to a verified AI runtime.
- Remote runtime should provide attestation, transparency, stateless processing, and retention limits.
- The client logs what was shared and why.
- Users can revoke grants and rotate group epochs where appropriate.

### Remote Provider AI

Explicit context-sharing mode only.

- The UI must say selected context is leaving the E2EE trust boundary.
- Never default for sensitive chats.
- Strongly prefer redaction, narrow context windows, and no retention/training terms where possible.

## Core Rules

- Explicit invitations only.
- No silent global AI access.
- Grants name readable data, writable actions, tool access, retention, expiration, and approvers.
- AI output is attributed with account, device, model/mode, grant id, and tool-use status.
- Prompt injection is assumed possible, so enforcement must happen outside the model.
- One-tap revoke invalidates grants and removes AI membership/context access.
- Sensitive rooms can block AI participants and AI context sharing entirely.

## Identity Objects

Example AI account:

```json
{
  "type": "mercury.ai.account.v1",
  "account_id": "ai:summarizer@mercury.example",
  "kind": "ai",
  "operator": "Mercury",
  "display_name": "Mercury AI",
  "allowed_modes": ["local", "remote_enclave"],
  "public_profile": {
    "capabilities": ["summarize", "draft_reply", "search_selected_context"],
    "default_retention": "none"
  }
}
```

Example AI device:

```json
{
  "type": "mercury.ai.device.v1",
  "ai_account_id": "ai:summarizer@mercury.example",
  "device_id": "aid:local:alice-phone:7f31",
  "mode": "local",
  "identity_key": "base64...",
  "capability_key": "base64...",
  "key_transparency_leaf": "sha256...",
  "created_at": "2026-05-27T15:00:00Z"
}
```

AI devices should appear in key transparency like human devices so clients can detect surprise key substitution.

## Invitation And Grant Flow

An AI invitation declares requested access:

```json
{
  "type": "mercury.ai.invite.v1",
  "room_id": "room:8db3",
  "ai_account_id": "ai:summarizer@mercury.example",
  "requested_by": "user:alice",
  "mode": "remote_enclave",
  "requested_scopes": [
    {"scope": "room.read", "range": "last_50_messages", "ttl_s": 900},
    {"scope": "room.write", "actions": ["reply", "summary"]},
    {"scope": "memory.write", "level": "none"}
  ],
  "tool_scopes": [
    {"tool": "calendar.lookup", "permission": "ask_each_time"}
  ],
  "consent_policy": "all_human_members",
  "expires_at": "2026-05-27T15:15:00Z"
}
```

A signed grant is a room event:

```json
{
  "type": "mercury.ai.grant.v1",
  "grant_id": "grant:01J...",
  "room_id": "room:8db3",
  "principal": "ai:summarizer@mercury.example",
  "device_ids": ["aid:remote-enclave:42"],
  "scopes": [
    {"scope": "room.read", "messages": ["msg:1000..msg:1050"]},
    {"scope": "room.write", "max_messages": 3}
  ],
  "denied": ["room.history_all", "attachments.read_all", "contacts.read"],
  "retention": {"prompt_store": false, "training": false, "memory": "none"},
  "expires_at": "2026-05-27T15:15:00Z",
  "approved_by": ["user:alice", "user:bob"],
  "signatures": {"user:alice": "base64...", "user:bob": "base64..."}
}
```

## Encrypted Context Bridge

For remote AI, the client builds a context package and encrypts it to the AI device or verified enclave using HPKE or an equivalent reviewed envelope-encryption mechanism.

```json
{
  "type": "mercury.ai.context_bridge.v1",
  "grant_id": "grant:01J...",
  "audience": "aid:remote-enclave:42",
  "context_policy": {
    "messages": ["msg:1000..msg:1050"],
    "attachments": "none",
    "redactions": ["phone_numbers", "hidden_metadata"]
  },
  "provenance_hash": "sha256...",
  "encryption": {
    "scheme": "HPKE-v1",
    "recipient_key_id": "kt:leaf:abc",
    "aad": "room:8db3|grant:01J...|expires:..."
  },
  "ciphertext": "base64..."
}
```

The bridge should include a provenance hash so clients can later prove what selected message set was given to the AI without logging plaintext by default.

## Tool Permissions

Tool access is granted by policy, not by model request.

```json
{
  "type": "mercury.ai.tool_policy.v1",
  "grant_id": "grant:01J...",
  "tools": [
    {
      "name": "chat.search_selected",
      "readOnlyHint": true,
      "openWorldHint": false,
      "approval": "auto"
    },
    {
      "name": "message.send",
      "readOnlyHint": false,
      "destructiveHint": false,
      "approval": "confirm_before_execute"
    },
    {
      "name": "file.export",
      "readOnlyHint": false,
      "openWorldHint": true,
      "approval": "blocked"
    }
  ]
}
```

High-risk chains are denied even if individual tools look harmless. Example: read private context, summarize it, and post to an external URL.

## Audit Events

Audit logs are visible to room members and exportable in managed deployments. Consumer mode should log hashes and metadata by default, not full plaintext prompts.

```json
{
  "type": "mercury.ai.audit_event.v1",
  "event_id": "evt:01J...",
  "room_id": "room:8db3",
  "ai_account_id": "ai:summarizer@mercury.example",
  "grant_id": "grant:01J...",
  "action": "tool_call",
  "tool": "message.send",
  "approval": {"required": true, "approved_by": "user:alice"},
  "input_hash": "sha256...",
  "output_hash": "sha256...",
  "created_at": "2026-05-27T15:03:10Z"
}
```

## Prompt-Injection Controls

All messages, files, webpages, quoted text, and tool responses are untrusted data.

Controls:

- Tool calls require grant checks independent of model output.
- External tool responses are labeled untrusted before entering context.
- Hidden instructions inside files or messages cannot change grants.
- Memory writes require explicit user approval.
- Cross-chat retrieval is off unless every source chat grants it.
- Sending messages in high-security chats requires confirmation unless the room explicitly allows autonomous AI sending.
- AI cannot request broader access by writing text into the chat; it must use a real grant request flow.

## UX Requirements

- Member list shows `AI`, mode, active scopes, and expiration.
- Invite sheet uses concrete language such as `Can read last 50 messages for 15 minutes`.
- In-chat state shows when AI is reading selected context.
- Per-response disclosure shows data and tools used.
- One-tap revoke is always available.
- Sensitive rooms can block all AI access.
- Draft-only mode is easy to select.
- `Show why AI had access` opens the audit trail.

## First AI Milestone

The first Mercury AI feature should be local draft/summarize for selected messages only:

- No automatic send.
- No cross-chat memory.
- No remote provider.
- Full audit metadata.
- Helix policy validator for grant shape and scope checks.

Initial executable policy exists in `docs/08_AI_GRANT_POLICY.md`.

## Implementation Status (honest residual)

The model above is the design target. The executable core today (`mercury-ai` + `mercury-core`'s AI
gates) implements the **commitment** and the **kind / count / local / draft-only gates**: it computes
the auditable context + draft SHA-256 digests, counts the selected messages, and the policy gate
enforces the action kind, read-scope category, local runtime, and draft-only invariants — refusing on
any honest attestation failure (prompt-injection enforcement lives outside the model, as above). What
is **not yet wired**:

- **Conversation / message provenance.** The grant validates scope *categories and counts*, and the
  digest commits the selected *bytes*, but neither yet binds those bytes to a specific conversation or
  to the named message range in the grant above. So "the AI saw only the granted messages" currently
  rests on the client selecting them, **not** on gate verification. When the `@mention → AI` egress
  path is wired, bind a conversation / room-epoch id into the grant **and** the context digest, and
  have the gate require every selected message's *authenticated* conversation to equal the grant's —
  turning the after-the-fact commitment into a preventive guarantee.
- **No live egress path.** `mercury-ai` has no production caller yet; the gates are exercised by tests
  and the UI simulation fixtures. The gates are the enforced core of the model; the rest is roadmap.
