// Multi-conversation controller — the REAL messenger model over `MercuryMessaging`.
//
// One identity (this app-server) talks to many peers. Each peer is a Conversation with its own
// authoritative history (`messaging.history(peer)`). `poll()` drains incoming for ALL peers and
// routes each by its `peer` field — an inbound message from an unknown peer auto-creates a
// conversation (you can read it; replying needs their contact card, which `addPeer` fetches).
//
// Framework-agnostic (unit-tested with plain vitest); `useLiveConversations.ts` is the thin React
// binding. Fail-closed: `start` failure is fatal; per-action failures surface a banner.

import {
  type ChatMessage,
  type DecisionViews,
  type GroupInfo,
  type MercuryMessaging,
  MessagingError,
} from "./messaging";
import { notifyIncoming } from "./notify";
import { parseInviteOrId } from "./invite";

export type LiveStatus = "connecting" | "ready" | "error";

export interface Conversation {
  /** Peer account id (lowercase hex). */
  peer: string;
  /** Authoritative message log with this peer (from the backend). */
  messages: ChatMessage[];
  /** True once we hold the peer's verified contact card (so we can open/send). */
  connected: boolean;
  /** Unread incoming count while this conversation is not active. */
  unread: number;
}

/** A real, timestamped record of something the client actually did — the live "binding" tail. */
export interface LiveEvent {
  t: number;
  kind: "ready" | "open" | "sent" | "received";
  detail: string;
}

export interface ConversationsState {
  accountId: string | null;
  status: LiveStatus;
  error: string | null;
  /** Which operation set `error`: "action" (a user send/addPeer) or "poll" (a background poll). A
   *  recovered background poll auto-clears ONLY its own "poll" error, never a user-action failure —
   *  otherwise a successful poll ~1.5s later would silently wipe a real send-failure banner. */
  errorSource?: "poll" | "action";
  conversations: Conversation[];
  activePeer: string | null;
  busy: boolean;
  /** Newest-first log of real operations (capped). */
  events: LiveEvent[];
  /** The three REAL mercury-core decision views for the ACTIVE peer (bootstrap / outbound /
   *  receive), or null when none have been fetched yet for the current peer. Cleared on every
   *  active-peer change and re-fetched, so it is never another peer's verdict. A fetch failure
   *  leaves it null (the UI then withholds send) — it is never replaced by a permissive default. */
  activeDecisions: DecisionViews | null;
  /** Lowercase-hex ids of MUTED conversations (notifications suppressed; delivery unaffected). */
  muted: string[];
  /** Lowercase-hex ids of BLOCKED contacts (sends refused + their incoming received then discarded client-side). */
  blocked: string[];
  /** Lowercase-hex ids of peers the user marked VERIFIED after a safety-number comparison. */
  verified: string[];
  /** Per-conversation disappearing-message TTL in seconds, keyed by peer hex (absent = off). */
  disappearing: Record<string, number>;
  /** Username aliases learned at add-by-username time (peer hex -> handle). Display-only. */
  aliases: Record<string, string>;
  /** This account's own claimed username, if any. Display-only. */
  myUsername: string | null;
  /** Group metadata, keyed by group id hex. A conversation whose peer is a key here is a GROUP. */
  groups: Record<string, GroupInfo>;
  /** Pinned conversation ids (persisted; the rail sorts pinned first). */
  pinned: string[];
  /** This account's read-receipt privacy toggle (delivery acks always flow). */
  readReceipts: boolean;
}

