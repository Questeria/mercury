// The mercury-core decision boundary, mirrored in TypeScript.
//
// The UI RENDERS these decisions; it never computes or second-guesses them
// (integration rules 1-3). `PlatformDecisionView` is the single shape every
// gate returns. In production these come from the real mercury-core binding
// (see `binding.ts`); in dev they come from the in-app simulator
// (`simulator.ts`) -- both behind the same shape so they swap cleanly.

export type DecisionSource =
  | "client_bootstrap"
  | "outbound_send"
  | "client_receive"
  | "policy_pipeline"
  | "client_policy";

export type ReasonLabel =
  | "ACCEPTED"
  | "ORDERING_GAP"
  | "SYNC_INCOMPLETE"
  | "RECOVERY_REQUIRED"
  | "RECIPIENT_TRUST_REJECTED"
  | "TOFU_PENDING"
  | "KEY_STALE"
  | "AI_GRANT_ABSENT"
  | "AI_GRANT_REVOKED"
  | "AI_GRANT_EXPIRED"
  | "SENDER_TRUST_REJECTED"
  | "REPLAY_DETECTED";

/** The 13-field platform decision view. The UI branches on `reason_label`
 *  and the boolean capability flags -- never on the wire-level `reason_code`. */
export interface PlatformDecisionView {
  source: DecisionSource;
  accepted: boolean;
  reason_code: number;
  /**
   * Canonical reason string. The simulator emits the `ReasonLabel` set above;
   * the real mercury-core emits a richer vocabulary (e.g. MESSAGE_POLICY_REJECTED,
   * AI_GRANT_REJECT, SENDER_DEVICE_TRUST_REJECTED). The `(string & {})` keeps the
   * known literals as autocomplete hints while accepting any core label. The UI
   * branches on capability flags + this label via `labelGloss()` (never on
   * `reason_code`).
   */
  reason_label: ReasonLabel | (string & {});
  can_open_message_ui: boolean;
  can_start_sync: boolean;
  can_send: boolean;
  can_receive: boolean;
  can_persist_ciphertext: boolean;
  requires_sync: boolean;
  requires_recovery: boolean;
  requires_client_retry: boolean;
  requires_user_action: boolean;
}

export type Tone = "ok" | "warn" | "bad";

/** Which detail panel (modal/sheet) is open. `profile:<who>` opens a profile. */
export type PanelId =
  | "trust"
  | "ai"
  | "peers"
  | "encryption"
  | "bootstrap"
  | "recovery"
  | "sync"
  | "notifications"
  | "updates"
  | "attachments"
  | "group"
  | "policy"
  | "onboarding"
  | "settings"
  | `profile:${string}`;

export interface Person {
  id: string;
  name: string;
  short: string;
  hue: number;
  isAi?: boolean;
  /** Optional profile-picture data URL (device-local). When set, the avatar shows
   *  the image instead of the hue-tinted initials. */
  avatar?: string;
  /** Optional exact custom avatar colour (hex). When set, the avatar fills with this
   *  precise colour (auto-contrast initials) instead of the hue-derived soft tint. */
  color?: string;
}

export type MessageKind =
  | "system"
  | "incoming"
  | "outgoing"
  | "gap"
  | "rejected_inbound"
  | "send_blocked";

/** A rendered message. Plaintext lives here only as ephemeral render state
 *  (integration rule 4: never persist plaintext durably). */
export interface Message {
  id: number;
  kind: MessageKind;
  author?: string;
  text: string;
  ts: number;
  status?: "sending" | "sent" | "delivered" | "read";
  /** AI messages stream char-by-char on first render; set for seeded/historical
   *  AI messages so they appear complete instead of re-typing. */
  streamed?: boolean;
  /** The decision that admitted/blocked this message (subset carried for display). */
  decision?: Partial<PlatformDecisionView> & { reason_label: PlatformDecisionView["reason_label"] };
  /** Small-file attachment riding the sealed channel (live app), if any. */
  att?: { name: string; mime: string; data_b64: string };
  /** GROUP messages: the sender's lowercase-hex account id, for attribution (live app). */
  fromId?: string;
}

// ---- simulation knobs (dev only; production derives these from mercury-core) ----
export type BootstrapState = "accepted" | "sync_incomplete" | "recovery_required";
export type TrustState =
  | "trusted"
  | "sendable_not_full"
  | "unverified"
  | "rejected"
  | "stale";
export type AiState = "granted" | "absent" | "revoked" | "expired";
export type ReceiveMode = "accepted" | "ordering_gap";
export type SenderTrust = "trusted" | "rejected";
export type ForceSendOutcome = "auto" | "accepted" | "tofu" | "rejected" | "stale";

export interface MercuryThreadInputs {
  bootstrap: BootstrapState;
  trust: TrustState;
  ai: AiState;
  receiveMode: ReceiveMode;
  senderTrust: SenderTrust;
  forceSendOutcome?: ForceSendOutcome;
}

export interface MercuryThreadState {
  messages: Message[];
  draft: string;
  setDraft: (v: string) => void;
  bootstrapView: PlatformDecisionView;
  outboundView: PlatformDecisionView;
  receiveView: PlatformDecisionView;
  pendingTofu: { text: string; decision: PlatformDecisionView } | null;
  acceptTofu: () => void;
  cancelTofu: () => void;
  retryGap: () => void;
  retryingGap: boolean;
  send: (overrideText?: string) => void;
  draftMentionsAi: boolean;
}
