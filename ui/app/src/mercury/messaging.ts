// The mercury MESSAGING boundary (distinct from the decision boundary in `binding.ts`).
//
// `binding.ts` carries POLICY DECISIONS (PlatformDecisionView); this module carries the
// actual end-to-end-encrypted MESSAGES. A `MercuryMessaging` drives a real
// `mercury-app::AppController` over the `mercury-app-server` dev HTTP shim's `POST /command`
// surface — the UI never sees keys or ciphertext, only plaintext it sent/received plus the
// peer account ids. Every method is fail-closed: a transport failure, a non-2xx status, or an
// in-band `{ok:false}` envelope throws `MessagingError` rather than silently degrading.
//
// Wire-shape parity: `ChatMessage` and every command/response below mirror the Rust types in
// `core/rust/mercury-app` (handle_command) exactly, so the two sides stay in lockstep.
//
// ── INTEGRATION (the remaining, USER-VISIBLE step) ──────────────────────────────────────────
// This module is the tested binding; it is NOT yet wired into the chat view. To go live:
//   1. Run the relay:        cargo run -p mercury-relay --bin mercury-relay-server
//   2. Run the app shim:     cargo run -p mercury-app-server         (loopback, DEV-ONLY)
//   3. In `useMercuryThread.ts`, replace the `setTimeout`/`setInterval` simulator fakes with
//      `createHttpMessaging(<shim-url>)`: `send()` -> `messaging.send(peer, text)`, and a poll
//      loop -> `messaging.poll()` appended to the rendered `Message[]`.
// Because this changes the live UI data flow, the two-window end-to-end check (two browser
// tabs actually chatting) must be verified VISUALLY by a human — it is not asserted here.

import type { PlatformDecisionView } from "./types";

/** Direction of a message relative to this client. Mirrors Rust `Direction`. */
export type MessageDirection = "outgoing" | "incoming";

/** An attachment carried inside a message (small files over the sealed channel). */
export interface ChatAttachment {
  name: string;
  mime: string;
  data_b64: string;
}

/** One message in the conversation log. Mirrors Rust `ChatMessage`. */
export interface ChatMessage {
  direction: MessageDirection;
  /** Lowercase-hex account id of the OTHER party (recipient if outgoing, sender if incoming). */
  peer: string;
  /** The message body as text (incoming non-UTF-8 bytes are decoded lossily by the backend). */
  text: string;
  /** Local monotonic ordering key (NOT the ratchet index). */
  seq: number;
  /** Wall-clock creation time (Unix seconds); 0 for messages logged before this field existed. */
  ts?: number;
  /** Outgoing only: the peer's device confirmed receipt (in-band sealed receipt). */
  delivered?: boolean;
  /** Outgoing only: queued in the outbox (peer's mailbox full / relay briefly unreachable). */
  pending?: boolean;
  /** Outgoing 1:1 only: the peer's device reported the message READ (their toggle permitting). */
  read?: boolean;
  /** The attachment carried by this message, if any. */
  att?: ChatAttachment | null;
  /** GROUP messages: the lowercase-hex id of the actual sender (`peer` holds the group id).
   *  Absent/null for 1:1 messages and group system notices. */
  from?: string | null;
}

/** One group's metadata as the backend reports it. */
export interface GroupInfo {
  name: string;
  creator: string;
  members: string[];
  pending: string[];
  ended: boolean;
}

/** The three REAL mercury-core decision views the controller exposes for the active peer.
 *  Mirrors Rust `mercury_app::DecisionViews` — each field is a genuine `PlatformDecisionView`
 *  built by mercury-core's constructors from the controller's ACTUAL state (account id present,
 *  real `can_send(peer)`, last receive outcome). The UI renders these verbatim; it computes no
 *  policy of its own. */
export interface DecisionViews {
  bootstrap: PlatformDecisionView;
  outbound: PlatformDecisionView;
  receive: PlatformDecisionView;
}

/** The messaging surface the UI depends on. All methods reach the real backend; all are
 *  fail-closed (they reject rather than return a degraded/fake result). */
