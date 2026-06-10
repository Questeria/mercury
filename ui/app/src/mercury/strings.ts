// UI-facing string maps + tone helpers (ported from the design handoff).
// These are presentation concerns only -- they never alter a decision.

import { REASON_DICT } from "./simulator";
import type { AiState, PlatformDecisionView, Tone, TrustState } from "./types";

/** A tone resolves to a themed CSS variable. */
export type ToneKey = Tone | "ai" | "muted" | "ink";

export function toneVar(tone: ToneKey): string {
  switch (tone) {
    case "ok":
      return "var(--mc-ok)";
    case "warn":
      return "var(--mc-warn)";
    case "bad":
      return "var(--mc-bad)";
    case "ai":
      return "var(--mc-ai)";
    case "muted":
      return "var(--mc-muted)";
    default:
      return "var(--mc-ink)";
  }
}

export const FU_TRUST: Record<TrustState, { tone: Tone; label: string; sub: string }> = {
  trusted: { tone: "ok", label: "Verified", sub: "safety number matched" },
  sendable_not_full: { tone: "warn", label: "Sendable", sub: "one key not fully verified" },
  unverified: { tone: "warn", label: "New device", sub: "first-use pending" },
  rejected: { tone: "bad", label: "Key change", sub: "unconfirmed - send blocked" },
  stale: { tone: "bad", label: "Key stale", sub: "rehandshake required" },
};

export const FU_AI: Record<AiState, { label: string; sub: string; tone: ToneKey }> = {
  granted: { label: "Mercury AI", sub: "scoped / read-only / 24h", tone: "ai" },
  absent: { label: "AI off", sub: "no grant in this room", tone: "muted" },
  revoked: { label: "AI revoked", sub: "access removed", tone: "bad" },
  expired: { label: "AI expired", sub: "grant lapsed", tone: "muted" },
};

/**
 * Plain-language gloss for each reason_label, scoped by decision source.
 * Covers both the design-handoff simulator vocabulary and the real mercury-core
 * labels (so "core (live)" reads as English, not a flag dump).
 */
export const REASON_EXPLAIN: Record<string, Record<string, string>> = {
  client_bootstrap: {
    ACCEPTED: "Client is ready. Message UI may open.",
    SYNC_INCOMPLETE: "Sync is not complete yet - message UI stays closed.",
    SYNC_GAP_DETECTED: "A gap in synced state was detected - sync must finish first.",
    SYNC_FAILED: "Sync failed - retry before opening the message UI.",
    RECOVERY_REQUIRED: "Account or device secrets need recovery before continuing.",
    KEY_TRANSPARENCY_NOT_READY: "Key transparency proofs aren't ready yet.",
    REPLAY_GAP_DETECTED: "A replay-checkpoint gap was detected - fetch missing state.",
    ACCOUNT_SECRET_MISSING: "Account secret is missing - recovery required.",
    DEVICE_SECRET_MISSING: "Device secret is missing - recovery required.",
  },
  outbound_send: {
    ACCEPTED: "Outgoing message will be sent and ciphertext persisted.",
    TOFU_PENDING: "First-use verification needed before this send goes out.",
    KEY_STALE: "Recipient's key is stale; re-handshake required.",
    RECIPIENT_TRUST_REJECTED: "Recipient's key change is unconfirmed - send is held.",
    DEVICE_TRUST_REJECTED: "A recipient device isn't trusted - send is held.",
    MESSAGE_POLICY_REJECTED: "Message policy rejected this send.",
    ROOM_MEMBERSHIP_TRANSITION_REJECTED: "A room-membership change is unconfirmed - send is held.",
    ROOM_MEMBERSHIP_EPOCH_NOT_ROTATED: "The room epoch hasn't rotated for the new membership.",
    LOCAL_STORE_SEALING_REJECTED: "Outgoing ciphertext could not be sealed locally.",
    AI_GRANT_ABSENT: "No AI grant exists in this room - @ai mentions are held.",
    AI_GRANT_REVOKED: "AI grant has been revoked - @ai mentions are held.",
    AI_GRANT_EXPIRED: "AI grant has lapsed - @ai mentions are held.",
  },
  client_receive: {
    ACCEPTED: "Incoming message accepted and handed to the renderer.",
    ORDERING_GAP: "Out-of-order message - retrying to fill the gap.",
    SENDER_TRUST_REJECTED: "Sender is not trusted - plaintext withheld.",
    SENDER_DEVICE_TRUST_REJECTED: "The sender's device isn't trusted - plaintext withheld.",
    MESSAGE_POLICY_REJECTED: "Message policy rejected this inbound - plaintext withheld.",
    REPLAY_DUPLICATE: "Duplicate message - dropped, nothing rendered.",
    REPLAY_STALE: "Stale (replayed) message - dropped, nothing rendered.",
    RELAY_DELIVERY_REJECTED: "The relay rejected this delivery.",
  },
  policy_pipeline: {
    ACCEPT: "Policy pipeline accepted.",
    AI_GRANT_REJECT: "AI grant policy rejected this - @ai access is held.",
    AI_LIFECYCLE_REJECT: "The AI grant lifecycle (expiry/revoke) rejected this.",
    AI_POST_REVOKE_ACCESS_REJECT: "AI access after revoke is blocked.",
    AI_HIGHSEC_ROTATION_REQUIRED: "High-security rooms require a key rotation first.",
    ENVELOPE_REJECT: "The message envelope was rejected.",
    ROOM_EPOCH_REJECT: "The room epoch was rejected.",
  },
};

