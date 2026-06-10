//! On-device AI engine for Mercury (Stage 10).
//!
//! Mercury treats an AI assistant as a VISIBLE PRINCIPAL that must never become a
//! hidden plaintext backdoor. This engine computes the auditable SHA-256 digests
//! that COMMIT what a local AI saw (the selected conversation context the user
//! chose to share) and what it produced (a draft reply), and produces the inputs
//! for mercury-core's AI policy gates ([`evaluate_ai_participant_action`] →
//! [`evaluate_ai_connector`]). The gates enforce the invariant: the AI is local,
//! user-selected, integrity-verified, DRAFT-ONLY (it can propose, never silently
//! send), with no plaintext bridge, no prompt retention, and no training; every
//! action is recorded in an audit trail bound to its input/output digests.
//!
//! The engine does NOT run a model — it takes the model's draft output as bytes.
//! The real work is the two audit digests (the auditable commitment), feeding the
//! gates, and recording the trail. Runtime/model SELECTION + integrity are honest
//! caller attestations ([`AiRuntimeAttestation`]) — never fabricated: the gate's
//! consent guarantee is only as strong as the user genuinely choosing the runtime
//! and model from a visible chooser, and the engine refuses to assert that for
//! them. Only audited SHA-256 is used; nothing crypto is hand-rolled.

#![forbid(unsafe_code)]

use sha2::{Digest as _, Sha256};

pub use mercury_core::{
    AiConnectorDecision, AiConnectorReason, AiConnectorRuntimeKind, AiGrantFacts, AiLifecycleFacts,
    AiParticipantDecision, AiParticipantReason, AiPolicyFacts, evaluate_ai_connector,
    evaluate_ai_participant_action,
};
use mercury_core::{AiConnectorInput, AiParticipantAction, AiParticipantRequest};

/// Domain separation for the two audit digests + the audit hash chain.
const CONTEXT_DOMAIN: &[u8] = b"mercury/ai/context/v1";
const DRAFT_DOMAIN: &[u8] = b"mercury/ai/draft/v1";
const CHAIN_DOMAIN: &[u8] = b"mercury/ai/audit-chain/v1";
const DIGEST_LEN: i32 = 32;
/// Genesis link for the audit hash chain (the previous-hash of the first entry).
const GENESIS_HASH: [u8; 32] = [0u8; 32];

/// Runtime + model facts the AI engine cannot compute from the content — the
/// caller (wired to the device UI + model-bundle verification) supplies its REAL
/// status; the engine never fabricates these. A `false` makes the gate honestly
/// refuse, which is the point: these fields exist to prevent SILENT model
/// substitution and to keep the AI a deliberate, visible choice.
/// - `runtime_kind`: which runtime class the AI is (default + only safe choice:
///   [`AiConnectorRuntimeKind::LocalDevice`]).
/// - `runtime_user_selected` / `model_user_selected`: the user explicitly chose
///   this runtime / model from a visible chooser.
/// - `connector_authenticated`: the local connector is authenticated to the device.
/// - `model_integrity_verified`: the model bundle's hash/signature is verified.
/// - `high_security_room`: the room is high-security (forces a LocalDevice runtime).
#[derive(Debug, Clone, Copy)]
pub struct AiRuntimeAttestation {
    pub runtime_kind: AiConnectorRuntimeKind,
    pub runtime_user_selected: bool,
    pub model_user_selected: bool,
    pub connector_authenticated: bool,
    pub model_integrity_verified: bool,
    pub high_security_room: bool,
}

impl AiRuntimeAttestation {
    /// A fully-attested standard LOCAL-DEVICE AI: the user selected the runtime +
    /// model, the connector is authenticated, the model integrity is verified,
    /// and the room is not high-security. Use only when these facts genuinely hold.
    pub const fn local_device() -> Self {
        Self {
            runtime_kind: AiConnectorRuntimeKind::LocalDevice,
            runtime_user_selected: true,
            model_user_selected: true,
            connector_authenticated: true,
            model_integrity_verified: true,
            high_security_room: false,
        }
    }
}

/// Build a standard, valid grant for a LOCAL DRAFT-ONLY AI in a standard room:
/// user-granted principal, local mode, read + draft scope, NO tool/retention/
/// training/prompt-store, active and unexpired over `[now_s, expires_at_s)`. The
/// caller reads the real grant from storage in production; this is the canonical
/// shape the gate accepts.
pub fn standard_local_draft_grant(now_s: i32, expires_at_s: i32) -> AiPolicyFacts {
    AiPolicyFacts {
        grant: AiGrantFacts {
            version: 1,
            principal_kind: 2,
            room_mode: 1,
            ai_mode: 1,
            ttl_s: 300,
            approver_count: 1,
            read_scope: 1,
            write_scope: 1,
            tool_scope: 0,
            retention_mode: 0,
            training_allowed: 0,
            prompt_store_allowed: 0,
        },
        lifecycle: AiLifecycleFacts {
            version: 1,
            grant_state: 1,
            revoke_reason: 0,
            now_s,
            expires_at_s,
            room_mode: 1,
            // Overwritten by the gate from the action's access kind; supply Read.
            access_kind: 0,
            epoch_rotated: 0,
        },
    }
}