export interface MercuryMessaging {
  /** This account's id (lowercase hex). */
  accountId(): Promise<string>;
  /** Publish our contact card to the directory so peers can discover us. */
  publishSelf(): Promise<void>;
  /** Fetch + verify a peer's contact card from the directory and remember it. */
  addContact(accountId: string): Promise<void>;
  /** Whether we can already reach `peer` (a session or a known contact card). */
  canSend(peer: string): Promise<boolean>;
  /** Send `text` to `peer` (opens the session on the first message). */
  send(peer: string, text: string): Promise<void>;
  /** Poll for newly-received messages, decrypted. `updated` lists conversations whose EXISTING
   *  messages changed (a delivery receipt landed / a pending send was accepted) — refresh those
   *  histories without counting anything as a new arrival. */
  poll(): Promise<{ messages: ChatMessage[]; updated: string[] }>;
  /** The full message history with `peer`, in local order. */
  history(peer: string): Promise<ChatMessage[]>;
  /** The three REAL mercury-core decision views (bootstrap / outbound to `peer` / last receive),
   *  built by the controller from its actual state. Fail-closed: a malformed or partial response
   *  throws `MessagingError` rather than yielding a degraded view. */
  decisions(peer: string): Promise<DecisionViews>;
  /** Long-poll the relay for a pending delivery (real-time receive). Resolves `true` when a
   *  delivery is likely pending (the caller should then `poll()`), or `false` on timeout / no
   *  long-poll support (the caller falls back to interval polling). Never throws — a transient
   *  failure resolves `false` so the delivery loop degrades to polling instead of stalling. */
  waitForDelivery(): Promise<boolean>;
  /** Delete a conversation locally (delete-for-me): removes its messages and the stored contact. */
  deleteConversation(peer: string): Promise<void>;
  /** Delete a conversation on BOTH sides: sends the peer a sealed control message that wipes the
   *  shared history on their device, then removes it locally. */
  deleteForEveryone(peer: string): Promise<void>;
  /** Clear a conversation's message history while keeping the contact. */
  clearHistory(peer: string): Promise<void>;
  /** The mutual identity safety number with `peer` for out-of-band MITM verification
   *  (`digits` = 60 decimal chars; `grouped` = 12 space-separated groups of 5). */
  safetyNumber(peer: string): Promise<{ digits: string; grouped: string }>;
  /** Mute a conversation (the UI suppresses its notifications). */
  mute(peer: string): Promise<void>;
  /** Unmute a conversation. */
  unmute(peer: string): Promise<void>;
  /** Block a contact (sends refused, their incoming dropped). */
  block(peer: string): Promise<void>;
  /** Unblock a contact. */
  unblock(peer: string): Promise<void>;
  /** Per-conversation flags: muted/blocked/verified id lists, disappearing TTLs, username
   *  aliases (peer hex -> handle), this account's own claimed handle, and group metadata
   *  (group id hex -> info). */
  conversationFlags(): Promise<{
    muted: string[];
    blocked: string[];
    verified: string[];
    disappearing: Record<string, number>;
    aliases: Record<string, string>;
    myUsername: string | null;
    groups: Record<string, GroupInfo>;
    pinned: string[];
    readReceipts: boolean;
  }>;
  /** Tell the backend the user is LOOKING at this 1:1 conversation (sends a read watermark if
   *  the privacy toggle allows). Cheap + idempotent. */
  markRead(peer: string): Promise<void>;
  /** Set the read-receipt privacy toggle (delivery acks always flow). */
  setReadReceipts(on: boolean): Promise<void>;
  /** Pin/unpin a conversation (persisted; the rail sorts pinned first). */
  setPinned(peer: string, on: boolean): Promise<void>;
  /** Create a group and invite existing contacts (≤15 others). Returns the new group id. */
  createGroup(name: string, members: string[]): Promise<{ group: string }>;
  /** Send `text` to every member of the group (one MLS ciphertext, fanned out sealed). */
  groupSend(group: string, text: string): Promise<void>;
  /** Leave a group (as creator: closes it for everyone — v1 is creator-administered). */
  leaveGroup(group: string): Promise<void>;
  /** Set (secs>0) or clear (secs=0) the disappearing-message timer for `peer` (seconds). */
  setDisappearing(peer: string, secs: number): Promise<void>;
  /** Mint a short-lived, single-use PAIRING CODE that brokers our contact card, so a peer can add
   *  us by typing a friendly code instead of a 64-hex account id. */
  pairingCode(): Promise<{ code: string }>;
  /** Add a contact from a PAIRING CODE: fetch + verify the brokered card. Returns whose card it was
   *  (`account_id`) so the UI can confirm. Fail-closed on an unknown/expired/used code. */
  addByPairing(code: string): Promise<{ account_id: string }>;
  /** Claim a USERNAME (key-transparency-backed handle) for this account, first-claimant-wins.
   *  `status` is one of `created | already_owned | taken | rejected | unavailable`. */
  claimUsername(username: string): Promise<{ status: string; username: string }>;
  /** Add a contact by USERNAME: resolve the handle, VERIFY its transparency inclusion proof against
   *  the pinned directory key, then fetch + verify the card. `tofu` is true when this resolve
   *  trusted the relay-served key on first use. Fail-closed on an unverifiable binding. */
  addByUsername(username: string): Promise<{ account_id: string; tofu: boolean }>;
  /** Mark a peer VERIFIED (the user compared safety numbers out-of-band). Local + persisted. */
  setVerified(peer: string): Promise<void>;
  /** Clear the VERIFIED mark. */
  unsetVerified(peer: string): Promise<void>;
  /** Send a file (≤4 MiB raw; over 512 KiB is chunked into ordered sealed frames) over the SAME
   *  sealed channel as text. Returns whether it was queued (`pending`) behind a full mailbox. */
  sendAttachment(
    peer: string,
    name: string,
    mime: string,
    dataB64: string,
  ): Promise<{ pending: boolean }>;
  /** Export the whole account (identity, sessions, contacts, history) as a passphrase-sealed
   *  backup blob (base64). The file's security IS the passphrase; ≥9 chars enforced. */
  backupExport(passphrase: string): Promise<{ data_b64: string }>;
}