/** Friendly names for the boolean capability flags. */
export const FLAG_LABEL: Record<string, string> = {
  can_open_message_ui: "Message UI may open",
  can_start_sync: "Background sync allowed",
  can_send: "Sending allowed",
  can_receive: "Receiving allowed",
  can_persist_ciphertext: "Ciphertext will persist",
  requires_sync: "Needs sync",
  requires_recovery: "Needs recovery",
  requires_client_retry: "Needs retry",
  requires_user_action: "Needs your confirmation",
};

/** The capability flags, in display order. */
export const CAPABILITY_FLAGS: (keyof PlatformDecisionView)[] = [
  "can_open_message_ui",
  "can_start_sync",
  "can_send",
  "can_receive",
  "can_persist_ciphertext",
  "requires_sync",
  "requires_recovery",
  "requires_client_retry",
  "requires_user_action",
];

export function explainDecision(view: PlatformDecisionView, source?: string): string {
  const bySource = REASON_EXPLAIN[source ?? view.source] ?? {};
  return (
    bySource[view.reason_label] ??
    (view.accepted ? "Decision accepted." : "Decision rejected by the gate.")
  );
}

// ---- reason-label gloss + tone (handoff + real mercury-core vocabularies) ----

/** Curated glosses/tones for the real mercury-core labels not in REASON_DICT. */
const CORE_GLOSS: Record<string, { gloss: string; tone: Tone }> = {
  // policy_pipeline
  ACCEPT: { gloss: "Accepted", tone: "ok" },
  BAD_VERSION: { gloss: "Unsupported protocol version", tone: "bad" },
  BAD_ACTOR_KIND: { gloss: "Invalid actor kind", tone: "bad" },
  BAD_ENVELOPE_REASON: { gloss: "Invalid envelope", tone: "bad" },
  BAD_ROOM_EPOCH_REASON: { gloss: "Invalid room epoch", tone: "bad" },
  BAD_AI_GRANT_REASON: { gloss: "Invalid AI grant", tone: "bad" },
  BAD_AI_LIFECYCLE_REASON: { gloss: "Invalid AI lifecycle", tone: "bad" },
  AI_COMPONENT_FOR_HUMAN: { gloss: "AI component addressed to a human", tone: "bad" },
  ENVELOPE_REJECT: { gloss: "Envelope rejected", tone: "bad" },
  ROOM_EPOCH_REJECT: { gloss: "Room epoch rejected", tone: "bad" },
  AI_GRANT_REJECT: { gloss: "AI grant rejected", tone: "bad" },
  AI_LIFECYCLE_REJECT: { gloss: "AI grant lifecycle rejected", tone: "bad" },
  AI_POST_REVOKE_ACCESS_REJECT: { gloss: "AI access after revoke - blocked", tone: "bad" },
  AI_HIGHSEC_ROTATION_REQUIRED: { gloss: "High-security key rotation required", tone: "warn" },
  UNKNOWN: { gloss: "Unknown reason", tone: "bad" },
  // outbound_send
  DEVICE_TRUST_REJECTED: { gloss: "Device trust rejected", tone: "bad" },
  ROOM_MEMBERSHIP_TRANSITION_REJECTED: { gloss: "Room membership change rejected", tone: "bad" },
  ROOM_MEMBERSHIP_EPOCH_NOT_ROTATED: { gloss: "Room epoch not rotated", tone: "bad" },
  MESSAGE_POLICY_REJECTED: { gloss: "Message policy rejected", tone: "bad" },
  LOCAL_STORE_SEALING_REJECTED: { gloss: "Local store sealing rejected", tone: "bad" },
  // client_receive
  RELAY_DELIVERY_REJECTED: { gloss: "Relay delivery rejected", tone: "bad" },
  RELAY_DELIVERY_NOT_DELIVERED: { gloss: "Relay delivery pending", tone: "warn" },
  DELIVERY_ACK_REJECTED: { gloss: "Delivery ack rejected", tone: "bad" },
  DUPLICATE_DELIVERY_ACK: { gloss: "Duplicate delivery ack", tone: "warn" },
  SENDER_DEVICE_TRUST_REJECTED: { gloss: "Sender device trust rejected", tone: "bad" },
  REPLAY_DUPLICATE: { gloss: "Replay - duplicate dropped", tone: "warn" },
  REPLAY_STALE: { gloss: "Replay - stale dropped", tone: "warn" },
  BAD_CIPHERTEXT_DIGEST_LENGTH: { gloss: "Malformed ciphertext digest", tone: "bad" },
  PLAINTEXT_IDENTITY_FORBIDDEN: { gloss: "Plaintext identity forbidden", tone: "bad" },
  // client_bootstrap
  MISSING_ACCOUNT_ID: { gloss: "Missing account id", tone: "bad" },
  MISSING_DEVICE_ID: { gloss: "Missing device id", tone: "bad" },
  PLAINTEXT_CACHE_FORBIDDEN: { gloss: "Plaintext cache forbidden", tone: "bad" },
  LOCAL_DEVICE_TRUST_REJECTED: { gloss: "This device's trust rejected", tone: "bad" },
  KEY_TRANSPARENCY_NOT_READY: { gloss: "Key transparency not ready", tone: "warn" },
  KEY_TRANSPARENCY_REJECTED: { gloss: "Key transparency rejected", tone: "bad" },
  ACCOUNT_SECRET_MISSING: { gloss: "Account secret missing", tone: "bad" },
  ACCOUNT_SECRET_CORRUPT: { gloss: "Account secret corrupt", tone: "bad" },
  DEVICE_SECRET_MISSING: { gloss: "Device secret missing", tone: "bad" },
  DEVICE_SECRET_CORRUPT: { gloss: "Device secret corrupt", tone: "bad" },
  ROOM_STATE_MISSING: { gloss: "Room state missing", tone: "warn" },
  ROOM_STATE_CORRUPT: { gloss: "Room state corrupt", tone: "bad" },
  PLAINTEXT_SECRET_FORBIDDEN: { gloss: "Plaintext secret forbidden", tone: "bad" },
  REPLAY_CHECKPOINT_MISSING: { gloss: "Replay checkpoint missing", tone: "warn" },
  REPLAY_CHECKPOINT_STALE: { gloss: "Replay checkpoint stale", tone: "warn" },
  REPLAY_GAP_DETECTED: { gloss: "Replay gap detected", tone: "warn" },
  SYNC_GAP_DETECTED: { gloss: "Sync gap detected", tone: "warn" },
  SYNC_FAILED: { gloss: "Sync failed", tone: "bad" },
};

// Handoff glosses win on the labels both vocabularies share.
const LABEL_GLOSS: Record<string, { gloss: string; tone: Tone }> = { ...CORE_GLOSS, ...REASON_DICT };

function prettify(label: string): string {
  const s = label.toLowerCase().replace(/_/g, " ");
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function defaultTone(label: string): Tone {
  if (label === "ACCEPT" || label === "ACCEPTED") return "ok";
  if (/GAP|INCOMPLETE|NOT_READY|STALE|PENDING|ROTATION_REQUIRED|RETRY/.test(label)) {
    return "warn";
  }
  return "bad";
}

/** Gloss + tone for any reason_label (handoff or real-core), with a graceful
 *  prettified fallback for labels not yet curated. */
export function labelGloss(label: string): { gloss: string; tone: Tone } {
  return LABEL_GLOSS[label] ?? { gloss: prettify(label), tone: defaultTone(label) };
}
