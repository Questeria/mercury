// React binding over the multi-conversation controller. Exposes the conversation list (for the
// rail), the active conversation mapped into the UI `Message` shape + a `MercuryThreadState` the
// designed <Composer> consumes, and the controller actions.
//
// The live backend enforces the REAL mercury-core policy server-side AND now reports its genuine
// PlatformDecisionViews (bootstrap / outbound / receive) for the active peer, which the controller
// fetches into `state.activeDecisions`. The composer's send gate reads the REAL outbound view (so a
// backend REJECT withholds send); a fail-closed sentinel covers only the pre-fetch window. The UI
// never holds keys or ciphertext.

import { useEffect, useMemo, useState, useSyncExternalStore } from "react";

import {
  createLiveConversations,
  type ConversationsState,
  type LiveConversations,
} from "./liveConversations";
import { previewText } from "./conversationPreview";
import { inviteActionId } from "./invite";
import { inTauri } from "./messaging";
import type { ChatMessage, MercuryMessaging } from "./messaging";
import { MERCURY_PEOPLE } from "./simulator";
import type { Message, MercuryThreadState, PlatformDecisionView, Person } from "./types";

const POLL_MS = 1500;

// Captured at module load (NOT during render — keeps the hook pure). Anchors the synthesized,
// ordered cosmetic timestamps for a backend that carries no per-message wall-clock time.
const SESSION_START = Date.now();

// Fail-closed placeholder shown ONLY before the first real decision view for a peer has arrived
// (and if a fetch fails). It is deliberately NON-permissive — can_send=false withholds the composer
// rather than guessing ACCEPT, so the send gate can never be open without a genuine backend verdict
// behind it. reason_code -1 marks "no backend code" (real mercury-core codes are unsigned), so this
// can never be mistaken for a real rc=0 ACCEPTED. Once `state.activeDecisions` is populated, the
// REAL mercury-core views (built by mercury-app from its actual state) replace this entirely.
const DECISIONS_PENDING: PlatformDecisionView = {
  source: "outbound_send",
  accepted: false,
  reason_code: -1,
  reason_label: "DECISION_PENDING",
  can_open_message_ui: false,
  can_start_sync: false,
  can_send: false,
  can_receive: false,
  can_persist_ciphertext: false,
  requires_sync: false,
  requires_recovery: false,
  requires_client_retry: true,
  requires_user_action: false,
};

export interface PeerView {
  peer: string;
  person: Person;
  unread: number;
  connected: boolean;
  lastText: string;
  count: number;
  /** Pinned by the user (persisted backend preference; the rail sorts pinned first). */
  pinned: boolean;
}

export interface LiveConversationsView {
  state: ConversationsState;
  controller: LiveConversations;
  peers: PeerView[];
  active: { peer: string; person: Person } | null;
  messages: Message[];
  composer: MercuryThreadState;
  mePerson: Person;
}

function hueFromHex(hex: string): number {
  return Math.round((Number.parseInt(hex.slice(0, 2) || "0", 16) / 255) * 360);
}

function personFor(peer: string, alias?: string, groupName?: string): Person {
  return {
    id: peer,
    // A group renders by its name; a username alias (learned at verified add-by-username time)
    // beats 64 hex for humans; the id stays the cryptographic identity either way.
    name: groupName ?? (alias ? `@${alias}` : `${peer.slice(0, 6)}…${peer.slice(-4)}`),
    short: (groupName ?? alias ?? peer).slice(0, 2).toUpperCase(),
    hue: hueFromHex(peer),
  };
}

function toMessage(c: ChatMessage): Message {
  return {
    id: c.seq,
    kind: c.direction === "outgoing" ? "outgoing" : "incoming",
    author: c.direction === "outgoing" ? "me" : "peer",
    text: c.text,
    // Real wall-clock time from the backend; legacy messages (ts 0/absent) fall back to the old
    // synthesized session-relative ordering so they never render as 1970.
    ts: c.ts && c.ts > 0 ? c.ts * 1000 : SESSION_START + c.seq * 1000,
    // HONEST status ladder: "sending" = queued in the outbox (peer's mailbox full / relay briefly
    // unreachable); "sent" = the relay accepted it; "delivered" = the peer's DEVICE confirmed
    // receipt via an in-band sealed receipt; "read" = the peer's device reported the conversation
    // open + focused (only when THEIR read-receipt toggle allows). delivered ≠ read, never inferred.
    status:
      c.direction === "outgoing"
        ? c.pending
          ? "sending"
          : c.read
            ? "read"
            : c.delivered
              ? "delivered"
              : "sent"
        : undefined,
    att: c.att ?? undefined,
    fromId: c.from ?? undefined,
    decision: { reason_label: "ACCEPTED" },
  };
}