/** A messaging-layer failure: transport unreachable, bad HTTP status, or an in-band command
 *  error. Carries a human-readable, key/plaintext-free message. */
export class MessagingError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "MessagingError";
  }
}

/** The `{ ok, result?, error? }` envelope every command returns (mirrors Rust `CommandResponse`). */
interface CommandEnvelope {
  ok: boolean;
  result?: unknown;
  error?: string;
}

/** POST one JSON command to the shim and unwrap its envelope, throwing `MessagingError` on any
 *  failure (network, non-2xx, or `ok:false`). */
async function runCommand(baseUrl: string, command: Record<string, unknown>): Promise<unknown> {
  let res: Response;
  try {
    res = await fetch(`${baseUrl}/command`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(command),
    });
  } catch (e) {
    throw new MessagingError(`cannot reach mercury-app-server: ${String(e)}`);
  }
  if (!res.ok) {
    throw new MessagingError(`mercury-app-server returned HTTP ${res.status}`);
  }
  let envelope: CommandEnvelope;
  try {
    envelope = (await res.json()) as CommandEnvelope;
  } catch {
    throw new MessagingError("mercury-app-server returned a non-JSON response");
  }
  if (!envelope.ok) {
    throw new MessagingError(envelope.error ?? "command failed");
  }
  return envelope.result;
}

/** The 10 boolean capability/requirement fields of a `PlatformDecisionView`. */
const DECISION_BOOLS = [
  "accepted",
  "can_open_message_ui",
  "can_start_sync",
  "can_send",
  "can_receive",
  "can_persist_ciphertext",
  "requires_sync",
  "requires_recovery",
  "requires_client_retry",
  "requires_user_action",
] as const;

/** Fail-closed parse of ONE `PlatformDecisionView`: all 13 fields must be present with the right
 *  primitive type, or we throw. A malicious or buggy app-server therefore cannot hand the UI a
 *  half-built view that would render as a capability it never granted (e.g. a missing `can_send`
 *  defaulting to truthy). */
function asDecisionView(v: unknown, field: string): PlatformDecisionView {
  if (!v || typeof v !== "object") {
    throw new MessagingError(`decisions.${field}: not an object`);
  }
  const o = v as Record<string, unknown>;
  for (const k of DECISION_BOOLS) {
    if (typeof o[k] !== "boolean") {
      throw new MessagingError(`decisions.${field}.${k}: not a boolean`);
    }
  }
  if (typeof o.source !== "string") {
    throw new MessagingError(`decisions.${field}.source: not a string`);
  }
  if (typeof o.reason_label !== "string") {
    throw new MessagingError(`decisions.${field}.reason_label: not a string`);
  }
  if (typeof o.reason_code !== "number" || !Number.isFinite(o.reason_code)) {
    throw new MessagingError(`decisions.${field}.reason_code: not a finite number`);
  }
  return o as unknown as PlatformDecisionView;
}