/// Build a standard, valid grant for a LOCAL AI permitted to propose CONFIRMED
/// SENDS — identical to [`standard_local_draft_grant`] but with `write_scope: 2`
/// (the step up the gate requires for `SendMessageWithConfirmation`; draft-only is
/// `write_scope: 1`). The user must still confirm each send; the AI never
/// auto-sends.
pub fn standard_local_send_grant(now_s: i32, expires_at_s: i32) -> AiPolicyFacts {
    let mut grant = standard_local_draft_grant(now_s, expires_at_s);
    grant.grant.write_scope = 2;
    grant
}

/// Build a standard, valid grant for a LOCAL AI permitted read-only TOOL USE at
/// `tool_scope` (1 = read-only local tools; 2 = also room-search-selected) —
/// identical to [`standard_local_draft_grant`] but with the given `tool_scope`
/// (draft-only is `tool_scope: 0`, no tools).
pub fn standard_local_tool_grant(now_s: i32, expires_at_s: i32, tool_scope: i32) -> AiPolicyFacts {
    let mut grant = standard_local_draft_grant(now_s, expires_at_s);
    grant.grant.tool_scope = tool_scope;
    grant
}

/// SHA-256 over the selected context messages, length-prefixed + domain-separated
/// so distinct message sets cannot collide via concatenation. This is the
/// auditable commitment to exactly what the AI was shown.
fn context_digest(selected_messages: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(CONTEXT_DOMAIN);
    hasher.update((selected_messages.len() as u64).to_le_bytes());
    for message in selected_messages {
        hasher.update((message.len() as u64).to_le_bytes());
        hasher.update(message);
    }
    hasher.finalize().into()
}

/// SHA-256 over the AI's draft output — the auditable commitment to exactly what
/// the AI produced.
fn draft_digest(draft: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DRAFT_DOMAIN);
    hasher.update(draft);
    hasher.finalize().into()
}

/// A prepared draft-assist action: the gate input plus the REAL audit digests
/// committing the AI's input context and draft output. Evaluate it against the
/// gate via [`PreparedDraft::evaluate`] (or record it in an [`AiAuditLog`]).
#[derive(Debug, Clone)]
pub struct PreparedDraft {
    connector_input: AiConnectorInput,
    context_digest: [u8; 32],
    draft_digest: [u8; 32],
}

/// Prepare a LOCAL AI draft-reply action over `selected_messages` (the context the
/// AI is shown) producing `draft` (the AI's proposed reply), under `grant` and
/// `attestation`. Computes the real context + draft SHA-256 digests, counts the
/// selected messages, and assembles the participant + connector gate inputs.
///
/// Remote/development runtimes are NOT allowed by default (`allow_remote_runtime`
/// / `allow_development_runtime` are fixed false): Mercury's default is a
/// local-only AI, so a non-local `runtime_kind` is honestly rejected by the gate.
pub fn prepare_draft(
    selected_messages: &[&[u8]],
    draft: &[u8],
    grant: AiPolicyFacts,
    attestation: &AiRuntimeAttestation,
) -> PreparedDraft {
    let context_digest = context_digest(selected_messages);
    let draft_digest = draft_digest(draft);
    let selected_message_count = i32::try_from(selected_messages.len()).unwrap_or(i32::MAX);

    let participant_request = AiParticipantRequest::new(
        AiParticipantAction::DraftReply,
        grant,
        true, // participant_visible
        true, // grant_visible
        selected_message_count,
        DIGEST_LEN, // input_digest_len (the 32-byte context digest)
        DIGEST_LEN, // output_digest_len (the 32-byte draft digest)
        0,          // plaintext_identity_fields
    );

    let connector_input = AiConnectorInput {
        participant_request,
        runtime_kind: attestation.runtime_kind,
        runtime_user_selected: attestation.runtime_user_selected,
        model_user_selected: attestation.model_user_selected,
        connector_authenticated: attestation.connector_authenticated,
        model_integrity_verified: attestation.model_integrity_verified,
        allow_development_runtime: false,
        allow_remote_runtime: false,
        high_security_room: attestation.high_security_room,
        context_digest_len: DIGEST_LEN,
        draft_output_digest_len: DIGEST_LEN,
        plaintext_bridge_fields: 0,
        prompt_retention_enabled: false,
        training_enabled: false,
        direct_send_enabled: false,
        tool_execution_enabled: false,
    };

    PreparedDraft {
        connector_input,
        context_digest,
        draft_digest,
    }
}

impl PreparedDraft {
    /// The assembled connector gate input.
    pub fn connector_input(&self) -> AiConnectorInput {
        self.connector_input
    }