export interface LiveConversations {
  getState(): ConversationsState;
  subscribe(listener: () => void): () => void;
  start(): Promise<void>;
  /** Fetch + verify the peer's card, open the conversation, and make it active. */
  addPeer(peer: string): Promise<void>;
  setActive(peer: string): void;
  /** Send to the active conversation's peer. */
  send(text: string): Promise<void>;
  /** Drain newly-received messages and route them to conversations. */
  poll(): Promise<void>;
  /** Delete a conversation locally (delete-for-me) and drop it from the list. */
  deleteConversation(peer: string): Promise<void>;
  /** Delete a conversation on both sides (wipes the peer's copy too) and drop it from the list. */
  deleteForEveryone(peer: string): Promise<void>;
  /** Clear a conversation's messages, keeping the contact. */
  clearHistory(peer: string): Promise<void>;
  /** The mutual safety number with `peer` (grouped + raw digits), or null on failure. */
  safetyNumber(peer: string): Promise<{ digits: string; grouped: string } | null>;
  /** Mute a conversation (suppress its notifications). */
  mute(peer: string): Promise<void>;
  /** Unmute a conversation. */
  unmute(peer: string): Promise<void>;
  /** Block a contact (sends refused; their incoming dropped). */
  block(peer: string): Promise<void>;
  /** Unblock a contact. */
  unblock(peer: string): Promise<void>;
  /** Set (secs>0) or clear (secs=0) the disappearing-message timer for `peer`. */
  setDisappearing(peer: string, secs: number): Promise<void>;
  /** Mint a short-lived, single-use pairing code that brokers our card. Returns `{ code }`. */
  pairingCode(): Promise<{ code: string }>;
  /** Add + open a contact from a pairing code. Returns whose card it was (`account_id`); throws on
   *  an unknown/expired/used code. */
  addByPairing(code: string): Promise<{ account_id: string }>;
  /** Claim a username for this account (first-claimant-wins). Returns `{ status, username }`. */
  claimUsername(username: string): Promise<{ status: string; username: string }>;
  /** Add + open a contact by username (key-transparency-verified). Returns `{ account_id, tofu }`;
   *  throws on an unverifiable/unknown handle. */
  addByUsername(username: string): Promise<{ account_id: string; tofu: boolean }>;
  /** Mark/unmark a peer VERIFIED (the user compared safety numbers out-of-band). Persisted. */
  setVerified(peer: string, on: boolean): Promise<void>;
  /** Send a file (≤4 MiB; over 512 KiB is chunked) to the ACTIVE peer over the sealed channel. Throws on failure. */
  sendAttachment(name: string, mime: string, dataB64: string): Promise<void>;
  /** Export the whole account as a passphrase-sealed backup blob (base64). Throws on failure
   *  (incl. a passphrase under 9 chars). */
  backupExport(passphrase: string): Promise<string>;
  /** The user is LOOKING at this conversation (active + focused): lets the backend send a read
   *  watermark when the privacy toggle allows. Silent best-effort. */
  markRead(peer: string): Promise<void>;
  /** Set the read-receipt privacy toggle (persisted). */
  setReadReceipts(on: boolean): Promise<void>;
  /** Pin/unpin a conversation (persisted; the rail sorts pinned first). */
  setPinned(peer: string, on: boolean): Promise<void>;
  /** Create a group from existing contacts, open its conversation, make it active. Throws with a
   *  readable message on failure (non-contact member, too many members...). */
  createGroup(name: string, members: string[]): Promise<{ group: string }>;
  /** Leave (or, as creator, close) a group and drop its conversation. */
  leaveGroup(group: string): Promise<void>;
}

function freshState(): ConversationsState {
  return {
    accountId: null,
    status: "connecting",
    error: null,
    conversations: [],
    activePeer: null,
    busy: false,
    events: [],
    activeDecisions: null,
    muted: [],
    blocked: [],
    verified: [],
    disappearing: {},
    aliases: {},
    myUsername: null,
    groups: {},
    pinned: [],
    readReceipts: true,
  };
}

function shortHex(hex: string): string {
  return hex.length > 12 ? `${hex.slice(0, 6)}…${hex.slice(-4)}` : hex;
}