/** Fail-closed parse of the `{bootstrap, outbound, receive}` envelope from the `decisions` command. */
function asDecisionViews(r: unknown): DecisionViews {
  if (!r || typeof r !== "object") {
    throw new MessagingError("decisions: not an object");
  }
  const o = r as Record<string, unknown>;
  return {
    bootstrap: asDecisionView(o.bootstrap, "bootstrap"),
    outbound: asDecisionView(o.outbound, "outbound"),
    receive: asDecisionView(o.receive, "receive"),
  };
}

/** Build the `MercuryMessaging` surface from a generic command runner (HTTP fetch in the browser
 *  dev path, Tauri `invoke` in the desktop app). The command/response shapes are transport-agnostic
 *  and mirror Rust `AppController::handle_command` exactly, so both transports stay in lockstep. */
function buildMessaging(
  run: (command: Record<string, unknown>) => Promise<unknown>,
  waitForDelivery: () => Promise<boolean>,
): MercuryMessaging {
  return {
    waitForDelivery,
    async accountId() {
      const r = (await run({ cmd: "account_id" })) as { account_id: string };
      return r.account_id;
    },
    async publishSelf() {
      await run({ cmd: "publish_self" });
    },
    async addContact(accountId: string) {
      await run({ cmd: "add_contact", account_id: accountId });
    },
    async canSend(peer: string) {
      const r = (await run({ cmd: "can_send", peer })) as { can_send: boolean };
      return r.can_send;
    },
    async send(peer: string, text: string) {
      await run({ cmd: "send", peer, text });
    },
    async poll() {
      const r = (await run({ cmd: "poll" })) as {
        messages: ChatMessage[];
        updated?: string[];
      };
      // `updated` is additive (older backends omit it) — default to none.
      return { messages: r.messages, updated: r.updated ?? [] };
    },
    async history(peer: string) {
      const r = (await run({ cmd: "history", peer })) as { messages: ChatMessage[] };
      return r.messages;
    },
    async decisions(peer: string) {
      return asDecisionViews(await run({ cmd: "decisions", peer }));
    },
    async deleteConversation(peer: string) {
      await run({ cmd: "delete_conversation", peer });
    },
    async deleteForEveryone(peer: string) {
      await run({ cmd: "delete_for_everyone", peer });
    },
    async clearHistory(peer: string) {
      await run({ cmd: "clear_history", peer });
    },
    async safetyNumber(peer: string) {
      const r = (await run({ cmd: "safety_number", peer })) as { digits: string; grouped: string };
      return { digits: r.digits, grouped: r.grouped };
    },
    async mute(peer: string) {
      await run({ cmd: "mute", peer });
    },
    async unmute(peer: string) {
      await run({ cmd: "unmute", peer });
    },
    async block(peer: string) {
      await run({ cmd: "block", peer });
    },
    async unblock(peer: string) {
      await run({ cmd: "unblock", peer });
    },
    async conversationFlags() {
      const r = (await run({ cmd: "conversation_flags" })) as {
        muted?: string[];
        blocked?: string[];
        verified?: string[];
        disappearing?: Record<string, number>;
        aliases?: Record<string, string>;
        my_username?: string | null;
        groups?: Record<string, GroupInfo>;
        pinned?: string[];
        read_receipts?: boolean;
      };
      return {
        muted: r.muted ?? [],
        blocked: r.blocked ?? [],
        verified: r.verified ?? [],
        disappearing: r.disappearing ?? {},
        aliases: r.aliases ?? {},
        myUsername: r.my_username ?? null,
        groups: r.groups ?? {},
        pinned: r.pinned ?? [],
        readReceipts: r.read_receipts ?? true,
      };
    },
    async markRead(peer: string) {
      await run({ cmd: "mark_read", peer });
    },
    async setReadReceipts(on: boolean) {
      await run({ cmd: "set_read_receipts", on });
    },
    async setPinned(peer: string, on: boolean) {
      await run({ cmd: "set_pinned", peer, on });
    },
    async createGroup(name: string, members: string[]) {
      const r = (await run({ cmd: "create_group", name, members })) as { group: string };
      return { group: r.group };
    },
    async groupSend(group: string, text: string) {
      await run({ cmd: "group_send", group, text });
    },
    async leaveGroup(group: string) {
      await run({ cmd: "leave_group", group });
    },
    async setDisappearing(peer: string, secs: number) {
      await run({ cmd: "set_disappearing", peer, secs });
    },
    async pairingCode() {
      const r = (await run({ cmd: "pairing_code" })) as { code: string };
      return { code: r.code };
    },
    async addByPairing(code: string) {
      const r = (await run({ cmd: "add_by_pairing", code })) as { account_id: string };
      return { account_id: r.account_id };
    },
    async claimUsername(username: string) {
      const r = (await run({ cmd: "claim_username", username })) as {
        status: string;
        username: string;
      };
      return { status: r.status, username: r.username };
    },
    async addByUsername(username: string) {
      const r = (await run({ cmd: "add_by_username", username })) as {
        account_id: string;
        tofu: boolean;
      };
      return { account_id: r.account_id, tofu: r.tofu };
    },
    async setVerified(peer: string) {
      await run({ cmd: "set_verified", peer });
    },
    async unsetVerified(peer: string) {
      await run({ cmd: "unset_verified", peer });
    },
    async sendAttachment(peer: string, name: string, mime: string, dataB64: string) {
      const r = (await run({
        cmd: "send_attachment",
        peer,
        name,
        mime,
        data_b64: dataB64,
      })) as { pending: boolean };
      return { pending: !!r.pending };
    },
    async backupExport(passphrase: string) {
      const r = (await run({ cmd: "backup_export", passphrase })) as { data_b64: string };
      return { data_b64: r.data_b64 };
    },
  };
}

