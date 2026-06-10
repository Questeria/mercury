// In-app PlatformDecisionView simulator (dev default).
//
// Faithful TypeScript port of the design-handoff `mercury-core.jsx` decision
// builders + scripted content. Production swaps these for the real
// mercury-core binding (see `binding.ts`) -- behind the SAME PlatformDecisionView
// shape, so callers never change.

import type {
  AiState,
  BootstrapState,
  ForceSendOutcome,
  Message,
  Person,
  PlatformDecisionView,
  ReasonLabel,
  ReceiveMode,
  SenderTrust,
  Tone,
  TrustState,
} from "./types";

export const MERCURY_PEOPLE: Record<string, Person> = {
  me: { id: "me", name: "You", short: "YOU", hue: 220 },
  rin: { id: "rin", name: "Rin Hadley", short: "RH", hue: 16 },
  jules: { id: "jules", name: "Jules Okafor", short: "JO", hue: 152 },
  ai: { id: "ai", name: "Mercury AI", short: "AI", hue: 268, isAi: true },
};

export const REASON_DICT: Record<ReasonLabel, { gloss: string; tone: Tone }> = {
  ACCEPTED: { gloss: "Accepted", tone: "ok" },
  ORDERING_GAP: { gloss: "Ordering gap — fetching missing", tone: "warn" },
  SYNC_INCOMPLETE: { gloss: "Sync incomplete", tone: "warn" },
  RECOVERY_REQUIRED: { gloss: "Recovery required", tone: "bad" },
  RECIPIENT_TRUST_REJECTED: { gloss: "Recipient trust rejected", tone: "bad" },
  TOFU_PENDING: { gloss: "First-use verification needed", tone: "warn" },
  KEY_STALE: { gloss: "Recipient key stale", tone: "bad" },
  AI_GRANT_ABSENT: { gloss: "No AI grant in this room", tone: "bad" },
  AI_GRANT_REVOKED: { gloss: "AI grant revoked", tone: "bad" },
  AI_GRANT_EXPIRED: { gloss: "AI grant expired", tone: "bad" },
  SENDER_TRUST_REJECTED: { gloss: "Sender trust rejected", tone: "bad" },
  REPLAY_DETECTED: { gloss: "Replay detected — dropped", tone: "bad" },
};

/** Wire-level integers. The UI must NOT branch on these (rule 2) -- surfaced
 *  only as diagnostic flavor. */
export const REASON_CODE: Record<ReasonLabel, number> = {
  ACCEPTED: 0,
  ORDERING_GAP: 10,
  REPLAY_DETECTED: 11,
  SENDER_TRUST_REJECTED: 12,
  RECIPIENT_TRUST_REJECTED: 20,
  TOFU_PENDING: 21,
  KEY_STALE: 22,
  AI_GRANT_ABSENT: 30,
  AI_GRANT_REVOKED: 31,
  AI_GRANT_EXPIRED: 32,
  SYNC_INCOMPLETE: 18,
  RECOVERY_REQUIRED: 19,
};

function base(
  source: PlatformDecisionView["source"],
  accepted: boolean,
  reason_label: ReasonLabel,
  over: Partial<PlatformDecisionView>,
): PlatformDecisionView {
  return {
    source,
    accepted,
    reason_label,
    reason_code: REASON_CODE[reason_label] ?? 0,
    can_open_message_ui: false,
    can_start_sync: false,
    can_send: false,
    can_receive: false,
    can_persist_ciphertext: false,
    requires_sync: false,
    requires_recovery: false,
    requires_client_retry: false,
    requires_user_action: false,
    ...over,
  };
}

export function decisionBootstrap(bootstrap: BootstrapState): PlatformDecisionView {
  if (bootstrap === "accepted")
    return base("client_bootstrap", true, "ACCEPTED", {
      can_open_message_ui: true,
      can_start_sync: true,
    });
  if (bootstrap === "sync_incomplete")
    return base("client_bootstrap", false, "SYNC_INCOMPLETE", {
      can_start_sync: true,
      requires_sync: true,
    });
  return base("client_bootstrap", false, "RECOVERY_REQUIRED", {
    requires_recovery: true,
  });
}

export function decisionOutbound(args: {
  trust: TrustState;
  ai: AiState;
  draftMentionsAi: boolean;
}): PlatformDecisionView {
  const { trust, ai, draftMentionsAi } = args;
  if (trust === "rejected")
    return base("outbound_send", false, "RECIPIENT_TRUST_REJECTED", {});
  if (trust === "stale")
    return base("outbound_send", false, "KEY_STALE", { requires_user_action: true });
  if (draftMentionsAi) {
    if (ai === "absent") return base("outbound_send", false, "AI_GRANT_ABSENT", {});
    if (ai === "revoked") return base("outbound_send", false, "AI_GRANT_REVOKED", {});
    if (ai === "expired") return base("outbound_send", false, "AI_GRANT_EXPIRED", {});
  }
  if (trust === "unverified")
    return base("outbound_send", true, "TOFU_PENDING", {
      can_send: true,
      can_persist_ciphertext: true,
      requires_user_action: true,
    });
  return base("outbound_send", true, "ACCEPTED", {
    can_send: true,
    can_persist_ciphertext: true,
  });
}