export function createLiveConversations(messaging: MercuryMessaging): LiveConversations {
  let state = freshState();
  // Reentrancy guard for poll(): true while a poll round is in flight, so a slow relay can't let
  // the 1.5s interval stack overlapping polls (M3).
  let polling = false;
  const listeners = new Set<() => void>();

  const emit = () => {
    for (const l of listeners) l();
  };
  const set = (patch: Partial<ConversationsState>) => {
    state = { ...state, ...patch };
    emit();
  };
  const errorText = (e: unknown): string =>
    e instanceof MessagingError ? e.message : `unexpected error: ${String(e)}`;

  const logEvent = (kind: LiveEvent["kind"], detail: string) => {
    set({ events: [{ t: Date.now(), kind, detail }, ...state.events].slice(0, 40) });
  };

  const findIndex = (peer: string) => state.conversations.findIndex((c) => c.peer === peer);

  // Replace one conversation by peer (immutably) and emit.
  const upsert = (peer: string, patch: Partial<Conversation>) => {
    const i = findIndex(peer);
    const next = [...state.conversations];
    if (i === -1) {
      next.push({ peer, messages: [], connected: false, unread: 0, ...patch });
    } else {
      next[i] = { ...next[i], ...patch };
    }
    set({ conversations: next });
  };

  const refresh = async (peer: string) => {
    const messages = await messaging.history(peer);
    upsert(peer, { messages });
  };

  // Fetch the REAL mercury-core decision views for `peer` and store them IF `peer` is still the
  // active conversation when the response lands (so a late reply never shows under another peer).
  // Fail-closed: a transport/parse failure does NOT surface the error banner (that is reserved for
  // user actions) and does NOT substitute a permissive view — it leaves `activeDecisions` as-is
  // (null until the first success), so the composer keeps send withheld rather than guessing ACCEPT.
  const refreshDecisions = async (peer: string) => {
    try {
      const views = await messaging.decisions(peer);
      if (state.activePeer === peer) set({ activeDecisions: views });
    } catch {
      /* decision fetch failed; keep the prior (same-peer) view or null — never invent an ACCEPT. */
    }
  };

  // Open a freshly-added contact (already verified by the backend): load its history, list it, make
  // it active. Shared by the pairing-code + username add paths (the tail of `addPeer`).
  const openContact = async (id: string) => {
    const messages = await messaging.history(id);
    upsert(id, { connected: true, messages, unread: 0 });
    set({ activePeer: id, activeDecisions: null });
    logEvent("open", `opened ${shortHex(id)}`);
    void refreshDecisions(id);
  };

  return {
    getState: () => state,

    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },

    async start() {
      set({ status: "connecting", error: null });
      try {
        const accountId = await messaging.accountId();
        await messaging.publishSelf();
        let flags = {
          muted: [] as string[],
          blocked: [] as string[],
          verified: [] as string[],
          disappearing: {} as Record<string, number>,
          aliases: {} as Record<string, string>,
          myUsername: null as string | null,
          groups: {} as Record<string, GroupInfo>,
          pinned: [] as string[],
          readReceipts: true,
        };
        try {
          flags = await messaging.conversationFlags();
        } catch {
          /* non-fatal: flags default to empty until the user re-toggles */
        }
        set({
          accountId,
          status: "ready",
          muted: flags.muted,
          blocked: flags.blocked,
          verified: flags.verified,
          disappearing: flags.disappearing,
          aliases: flags.aliases,
          myUsername: flags.myUsername,
          groups: flags.groups,
          pinned: flags.pinned,
          readReceipts: flags.readReceipts,
        });
        // Groups persist server-side state; make sure each has a visible conversation row even
        // before its first message of this session.
        for (const gid of Object.keys(flags.groups)) {
          if (findIndex(gid) === -1) {
            const messages = await messaging.history(gid);
            upsert(gid, { connected: true, messages, unread: 0 });
          }
        }
        logEvent("ready", "session ready · card published");
      } catch (e) {
        set({ status: "error", error: errorText(e) });
      }
    },

    async addPeer(peer) {
      // Accept a bare account id OR an invite link (mercury://add/<id> or a web /add/#<id> URL), so
      // people never retype 64 hex chars. The fetched card is still verified against the extracted
      // id, so a bad parse fails closed ("no such contact"), never a MITM.
      const id = parseInviteOrId(peer);
      if (!id) {
        set({ error: "Enter a valid account id or invite link.", errorSource: "action" });
        return;
      }
      if (id === state.accountId) {
        set({ error: "That's your own id — paste the other person's.", errorSource: "action" });
        return;
      }
      set({ busy: true, error: null });
      try {
        await messaging.addContact(id);
        const messages = await messaging.history(id);
        upsert(id, { connected: true, messages, unread: 0 });
        // Switch active peer and clear the prior peer's verdicts in the SAME update, so the
        // inspector never momentarily shows the old peer's decision under the new one.
        set({ activePeer: id, busy: false, activeDecisions: null });
        logEvent("open", `opened ${shortHex(id)}`);
        void refreshDecisions(id);
      } catch (e) {
        set({ busy: false, error: errorText(e), errorSource: "action" });
      }
    },

    setActive(peer) {
      const i = findIndex(peer);
      if (i === -1) return;
      const next = [...state.conversations];
      next[i] = { ...next[i], unread: 0 };
      // Clear the prior peer's verdicts atomically with the switch, then fetch this peer's.
      set({ conversations: next, activePeer: peer, error: null, activeDecisions: null });
      void refreshDecisions(peer);
    },

    async send(text) {
      const body = text.trim();
      const peer = state.activePeer;
      if (!body || !peer) return;
      set({ busy: true, error: null });
      try {
        // A conversation whose id is a known group routes through the MLS group path; everything
        // else is the 1:1 sealed ratchet. Same composer, same discipline.
        if (state.groups[peer]) {
          await messaging.groupSend(peer, body);
        } else {
          await messaging.send(peer, body);
        }
        await refresh(peer);
        set({ busy: false });
        logEvent("sent", `→ ${shortHex(peer)}`);
        void refreshDecisions(peer);
      } catch (e) {
        set({ busy: false, error: errorText(e), errorSource: "action" });
      }
    },

    async poll() {
      if (polling) return; // M3: don't stack polls while one is still in flight.
      polling = true;
      try {
        const { messages: incoming, updated } = await messaging.poll();
        // Conversations whose EXISTING messages changed (a delivery receipt landed, a pending
        // send was accepted, a group roster moved): refresh their history silently — no unread
        // bump, no notification.
        for (const peer of updated) {
          if (findIndex(peer) !== -1) await refresh(peer);
        }
        // Group activity (a roster change, a join, a new group appearing via a Welcome) also
        // changes the flags' group metadata — refetch it when anything group-shaped moved.
        const groupActivity =
          updated.some((p) => state.groups[p]) ||
          incoming.some((m) => state.groups[m.peer] || m.from != null || m.text.startsWith("\u{1f465}"));
        if (groupActivity) {
          try {
            const flags = await messaging.conversationFlags();
            set({ groups: flags.groups });
          } catch {
            /* keep the previous group view; the next poll retries */
          }
        }
        if (incoming.length > 0) {
          // Route by sender: refresh each affected conversation's authoritative history, creating a
          // conversation for a first-time sender, and bump unread when it isn't the active thread.
          const peers = [...new Set(incoming.map((m) => m.peer))];
          for (const peer of peers) {
            const messages = await messaging.history(peer);
            const i = findIndex(peer);
            const wasActive = state.activePeer === peer;
            const prevUnread = i === -1 ? 0 : state.conversations[i].unread;
            const arrived = incoming.filter((m) => m.peer === peer).length;
            upsert(peer, {
              messages,
              unread: wasActive ? 0 : prevUnread + arrived,
            });
            logEvent("received", `← ${shortHex(peer)}`);
          }
          // Background OS notification for arrivals you are NOT currently looking at: everything when
          // the window is unfocused/hidden, else only conversations OTHER than the active one — so
          // the chat you have open and focused never spam-notifies (L5). Desktop-only inside notify.
          const focused = typeof document !== "undefined" && document.hasFocus();
          const unseen = (focused ? incoming.filter((m) => m.peer !== state.activePeer) : incoming)
            .filter((m) => !state.muted.includes(m.peer));
          if (unseen.length > 0) {
            const unseenPeers = [...new Set(unseen.map((m) => m.peer))];
            void notifyIncoming(unseen.length, shortHex(unseenPeers[0]));
          }
          // A receive updates the controller's last-receive outcome, so the RECEIVE view for the
          // active peer may have changed — re-fetch it (peer-guarded, fail-closed).
          if (state.activePeer) void refreshDecisions(state.activePeer);
        }
        // The poll succeeded (zero or more messages): clear a stale error banner left by an EARLIER
        // failed poll so a recovered relay doesn't keep showing an error — but ONLY a poll-sourced
        // error, never a user-action (send/addPeer) failure the user still needs to see (M4).
        if (state.error && state.errorSource === "poll") set({ error: null, errorSource: undefined });
      } catch (e) {
        set({ error: errorText(e), errorSource: "poll" });
      } finally {
        polling = false;
      }
    },

    async deleteConversation(peer) {
      try {
        await messaging.deleteConversation(peer);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      const conversations = state.conversations.filter((c) => c.peer !== peer);
      const clearingActive = state.activePeer === peer;
      set({
        conversations,
        activePeer: clearingActive ? null : state.activePeer,
        activeDecisions: clearingActive ? null : state.activeDecisions,
      });
    },

    async deleteForEveryone(peer) {
      try {
        await messaging.deleteForEveryone(peer);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      const conversations = state.conversations.filter((c) => c.peer !== peer);
      const clearingActive = state.activePeer === peer;
      set({
        conversations,
        activePeer: clearingActive ? null : state.activePeer,
        activeDecisions: clearingActive ? null : state.activeDecisions,
      });
    },

    async clearHistory(peer) {
      try {
        await messaging.clearHistory(peer);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      upsert(peer, { messages: [] });
    },

    async safetyNumber(peer) {
      try {
        return await messaging.safetyNumber(peer);
      } catch {
        return null;
      }
    },

    async mute(peer) {
      try {
        await messaging.mute(peer);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      if (!state.muted.includes(peer)) set({ muted: [...state.muted, peer] });
    },

    async unmute(peer) {
      try {
        await messaging.unmute(peer);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      set({ muted: state.muted.filter((p) => p !== peer) });
    },

    async block(peer) {
      try {
        await messaging.block(peer);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      if (!state.blocked.includes(peer)) set({ blocked: [...state.blocked, peer] });
    },

    async unblock(peer) {
      try {
        await messaging.unblock(peer);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      set({ blocked: state.blocked.filter((p) => p !== peer) });
    },

    async setDisappearing(peer, secs) {
      try {
        await messaging.setDisappearing(peer, secs);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      const disappearing = { ...state.disappearing };
      if (secs > 0) disappearing[peer] = secs;
      else delete disappearing[peer];
      set({ disappearing });
    },

    async pairingCode() {
      // No state mutation — the panel displays the returned code. Errors propagate to the caller.
      return await messaging.pairingCode();
    },

    async addByPairing(code) {
      const trimmed = code.trim();
      if (!trimmed) throw new Error("Enter a pairing code.");
      const { account_id } = await messaging.addByPairing(trimmed);
      if (account_id === state.accountId) {
        throw new Error("That code is your own — share it with someone else.");
      }
      await openContact(account_id);
      return { account_id };
    },

    async claimUsername(username) {
      const result = await messaging.claimUsername(username);
      if (result.status === "created" || result.status === "already_owned") {
        set({ myUsername: result.username });
      }
      return result;
    },

    async addByUsername(username) {
      const trimmed = username.trim();
      if (!trimmed) throw new Error("Enter a username.");
      const { account_id, tofu } = await messaging.addByUsername(trimmed);
      if (account_id === state.accountId) {
        throw new Error("That's your own username.");
      }
      // Remember the handle for display ("@bob" beats 64 hex in the rail).
      set({ aliases: { ...state.aliases, [account_id]: trimmed.toLowerCase() } });
      await openContact(account_id);
      return { account_id, tofu };
    },

    async setVerified(peer, on) {
      try {
        if (on) await messaging.setVerified(peer);
        else await messaging.unsetVerified(peer);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      const verified = on
        ? [...new Set([...state.verified, peer])]
        : state.verified.filter((p) => p !== peer);
      set({ verified });
    },

    async sendAttachment(name, mime, dataB64) {
      const peer = state.activePeer;
      if (!peer) throw new Error("Open a conversation first.");
      set({ busy: true, error: null });
      try {
        await messaging.sendAttachment(peer, name, mime, dataB64);
        await refresh(peer);
        set({ busy: false });
        logEvent("sent", `📎 → ${shortHex(peer)}`);
      } catch (e) {
        set({ busy: false, error: errorText(e), errorSource: "action" });
        throw e;
      }
    },

    async backupExport(passphrase) {
      const { data_b64 } = await messaging.backupExport(passphrase);
      return data_b64;
    },

    async markRead(peer) {
      try {
        await messaging.markRead(peer);
      } catch {
        /* cosmetic — never surface an error for a receipt */
      }
    },

    async setReadReceipts(on) {
      try {
        await messaging.setReadReceipts(on);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      set({ readReceipts: on });
    },

    async setPinned(peer, on) {
      try {
        await messaging.setPinned(peer, on);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      const pinned = on
        ? [...new Set([...state.pinned, peer])]
        : state.pinned.filter((p) => p !== peer);
      set({ pinned });
    },

    async createGroup(name, members) {
      const { group } = await messaging.createGroup(name, members);
      // Pick up the new group's metadata + system line, then open it.
      try {
        const flags = await messaging.conversationFlags();
        set({ groups: flags.groups });
      } catch {
        /* metadata refresh failed — the next poll will catch up */
      }
      const messages = await messaging.history(group);
      upsert(group, { connected: true, messages, unread: 0 });
      set({ activePeer: group, activeDecisions: null });
      logEvent("open", `group ${shortHex(group)} created`);
      void refreshDecisions(group);
      return { group };
    },

    async leaveGroup(group) {
      try {
        await messaging.leaveGroup(group);
      } catch (e) {
        set({ error: errorText(e), errorSource: "action" });
        return;
      }
      const groups = { ...state.groups };
      delete groups[group];
      const conversations = state.conversations.filter((c) => c.peer !== group);
      const clearingActive = state.activePeer === group;
      set({
        groups,
        conversations,
        activePeer: clearingActive ? null : state.activePeer,
        activeDecisions: clearingActive ? null : state.activeDecisions,
      });
    },
  };
}