    /// The auditable SHA-256 digest of the selected context the AI was shown.
    pub fn context_digest(&self) -> &[u8; 32] {
        &self.context_digest
    }

    /// The auditable SHA-256 digest of the AI's draft output.
    pub fn draft_digest(&self) -> &[u8; 32] {
        &self.draft_digest
    }

    /// Run the full AI policy gate chain (participant → connector) on this action.
    pub fn evaluate(&self) -> AiConnectorDecision {
        evaluate_ai_connector(self.connector_input)
    }
}

/// A prepared confirmed-SEND proposal: the AI, over some selected context, proposes
/// to send a specific message that the USER must confirm. Carries the participant
/// gate input plus the REAL audit digests (the context the AI saw + the message it
/// proposes to send). A confirmed send goes through the participant gate only (not
/// the draft-specific connector gate).
#[derive(Debug, Clone)]
pub struct PreparedSend {
    participant_request: AiParticipantRequest,
    context_digest: [u8; 32],
    draft_digest: [u8; 32],
}

/// Prepare a confirmed-SEND proposal: over `selected_messages`, the AI proposes to
/// send `message_to_send` (requiring user confirmation). Computes the real context
/// and message digests, then assembles the participant gate input. Requires a grant
/// with confirmed-send write scope ([`standard_local_send_grant`]); a draft-only
/// grant is honestly rejected by the gate. The engine NEVER auto-sends — an
/// accepted decision still `requires_user_confirmation`.
pub fn prepare_send(
    selected_messages: &[&[u8]],
    message_to_send: &[u8],
    grant: AiPolicyFacts,
) -> PreparedSend {
    let context_digest = context_digest(selected_messages);
    let draft_digest = draft_digest(message_to_send);
    let selected_message_count = i32::try_from(selected_messages.len()).unwrap_or(i32::MAX);
    let participant_request = AiParticipantRequest::new(
        AiParticipantAction::SendMessageWithConfirmation,
        grant,
        true, // participant_visible
        true, // grant_visible
        selected_message_count,
        DIGEST_LEN, // input_digest_len (the context digest)
        DIGEST_LEN, // output_digest_len (the message-to-send digest)
        0,          // plaintext_identity_fields
    );
    PreparedSend {
        participant_request,
        context_digest,
        draft_digest,
    }
}

impl PreparedSend {
    /// The auditable digest of the context the AI was shown.
    pub fn context_digest(&self) -> &[u8; 32] {
        &self.context_digest
    }

    /// The auditable digest of the message the AI proposes to send.
    pub fn draft_digest(&self) -> &[u8; 32] {
        &self.draft_digest
    }

    /// Evaluate the confirmed-send proposal against the participant gate.
    pub fn evaluate(&self) -> AiParticipantDecision {
        evaluate_ai_participant_action(self.participant_request)
    }
}

/// The READ-ONLY tool actions Mercury's local AI may take over selected context.
/// (Open-world external tools are deliberately NOT offered — the gate forbids
/// them, and exposing only the read-only kinds keeps that secure default explicit.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiToolAction {
    /// A read-only local tool over the selected context (needs tool scope >= 1).
    ReadOnlyLocal,
    /// A room-search tool over the selected context (needs the broader tool scope 2).
    RoomSearchSelected,
}

impl AiToolAction {
    fn participant_action(self) -> AiParticipantAction {
        match self {
            Self::ReadOnlyLocal => AiParticipantAction::UseReadOnlyLocalTool,
            Self::RoomSearchSelected => AiParticipantAction::UseRoomSearchSelectedTool,
        }
    }
}

/// A prepared read-only TOOL-USE action: the AI invokes a tool over selected
/// context. Carries the participant gate input plus the REAL audit digests (the
/// context the AI saw + the tool request it issued). Tool use goes through the
/// participant gate only.
#[derive(Debug, Clone)]
pub struct PreparedToolUse {
    participant_request: AiParticipantRequest,
    context_digest: [u8; 32],
    draft_digest: [u8; 32],
    tool_action: AiToolAction,
}

/// Prepare a read-only tool use: over `selected_messages`, the AI issues
/// `tool_request` to a `tool_action` tool. Computes the real context + request
/// digests and assembles the participant gate input. Requires a grant whose tool
/// scope permits the action ([`standard_local_tool_grant`]); an insufficient scope
/// is honestly rejected. Tools require selected context, so an empty selection is
/// rejected by the gate.
pub fn prepare_tool_use(
    selected_messages: &[&[u8]],
    tool_request: &[u8],
    tool_action: AiToolAction,
    grant: AiPolicyFacts,
) -> PreparedToolUse {
    let context_digest = context_digest(selected_messages);
    let draft_digest = draft_digest(tool_request);
    let selected_message_count = i32::try_from(selected_messages.len()).unwrap_or(i32::MAX);
    let participant_request = AiParticipantRequest::new(
        tool_action.participant_action(),
        grant,
        true, // participant_visible
        true, // grant_visible
        selected_message_count,
        DIGEST_LEN, // input_digest_len (the context digest)
        DIGEST_LEN, // output_digest_len (the tool-request digest)
        0,          // plaintext_identity_fields
    );
    PreparedToolUse {
        participant_request,
        context_digest,
        draft_digest,
        tool_action,
    }
}