/**
 * Real messaging binding over the `mercury-app-server` shim at `baseUrl`
 * (e.g. `http://127.0.0.1:7879`). The shim is DEV-ONLY (loopback, holds keys); the production
 * exposure is the Tauri in-process IPC binding below ([`createTauriMessaging`]).
 */
export function createHttpMessaging(baseUrl: string): MercuryMessaging {
  const base = baseUrl.replace(/\/+$/, "");
  // The dev shim has no long-poll endpoint, so the browser path uses interval polling instead;
  // waitForDelivery resolves false immediately (it is not used by the browser delivery loop).
  return buildMessaging((command) => runCommand(base, command), async () => false);
}

/** True when running inside the Tauri desktop app (the in-process IPC bridge is available). */
export function inTauri(): boolean {
  return (
    typeof window !== "undefined" &&
    ("__TAURI_INTERNALS__" in window || "__TAURI__" in window)
  );
}

/** Run one JSON command over Tauri's in-process IPC and unwrap the SAME `{ok,result|error}` envelope
 *  the HTTP shim returns. The `@tauri-apps/api` module is imported lazily so the plain-browser build
 *  (which never calls this) carries no hard dependency on the Tauri runtime being present. */
async function runTauriCommand(command: Record<string, unknown>): Promise<unknown> {
  let out: string;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    out = await invoke<string>("command", { body: JSON.stringify(command) });
  } catch (e) {
    throw new MessagingError(`tauri command failed: ${String(e)}`);
  }
  let envelope: CommandEnvelope;
  try {
    envelope = JSON.parse(out) as CommandEnvelope;
  } catch {
    throw new MessagingError("tauri command returned a non-JSON response");
  }
  if (!envelope.ok) {
    throw new MessagingError(envelope.error ?? "command failed");
  }
  return envelope.result;
}

/**
 * Real messaging binding INSIDE the Tauri desktop app: the SAME `AppController` command surface,
 * driven over Tauri's in-process IPC (`invoke`) rather than an HTTP socket — no key or ciphertext
 * ever leaves the native process, and there is no open port. Identical envelope semantics to
 * [`createHttpMessaging`], so the UI above is unchanged.
 */
/** Long-poll the relay for a pending delivery via the dedicated `wait_for_delivery` Tauri command
 *  (NOT the `command` bridge — it runs outside the controller lock so it can block ~25s). Never
 *  throws: any error resolves `false`, so the delivery loop falls back to polling. */
async function runTauriWait(): Promise<boolean> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<boolean>("wait_for_delivery");
  } catch {
    return false;
  }
}

export function createTauriMessaging(): MercuryMessaging {
  return buildMessaging(runTauriCommand, runTauriWait);
}