export function useLiveConversations(messaging: MercuryMessaging): LiveConversationsView {
  const [controller] = useState<LiveConversations>(() => createLiveConversations(messaging));
  const state = useSyncExternalStore(controller.subscribe, controller.getState);
  const [draft, setDraft] = useState("");

  useEffect(() => {
    void controller.start();

    if (inTauri()) {
      // Desktop: REAL-TIME delivery. Drain anything pending, then long-poll the relay until it
      // wakes us (a message arrived) or its ~25s timeout — far lower-latency and far cheaper than
      // fixed polling, and it runs in the foreground AND minimized-to-tray. A cadence floor keeps
      // the loop sane if the wait ever returns instantly (e.g. transiently no long-poll support),
      // so it can never hot-loop.
      let stopped = false;
      const FLOOR_MS = 1000;
      const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
      const run = async () => {
        while (!stopped) {
          const started = Date.now();
          await controller.poll();
          if (stopped) break;
          await messaging.waitForDelivery(); // never throws; resolves false on timeout/none
          const elapsed = Date.now() - started;
          if (!stopped && elapsed < FLOOR_MS) await sleep(FLOOR_MS - elapsed);
        }
      };
      void run();
      return () => {
        stopped = true;
      };
    }

    // Browser dev: the shim has no long-poll endpoint — poll on an interval, skipping while the tab
    // is backgrounded (saves work/battery), and poll once when it becomes visible again.
    const tick = () => {
      if (!document.hidden) void controller.poll();
    };
    const handle = window.setInterval(tick, POLL_MS);
    const onVisible = () => {
      if (!document.hidden) void controller.poll();
    };
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      window.clearInterval(handle);
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, [controller, messaging]);

  // Click-to-add: when a `mercury://add/<id>` invite is opened — at cold start, or while Mercury is
  // already running (forwarded by the single-instance plugin) — add that contact. Desktop-only and
  // lazy-imported so the browser build carries no Tauri dependency. `controller.addPeer` parses the
  // link and fails closed on a malformed one.
  useEffect(() => {
    if (!inTauri()) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void (async () => {
      try {
        const dl = await import("@tauri-apps/plugin-deep-link");
        // Only an explicit `mercury://add/<id>` invite adds a contact (inviteActionId requires the
        // `add` action), so a future `mercury://<other-verb>/…` is never mis-handled as an add.
        const handleUrl = (url: string) => {
          const id = inviteActionId(url);
          if (id) void controller.addPeer(id);
        };
        const initial = await dl.getCurrent();
        if (!cancelled && initial) for (const url of initial) handleUrl(url);
        unlisten = await dl.onOpenUrl((urls) => {
          for (const url of urls) handleUrl(url);
        });
      } catch {
        /* deep-link plugin unavailable — pasting an id/link still works */
      }
    })();
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }, [controller]);

  const peers = useMemo<PeerView[]>(() => {
    const views = state.conversations.map((c) => ({
      peer: c.peer,
      person: personFor(c.peer, state.aliases[c.peer], state.groups[c.peer]?.name),
      unread: c.unread,
      connected: c.connected,
      lastText: c.messages.length ? previewText(c.messages[c.messages.length - 1]) : "",
      count: c.messages.length,
      pinned: state.pinned.includes(c.peer),
    }));
    // Pinned conversations first; otherwise keep arrival order (stable sort).
    return views.sort((a, b) => Number(b.pinned) - Number(a.pinned));
  }, [state.conversations, state.aliases, state.groups, state.pinned]);

  const activeConv = useMemo(
    () => state.conversations.find((c) => c.peer === state.activePeer) ?? null,
    [state.conversations, state.activePeer],
  );

  const active = useMemo(
    () =>
      activeConv
        ? {
            peer: activeConv.peer,
            person: personFor(
              activeConv.peer,
              state.aliases[activeConv.peer],
              state.groups[activeConv.peer]?.name,
            ),
          }
        : null,
    [activeConv, state.aliases, state.groups],
  );

  const messages = useMemo(
    () => (activeConv ? activeConv.messages.map(toMessage) : []),
    [activeConv],
  );

  // The REAL mercury-core decision views for the active peer (fetched by the controller). Until the
  // first fetch lands — or if it fails — fall back to the fail-closed DECISIONS_PENDING sentinel so
  // the composer's send gate is never open without a genuine backend verdict.
  const decisions = state.activeDecisions;

  const composer = useMemo<MercuryThreadState>(
    () => ({
      messages,
      draft,
      setDraft,
      bootstrapView: decisions?.bootstrap ?? DECISIONS_PENDING,
      outboundView: decisions?.outbound ?? DECISIONS_PENDING,
      receiveView: decisions?.receive ?? DECISIONS_PENDING,
      pendingTofu: null,
      acceptTofu: () => {},
      cancelTofu: () => {},
      retryGap: () => {},
      retryingGap: false,
      send: (override?: string) => {
        const text = (override ?? draft).trim();
        if (!text) return;
        setDraft("");
        void controller.send(text);
      },
      draftMentionsAi: false,
    }),
    [messages, draft, controller, decisions],
  );

  return { state, controller, peers, active, messages, composer, mePerson: MERCURY_PEOPLE.me };
}