impl PreparedToolUse {
    /// The auditable digest of the context the AI was shown.
    pub fn context_digest(&self) -> &[u8; 32] {
        &self.context_digest
    }

    /// The auditable digest of the tool request the AI issued.
    pub fn draft_digest(&self) -> &[u8; 32] {
        &self.draft_digest
    }

    /// Evaluate the tool-use action against the participant gate.
    pub fn evaluate(&self) -> AiParticipantDecision {
        evaluate_ai_participant_action(self.participant_request)
    }
}

/// Stable per-action code bound into the audit chain (so the action kind can't
/// be silently rewritten without breaking the hash).
fn action_code(action: AiParticipantAction) -> u32 {
    match action {
        AiParticipantAction::ReadSelectedContext => 1,
        AiParticipantAction::DraftReply => 2,
        AiParticipantAction::SendMessageWithConfirmation => 3,
        AiParticipantAction::AutonomousSend => 4,
        AiParticipantAction::UseReadOnlyLocalTool => 5,
        AiParticipantAction::UseRoomSearchSelectedTool => 6,
        AiParticipantAction::UseOpenWorldExternalTool => 7,
        AiParticipantAction::StorePrompt => 8,
        AiParticipantAction::TrainOnContext => 9,
        AiParticipantAction::WriteMemory => 10,
    }
}

/// Map a participant-gate reason to a stable code for the audit chain (the
/// participant reason enum has no `code()` of its own; this is the audit binding).
fn participant_reason_code(reason: AiParticipantReason) -> i32 {
    match reason {
        AiParticipantReason::Accepted => 0,
        AiParticipantReason::ParticipantVisibilityRequired => 1,
        AiParticipantReason::GrantVisibilityRequired => 2,
        AiParticipantReason::PlaintextIdentityForbidden => 3,
        AiParticipantReason::BadAuditDigestLength => 4,
        AiParticipantReason::GrantPolicyRejected => 5,
        AiParticipantReason::GrantLifecycleRejected => 6,
        AiParticipantReason::SelectedContextRequired => 7,
        AiParticipantReason::ReadScopeRequired => 8,
        AiParticipantReason::WriteScopeRequired => 9,
        AiParticipantReason::ToolScopeRequired => 10,
        AiParticipantReason::PromptStoreForbidden => 11,
        AiParticipantReason::TrainingForbidden => 12,
        AiParticipantReason::MemoryWriteForbidden => 13,
        AiParticipantReason::AutonomousSendForbidden => 14,
    }
}

/// The chained entry hash: `SHA-256(CHAIN_DOMAIN || prev_hash || sequence ||
/// action || accepted || reason || context_digest || draft_digest)`. Linking each
/// entry to its predecessor makes the trail tamper-evident: altering, removing, or
/// reordering any record changes the recomputed hash and breaks the chain.
#[allow(clippy::too_many_arguments)]
fn compute_entry_hash(
    prev_hash: &[u8; 32],
    sequence: u64,
    action: AiParticipantAction,
    accepted: bool,
    reason_code: i32,
    context_digest: &[u8; 32],
    draft_digest: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(CHAIN_DOMAIN);
    hasher.update(prev_hash);
    hasher.update(sequence.to_le_bytes());
    hasher.update(action_code(action).to_le_bytes());
    hasher.update([accepted as u8]);
    hasher.update(reason_code.to_le_bytes());
    hasher.update(context_digest);
    hasher.update(draft_digest);
    hasher.finalize().into()
}

/// One recorded AI action: the gate outcome bound to the REAL input/output
/// digests, plus its position (`sequence`) and chained `entry_hash` linking it to
/// the prior record, so an auditor can later verify exactly what the AI saw and
/// produced AND that the trail has not been tampered with. `reason_code` is the
/// gate decision's reason code (a draft action carries an `AiConnectorReason`
/// code; a confirmed-send carries an `AiParticipantReason` code — disambiguated by
/// `action`). REJECTED actions are recorded too — the trail is complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiActionAudit {
    pub action: AiParticipantAction,
    pub accepted: bool,
    pub reason_code: i32,
    pub context_digest: [u8; 32],
    pub draft_digest: [u8; 32],
    pub sequence: u64,
    pub entry_hash: [u8; 32],
}

