// The LIVE Mercury app — the polished design wired to the real backend. A full multi-conversation
// messenger: rail (conversation list + add-peer + identity + search), the active thread (status
// data-strip, day-divided message bubbles, composer), and a Session panel.
//
// HONESTY: the design's data-strip/Inspector render mercury-core DECISION views; the live backend
// does not expose per-message decisions to the UI, so those areas show only real session FACTS
// (account ids, crypto posture, message flow). The UI never holds keys or ciphertext.

import { Fragment, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type { CSSProperties, FormEvent } from "react";

import { Avatar } from "./components/Avatar";
import { Composer } from "./components/Composer";
import { ConversationMenu } from "./components/ConversationMenu";
import { NewGroupModal } from "./components/NewGroupModal";
import { ConnectionPanel } from "./components/panels/ConnectionPanel";
import { getConnectionPrefs, type ConnectionPrefs } from "./connectionPrefs";
import { MercuryLogo } from "./components/MercuryLogo";
import { QrCode } from "./components/QrCode";
import { TitleBar } from "./components/TitleBar";
import { UpdateBanner } from "./components/UpdateBanner";
import { AiPanel } from "./components/panels/AiPanel";
import { AttachmentsPanel } from "./components/panels/AttachmentsPanel";
import { GroupSafetyPanel } from "./components/panels/GroupSafetyPanel";
import { NotificationsPanel } from "./components/panels/NotificationsPanel";
import { OnboardingPanel } from "./components/panels/OnboardingPanel";
import { PolicyPanel } from "./components/panels/PolicyPanel";
import { RecoveryPanel } from "./components/panels/RecoveryPanel";
import { TrustPanel } from "./components/panels/TrustPanel";
import { UpdatesPanel } from "./components/panels/UpdatesPanel";
import { EmbeddedContext, PreviewContext } from "./components/panels/panelContext";
import { downloadViaBlob, saveToDownloads } from "./b64file";
import { fmtDay, fmtMsgTime, sameDay } from "./format";
import { inviteLink } from "./invite";
import { createHttpMessaging, createTauriMessaging, inTauri } from "./messaging";
import { explainDecision } from "./strings";
import { useLiveConversations, type PeerView } from "./useLiveConversations";
import type { LiveEvent } from "./liveConversations";
import type { AiState, Message as Msg, PanelId, PlatformDecisionView, Person, TrustState } from "./types";
import msg from "./components/Message.module.css";
import styles from "./LiveMercuryApp.module.css";
import "./theme.css";

function short(id: string): string {
  return id.length > 16 ? `${id.slice(0, 6)}…${id.slice(-4)}` : id;
}

// Display Person for a GROUP message sender (attributed by `m.fromId`). Mirrors the conversation
// layer's `personFor` naming + hue derivation (which is not exported): alias/@handle beats 64 hex,
// hue comes from the id's first byte — so a member keeps one consistent colour everywhere.
function groupSender(id: string, aliases: Record<string, string>): Person {
  const alias = aliases[id];
  return {
    id,
    name: alias ? `@${alias}` : short(id),
    short: (alias ?? id).slice(0, 2).toUpperCase(),
    hue: Math.round((Number.parseInt(id.slice(0, 2) || "0", 16) / 255) * 360),
  };
}

// Search-hit wash behind a message whose text matches the rail query: a subtle, STATIC background
// tint applied inline (no animation), cleared the moment the query empties.
const SEARCH_HIT_BG = "color-mix(in srgb, var(--mc-warn) 13%, transparent)";
const SEARCH_HIT: CSSProperties = { background: SEARCH_HIT_BG, borderRadius: 14 };

// 👥 system notices in a group thread (member joined / left / group closed): a centered, muted
// single line — no bubble, no avatar — matching the day-divider's quiet aesthetic.
function GroupNotice({ text, highlight }: { text: string; highlight?: boolean }) {
  return (
    <div
      className="mono"
      style={{
        alignSelf: "center",
        maxWidth: "92%",
        whiteSpace: "nowrap",
        overflow: "hidden",
        textOverflow: "ellipsis",
        textAlign: "center",
        fontSize: 10.5,
        color: "var(--mc-muted)",
        padding: "1px 8px",
        borderRadius: 8,
        background: highlight ? SEARCH_HIT_BG : undefined,
      }}
    >
      {text}
    </div>
  );
}

const hueStyle = (hue: number, maxWidth: string): CSSProperties =>
  ({ maxWidth, ["--hue" as string]: String(hue) }) as CSSProperties;

const MOBILE_BP = 760;

type LegacyModalState =
  | { kind: "settings" }
  | { kind: "encryption" }
  | { kind: "profile"; person: Person; id: string; self: boolean };

type ModalState = null | LegacyModalState | { kind: LiveFeatureKind };

type LiveFeatureKind =
  | "trust"
  | "ai"
  | "updates"
  | "attachments"
  | "notifications"
  | "recovery"
  | "group"
  | "policy"
  | "onboarding"
  | "connection";

const LIVE_FEATURE_KINDS = new Set<string>([
  "trust",
  "ai",
  "updates",
  "attachments",
  "notifications",
  "recovery",
  "group",
  "policy",
  "onboarding",
  "connection",
]);

function isLiveFeatureKind(kind: string): kind is LiveFeatureKind {
  return LIVE_FEATURE_KINDS.has(kind);
}

export type ThemeChoice = "light" | "dark" | "auto";
const THEME_KEY = "mercury.theme";

function readThemeChoice(): ThemeChoice {
  const v = typeof window !== "undefined" ? window.localStorage.getItem(THEME_KEY) : null;
  return v === "light" || v === "dark" ? v : "auto";
}

// Local, device-only profile (display name / status / avatar colour). Shown in your own app;
// peer-visible profiles are future protocol work.
export type Profile = { name: string; hue: number; status: string; avatar?: string; color?: string };
const PROFILE_KEY = "mercury.profile";
const PROFILE_HUES = [265, 222, 192, 158, 128, 92, 38, 12, 322];

function readProfile(): Profile {
  try {
    const raw = typeof window !== "undefined" ? window.localStorage.getItem(PROFILE_KEY) : null;
    if (raw) {
      const p = JSON.parse(raw) as Partial<Profile>;
      return {
        name: typeof p.name === "string" ? p.name : "",
        hue: typeof p.hue === "number" ? p.hue : 265,
        status: typeof p.status === "string" ? p.status : "",
        avatar: typeof p.avatar === "string" ? p.avatar : undefined,
        color: typeof p.color === "string" ? p.color : undefined,
      };
    }
  } catch {
    /* ignore malformed storage */
  }
  return { name: "", hue: 265, status: "" };
}

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "You";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

// hsl(h, 68%, 55%) <-> hex, matching how the avatar preset dots render — so the custom
// colour picker round-trips against the same hue the avatar tint uses.
function hueToHex(h: number): string {
  const s = 0.68;
  const l = 0.55;
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  const seg = Math.floor((((h % 360) + 360) % 360) / 60) % 6;
  const rgb = [
    [c, x, 0],
    [x, c, 0],
    [0, c, x],
    [0, x, c],
    [x, 0, c],
    [c, 0, x],
  ][seg];
  const to = (v: number) =>
    Math.round((v + m) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${to(rgb[0])}${to(rgb[1])}${to(rgb[2])}`;
}

// Downscale a chosen image to a small square (cover) and return a compact JPEG data URL
// suitable for localStorage. Resolves to null on failure.
async function imageToAvatarDataUrl(file: File): Promise<string | null> {
  try {
    const bitmap = await createImageBitmap(file);
    const SIZE = 128;
    const canvas = document.createElement("canvas");
    canvas.width = SIZE;
    canvas.height = SIZE;
    const ctx = canvas.getContext("2d");
    if (!ctx) return null;
    const scale = Math.max(SIZE / bitmap.width, SIZE / bitmap.height);
    const w = bitmap.width * scale;
    const h = bitmap.height * scale;
    ctx.drawImage(bitmap, (SIZE - w) / 2, (SIZE - h) / 2, w, h);
    bitmap.close?.();
    return canvas.toDataURL("image/jpeg", 0.85);
  } catch {
    return null;
  }
}

/** Subscribe to the OS color-scheme preference (external-store read, no effect setState). */
function useSystemDark(): boolean {
  return useSyncExternalStore(
    (cb) => {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      mq.addEventListener("change", cb);
      return () => mq.removeEventListener("change", cb);
    },
    () => window.matchMedia("(prefers-color-scheme: dark)").matches,
    () => true,
  );
}

// Viewport width. Starts at a desktop default and corrects in an effect (so render stays pure —
// no `window` read during render).
function useViewport(): number {
  const [w, setW] = useState(1280);
  useEffect(() => {
    const onResize = () => setW(window.innerWidth);
    onResize();
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);
  return w;
}

export function LiveMercuryApp({ backendUrl }: { backendUrl: string }) {
  // Desktop (Tauri): drive the in-process AppController over IPC — no socket, keys stay native.
  // Browser dev: drive the loopback HTTP shim at `backendUrl`.
  const messaging = useMemo(
    () => (inTauri() ? createTauriMessaging() : createHttpMessaging(backendUrl)),
    [backendUrl],
  );
  const { state, controller, peers, active, messages, composer, mePerson } =
    useLiveConversations(messaging);
  const [peerInput, setPeerInput] = useState("");
  // Connection-method prefs (Settings → Connections), kept live so the rail add-box reflects toggles.
  const [connPrefs, setConnPrefs] = useState(getConnectionPrefs);
  const [addHint, setAddHint] = useState<string | null>(null);
  useEffect(() => {
    const onPrefs = (e: Event) =>
      setConnPrefs((e as CustomEvent<ConnectionPrefs>).detail ?? getConnectionPrefs());
    window.addEventListener("mercury-connprefs", onPrefs);
    return () => window.removeEventListener("mercury-connprefs", onPrefs);
  }, []);
  const [query, setQuery] = useState("");
  const [railCollapsed, setRailCollapsed] = useState(false);
  const [railView, setRailView] = useState<"messages" | "contacts">("messages");
  const [panelOpen, setPanelOpen] = useState(true);
  const [copied, setCopied] = useState<"me" | "peer" | null>(null);
  const mobile = useViewport() < MOBILE_BP;
  const [mobileView, setMobileView] = useState<"list" | "thread">("list");
  const [mobileInspector, setMobileInspector] = useState(false);
  const [modal, setModal] = useState<ModalState>(null);
  const [convMenu, setConvMenu] = useState(false);
  const [moreMenu, setMoreMenu] = useState(false);
  const [newGroupOpen, setNewGroupOpen] = useState(false);
  const [trustState, setTrustState] = useState<TrustState>("trusted");
  const [aiState, setAiState] = useState<AiState>("absent");
  const [theme, setThemeState] = useState<ThemeChoice>(readThemeChoice);
  const systemDark = useSystemDark();
  const dark = theme === "dark" || (theme === "auto" && systemDark);
  const setTheme = (t: ThemeChoice) => {
    setThemeState(t);
    try {
      window.localStorage.setItem(THEME_KEY, t);
    } catch {
      /* storage unavailable; ignore */
    }
  };
  const openFeaturePanel = (panel: PanelId | LiveFeatureKind) => {
    if (panel === "settings") {
      setModal({ kind: "settings" });
      return;
    }
    if (panel === "encryption") {
      setModal({ kind: "encryption" });
      return;
    }
    if (typeof panel === "string" && isLiveFeatureKind(panel)) {
      setModal({ kind: panel });
    }
  };

  const [profile, setProfileState] = useState<Profile>(readProfile);
  const setProfile = (p: Profile) => {
    setProfileState(p);
    try {
      window.localStorage.setItem(PROFILE_KEY, JSON.stringify(p));
    } catch {
      /* storage unavailable; ignore */
    }
  };
  // "You" as a Person, customised by the local profile (falls back to the derived identity).
  const me: Person = {
    ...mePerson,
    name: profile.name.trim() || mePerson.name,
    short: profile.name.trim() ? initials(profile.name) : mePerson.short,
    hue: profile.hue,
    avatar: profile.avatar,
    color: profile.color,
  };

  const selectConv = (peer: string) => {
    controller.setActive(peer);
    if (mobile) setMobileView("thread");
  };

  const addPeer = (e: FormEvent) => {
    e.preventDefault();
    const v = peerInput.trim();
    if (!v) return;
    const handle = v.replace(/^@+/, "");
    // An invite link / account id is long, hex, or URL-shaped; anything short is a pairing code.
    const looksLikeId = v.length >= 32 || /[:/#]/.test(v) || /^[0-9a-f]{16,}$/i.test(v);
    const commit = (run: () => void) => {
      run();
      setPeerInput("");
      setAddHint(null);
      if (mobile) setMobileView("thread");
    };
    // Route only to a method ENABLED in Settings → Connections; otherwise say where to turn it on.
    if (v.startsWith("@")) {
      if (connPrefs.username) return commit(() => void controller.addByUsername(handle));
      return setAddHint("Usernames are off — turn them on in Settings → Connections.");
    }
    if (looksLikeId) {
      if (connPrefs.link) return commit(() => void controller.addPeer(v));
      return setAddHint("Invite link / ID is off — turn it on in Settings → Connections.");
    }
    if (connPrefs.pairing) return commit(() => void controller.addByPairing(v));
    if (connPrefs.username) return commit(() => void controller.addByUsername(handle));
    if (connPrefs.link) return commit(() => void controller.addPeer(v));
    setAddHint("No connect method is enabled — turn one on in Settings → Connections.");
  };

  const copy = async (which: "me" | "peer", value: string | null) => {
    if (!value) return;
    try {
      await navigator.clipboard.writeText(value);
      setCopied(which);
      window.setTimeout(() => setCopied(null), 1200);
    } catch {
      /* clipboard unavailable; ignore */
    }
  };

  const q = query.trim().toLowerCase();
  // Search matches a conversation when the person/group NAME or ANY of its messages' text contains
  // the query (case-insensitive) — one cheap pass over the in-memory histories, memoized.
  const shownPeers = useMemo(() => {
    if (!q) return peers;
    return peers.filter(
      (p) =>
        p.person.name.toLowerCase().includes(q) ||
        (state.conversations.find((c) => c.peer === p.peer)?.messages ?? []).some((m) =>
          m.text.toLowerCase().includes(q),
        ),
    );
  }, [q, peers, state.conversations]);
  // Contacts view: 1:1 people only (no groups), filtered by the same search box, sorted A->Z.
  const contactPeers = useMemo(() => {
    const list = peers.filter((p) => !state.groups[p.peer]);
    const filtered = q ? list.filter((p) => p.person.name.toLowerCase().includes(q)) : list;
    return [...filtered].sort((a, b) => a.person.name.localeCompare(b.person.name));
  }, [peers, state.groups, q]);
  // Ids of the ACTIVE thread's messages whose text contains the query: those bubbles get a subtle
  // static tint and the thread header shows the count. Empty set when the query is empty.
  const activeMatchIds = useMemo(() => {
    const ids = new Set<number>();
    if (q) for (const m of messages) if (m.text.toLowerCase().includes(q)) ids.add(m.id);
    return ids;
  }, [q, messages]);

  const activeConv = active
    ? state.conversations.find((c) => c.peer === active.peer) ?? null
    : null;
  const inCount = activeConv ? activeConv.messages.filter((m) => m.direction === "incoming").length : 0;
  const outCount = activeConv ? activeConv.messages.length - inCount : 0;
  // A conversation whose id is a key in state.groups IS a group — its REAL backend metadata.
  const activeGroup = active ? state.groups[active.peer] ?? null : null;

  // READ watermark (1:1 only): when the active conversation changes, and again whenever the window
  // regains focus while it stays open, tell the backend the user is LOOKING at it. markRead is
  // silent best-effort and the backend only emits a receipt when the privacy toggle allows; groups
  // never get one (the backend has no group read receipts). Never fires with no active peer.
  const activePeerId = active?.peer ?? null;
  const activeIsGroup = activeGroup != null;
  useEffect(() => {
    if (!activePeerId || activeIsGroup) return;
    void controller.markRead(activePeerId);
    const onFocus = () => void controller.markRead(activePeerId);
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [activePeerId, activeIsGroup, controller]);

  const tauri = inTauri();
  return (
    <div className="mercury" data-theme={dark ? "dark" : "light"}>
      {tauri && <TitleBar />}
      <UpdateBanner />
      <div
        className={styles.shell}
        data-mobile={mobile ? "" : undefined}
        data-view={mobileView}
        data-inspector={(mobile ? mobileInspector : panelOpen) ? "" : undefined}
        style={tauri ? { top: 36 } : undefined}
      >
        <div className="shimmer-bg" />

        {/* ---------- floating top-right quick controls ---------- */}
        <div className={styles.quickBar} role="toolbar" aria-label="Quick controls">
          <button
            type="button"
            className={styles.quickBtn}
            onClick={() => setModal({ kind: "settings" })}
            data-tip="Settings"
            aria-label="Settings"
          >
            ⚙
          </button>
          <button
            type="button"
            className={styles.quickBtn}
            onClick={() => setTheme(dark ? "light" : "dark")}
            data-tip={dark ? "Light mode" : "Dark mode"}
            aria-label={dark ? "Switch to light mode" : "Switch to dark mode"}
          >
            {dark ? "☀" : "☾"}
          </button>
          <button
            type="button"
            className={styles.quickBtn}
            data-active={moreMenu ? "" : undefined}
            onClick={() => setMoreMenu((o) => !o)}
            data-tip="More"
            aria-label="More options"
            aria-haspopup="menu"
            aria-expanded={moreMenu}
          >
            ⋯
          </button>
        </div>

        {/* top-right overflow menu: inspector collapse + conversation settings */}
        {moreMenu && (
          <>
            <div className={styles.moreBackdrop} onClick={() => setMoreMenu(false)} />
            <div className={`${styles.moreMenu} card-in`} role="menu" aria-label="More options">
              <button
                type="button"
                role="menuitemcheckbox"
                aria-checked={mobile ? mobileInspector : panelOpen}
                className={styles.moreItem}
                onClick={() => {
                  if (mobile) setMobileInspector((o) => !o);
                  else setPanelOpen((o) => !o);
                  setMoreMenu(false);
                }}
              >
                <span className={styles.moreIcon}>ⓘ</span>
                {(mobile ? mobileInspector : panelOpen) ? "Hide inspector" : "Show inspector"}
              </button>
              {active && (
                <button
                  type="button"
                  role="menuitem"
                  className={styles.moreItem}
                  onClick={() => {
                    setConvMenu(true);
                    setMoreMenu(false);
                  }}
                >
                  <span className={styles.moreIcon}>⚙</span>
                  Conversation settings
                </button>
              )}
            </div>
          </>
        )}

        {/* ---------- rail ---------- */}
        <aside className={styles.rail} data-collapsed={railCollapsed ? "" : undefined}>
          <div className={styles.railHead}>
            <span className={styles.logo} aria-hidden>
              <MercuryLogo size={46} />
            </span>
            <span className={`${styles.wordmark} iris-text`}>Mercury</span>
            <span className={`${styles.version} mono`}>v0.1.38</span>
            <button
              className={styles.railToggle}
              type="button"
              onClick={() => setRailCollapsed((v) => !v)}
              aria-label={railCollapsed ? "Expand conversation list" : "Collapse conversation list"}
              aria-expanded={!railCollapsed}
              data-tip={railCollapsed ? "Expand" : "Collapse"}
            >
              {railCollapsed ? "›" : "‹"}
            </button>
          </div>

          <label className={styles.search}>
            <span className={styles.searchIcon}>⌕</span>
            <input
              className={styles.searchInput}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={railView === "messages" ? "Search messages" : "Search contacts"}
              aria-label={railView === "messages" ? "Search messages (names and text)" : "Search contacts"}
            />
          </label>

          {(connPrefs.link || connPrefs.username || connPrefs.pairing) && (
            <form className={styles.addForm} onSubmit={addPeer}>
              <input
                className={styles.addInput}
                value={peerInput}
                onChange={(e) => {
                  setPeerInput(e.target.value);
                  if (addHint) setAddHint(null);
                }}
                placeholder={`+ ${[
                  connPrefs.link && "id / link",
                  connPrefs.username && "@username",
                  connPrefs.pairing && "code",
                ]
                  .filter(Boolean)
                  .join(", ")}`}
                autoComplete="off"
                spellCheck={false}
                aria-label="Add someone using a connection method you've enabled in Settings"
              />
              <button className={styles.addBtn} type="submit" disabled={state.busy || state.status !== "ready"}>
                Add
              </button>
            </form>
          )}
          {!railCollapsed && !(connPrefs.link || connPrefs.username || connPrefs.pairing) && (
            <div className={styles.railEmpty} style={{ padding: "6px 4px" }}>
              All connect methods are off. Turn one on in Settings → Connections.
            </div>
          )}
          {!railCollapsed && addHint && (
            <div
              className={styles.railEmpty}
              style={{ padding: "0 4px 4px", color: "var(--mc-warn)", fontSize: 11.5 }}
            >
              {addHint}
            </div>
          )}

          {!railCollapsed && (
          <button
            type="button"
            onClick={() => setNewGroupOpen(true)}
            disabled={state.busy || state.status !== "ready"}
            aria-label="Create a new group from existing contacts"
            data-tip="Group up existing contacts"
            style={{
              alignSelf: "flex-start",
              marginTop: -6,
              font: "inherit",
              fontSize: 11.5,
              fontWeight: 600,
              color: "var(--mc-ink2)",
              background: "var(--mc-bg)",
              border: "1px solid var(--mc-border)",
              borderRadius: 9,
              padding: "5px 10px",
              cursor: "pointer",
            }}
          >
            + New group
          </button>
          )}

          <div className={styles.railSeg} role="tablist" aria-label="Rail view">
            <button
              type="button"
              role="tab"
              aria-selected={railView === "messages"}
              className={styles.railSegBtn}
              data-active={railView === "messages" ? "" : undefined}
              onClick={() => setRailView("messages")}
            >
              Messages
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={railView === "contacts"}
              className={styles.railSegBtn}
              data-active={railView === "contacts" ? "" : undefined}
              onClick={() => setRailView("contacts")}
            >
              Contacts
            </button>
          </div>

          <div className={`${styles.kicker} mono`}>
            {railView === "messages" ? "conversations" : "contacts"}
          </div>
          <div className={styles.convList}>
            {railView === "messages" ? (
              <>
                {shownPeers.length === 0 && (
                  <div className={styles.railEmpty}>
                    {peers.length === 0 ? "No conversations yet." : "No matches."}
                  </div>
                )}
                {shownPeers.map((p) => (
                  <ConvRow
                    key={p.peer}
                    p={p}
                    active={active?.peer === p.peer}
                    onClick={() => selectConv(p.peer)}
                    collapsed={railCollapsed}
                  />
                ))}
              </>
            ) : (
              <>
                {contactPeers.length === 0 && (
                  <div className={styles.railEmpty}>
                    {peers.length === 0 ? "No contacts yet." : "No matches."}
                  </div>
                )}
                {contactPeers.map((p) => (
                  <ConvRow
                    key={p.peer}
                    p={p}
                    active={active?.peer === p.peer}
                    onClick={() => selectConv(p.peer)}
                    collapsed={railCollapsed}
                    contactMode
                  />
                ))}
              </>
            )}
          </div>

          <button
            className={styles.idCard}
            onClick={() =>
              state.accountId &&
              setModal({ kind: "profile", person: me, id: state.accountId, self: true })
            }
            type="button"
            data-tip="Your profile"
          >
            <Avatar person={me} size={28} />
            <div className={styles.idText}>
              <div className={styles.idName}>{me.name}</div>
              <div className={`${styles.idSub} mono`}>
                {state.accountId ? short(state.accountId) : "loading…"}
              </div>
              {state.myUsername && (
                <div className={`${styles.idSub} mono`} title="Your claimed username">
                  @{state.myUsername}
                </div>
              )}
            </div>
            <span className={`${styles.dot} ${styles[state.status]}`} data-tip={state.status} />
            <span className={styles.idArrow}>›</span>
          </button>
        </aside>

        {/* ---------- center ---------- */}
        <main className={styles.center}>
          {!active ? (
            <Welcome
              accountId={state.accountId}
              error={state.error}
              busy={state.busy}
              status={state.status}
              hasPeers={peers.length > 0}
              peerInput={peerInput}
              setPeerInput={setPeerInput}
              onAdd={addPeer}
              onCopy={() => copy("me", state.accountId)}
              copied={copied === "me"}
              onRestore={() => setModal({ kind: "recovery" })}
              controller={controller}
              connPrefs={connPrefs}
              myUsername={state.myUsername}
            />
          ) : (
            <div className={styles.thread}>
              <header className={styles.threadHead}>
                {mobile && (
                  <button
                    className={styles.backBtn}
                    type="button"
                    onClick={() => setMobileView("list")}
                    aria-label="Back to conversations"
                  >
                    ‹
                  </button>
                )}
                <Avatar
                  person={active.person}
                  size={32}
                  onClick={() =>
                    activeGroup
                      ? openFeaturePanel("group")
                      : setModal({ kind: "profile", person: active.person, id: active.peer, self: false })
                  }
                />
                <div className={styles.threadHeadText}>
                  <div className={styles.threadPeer}>
                    <span className={styles.lock}>🔒</span> {active.person.name}
                    {/* The ✓ verified pill records a 1:1 safety-number comparison — a 1:1-only
                        concept, so it never shows on a group conversation. */}
                    {!activeGroup && state.verified.includes(active.peer) && (
                      <span
                        title="You compared safety numbers for this contact"
                        style={{
                          marginLeft: 8,
                          fontSize: 10.5,
                          fontWeight: 600,
                          color: "var(--mc-ok)",
                          background: "var(--mc-ok-soft)",
                          border: "1px solid color-mix(in srgb, var(--mc-ok) 35%, transparent)",
                          borderRadius: 999,
                          padding: "1px 8px",
                          verticalAlign: "middle",
                        }}
                      >
                        ✓ verified
                      </span>
                    )}
                  </div>
                  {activeGroup ? (
                    <button
                      type="button"
                      className={`${styles.threadMeta} mono`}
                      onClick={() => openFeaturePanel("group")}
                      title="Group members"
                      aria-label="Show group members"
                      style={{
                        background: "none",
                        border: "none",
                        padding: 0,
                        textAlign: "left",
                        cursor: "pointer",
                      }}
                    >
                      {activeGroup.members.length} member{activeGroup.members.length === 1 ? "" : "s"}
                      {activeGroup.pending.length > 0 ? ` (+${activeGroup.pending.length} invited)` : ""}
                      {activeGroup.ended ? " · closed" : " · end-to-end encrypted"}
                    </button>
                  ) : (
                    <div className={`${styles.threadMeta} mono`}>direct · end-to-end encrypted</div>
                  )}
                </div>
                <span className={`${styles.dot} ${styles[state.status]}`} data-tip={state.status} />
              </header>

              <DataStrip
                onClick={() =>
                  activeGroup ? openFeaturePanel("group") : setModal({ kind: "encryption" })
                }
                outbound={state.activeDecisions?.outbound ?? null}
                group={Boolean(activeGroup)}
              />

              {q && activeMatchIds.size > 0 && (
                <div
                  className="mono"
                  style={{ padding: "7px 18px 0", fontSize: 10.5, color: "var(--mc-muted)" }}
                >
                  {activeMatchIds.size === 1 ? "1 match" : `${activeMatchIds.size} matches`} in this
                  conversation
                </div>
              )}

              {state.error && (
                <div className={styles.banner} role="alert">
                  {state.error}
                </div>
              )}

              <div className={styles.list}>
                {messages.length === 0 && (
                  <div className={styles.empty}>
                    No messages yet — say hello. This thread is sealed end-to-end.
                  </div>
                )}
                {messages.map((m, i) => {
                  const prev = messages[i - 1];
                  const needDay = !prev || !sameDay(prev.ts, m.ts);
                  // GROUP system notices (member joined/left, group closed) arrive as incoming
                  // messages with no sender attribution and a leading 👥 — rendered as a quiet
                  // centered line, not a bubble.
                  if (activeGroup && m.kind === "incoming" && !m.fromId && m.text.startsWith("👥")) {
                    return (
                      <Fragment key={m.id}>
                        {needDay && <DayDivider ts={m.ts} />}
                        <GroupNotice text={m.text} highlight={activeMatchIds.has(m.id)} />
                      </Fragment>
                    );
                  }
                  // GROUP incoming messages carry the REAL sender id (`fromId`, MLS-attributed);
                  // label the first message of each sender run.
                  const sender =
                    activeGroup && m.kind === "incoming" && m.fromId
                      ? groupSender(m.fromId, state.aliases)
                      : null;
                  const showSender = sender != null && prev?.fromId !== m.fromId;
                  return (
                    <Fragment key={m.id}>
                      {needDay && <DayDivider ts={m.ts} />}
                      <Bubble
                        m={m}
                        me={me}
                        peer={active.person}
                        sender={sender}
                        showSender={showSender}
                        highlight={activeMatchIds.has(m.id)}
                        onProfile={
                          sender
                            ? undefined // no fetched card for arbitrary group members — no profile sheet
                            : () =>
                                setModal({
                                  kind: "profile",
                                  person: active.person,
                                  id: active.peer,
                                  self: false,
                                })
                        }
                      />
                    </Fragment>
                  );
                })}
              </div>

              {activeGroup?.ended ? (
                // The backend refuses sends to a closed group; the UI doesn't offer them either.
                <div
                  role="note"
                  style={{
                    margin: "0 18px 16px",
                    padding: "12px 16px",
                    borderRadius: 12,
                    border: "1px dashed var(--mc-border-strong)",
                    background: "var(--mc-surface)",
                    color: "var(--mc-muted)",
                    fontSize: 12.5,
                    textAlign: "center",
                  }}
                >
                  This group was closed by its creator — sending is disabled.
                </div>
              ) : (
                <Composer
                  thread={composer}
                  mobile={mobile}
                  onOpenPanel={openFeaturePanel}
                  onAttach={(f) => controller.sendAttachment(f.name, f.mime, f.dataB64)}
                />
              )}
            </div>
          )}
        </main>

        {/* ---------- inspector panel (docked on desktop, overlay on mobile) ---------- */}
        {active && (mobile ? mobileInspector : panelOpen) && (
          <aside className={styles.panel}>
            <div className={styles.panelHead}>
              <span className={styles.panelTitle}>Inspector</span>
              <span className={styles.panelSub}>binding view</span>
              <span className={`${styles.panelLive} mono`}>● live</span>
              <button
                className={styles.panelClose}
                type="button"
                onClick={() => (mobile ? setMobileInspector(false) : setPanelOpen(false))}
                aria-label="Hide inspector"
              >
                ✕
              </button>
            </div>
            <div className={styles.panelScroll}>
              <Field label="You" value={state.accountId} onCopy={() => copy("me", state.accountId)} copied={copied === "me"} />
              <Field
                label={activeGroup ? "Group" : "Peer"}
                value={active.peer}
                onCopy={() => copy("peer", active.peer)}
                copied={copied === "peer"}
              />
              {activeGroup ? (
                // Honest group crypto facts: real MLS, classical (not yet PQ) in v1 — see
                // docs/GROUPS.md. Never show the 1:1 PQ-hybrid claims on a group thread.
                <div className={styles.card}>
                  <div className={`${styles.cardKicker} mono`}>encryption</div>
                  <Row k="who can read" v="members only" tone="ok" tip="Group messages are sealed so only members can read them — not the relay, not us." />
                  <Row k="protocol" v="MLS group" tone="irid" tip="Uses the MLS group-messaging standard to share keys among members." />
                  <Row k="quantum-safe" v="not yet (v1)" tip="Groups use classical-strength encryption in v1; quantum-safe groups are on the way." />
                  <Row k="members" v={`${activeGroup.members.length} / 16`} tip="People in this group (up to 16)." />
                </div>
              ) : (
                <div className={styles.card}>
                  <div className={`${styles.cardKicker} mono`}>encryption</div>
                  <Row k="who can read" v="only you two" tone="ok" tip="End-to-end encrypted: only you and the person you're messaging can read these — not the relay, not us." />
                  <Row k="quantum-safe" v="yes" tone="ok" tip="A hybrid X25519 + ML-KEM-768 handshake keeps messages private even against future quantum computers." />
                  <Row k="forward secrecy" v="yes" tone="ok" tip="Every message uses a fresh key (a double ratchet), so even a stolen key can't reveal past or future messages." />
                </div>
              )}
              <div className={styles.card}>
                <div className={`${styles.cardKicker} mono`}>this conversation</div>
                {!activeGroup && (
                  <Row
                    k="safety number"
                    v={state.verified.includes(active.peer) ? "verified" : "not verified"}
                    tone={state.verified.includes(active.peer) ? "ok" : undefined}
                    tip="Compare your safety number out-of-band (in person or a call you recognise), then mark the contact verified — that's the one check that proves nobody is impersonating them."
                  />
                )}
                {!activeGroup && (
                  <Row
                    k="ai access"
                    v={state.blocked.includes(active.peer) ? "blocked" : "only when shared"}
                    tone={state.blocked.includes(active.peer) ? undefined : "ok"}
                    tip="The AI assistant only sees a message when you send it to @ai. 'Blocked' turns even that off for this chat."
                  />
                )}
                {activeGroup && (
                  <Row k="members" v={`${activeGroup.members.length} / 16`} tip="People in this group (up to 16)." />
                )}
                <Row
                  k="disappearing"
                  v={fmtDisappear(state.disappearing[active.peer] ?? 0)}
                  tone={(state.disappearing[active.peer] ?? 0) > 0 ? "irid" : undefined}
                  tip="When on, messages auto-delete on both sides after this long. Set it from Conversation settings (the ⋯ menu, top-right)."
                />
                <Row
                  k="pinned"
                  v={state.pinned.includes(active.peer) ? "yes" : "no"}
                  tip="Pinned chats stay at the top of your conversation list."
                />
                <Row
                  k="notifications"
                  v={state.muted.includes(active.peer) ? "muted" : "on"}
                  tip="Whether this chat can notify you. Mute or unmute from Conversation settings (the ⋯ menu, top-right)."
                />
                <Row
                  k="shared files"
                  v={String(activeConv ? activeConv.messages.filter((m) => m.att).length : 0)}
                  tip="Files exchanged in this chat — all end-to-end encrypted, same as messages."
                />
              </div>
              <div className={styles.card}>
                <div className={`${styles.cardKicker} mono`}>the server</div>
                <Row k="role" v="blind relay" tip="The server only stores and forwards sealed bytes between devices — there's no separate message or file server." />
                <Row k="can read" v="nothing" tone="ok" tip="The relay only ever sees ciphertext — never your messages, names, or files." />
                <Row k="sent" v={String(outCount)} tip="Messages you've sent in this chat." />
                <Row k="received" v={String(inCount)} tip="Messages you've received in this chat." />
              </div>
              <EventTail events={state.events} />
              {state.activeDecisions ? (
                <details className={`${styles.decisionRaw} mono`}>
                  <summary>mercury-core decision views</summary>
                  <div className={styles.decisions} style={{ marginTop: 8 }}>
                    <DecisionCard title="bootstrap" view={state.activeDecisions.bootstrap} />
                    <DecisionCard title="outbound send" view={state.activeDecisions.outbound} />
                    <DecisionCard title="last receive" view={state.activeDecisions.receive} />
                  </div>
                </details>
              ) : (
                <div className={`${styles.decisionsPending} mono`}>
                  fetching mercury-core decision views…
                </div>
              )}
              <div className={styles.panelNote}>
                The server stores and forwards only sealed bytes. Keys never leave the device; this
                screen only sees plaintext you send or receive.
              </div>
            </div>
          </aside>
        )}
      </div>

      {convMenu && active && (
        <ConversationMenu
          controller={controller}
          peer={active.peer}
          person={active.person}
          muted={state.muted.includes(active.peer)}
          blocked={state.blocked.includes(active.peer)}
          pinned={state.pinned.includes(active.peer)}
          onPin={(on) => void controller.setPinned(active.peer, on)}
          disappearingSecs={state.disappearing[active.peer] ?? 0}
          group={activeGroup}
          isCreator={Boolean(activeGroup && state.accountId && activeGroup.creator === state.accountId)}
          onClose={() => setConvMenu(false)}
        />
      )}

      {newGroupOpen && (
        <NewGroupModal
          controller={controller}
          candidates={peers
            .filter((p) => !state.groups[p.peer]) // groups can't be members — 1:1 contacts only
            .map((p) => ({ id: p.peer, name: p.person.name }))}
          onClose={() => setNewGroupOpen(false)}
          onCreated={() => {
            setNewGroupOpen(false);
            if (mobile) setMobileView("thread");
          }}
        />
      )}

      {modal && isLiveFeatureKind(modal.kind) && (
        <LiveFeaturePanel
          kind={modal.kind}
          mobile={mobile}
          thread={composer}
          controller={controller}
          convState={state}
          active={active}
          trust={trustState}
          setTrust={setTrustState}
          ai={aiState}
          setAi={setAiState}
          onClose={() => setModal(null)}
          openPanel={openFeaturePanel}
        />
      )}

      {modal && modal.kind === "settings" && (
        <SettingsHub
          mobile={mobile}
          onClose={() => setModal(null)}
          accountId={state.accountId}
          backendUrl={backendUrl}
          convCount={peers.length}
          theme={theme}
          setTheme={setTheme}
          profile={profile}
          setProfile={setProfile}
          me={me}
          thread={composer}
          controller={controller}
          convState={state}
          active={active}
          trust={trustState}
          setTrust={setTrustState}
          ai={aiState}
          setAi={setAiState}
          openPanel={openFeaturePanel}
        />
      )}

      {modal && !isLiveFeatureKind(modal.kind) && modal.kind !== "settings" && (
        <ModalHost
          modal={modal as LegacyModalState}
          convCount={peers.length}
          profile={profile}
          setProfile={setProfile}
          onClose={() => setModal(null)}
        />
      )}
    </div>
  );
}

function ConvRow({ p, active, onClick, collapsed, contactMode }: { p: PeerView; active: boolean; onClick: () => void; collapsed?: boolean; contactMode?: boolean }) {
  return (
    <button className={styles.convRow} data-active={active ? "" : undefined} onClick={onClick} type="button" data-tip={collapsed ? p.person.name : undefined}>
      <Avatar person={p.person} size={30} />
      <div className={styles.convText}>
        <div className={styles.convName}>
          {p.pinned && (
            <span title="Pinned conversation" style={{ marginRight: 4, fontSize: 10, opacity: 0.7 }}>
              📌
            </span>
          )}
          {p.person.name}
        </div>
        <div className={`${styles.convLast} mono`}>
          {contactMode
            ? p.connected
              ? "connected"
              : "not connected yet"
            : p.lastText
              ? p.lastText
              : p.connected
                ? "no messages yet"
                : "incoming"}
        </div>
      </div>
      {p.unread > 0 && <span className={styles.unread}>{p.unread}</span>}
    </button>
  );
}

/** Human-readable disappearing-timer label for the inspector (e.g. 30m / 6h / 7d / off). */
function fmtDisappear(secs: number): string {
  if (!secs) return "off";
  if (secs < 3600) return `${Math.round(secs / 60)}m`;
  if (secs < 86400) return `${Math.round(secs / 3600)}h`;
  return `${Math.round(secs / 86400)}d`;
}

function DataStrip({
  onClick,
  outbound,
  group,
}: {
  onClick: () => void;
  outbound: PlatformDecisionView | null;
  group?: boolean;
}) {
  // The "send" chip mirrors the REAL outbound decision for the active peer: a green "ready" only
  // when the backend actually accepted the send, "held" when it rejected, "…" before the first
  // verdict arrives. The other chips are static, true crypto facts (not decisions) — and they
  // differ for GROUPS, which are real MLS but classical (not yet PQ) in v1; saying "ml-kem-768"
  // on a group thread would be a lie. See docs/GROUPS.md.
  // Plain-language status for a beginner: WHAT it means, not the algorithm names. Each chip carries a
  // hover tooltip with the one-line "why", and the whole strip still clicks through to the full
  // technical breakdown. Group threads are honestly distinguished (real MLS, classical in v1).
  const sendCell: { label: string; tip: string; tone?: "ok" | "irid"; dot?: boolean } = !outbound
    ? { label: "Checking…", tip: "Working out whether this message can be sent." }
    : outbound.accepted
      ? { label: "Ready to send", tip: "Checked — you can send right now.", tone: "ok", dot: true }
      : {
          label: "Sending paused",
          tip: "Sending is on hold — usually the contact still needs to be set up or verified.",
        };
  const cells: { label: string; tip: string; tone?: "ok" | "irid"; dot?: boolean }[] = group
    ? [
        {
          label: "End-to-end encrypted",
          tip: "Group messages are sealed so only members can read them — not the relay, not us.",
          tone: "ok",
          dot: true,
        },
        {
          label: "Group encryption (MLS)",
          tip: "Uses the MLS group-messaging standard. Groups are classical-strength in v1 — quantum-safe groups are on the way.",
          tone: "irid",
        },
        sendCell,
      ]
    : [
        {
          label: "End-to-end encrypted",
          tip: "Only you and the person you're messaging can read these — not the relay, not us, nobody in between.",
          tone: "ok",
          dot: true,
        },
        {
          label: "Quantum-safe",
          tip: "A hybrid X25519 + ML-KEM-768 handshake keeps your messages private even against future quantum computers.",
          tone: "irid",
        },
        sendCell,
      ];
  return (
    <button
      className={styles.dataStrip}
      type="button"
      onClick={onClick}
      aria-label="What this means — open the full encryption details"
    >
      {cells.map((c) => (
        <span key={c.label} className={styles.cell} title={c.tip}>
          {c.dot && <span className={styles.cellDot} data-tone={c.tone} />}
          {c.tone === "irid" ? (
            <span className="iris-text" style={{ fontWeight: 600 }}>
              {c.label}
            </span>
          ) : (
            <span className={styles.cellVal} data-tone={c.tone}>
              {c.label}
            </span>
          )}
        </span>
      ))}
      <span
        className={styles.cell}
        style={{ marginLeft: "auto", color: "var(--mc-muted)" }}
        title="See the full encryption breakdown."
      >
        details ›
      </span>
    </button>
  );
}

function DayDivider({ ts }: { ts: number }) {
  return (
    <div className={styles.dayDivider}>
      <span className={styles.rule} />
      <span className={`${styles.dayLabel} mono`}>{fmtDay(ts)}</span>
      <span className={styles.rule} />
    </div>
  );
}

function Field({
  label,
  value,
  onCopy,
  copied,
}: {
  label: string;
  value: string | null;
  onCopy: () => void;
  copied: boolean;
}) {
  return (
    <button className={styles.field} type="button" onClick={onCopy} data-tip="Copy">
      <span className={`${styles.fieldLabel} mono`}>{label}</span>
      <span className={`${styles.fieldVal} mono`}>{copied ? "copied ✓" : value ? short(value) : "…"}</span>
    </button>
  );
}

function Row({ k, v, tone, tip }: { k: string; v: string; tone?: "ok" | "irid"; tip?: string }) {
  return (
    <div className={styles.row} title={tip}>
      <span className={`${styles.rowKey} mono`}>{k}</span>
      {tone === "irid" ? (
        <span className="iris-text" style={{ fontWeight: 600, fontSize: 12.5 }}>
          {v}
        </span>
      ) : (
        <span className={styles.rowVal} data-tone={tone}>
          {v}
        </span>
      )}
    </div>
  );
}

// The capability/requirement flags surfaced as bullets on each decision card. `can_*` flags read
// green when granted; `requires_*` flags read amber when active; both read muted when off.
const DECISION_FLAGS: { key: keyof PlatformDecisionView; label: string; kind: "can" | "req" }[] = [
  { key: "can_open_message_ui", label: "message ui", kind: "can" },
  { key: "can_send", label: "send", kind: "can" },
  { key: "can_receive", label: "receive", kind: "can" },
  { key: "requires_user_action", label: "needs confirm", kind: "req" },
  { key: "requires_client_retry", label: "needs retry", kind: "req" },
];

// Every field of the view, in wire order — shown verbatim under "raw fields" so what the inspector
// renders is exactly what mercury-core produced (no curation, no hidden capability).
const DECISION_RAW_FIELDS: (keyof PlatformDecisionView)[] = [
  "source",
  "accepted",
  "reason_code",
  "reason_label",
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

// A REAL mercury-core decision view, rendered verbatim. `accepted` tints the card + badge; the
// reason label/code, plain-language gloss, capability bullets, and a collapsible raw-field dump all
// come straight off the backend-produced view — the UI invents nothing.
function DecisionCard({ title, view }: { title: string; view: PlatformDecisionView }) {
  return (
    <div className={styles.decision} data-accepted={view.accepted}>
      <div className={styles.decisionHead}>
        <span className={`${styles.decisionSrc} mono`}>{title}</span>
        <span className={styles.decisionBadge} data-accepted={view.accepted}>
          {view.accepted ? "✓ ACCEPT" : "✕ REJECT"}
        </span>
      </div>
      <div className={styles.decisionMeta}>
        <span className={`${styles.decisionLabel} mono`}>{view.reason_label}</span>
        <span className={`${styles.decisionRc} mono`}>rc={view.reason_code}</span>
      </div>
      <div className={styles.decisionExplain}>{explainDecision(view)}</div>
      <div className={styles.decisionFlags}>
        {DECISION_FLAGS.map((f) => {
          const on = Boolean(view[f.key]);
          const tone = on ? (f.kind === "can" ? "ok" : "warn") : "off";
          return (
            <div key={f.key} className={`${styles.decisionFlag} mono`} data-on={on ? "" : undefined}>
              <span className={styles.decisionFlagDot} data-tone={tone} />
              {f.label}
            </div>
          );
        })}
      </div>
      <details className={`${styles.decisionRaw} mono`}>
        <summary>› raw fields</summary>
        <div className={styles.decisionRawGrid}>
          {DECISION_RAW_FIELDS.map((k) => (
            <Fragment key={k}>
              <span className={styles.decisionRawKey}>{k}</span>
              <span className={styles.decisionRawVal}>{String(view[k])}</span>
            </Fragment>
          ))}
        </div>
      </details>
    </div>
  );
}

function EventTail({ events }: { events: LiveEvent[] }) {
  return (
    <div className={styles.tail}>
      <div className={`${styles.cardKicker} mono`}>event log</div>
      {events.length === 0 ? (
        <div className={styles.tailEmpty}>No events yet.</div>
      ) : (
        events.map((e, i) => (
          <div key={`${e.t}-${i}`} className={`${styles.tailRow} mono`}>
            <span className={styles.tailDot} data-kind={e.kind} />
            <span className={styles.tailDetail}>{e.detail}</span>
            <span className={styles.tailTime}>{fmtMsgTime(e.t)}</span>
          </div>
        ))
      )}
    </div>
  );
}

// Honest outgoing-status text from the backend's real delivery ladder: "sending" = queued in the
// outbox, "sent" = the relay accepted it, "delivered" = the peer's DEVICE confirmed receipt,
// "read" = the peer's device reported the message was actually viewed (1:1 only by construction —
// the backend never sets read on groups — and only when the peer's read receipts are on).
function statusText(status: Msg["status"]): string {
  if (status === "sending") return "… sending";
  if (status === "read") return "✓✓ read";
  if (status === "delivered") return "✓ delivered";
  return "● sent";
}

// A real attachment riding the sealed channel: inline preview for images, always the filename +
// a Save action (desktop → Downloads via the Rust shell, with the saved path surfaced briefly;
// browser → a plain Blob download).
function AttachmentBody({ att }: { att: NonNullable<Msg["att"]> }) {
  const [note, setNote] = useState<string | null>(null);
  const tauri = inTauri();
  const save = async () => {
    try {
      if (tauri) {
        const path = await saveToDownloads(att.name, att.data_b64);
        setNote(`saved → ${path}`);
      } else {
        downloadViaBlob(att.name, att.mime, att.data_b64);
        return;
      }
    } catch (e) {
      setNote(e instanceof Error ? e.message : String(e));
    }
    window.setTimeout(() => setNote(null), 4000);
  };
  return (
    <span style={{ position: "relative", display: "block" }}>
      {att.mime.startsWith("image/") && (
        <img
          src={`data:${att.mime};base64,${att.data_b64}`}
          alt={att.name}
          style={{ maxWidth: 260, maxHeight: 200, borderRadius: 8, display: "block", marginBottom: 6 }}
        />
      )}
      <span style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <span style={{ fontSize: 12.5, wordBreak: "break-all" }}>📎 {att.name}</span>
        <button
          type="button"
          onClick={() => void save()}
          style={{
            border: "1px solid var(--mc-border)",
            borderRadius: 7,
            background: "var(--mc-surface)",
            color: "var(--mc-ink2)",
            fontSize: 10.5,
            fontWeight: 600,
            padding: "2px 8px",
            cursor: "pointer",
            flexShrink: 0,
          }}
        >
          {tauri ? "Save" : "Download"}
        </button>
      </span>
      {note && (
        <span style={{ display: "block", fontSize: 10.5, opacity: 0.8, marginTop: 3, wordBreak: "break-all" }}>
          {note}
        </span>
      )}
    </span>
  );
}

function Bubble({
  m,
  me,
  peer,
  sender,
  showSender,
  highlight,
  onProfile,
}: {
  m: Msg;
  me: Person;
  peer: Person;
  /** GROUP threads: the attributed sender of an incoming message (from `m.fromId`). When set, the
   *  avatar/hue/name belong to the sender, and the name label shows only on the first message of
   *  a run (`showSender`). Absent in 1:1 threads, which keep their always-on peer name row. */
  sender?: Person | null;
  showSender?: boolean;
  /** This message's text matches the rail search query — wash it with the static search tint. */
  highlight?: boolean;
  onProfile?: () => void;
}) {
  if (m.kind === "outgoing") {
    return (
      <div
        className={`${msg.outgoingRow} rise`}
        style={{ ...hueStyle(me.hue, "74%"), ...(highlight ? SEARCH_HIT : undefined) }}
      >
        <div className={msg.outBubble}>
          <span className="iris-ring" style={{ borderRadius: 12 }} />
          {m.att && <AttachmentBody att={m.att} />}
          {m.text && <span style={{ position: "relative" }}>{m.text}</span>}
        </div>
        <div className={msg.outFooter}>
          <span className={msg.delivered}>{statusText(m.status)}</span>
          <span>· {fmtMsgTime(m.ts)}</span>
        </div>
      </div>
    );
  }
  const who = sender ?? peer;
  const showName = sender ? showSender === true : true;
  return (
    <div
      className={`${msg.incomingRow} rise`}
      style={{ ...hueStyle(who.hue, "78%"), ...(highlight ? SEARCH_HIT : undefined) }}
    >
      <Avatar person={who} size={28} onClick={onProfile} />
      <div className={msg.bubbleCol}>
        {showName && (
          <div className={msg.nameRow}>
            <span className={msg.authorName}>{who.name}</span>
          </div>
        )}
        <div className={`${msg.bubble} ${msg.tintBubble}`}>
          {m.att && <AttachmentBody att={m.att} />}
          {m.text && <span style={{ position: "relative" }}>{m.text}</span>}
        </div>
        <div className={msg.time}>{fmtMsgTime(m.ts)}</div>
      </div>
    </div>
  );
}

function LiveFeaturePanel({
  kind,
  mobile,
  thread,
  convState,
  active,
  trust,
  setTrust,
  ai,
  setAi,
  onClose,
  openPanel,
  controller,
}: {
  kind: LiveFeatureKind;
  mobile: boolean;
  thread: ReturnType<typeof useLiveConversations>["composer"];
  controller: ReturnType<typeof useLiveConversations>["controller"];
  convState: ReturnType<typeof useLiveConversations>["state"];
  active: ReturnType<typeof useLiveConversations>["active"];
  trust: TrustState;
  setTrust: (v: TrustState) => void;
  ai: AiState;
  setAi: (v: AiState) => void;
  onClose: () => void;
  openPanel: (panel: PanelId | LiveFeatureKind) => void;
}) {
  // Panels that are REAL (backend-wired or pure honest documentation), so they must NOT show the
  // "UI mockup — enforces nothing" preview banner: `updates` (auto-update), `policy` (genuine
  // mercury-core decision views), `connection` (real pairing-code + KT-username operations),
  // `trust` (real safety number + persisted verified mark), `notifications` (real master toggle +
  // autostart + mute list), `recovery` (real encrypted backup export/restore), `attachments`
  // (documentation of the shipped composer attachment feature), `onboarding` (accurate feature
  // documentation), and `ai` (honest model description + real block/unblock).
  //
  // `group` is REAL only when the ACTIVE conversation is an actual MLS group (its id is a key in
  // state.groups): the panel then shows the live roster + honest v1 limits and drives the real
  // leave/close action — no banner. With no real group active it falls back to the demo mockup
  // and KEEPS the preview banner, so the mockup can never pass for the real feature.
  const activeGroupInfo = active ? convState.groups[active.peer] ?? null : null;
  const isPreview = kind === "group" && activeGroupInfo == null;
  const node = (() => {
  if (kind === "trust") {
    return (
      <TrustPanel
        mobile={mobile}
        onClose={onClose}
        controller={controller}
        activePeer={active}
        verified={active ? convState.verified.includes(active.peer) : false}
      />
    );
  }
  if (kind === "ai") {
    return (
      <AiPanel
        mobile={mobile}
        onClose={onClose}
        controller={controller}
        activePeer={active}
        blocked={active ? convState.blocked.includes(active.peer) : false}
      />
    );
  }
  if (kind === "updates") {
    return <UpdatesPanel mobile={mobile} onClose={onClose} />;
  }
  if (kind === "attachments") {
    return <AttachmentsPanel mobile={mobile} onClose={onClose} />;
  }
  if (kind === "notifications") {
    return (
      <NotificationsPanel
        mobile={mobile}
        onClose={onClose}
        controller={controller}
        muted={convState.muted}
        aliases={convState.aliases}
        readReceipts={convState.readReceipts}
      />
    );
  }
  if (kind === "recovery") {
    return <RecoveryPanel mobile={mobile} onClose={onClose} controller={controller} />;
  }
  if (kind === "group") {
    if (active && activeGroupInfo) {
      // REAL group: live roster + honest limits + real leave/close. No preview banner.
      return (
        <GroupSafetyPanel
          mobile={mobile}
          onClose={onClose}
          controller={controller}
          group={activeGroupInfo}
          activePeer={active.peer}
          aliases={convState.aliases}
          accountId={convState.accountId}
        />
      );
    }
    // No real group active: the demo mockup, still behind the preview banner.
    return (
      <GroupSafetyPanel
        mobile={mobile}
        onClose={onClose}
        trust={trust}
        ai={ai}
        onOpenTrust={() => openPanel("trust")}
        onRotateEpoch={() => {
          setTrust("trusted");
          if (ai === "revoked") setAi("absent");
        }}
      />
    );
  }
  if (kind === "policy") {
    return <PolicyPanel mobile={mobile} onClose={onClose} thread={thread} />;
  }
  if (kind === "connection") {
    return <ConnectionPanel mobile={mobile} onClose={onClose} controller={controller} />;
  }
  return <OnboardingPanel mobile={mobile} onClose={onClose} openPanel={openPanel} />;
  })();
  return <PreviewContext.Provider value={isPreview}>{node}</PreviewContext.Provider>;
}

function Welcome({
  accountId,
  error,
  busy,
  status,
  hasPeers,
  peerInput,
  setPeerInput,
  onAdd,
  onCopy,
  copied,
  onRestore,
  controller,
  connPrefs,
  myUsername,
}: {
  accountId: string | null;
  error: string | null;
  busy: boolean;
  status: string;
  hasPeers: boolean;
  peerInput: string;
  setPeerInput: (v: string) => void;
  onAdd: (e: FormEvent) => void;
  onCopy: () => void;
  copied: boolean;
  onRestore: () => void;
  controller: ReturnType<typeof useLiveConversations>["controller"];
  connPrefs: ConnectionPrefs;
  myUsername: string | null;
}) {
  const [showQr, setShowQr] = useState(false);
  const [linkCopied, setLinkCopied] = useState(false);
  const [pairCode, setPairCode] = useState<string | null>(null);
  const [claimName, setClaimName] = useState("");
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);
  const link = accountId ? inviteLink(accountId) : "";
  const prefs = connPrefs;
  const copyLink = async () => {
    if (!link) return;
    try {
      await navigator.clipboard.writeText(link);
      setLinkCopied(true);
      window.setTimeout(() => setLinkCopied(false), 1200);
    } catch {
      /* clipboard unavailable; ignore */
    }
  };
  // Share-your-handle via the OTHER enabled methods (mirrors the Connections panel): a one-time
  // pairing code, and claiming a username so people can add you by handle.
  const showPairingCode = async () => {
    setBusyAction("pair");
    setFeedback(null);
    try {
      const { code } = await controller.pairingCode();
      setPairCode(code);
    } catch (e) {
      setFeedback(e instanceof Error ? e.message : String(e));
    } finally {
      setBusyAction(null);
    }
  };
  const claimUsername = async () => {
    const name = claimName.trim();
    if (!name) return;
    setBusyAction("claim");
    setFeedback(null);
    try {
      const { status: st, username } = await controller.claimUsername(name);
      setFeedback(
        st === "created" || st === "already_owned"
          ? `You now own @${username} — share it so people can add you by handle.`
          : st === "taken"
            ? `@${username} is already taken.`
            : st === "rejected"
              ? "Usernames are 3–20 chars: a–z, 0–9, _ and must start with a letter."
              : "The directory is busy — try again shortly.",
      );
      if (st === "created" || st === "already_owned") setClaimName("");
    } catch (e) {
      setFeedback(e instanceof Error ? e.message : String(e));
    } finally {
      setBusyAction(null);
    }
  };
  return (
    <div className={styles.connectWrap}>
      <form className={styles.connectCard} onSubmit={onAdd}>
        <div className={styles.connectTitle}>
          {hasPeers ? "Select a conversation" : "Start a conversation"}
        </div>
        <p className={styles.connectLead}>
          Share your invite link or QR so the other person can add you in one tap — or paste theirs
          below. Each side runs its own keys; this screen only ever sees plaintext.
        </p>
        <div className={styles.invite}>
          {prefs.link && (
            <button
              className={styles.inviteBtn}
              type="button"
              onClick={copyLink}
              disabled={!accountId}
              data-tip="Copy a link they can click to add you"
            >
              {linkCopied ? "invite link copied ✓" : "Copy invite link"}
            </button>
          )}
          {prefs.qr && (
            <button
              className={styles.inviteGhost}
              type="button"
              onClick={() => setShowQr((v) => !v)}
              disabled={!accountId}
              aria-expanded={showQr}
            >
              {showQr ? "Hide QR" : "Show QR"}
            </button>
          )}
          {prefs.pairing && (
            <button
              className={styles.inviteGhost}
              type="button"
              onClick={showPairingCode}
              disabled={!accountId || busyAction !== null}
              data-tip="A short, single-use code (~10 min) they can type to add you"
            >
              {busyAction === "pair" ? "…" : "Show pairing code"}
            </button>
          )}
        </div>
        {prefs.qr && showQr && link && (
          <div className={styles.qrWrap}>
            <QrCode value={link} size={188} />
            <span className={styles.qrCaption}>Scan to add you on Mercury</span>
          </div>
        )}
        {pairCode && (
          <div className={styles.yourId} style={{ cursor: "default" }}>
            <span className={`${styles.yourIdLabel} mono`}>your pairing code · one-time · ~10 min</span>
            <span className={`${styles.yourIdVal} mono`} style={{ fontSize: 18, fontWeight: 700, letterSpacing: 2 }}>
              {pairCode}
            </span>
          </div>
        )}
        {prefs.username &&
          (myUsername ? (
            <p className={styles.connectLead} style={{ margin: "0 0 2px" }}>
              Your handle: <strong>@{myUsername}</strong> — share it so people can add you by username.
            </p>
          ) : (
            <div style={{ display: "flex", gap: 6 }}>
              <input
                className={styles.connectInput}
                style={{ flex: 1, marginTop: 0 }}
                value={claimName}
                onChange={(e) => setClaimName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    void claimUsername();
                  }
                }}
                placeholder="claim a username so people can add you by handle"
                autoComplete="off"
                spellCheck={false}
                aria-label="Claim a username"
              />
              <button
                type="button"
                className={styles.inviteGhost}
                onClick={claimUsername}
                disabled={busyAction !== null || !claimName.trim()}
              >
                {busyAction === "claim" ? "…" : "Claim"}
              </button>
            </div>
          ))}
        {feedback && (
          <p className={styles.connectLead} style={{ margin: "2px 0 0", color: "var(--mc-accent)", fontSize: 12 }}>
            {feedback}
          </p>
        )}
        <button className={styles.yourId} type="button" onClick={onCopy} data-tip="Copy your raw account id">
          <span className={`${styles.yourIdLabel} mono`}>your id</span>
          <span className={`${styles.yourIdVal} mono`}>
            {copied ? "copied ✓" : accountId ? accountId : "loading…"}
          </span>
        </button>
        <input
          className={styles.connectInput}
          value={peerInput}
          onChange={(e) => setPeerInput(e.target.value)}
          placeholder={`paste ${
            [prefs.link && "an id / invite link", prefs.username && "@username", prefs.pairing && "a pairing code"]
              .filter(Boolean)
              .join(" · ") || "their handle"
          }`}
          autoComplete="off"
          spellCheck={false}
          aria-label="Add someone by any enabled connection method"
        />
        <button className={styles.primary} type="submit" disabled={busy || status !== "ready"}>
          {busy ? "Connecting…" : status === "ready" ? "Connect" : "Starting…"}
        </button>
        {error && <div className={styles.connectErr}>{error}</div>}
      </form>
      {!hasPeers && (
        <button
          type="button"
          onClick={onRestore}
          style={{
            marginTop: 16,
            alignSelf: "center",
            background: "none",
            border: "none",
            color: "var(--mc-ink2)",
            fontSize: 12,
            cursor: "pointer",
            textDecoration: "underline",
            textUnderlineOffset: 3,
          }}
          data-tip="Reinstalled, or on a new machine? Restore your account from an encrypted backup."
        >
          Already have an account? Restore from a backup
        </button>
      )}
    </div>
  );
}

// Appearance glyphs: a rayed sun (Light), a crescent (Dark), and a half-filled
// contrast disc (Auto). currentColor lets them inherit + cross-fade with the option.
function ThemeIcon({ choice }: { choice: ThemeChoice }) {
  if (choice === "light") {
    return (
      <svg className={styles.themeIcon} viewBox="0 0 24 24" width={22} height={22} fill="none" stroke="currentColor" strokeWidth={1.7} strokeLinecap="round" aria-hidden>
        <circle cx="12" cy="12" r="4.6" />
        <line x1="12" y1="1.6" x2="12" y2="3.8" />
        <line x1="12" y1="20.2" x2="12" y2="22.4" />
        <line x1="1.6" y1="12" x2="3.8" y2="12" />
        <line x1="20.2" y1="12" x2="22.4" y2="12" />
        <line x1="4.5" y1="4.5" x2="6.1" y2="6.1" />
        <line x1="17.9" y1="17.9" x2="19.5" y2="19.5" />
        <line x1="19.5" y1="4.5" x2="17.9" y2="6.1" />
        <line x1="6.1" y1="17.9" x2="4.5" y2="19.5" />
      </svg>
    );
  }
  if (choice === "dark") {
    return (
      <svg className={styles.themeIcon} viewBox="0 0 24 24" width={22} height={22} fill="none" stroke="currentColor" strokeWidth={1.7} strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
      </svg>
    );
  }
  return (
    <svg className={styles.themeIcon} viewBox="0 0 24 24" width={22} height={22} fill="none" stroke="currentColor" strokeWidth={1.7} aria-hidden>
      <circle cx="12" cy="12" r="8.6" />
      <path d="M12 3.4 A8.6 8.6 0 0 1 12 20.6 Z" fill="currentColor" stroke="none" />
    </svg>
  );
}

function ModalHost({
  modal,
  convCount,
  profile,
  setProfile,
  onClose,
}: {
  modal: LegacyModalState;
  convCount: number;
  profile: Profile;
  setProfile: (p: Profile) => void;
  onClose: () => void;
}) {
  const [copied, setCopied] = useState(false);
  const copy = async (v: string | null) => {
    if (!v) return;
    try {
      await navigator.clipboard.writeText(v);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    } catch {
      /* clipboard unavailable; ignore */
    }
  };
  const title =
    modal.kind === "settings"
      ? "Settings"
      : modal.kind === "encryption"
        ? "Encryption"
        : modal.self
          ? "Your profile"
          : "Profile";

  return (
    <div className={styles.scrim} onClick={onClose}>
      <div
        className={styles.modal}
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
      >
        <div className={styles.modalHead}>
          <span className={styles.modalTitle}>{title}</span>
          <button className={styles.modalClose} type="button" onClick={onClose} aria-label="Close">
            ✕
          </button>
        </div>
        <div className={styles.modalBody}>
          {modal.kind === "encryption" && (
            <>
              <div className={styles.card}>
                <div className={`${styles.cardKicker} mono`}>what protects your messages</div>
                <Row
                  k="who can read them"
                  v="only you two"
                  tone="ok"
                  tip="End-to-end encrypted: messages are sealed on your device and can only be opened on the other person's — not the relay, not us."
                />
                <Row
                  k="quantum-safe"
                  v="yes"
                  tone="ok"
                  tip="The handshake combines classical X25519 with post-quantum ML-KEM-768 — an attacker would have to break BOTH, so it stays private even against future quantum computers."
                />
              </div>
              <div className={styles.card}>
                <div className={`${styles.cardKicker} mono`}>how it stays safe over time</div>
                <Row
                  k="fresh key per message"
                  v="yes"
                  tone="ok"
                  tip="A double ratchet gives every message its own key (forward secrecy), so a stolen key can't unlock your past messages."
                />
                <Row
                  k="heals if a device is hacked"
                  v="yes"
                  tone="ok"
                  tip="If a device is compromised, the ratchet recovers so future messages become private again (post-compromise / break-in recovery)."
                />
                <Row
                  k="tamper-proof"
                  v="yes"
                  tone="ok"
                  tip="Every message is encrypted AND authenticated (AEAD), so nobody can read it or quietly change it in transit."
                />
              </div>
              <div className={styles.card}>
                <div className={`${styles.cardKicker} mono`}>what the server can see</div>
                <Row
                  k="the server's job"
                  v="just relays bytes"
                  tip="The relay only stores and forwards sealed bytes — there's no separate message or file server."
                />
                <Row
                  k="server can read"
                  v="nothing"
                  tone="ok"
                  tip="It only ever sees ciphertext — never your messages, names, or files."
                />
                <Row
                  k="how people find you"
                  v="verifiable directory"
                  tip="Usernames live in a key-transparency directory with self-authenticating entries, so the server can't quietly swap in a fake key for someone."
                />
              </div>
              <div className={styles.panelNote}>
                In plain terms: every message is locked on your device so only the person you're
                messaging can open it, each message gets its own key, and the relay only ever moves
                sealed bytes it can't read. Hover any line for the detail.
              </div>
            </>
          )}
          {modal.kind === "profile" && (
            <>
              <div className={styles.profileHead}>
                <Avatar
                  person={
                    modal.self
                      ? {
                          ...modal.person,
                          name: profile.name.trim() || "You",
                          short: profile.name.trim() ? initials(profile.name) : "Y",
                          hue: profile.hue,
                          avatar: profile.avatar,
                          color: profile.color,
                        }
                      : modal.person
                  }
                  size={54}
                />
                <div className={styles.profileName}>
                  {modal.self ? profile.name.trim() || "You" : modal.person.name}
                </div>
                <div className={styles.profileTag}>
                  {modal.self ? profile.status.trim() || "your identity" : "verified contact"}
                </div>
              </div>
              <ModalField label="Account id" value={modal.id} onCopy={() => copy(modal.id)} copied={copied} />
              {modal.self ? (
                <>
                  <div className={styles.card}>
                    <div className={`${styles.cardKicker} mono`}>customise your profile</div>
                    <div className={styles.profileEdit}>
                      <span className={`${styles.fieldLabel} mono`}>profile picture</span>
                      <div className={styles.avatarEditRow}>
                        <Avatar
                          person={{
                            ...modal.person,
                            name: profile.name.trim() || "You",
                            short: profile.name.trim() ? initials(profile.name) : "Y",
                            hue: profile.hue,
                            avatar: profile.avatar,
                            color: profile.color,
                          }}
                          size={48}
                        />
                        <label className={styles.uploadBtn}>
                          {profile.avatar ? "Change photo" : "Upload photo"}
                          <input
                            type="file"
                            accept="image/*"
                            hidden
                            onChange={async (e) => {
                              const f = e.target.files?.[0];
                              e.target.value = "";
                              if (!f) return;
                              const url = await imageToAvatarDataUrl(f);
                              if (url) setProfile({ ...profile, avatar: url });
                            }}
                          />
                        </label>
                        {profile.avatar && (
                          <button
                            type="button"
                            className={styles.removeBtn}
                            onClick={() => setProfile({ ...profile, avatar: undefined })}
                          >
                            Remove
                          </button>
                        )}
                      </div>
                    </div>
                    <label className={styles.profileEdit}>
                      <span className={`${styles.fieldLabel} mono`}>display name</span>
                      <input
                        className={styles.profileInput}
                        value={profile.name}
                        maxLength={40}
                        onChange={(e) => setProfile({ ...profile, name: e.target.value })}
                        placeholder="Your name"
                      />
                    </label>
                    <label className={styles.profileEdit}>
                      <span className={`${styles.fieldLabel} mono`}>status</span>
                      <input
                        className={styles.profileInput}
                        value={profile.status}
                        maxLength={80}
                        onChange={(e) => setProfile({ ...profile, status: e.target.value })}
                        placeholder="What's your status?"
                      />
                    </label>
                    <div className={styles.profileEdit}>
                      <span className={`${styles.fieldLabel} mono`}>avatar colour</span>
                      <div className={styles.hueRow}>
                        {PROFILE_HUES.map((h) => (
                          <button
                            key={h}
                            type="button"
                            className={styles.hueDot}
                            data-active={profile.hue === h && !profile.color ? "" : undefined}
                            style={{ background: `hsl(${h} 68% 55%)` }}
                            onClick={() => setProfile({ ...profile, hue: h, color: undefined })}
                            aria-label={`avatar colour ${h}`}
                          />
                        ))}
                        <label
                          className={styles.customHue}
                          data-active={profile.color ? "" : undefined}
                          style={{ background: profile.color ?? `hsl(${profile.hue} 68% 55%)` }}
                          data-tip="Custom colour"
                        >
                          <span className={styles.customHuePlus}>+</span>
                          <input
                            type="color"
                            className={styles.customHueInput}
                            value={profile.color ?? hueToHex(profile.hue)}
                            onChange={(e) => setProfile({ ...profile, color: e.target.value })}
                            aria-label="Custom avatar colour"
                          />
                        </label>
                      </div>
                    </div>
                  </div>
                  <div className={styles.card}>
                    <div className={`${styles.cardKicker} mono`}>this device</div>
                    <Row k="keys" v="on device only" tone="ok" />
                    <Row k="conversations" v={String(convCount)} />
                  </div>
                  <div className={styles.panelNote}>
                    Your profile is saved on this device and shown in your app. Sharing your name with
                    contacts is coming — for now, share your account id so they can reach you.
                  </div>
                </>
              ) : (
                <>
                  <div className={styles.card}>
                    <div className={`${styles.cardKicker} mono`}>contact</div>
                    <Row k="card" v="verified" tone="ok" />
                    <Row k="source" v="directory" />
                    <Row k="session" v="connected" tone="ok" />
                  </div>
                  <div className={styles.panelNote}>
                    This contact&apos;s card was fetched from the directory and verified against
                    their account id — the relay can&apos;t substitute or forge it.
                  </div>
                </>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function ModalField({
  label,
  value,
  onCopy,
  copied,
}: {
  label: string;
  value: string | null;
  onCopy: () => void;
  copied: boolean;
}) {
  return (
    <button className={styles.modalField} type="button" onClick={onCopy} data-tip="Copy">
      <span className={`${styles.fieldLabel} mono`}>{label}</span>
      <span className={`${styles.modalFieldVal} mono`}>{copied ? "copied ✓" : (value ?? "…")}</span>
    </button>
  );
}

/** Self-profile editor (avatar/name/status/colour incl. the custom <input type=color> picker).
 *  Authored as a standalone block so the unified Settings hub renders it inline; the standalone
 *  profile modal keeps its own copy unchanged. Reads/writes the same `profile` state. */
function SelfProfileFields({
  profile,
  setProfile,
  person,
  accountId,
  convCount,
  onCopyId,
  copied,
}: {
  profile: Profile;
  setProfile: (p: Profile) => void;
  person: Person;
  accountId: string | null;
  convCount: number;
  onCopyId: () => void;
  copied: boolean;
}) {
  const display: Person = {
    ...person,
    name: profile.name.trim() || "You",
    short: profile.name.trim() ? initials(profile.name) : "Y",
    hue: profile.hue,
    avatar: profile.avatar,
    color: profile.color,
  };
  return (
    <>
      <div className={styles.profileHead}>
        <Avatar person={display} size={54} />
        <div className={styles.profileName}>{profile.name.trim() || "You"}</div>
        <div className={styles.profileTag}>{profile.status.trim() || "your identity"}</div>
      </div>
      <ModalField label="Account id" value={accountId} onCopy={onCopyId} copied={copied} />
      <div className={styles.card}>
        <div className={`${styles.cardKicker} mono`}>customise your profile</div>
        <div className={styles.profileEdit}>
          <span className={`${styles.fieldLabel} mono`}>profile picture</span>
          <div className={styles.avatarEditRow}>
            <Avatar person={display} size={48} />
            <label className={styles.uploadBtn}>
              {profile.avatar ? "Change photo" : "Upload photo"}
              <input
                type="file"
                accept="image/*"
                hidden
                onChange={async (e) => {
                  const f = e.target.files?.[0];
                  e.target.value = "";
                  if (!f) return;
                  const url = await imageToAvatarDataUrl(f);
                  if (url) setProfile({ ...profile, avatar: url });
                }}
              />
            </label>
            {profile.avatar && (
              <button
                type="button"
                className={styles.removeBtn}
                onClick={() => setProfile({ ...profile, avatar: undefined })}
              >
                Remove
              </button>
            )}
          </div>
        </div>
        <label className={styles.profileEdit}>
          <span className={`${styles.fieldLabel} mono`}>display name</span>
          <input
            className={styles.profileInput}
            value={profile.name}
            maxLength={40}
            onChange={(e) => setProfile({ ...profile, name: e.target.value })}
            placeholder="Your name"
          />
        </label>
        <label className={styles.profileEdit}>
          <span className={`${styles.fieldLabel} mono`}>status</span>
          <input
            className={styles.profileInput}
            value={profile.status}
            maxLength={80}
            onChange={(e) => setProfile({ ...profile, status: e.target.value })}
            placeholder="What's your status?"
          />
        </label>
        <div className={styles.profileEdit}>
          <span className={`${styles.fieldLabel} mono`}>avatar colour</span>
          <div className={styles.hueRow}>
            {PROFILE_HUES.map((h) => (
              <button
                key={h}
                type="button"
                className={styles.hueDot}
                data-active={profile.hue === h && !profile.color ? "" : undefined}
                style={{ background: `hsl(${h} 68% 55%)` }}
                onClick={() => setProfile({ ...profile, hue: h, color: undefined })}
                aria-label={`avatar colour ${h}`}
              />
            ))}
            <label
              className={styles.customHue}
              data-active={profile.color ? "" : undefined}
              style={{ background: profile.color ?? `hsl(${profile.hue} 68% 55%)` }}
              data-tip="Custom colour"
            >
              <span className={styles.customHuePlus}>+</span>
              <input
                type="color"
                className={styles.customHueInput}
                value={profile.color ?? hueToHex(profile.hue)}
                onChange={(e) => setProfile({ ...profile, color: e.target.value })}
                aria-label="Custom avatar colour"
              />
            </label>
          </div>
        </div>
      </div>
      <div className={styles.card}>
        <div className={`${styles.cardKicker} mono`}>this device</div>
        <Row k="keys" v="on device only" tone="ok" />
        <Row k="conversations" v={String(convCount)} />
      </div>
    </>
  );
}

const HUB_SECTIONS = [
  { id: "profile", label: "Profile", keywords: "name color avatar id account display picture handle" },
  { id: "appearance", label: "Appearance", keywords: "theme dark light mode colour color" },
  { id: "privacy", label: "Privacy & trust", keywords: "encryption security trust safety number block verify policy keys ratchet" },
  { id: "notifications", label: "Notifications", keywords: "alerts sound mute push badge ping" },
  { id: "ai", label: "AI access", keywords: "assistant ai automation agent model" },
  { id: "account", label: "Account & backup", keywords: "backup restore recovery delete data device passphrase export erase wipe" },
  { id: "connections", label: "Connections", keywords: "connect qr link username pairing invite relay add contact" },
  { id: "updates", label: "Updates", keywords: "update version auto-update download install check signed upgrade release" },
  { id: "about", label: "About", keywords: "build version onboarding help license info" },
] as const;
type HubSection = (typeof HUB_SECTIONS)[number]["id"];

/** Unified Settings hub: one modal, left nav + content pane. Feature panels render INLINE via
 *  EmbeddedContext (no nested modal); simple sections render their controls directly. Every panel
 *  is wired to the SAME backend handlers as its standalone modal — no logic divergence. */
function SettingsHub({
  mobile,
  onClose,
  accountId,
  backendUrl,
  convCount,
  theme,
  setTheme,
  profile,
  setProfile,
  me,
  thread,
  controller,
  convState,
  active,
  trust,
  setTrust,
  ai,
  setAi,
  openPanel,
}: {
  mobile: boolean;
  onClose: () => void;
  accountId: string | null;
  backendUrl: string;
  convCount: number;
  theme: ThemeChoice;
  setTheme: (t: ThemeChoice) => void;
  profile: Profile;
  setProfile: (p: Profile) => void;
  me: Person;
  thread: ReturnType<typeof useLiveConversations>["composer"];
  controller: ReturnType<typeof useLiveConversations>["controller"];
  convState: ReturnType<typeof useLiveConversations>["state"];
  active: ReturnType<typeof useLiveConversations>["active"];
  trust: TrustState;
  setTrust: (v: TrustState) => void;
  ai: AiState;
  setAi: (v: AiState) => void;
  openPanel: (panel: PanelId | LiveFeatureKind) => void;
}) {
  const [section, setSection] = useState<HubSection>("profile");
  const [hubQuery, setHubQuery] = useState("");
  const hubQ = hubQuery.trim().toLowerCase();
  const filteredSections = hubQ
    ? HUB_SECTIONS.filter((s) => s.label.toLowerCase().includes(hubQ) || s.keywords.includes(hubQ))
    : HUB_SECTIONS;
  const [copied, setCopied] = useState(false);
  const copyId = async () => {
    if (!accountId) return;
    try {
      await navigator.clipboard.writeText(accountId);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    } catch {
      /* clipboard unavailable; ignore */
    }
  };
  const embed = (kind: LiveFeatureKind) => (
    <EmbeddedContext.Provider value={true}>
      <LiveFeaturePanel
        kind={kind}
        mobile={mobile}
        thread={thread}
        controller={controller}
        convState={convState}
        active={active}
        trust={trust}
        setTrust={setTrust}
        ai={ai}
        setAi={setAi}
        onClose={onClose}
        openPanel={openPanel}
      />
    </EmbeddedContext.Provider>
  );
  return (
    <div className={styles.scrim} onClick={onClose}>
      <div
        className={`${styles.modal} ${styles.hubModal}`}
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
      >
        <div className={styles.modalHead}>
          <span className={styles.modalTitle}>Settings</span>
          <button className={styles.modalClose} type="button" onClick={onClose} aria-label="Close">
            ✕
          </button>
        </div>
        <div className={styles.hub}>
          <nav className={styles.hubNav} aria-label="Settings sections">
            <label className={styles.hubSearch}>
              <span className={styles.searchIcon}>⌕</span>
              <input
                className={styles.searchInput}
                value={hubQuery}
                onChange={(e) => {
                  const v = e.target.value;
                  setHubQuery(v);
                  const q = v.trim().toLowerCase();
                  if (q) {
                    // Jump the content pane to the first match so search feels live, not just a filter.
                    const hit = HUB_SECTIONS.filter(
                      (s) => s.label.toLowerCase().includes(q) || s.keywords.includes(q),
                    );
                    if (hit.length && !hit.some((h) => h.id === section)) setSection(hit[0].id);
                  }
                }}
                placeholder="Search settings"
                aria-label="Search settings"
              />
            </label>
            {filteredSections.length === 0 ? (
              <div className={styles.hubNavEmpty}>No settings match “{hubQuery.trim()}”.</div>
            ) : (
              filteredSections.map((s) => (
                <button
                  key={s.id}
                  type="button"
                  className={styles.hubNavItem}
                  data-active={section === s.id ? "" : undefined}
                  aria-current={section === s.id ? "page" : undefined}
                  onClick={() => setSection(s.id)}
                >
                  {s.label}
                </button>
              ))
            )}
          </nav>
          <div className={styles.hubContent}>
            {section === "profile" && (
              <SelfProfileFields
                profile={profile}
                setProfile={setProfile}
                person={me}
                accountId={accountId}
                convCount={convCount}
                onCopyId={copyId}
                copied={copied}
              />
            )}
            {section === "appearance" && (
              <div className={styles.card}>
                <div className={`${styles.cardKicker} mono`}>appearance</div>
                <div className={styles.themePicker}>
                  {(["light", "dark", "auto"] as const).map((t) => (
                    <button
                      key={t}
                      type="button"
                      className={styles.themeOption}
                      data-active={theme === t ? "" : undefined}
                      onClick={() => setTheme(t)}
                    >
                      <ThemeIcon choice={t} />
                      {t === "auto" ? "Auto" : t === "light" ? "Light" : "Dark"}
                    </button>
                  ))}
                </div>
                <div className={styles.themeHint}>Auto follows your system setting.</div>
              </div>
            )}
            {section === "privacy" && (
              <>
                {embed("trust")}
                <div className={styles.hubDivider} />
                {embed("policy")}
                <div className={styles.hubDivider} />
                <div className={styles.card}>
                  <div className={`${styles.cardKicker} mono`}>encryption</div>
                  <Row k="key agreement" v="X25519 + ML-KEM-768" tone="irid" />
                  <Row k="messages" v="double ratchet · FS" tone="ok" />
                  <button
                    type="button"
                    className={styles.actionBtn}
                    style={{ marginTop: 8 }}
                    onClick={() => openPanel("encryption")}
                  >
                    Encryption details
                  </button>
                </div>
              </>
            )}
            {section === "notifications" && embed("notifications")}
            {section === "ai" && embed("ai")}
            {section === "account" && (
              <>
                <div className={styles.card}>
                  <div className={`${styles.cardKicker} mono`}>this device</div>
                  <Row k="backend" v={inTauri() ? "in-process (Tauri)" : "loopback shim"} />
                  <Row k="conversations" v={String(convCount)} />
                </div>
                {embed("recovery")}
                <div className={styles.hubDivider} />
                {embed("attachments")}
              </>
            )}
            {section === "connections" && embed("connection")}
            {section === "updates" && embed("updates")}
            {section === "about" && (
              <>
                <div className={styles.card}>
                  <div className={`${styles.cardKicker} mono`}>this build</div>
                  <Row k="app" v="Mercury" />
                  <Row k="backend" v={inTauri() ? "in-process (Tauri)" : backendUrl} />
                </div>
                {embed("onboarding")}
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