export function decisionReceive(args: {
  receiveMode: ReceiveMode;
  senderTrust: SenderTrust;
}): PlatformDecisionView {
  const { receiveMode, senderTrust } = args;
  if (senderTrust === "rejected")
    return base("client_receive", false, "SENDER_TRUST_REJECTED", {
      requires_user_action: true,
    });
  if (receiveMode === "ordering_gap")
    return base("client_receive", false, "ORDERING_GAP", {
      requires_client_retry: true,
    });
  return base("client_receive", true, "ACCEPTED", {
    can_receive: true,
    can_open_message_ui: true,
  });
}

/** Demo override: force a specific outbound outcome regardless of trust state. */
export function outboundForce(outcome: Exclude<ForceSendOutcome, "auto">): PlatformDecisionView {
  switch (outcome) {
    case "accepted":
      return base("outbound_send", true, "ACCEPTED", {
        can_send: true,
        can_persist_ciphertext: true,
      });
    case "tofu":
      return base("outbound_send", true, "TOFU_PENDING", {
        can_send: true,
        can_persist_ciphertext: true,
        requires_user_action: true,
      });
    case "rejected":
      return base("outbound_send", false, "RECIPIENT_TRUST_REJECTED", {});
    case "stale":
      return base("outbound_send", false, "KEY_STALE", { requires_user_action: true });
  }
}

export const now = (): number => Date.now();

export function seedThread(): Message[] {
  const t = Date.now();
  const day = 86_400_000;
  return [
    {
      id: 0,
      kind: "system",
      text: "Room opened · 4 participants · end-to-end encrypted",
      ts: t - day - 8 * 3600_000,
    },
    {
      id: 1,
      kind: "incoming",
      author: "rin",
      text: "Spec for the receive gate is up. Ordering check is moving into core next sprint.",
      ts: t - day - 6 * 3600_000,
      decision: { reason_label: "ACCEPTED" },
    },
    {
      id: 2,
      kind: "incoming",
      author: "jules",
      text: "Will the platform view still surface requires_client_retry separately, or will you fold it into requires_sync?",
      ts: t - day - 5.8 * 3600_000,
      decision: { reason_label: "ACCEPTED" },
    },
    {
      id: 3,
      kind: "incoming",
      author: "rin",
      text: "Pushed the receive-gate rewrite. Ordering check is now in core, not the binding.",
      ts: t - 6800_000,
      decision: { reason_label: "ACCEPTED" },
    },
    {
      id: 4,
      kind: "incoming",
      author: "rin",
      text: "Separate. Retry is per-message; sync is per-room. UI needs both.",
      ts: t - 6650_000,
      decision: { reason_label: "ACCEPTED" },
    },
    {
      id: 5,
      kind: "outgoing",
      author: "me",
      text: "Good. I’ll wire the thread shell against from_client_receive and use the reason_label, not the codes.",
      ts: t - 6200_000,
      status: "delivered",
      decision: { reason_label: "ACCEPTED" },
    },
    {
      id: 6,
      kind: "incoming",
      author: "jules",
      text: "Per the report — never re-run policy from raw fields. Read the view and surface it.",
      ts: t - 5900_000,
      decision: { reason_label: "ACCEPTED" },
    },
    {
      id: 7,
      kind: "incoming",
      author: "ai",
      text: "Summary (scoped to this grant): the ordering check moved into core; the thread renders from_client_receive; the UI surfaces requires_client_retry rather than inferring it.",
      ts: t - 5600_000,
      streamed: true,
      decision: { reason_label: "ACCEPTED" },
    },
  ];
}

const INBOUND_SCRIPT: { author: string; text: string }[] = [
  {
    author: "rin",
    text: "Quick check — what did the binding return for that last send? Was can_persist_ciphertext flipped?",
  },
  {
    author: "jules",
    text: 'I’m drafting the trust prompt copy. Going with "First-use verification needed" for TOFU.',
  },
  {
    author: "rin",
    text: "Pushed a fixture for ordering_gap with reason_code 10. Test it against your retry path.",
  },
  {
    author: "jules",
    text: "AI grant lifecycle is in. Revoke flows through policy_pipeline now, not the binding.",
  },
  {
    author: "rin",
    text: "Reminder: never store plaintext durably. The thread is a render of the binding’s view.",
  },
];

let inboundIdx = 0;
export function nextInbound(): { author: string; text: string } {
  const m = INBOUND_SCRIPT[inboundIdx % INBOUND_SCRIPT.length];
  inboundIdx++;
  return m;
}

export function aiReplyFor(prompt: string): string {
  const p = prompt.toLowerCase();
  if (p.includes("summarize") || p.includes("summary")) {
    return "Summary (scoped to this grant): Rin moved the ordering check into core; the binding now reads from_client_receive; UI must surface requires_client_retry, not infer it.";
  }
  if (p.includes("reason") || p.includes("code")) {
    return "reason_code is the wire-level integer; reason_label is the canonical string. Per integration rules, UI should display reason_label.";
  }
  return "Acknowledged. I have read-only access to this room’s decrypted content for the duration of this grant; nothing is persisted past expiry.";
}