impl AiActionAudit {
    /// The canonical 32-byte commitment for anchoring this AI action into an
    /// EXTERNAL transparency log (e.g. mercury-audit's sealed-audit event log). It
    /// is the record's hash-chain `entry_hash`, which already commits the action,
    /// its accept/reason, both content digests, the sequence, and the prior entry —
    /// so anchoring it ties the external log to exactly this AI action while
    /// revealing nothing about the AI's context or draft content (the anchor is a
    /// hash, not the plaintext). The value is domain-separated (computed under
    /// `mercury/ai/audit-chain/v1`), so it cannot be confused with a commitment from
    /// another subsystem.
    pub fn transparency_anchor_bytes(&self) -> [u8; 32] {
        self.entry_hash
    }
}

/// Verify an audit trail's hash chain: each record's `sequence` must equal its
/// index, and its `entry_hash` must equal the recomputed
/// `SHA-256(prev || content)` linking it to the prior record (genesis for the
/// first). Returns `false` on any tampering, removal, or reordering.
pub fn verify_chain(records: &[AiActionAudit]) -> bool {
    let mut prev_hash = GENESIS_HASH;
    for (index, record) in records.iter().enumerate() {
        if record.sequence != index as u64 {
            return false;
        }
        let expected = compute_entry_hash(
            &prev_hash,
            record.sequence,
            record.action,
            record.accepted,
            record.reason_code,
            &record.context_digest,
            &record.draft_digest,
        );
        if expected != record.entry_hash {
            return false;
        }
        prev_hash = record.entry_hash;
    }
    true
}

/// An append-only, HASH-CHAINED audit trail of AI actions, each bound to its
/// content digests and linked to its predecessor. This is the "never a hidden
/// backdoor" mechanism: every time the AI is given context / proposes a draft it
/// is committed here, and the chain head ([`AiAuditLog::head_hash`]) is a single
/// 32-byte value committing the ENTIRE trail — publish/witness it and any later
/// alteration of any record is detectable ([`AiAuditLog::verify`]).
#[derive(Debug, Clone, Default)]
pub struct AiAuditLog {
    records: Vec<AiActionAudit>,
}

impl AiAuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an action outcome to the tamper-evident trail, chained to the prior
    /// record. Shared by the draft and confirmed-send paths.
    fn append(
        &mut self,
        action: AiParticipantAction,
        accepted: bool,
        reason_code: i32,
        context_digest: [u8; 32],
        draft_digest: [u8; 32],
    ) {
        let sequence = self.records.len() as u64;
        let prev_hash = self.head_hash();
        let entry_hash = compute_entry_hash(
            &prev_hash,
            sequence,
            action,
            accepted,
            reason_code,
            &context_digest,
            &draft_digest,
        );
        self.records.push(AiActionAudit {
            action,
            accepted,
            reason_code,
            context_digest,
            draft_digest,
            sequence,
            entry_hash,
        });
    }

    /// Evaluate a prepared draft action against the gate and append it (accepted
    /// or not) to the tamper-evident trail, bound to its real digests and chained
    /// to the prior record. Returns the gate decision.
    pub fn run(&mut self, prepared: &PreparedDraft) -> AiConnectorDecision {
        let decision = prepared.evaluate();
        self.append(
            AiParticipantAction::DraftReply,
            decision.accepted,
            decision.reason.code(),
            prepared.context_digest,
            prepared.draft_digest,
        );
        decision
    }

    /// Evaluate a confirmed-SEND proposal against the participant gate and append
    /// it (accepted or not) to the tamper-evident trail. Returns the participant
    /// decision; an accepted send still `requires_user_confirmation` (the engine
    /// never auto-sends).
    pub fn run_send(&mut self, prepared: &PreparedSend) -> AiParticipantDecision {
        let decision = prepared.evaluate();
        self.append(
            AiParticipantAction::SendMessageWithConfirmation,
            decision.accepted,
            participant_reason_code(decision.reason),
            prepared.context_digest,
            prepared.draft_digest,
        );
        decision
    }

    /// Evaluate a read-only tool-use action against the participant gate and append
    /// it (accepted or not) to the tamper-evident trail. Returns the decision.
    pub fn run_tool(&mut self, prepared: &PreparedToolUse) -> AiParticipantDecision {
        let decision = prepared.evaluate();
        self.append(
            prepared.tool_action.participant_action(),
            decision.accepted,
            participant_reason_code(decision.reason),
            prepared.context_digest,
            prepared.draft_digest,
        );
        decision
    }

    /// The recorded audit trail.
    pub fn records(&self) -> &[AiActionAudit] {
        &self.records
    }

    /// The chain head — the most recent entry hash, or [`GENESIS_HASH`] when
    /// empty. A single 32-byte commitment to the whole trail.
    pub fn head_hash(&self) -> [u8; 32] {
        self.records
            .last()
            .map_or(GENESIS_HASH, |record| record.entry_hash)
    }

    /// Verify the trail's hash chain is intact (no tampering, removal, reorder).
    pub fn verify(&self) -> bool {
        verify_chain(&self.records)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i32 = 1_000;
    const EXPIRES: i32 = 1_300;

    fn grant() -> AiPolicyFacts {
        standard_local_draft_grant(NOW, EXPIRES)
    }

    #[test]
    fn local_draft_is_gate_accepted_and_audited() {
        let messages: [&[u8]; 2] = [b"hi, are we still on for friday?", b"yes -- 7pm works"];
        let draft = b"Confirming Friday at 7pm. See you then!";
        let prepared = prepare_draft(
            &messages,
            draft,
            grant(),
            &AiRuntimeAttestation::local_device(),
        );

        let mut log = AiAuditLog::new();
        let decision = log.run(&prepared);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert!(decision.can_call_model);
        assert!(decision.can_emit_draft);
        assert!(
            !decision.can_send_message,
            "draft-only: AI cannot silently send"
        );
        assert!(!decision.can_use_tool);
        assert!(decision.requires_user_review);
        assert!(decision.forbids_prompt_retention && decision.forbids_training);
        assert!(!decision.plaintext_bytes_exposed);

        // The action was recorded bound to the REAL digests.
        assert_eq!(log.len(), 1);
        let record = log.records()[0];
        assert!(record.accepted);
        assert_eq!(record.context_digest, *prepared.context_digest());
        assert_eq!(record.draft_digest, *prepared.draft_digest());
    }

    #[test]
    fn digests_commit_the_real_bytes_not_a_constant() {
        let messages: [&[u8]; 1] = [b"the selected context"];
        let a = prepare_draft(
            &messages,
            b"draft A",
            grant(),
            &AiRuntimeAttestation::local_device(),
        );
        let b = prepare_draft(
            &messages,
            b"draft B",
            grant(),
            &AiRuntimeAttestation::local_device(),
        );

        // Same context -> same context digest; different drafts -> different draft
        // digests (so the engine hashes the actual bytes, not a placeholder).
        assert_eq!(a.context_digest(), b.context_digest());
        assert_ne!(a.draft_digest(), b.draft_digest());

        // Different context -> different context digest.
        let other_ctx: [&[u8]; 1] = [b"a DIFFERENT context"];
        let c = prepare_draft(
            &other_ctx,
            b"draft A",
            grant(),
            &AiRuntimeAttestation::local_device(),
        );
        assert_ne!(a.context_digest(), c.context_digest());

        // The digest matches an independent recomputation of the real draft bytes.
        let expected: [u8; 32] = {
            let mut h = Sha256::new();
            h.update(DRAFT_DOMAIN);
            h.update(b"draft A");
            h.finalize().into()
        };
        assert_eq!(a.draft_digest(), &expected);
    }

    #[test]
    fn missing_selected_context_is_rejected() {
        let none: [&[u8]; 0] = [];
        let prepared = prepare_draft(
            &none,
            b"draft",
            grant(),
            &AiRuntimeAttestation::local_device(),
        );
        let decision = prepared.evaluate();
        assert!(!decision.accepted);
        // The participant gate rejects (no context); the connector surfaces it.
        assert_eq!(decision.reason, AiConnectorReason::ParticipantRejected);
    }

    #[test]
    fn unselected_runtime_is_rejected() {
        let messages: [&[u8]; 1] = [b"ctx"];
        let mut attestation = AiRuntimeAttestation::local_device();
        attestation.runtime_user_selected = false;
        let prepared = prepare_draft(&messages, b"draft", grant(), &attestation);
        let decision = prepared.evaluate();
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AiConnectorReason::RuntimeNotUserSelected);
    }

    #[test]
    fn unverified_model_integrity_is_rejected() {
        let messages: [&[u8]; 1] = [b"ctx"];
        let mut attestation = AiRuntimeAttestation::local_device();
        attestation.model_integrity_verified = false;
        let prepared = prepare_draft(&messages, b"draft", grant(), &attestation);
        assert_eq!(
            prepared.evaluate().reason,
            AiConnectorReason::ModelIntegrityUnverified
        );
    }

    #[test]
    fn high_security_room_forbids_non_local_runtime() {
        let messages: [&[u8]; 1] = [b"ctx"];
        let attestation = AiRuntimeAttestation {
            runtime_kind: AiConnectorRuntimeKind::UserHostedLocalNetwork,
            high_security_room: true,
            ..AiRuntimeAttestation::local_device()
        };
        let prepared = prepare_draft(&messages, b"draft", grant(), &attestation);
        assert_eq!(
            prepared.evaluate().reason,
            AiConnectorReason::HighSecurityRequiresLocalRuntime
        );
    }

    #[test]
    fn a_training_enabled_grant_is_rejected() {
        // A grant that allows training violates the no-training invariant -> the
        // participant grant-policy check rejects, surfaced as ParticipantRejected.
        let messages: [&[u8]; 1] = [b"ctx"];
        let mut bad_grant = grant();
        bad_grant.grant.training_allowed = 1;
        let prepared = prepare_draft(
            &messages,
            b"draft",
            bad_grant,
            &AiRuntimeAttestation::local_device(),
        );
        let decision = prepared.evaluate();
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AiConnectorReason::ParticipantRejected);
    }

    #[test]
    fn rejected_actions_are_still_audited() {
        // The audit trail is complete: a rejected action is recorded too, bound to
        // its digests (so a denied AI access attempt is not invisible).
        let none: [&[u8]; 0] = [];
        let prepared = prepare_draft(
            &none,
            b"draft",
            grant(),
            &AiRuntimeAttestation::local_device(),
        );
        let mut log = AiAuditLog::new();
        let decision = log.run(&prepared);
        assert!(!decision.accepted);
        assert_eq!(log.len(), 1);
        assert!(!log.records()[0].accepted);
        assert_eq!(log.records()[0].draft_digest, *prepared.draft_digest());
    }

    fn run_n(log: &mut AiAuditLog, n: usize) {
        for i in 0..n {
            let msg = format!("message number {i}");
            let messages: [&[u8]; 1] = [msg.as_bytes()];
            let draft = format!("draft {i}");
            let prepared = prepare_draft(
                &messages,
                draft.as_bytes(),
                grant(),
                &AiRuntimeAttestation::local_device(),
            );
            log.run(&prepared);
        }
    }

    #[test]
    fn audit_chain_verifies_and_advances() {
        let mut log = AiAuditLog::new();
        assert_eq!(log.head_hash(), GENESIS_HASH);
        assert!(log.verify());

        run_n(&mut log, 3);

        assert!(log.verify(), "an untampered chain must verify");
        assert_eq!(log.len(), 3);
        // Sequences are 0,1,2 and the head is the last entry hash, not genesis.
        for (i, record) in log.records().iter().enumerate() {
            assert_eq!(record.sequence, i as u64);
        }
        assert_ne!(log.head_hash(), GENESIS_HASH);
        assert_eq!(log.head_hash(), log.records()[2].entry_hash);
        // Distinct entries -> distinct linked hashes.
        assert_ne!(log.records()[0].entry_hash, log.records()[1].entry_hash);
        assert_ne!(log.records()[1].entry_hash, log.records()[2].entry_hash);
    }

    #[test]
    fn tampering_a_record_breaks_verification() {
        let mut log = AiAuditLog::new();
        run_n(&mut log, 3);
        assert!(verify_chain(log.records()));

        // Flip a byte of the middle record's draft digest WITHOUT updating its
        // stored entry hash: the recomputed hash no longer matches -> broken.
        let mut tampered = log.records().to_vec();
        tampered[1].draft_digest[0] ^= 0x01;
        assert!(
            !verify_chain(&tampered),
            "content tampering must be detected"
        );

        // Tampering the accept flag is likewise caught.
        let mut flipped = log.records().to_vec();
        flipped[0].accepted = !flipped[0].accepted;
        assert!(!verify_chain(&flipped));
    }

    #[test]
    fn removing_or_reordering_breaks_verification() {
        let mut log = AiAuditLog::new();
        run_n(&mut log, 3);

        // Drop the middle record: the survivor's sequence no longer matches its
        // index (and the linkage breaks).
        let mut removed = log.records().to_vec();
        removed.remove(1);
        assert!(!verify_chain(&removed), "removal must be detected");

        // Swap two records: sequences no longer match their positions.
        let mut reordered = log.records().to_vec();
        reordered.swap(0, 1);
        assert!(!verify_chain(&reordered), "reordering must be detected");
    }

    #[test]
    fn different_content_yields_a_different_head() {
        // Two logs with the same action shape but a different draft must produce
        // different chain heads -- the head commits the actual content.
        let messages: [&[u8]; 1] = [b"same context"];
        let mut a = AiAuditLog::new();
        let mut b = AiAuditLog::new();
        a.run(&prepare_draft(
            &messages,
            b"draft A",
            grant(),
            &AiRuntimeAttestation::local_device(),
        ));
        b.run(&prepare_draft(
            &messages,
            b"draft B",
            grant(),
            &AiRuntimeAttestation::local_device(),
        ));
        assert_ne!(a.head_hash(), b.head_hash());
        assert!(a.verify() && b.verify());
    }

    #[test]
    fn confirmed_send_is_accepted_under_a_send_grant_and_audited() {
        let context: [&[u8]; 1] = [b"can you send my reply?"];
        let message = b"On my way, be there in 10.";
        let prepared = prepare_send(&context, message, standard_local_send_grant(NOW, EXPIRES));

        let mut log = AiAuditLog::new();
        let decision = log.run_send(&prepared);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert!(decision.can_send_message);
        // The user must still confirm -- the AI never auto-sends.
        assert!(decision.requires_user_confirmation);

        assert_eq!(log.len(), 1);
        assert_eq!(
            log.records()[0].action,
            AiParticipantAction::SendMessageWithConfirmation
        );
        assert_eq!(log.records()[0].draft_digest, *prepared.draft_digest());
        assert!(log.verify());
    }

    #[test]
    fn a_draft_only_grant_cannot_authorize_a_send() {
        // The draft grant has write_scope 1; a confirmed send needs write_scope 2.
        // The engine forwards the real grant -> the gate rejects WriteScopeRequired.
        let context: [&[u8]; 1] = [b"ctx"];
        let prepared = prepare_send(&context, b"a message", grant());
        let mut log = AiAuditLog::new();
        let decision = log.run_send(&prepared);
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AiParticipantReason::WriteScopeRequired);
        // The denied send is still recorded in the tamper-evident trail.
        assert_eq!(log.len(), 1);
        assert!(!log.records()[0].accepted);
        assert!(log.verify());
    }

    #[test]
    fn a_mixed_draft_and_send_trail_chains_and_verifies() {
        let context: [&[u8]; 1] = [b"shared context"];
        let mut log = AiAuditLog::new();
        log.run(&prepare_draft(
            &context,
            b"a draft",
            grant(),
            &AiRuntimeAttestation::local_device(),
        ));
        log.run_send(&prepare_send(
            &context,
            b"a send",
            standard_local_send_grant(NOW, EXPIRES),
        ));

        assert_eq!(log.len(), 2);
        assert_eq!(log.records()[0].action, AiParticipantAction::DraftReply);
        assert_eq!(
            log.records()[1].action,
            AiParticipantAction::SendMessageWithConfirmation
        );
        // The single chain links both action kinds and verifies end to end.
        assert!(log.verify());
        assert!(!verify_chain(&{
            let mut tampered = log.records().to_vec();
            tampered[0].draft_digest[0] ^= 0x01;
            tampered
        }));
    }

    #[test]
    fn read_only_local_tool_is_accepted_and_audited() {
        let context: [&[u8]; 1] = [b"summarize the thread for me"];
        let prepared = prepare_tool_use(
            &context,
            b"local_summarize(thread)",
            AiToolAction::ReadOnlyLocal,
            standard_local_tool_grant(NOW, EXPIRES, 1),
        );
        let mut log = AiAuditLog::new();
        let decision = log.run_tool(&prepared);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert!(decision.can_use_tool);
        assert!(!decision.can_send_message);
        assert_eq!(
            log.records()[0].action,
            AiParticipantAction::UseReadOnlyLocalTool
        );
        assert!(log.verify());
    }

    #[test]
    fn room_search_needs_the_broader_tool_scope() {
        let context: [&[u8]; 1] = [b"find where we agreed the date"];
        // tool_scope 1 permits read-only-local but NOT room-search.
        let narrow = prepare_tool_use(
            &context,
            b"room_search(date)",
            AiToolAction::RoomSearchSelected,
            standard_local_tool_grant(NOW, EXPIRES, 1),
        );
        assert_eq!(
            narrow.evaluate().reason,
            AiParticipantReason::ToolScopeRequired
        );
        // tool_scope 2 permits it.
        let broad = prepare_tool_use(
            &context,
            b"room_search(date)",
            AiToolAction::RoomSearchSelected,
            standard_local_tool_grant(NOW, EXPIRES, 2),
        );
        assert!(broad.evaluate().accepted);
    }

    #[test]
    fn tool_use_is_rejected_without_tool_scope() {
        // The draft grant has tool_scope 0 -> no tools.
        let context: [&[u8]; 1] = [b"ctx"];
        let prepared = prepare_tool_use(
            &context,
            b"local_tool()",
            AiToolAction::ReadOnlyLocal,
            grant(),
        );
        let mut log = AiAuditLog::new();
        let decision = log.run_tool(&prepared);
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AiParticipantReason::ToolScopeRequired);
        // The denied tool attempt is still recorded.
        assert_eq!(log.len(), 1);
        assert!(!log.records()[0].accepted);
        assert!(log.verify());
    }

    #[test]
    fn the_full_action_surface_chains_in_one_trail() {
        // draft + confirmed-send + tool use all append to ONE tamper-evident chain.
        let context: [&[u8]; 1] = [b"the conversation context"];
        let mut log = AiAuditLog::new();
        log.run(&prepare_draft(
            &context,
            b"a draft",
            grant(),
            &AiRuntimeAttestation::local_device(),
        ));
        log.run_send(&prepare_send(
            &context,
            b"a send",
            standard_local_send_grant(NOW, EXPIRES),
        ));
        log.run_tool(&prepare_tool_use(
            &context,
            b"a tool call",
            AiToolAction::ReadOnlyLocal,
            standard_local_tool_grant(NOW, EXPIRES, 1),
        ));

        assert_eq!(log.len(), 3);
        assert!(log.verify());
        assert_eq!(log.records()[0].action, AiParticipantAction::DraftReply);
        assert_eq!(
            log.records()[1].action,
            AiParticipantAction::SendMessageWithConfirmation
        );
        assert_eq!(
            log.records()[2].action,
            AiParticipantAction::UseReadOnlyLocalTool
        );
    }
}
