import { useEffect, useRef } from "react";
import type { CSSProperties, ReactNode } from "react";

import { fmtDay, sameDay } from "../format";
import { GearIcon, Helix, RailIcon, RefreshIcon } from "../icons";
import { MERCURY_PEOPLE } from "../simulator";
import { FU_AI, FU_TRUST, toneVar } from "../strings";
import type { AiState, MercuryThreadState, PanelId, TrustState } from "../types";
import { Avatar } from "./Avatar";
import { GapBanner } from "./Banners";
import { BootstrapLock } from "./BootstrapLock";
import { Composer } from "./Composer";
import { Message } from "./Message";
import { Tofu } from "./Tofu";
import styles from "./Thread.module.css";

interface ThreadProps {
  thread: MercuryThreadState;
  mobile: boolean;
  trust: TrustState;
  ai: AiState;
  inspectorOpen: boolean;
  onToggleInspector: () => void;
  railOpen: boolean;
  onToggleRail: () => void;
  onOpenPanel: (panel: PanelId) => void;
  onOpenProfile: (who: string) => void;
}

export function Thread({
  thread,
  mobile,
  trust,
  ai,
  inspectorOpen,
  onToggleInspector,
  railOpen,
  onToggleRail,
  onOpenPanel,
  onOpenProfile,
}: ThreadProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const sticky = useRef(true);

  const onScroll = () => {
    const el = scrollRef.current;
    if (!el) return;
    sticky.current = el.scrollHeight - el.scrollTop - el.clientHeight < 80;
  };

  useEffect(() => {
    const el = scrollRef.current;
    if (!el || !sticky.current) return;
    el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
  }, [thread.messages.length, thread.pendingTofu]);

  if (!thread.bootstrapView.can_open_message_ui) {
    return <BootstrapLock view={thread.bootstrapView} onOpenPanel={onOpenPanel} />;
  }

  return (
    <>
      <Header
        mobile={mobile}
        trust={trust}
        ai={ai}
        inspectorOpen={inspectorOpen}
        onToggleInspector={onToggleInspector}
        railOpen={railOpen}
        onToggleRail={onToggleRail}
        onOpenPanel={onOpenPanel}
        onOpenProfile={onOpenProfile}
      />
      <div
        ref={scrollRef}
        onScroll={onScroll}
        className={styles.list}
        data-mobile={mobile ? "" : undefined}
      >
        {thread.messages.length === 0 && <EmptyThread />}
        {thread.messages.map((m, i) => {
          const prev = thread.messages[i - 1];
          const needsDay = !prev || !sameDay(prev.ts, m.ts);
          return (
            <div key={m.id} className={styles.msgGroup}>
              {needsDay && <DayDivider ts={m.ts} />}
              <Message m={m} prev={prev} mobile={mobile} onOpenProfile={onOpenProfile} />
            </div>
          );
        })}
        {thread.retryingGap && <GapBanner />}
      </div>
      {thread.pendingTofu && (
        <Tofu
          pending={thread.pendingTofu}
          onAccept={thread.acceptTofu}
          onCancel={thread.cancelTofu}
          mobile={mobile}
        />
      )}
      <Composer thread={thread} mobile={mobile} onOpenPanel={onOpenPanel} />
    </>
  );
}

// ---- header ----
interface HeaderProps {
  mobile: boolean;
  trust: TrustState;
  ai: AiState;
  inspectorOpen: boolean;
  onToggleInspector: () => void;
  railOpen: boolean;
  onToggleRail: () => void;
  onOpenPanel: (panel: PanelId) => void;
  onOpenProfile: (who: string) => void;
}

function Header({
  mobile,
  trust,
  ai,
  inspectorOpen,
  onToggleInspector,
  railOpen,
  onToggleRail,
  onOpenPanel,
  onOpenProfile,
}: HeaderProps) {
  return (
    <div className={styles.header} data-mobile={mobile ? "" : undefined}>
      <div className={styles.headerRow}>
        <TogglePill
          open={railOpen}
          onClick={onToggleRail}
          icon={<RailIcon size={12} />}
          label="rooms"
          tip={railOpen ? "Hide rooms" : "Show rooms"}
        />
        <span className={styles.hash}>#</span>
        <span className={styles.title}>mercury-core</span>
        <div style={{ flex: 1 }} />
        <Avatar person={MERCURY_PEOPLE.me} size={26} onClick={() => onOpenProfile("me")} />
        <button
          type="button"
          className={styles.gearBtn}
          onClick={() => onOpenPanel("settings")}
          data-tip="Settings"
          aria-label="Open settings"
        >
          <GearIcon size={13} />
        </button>
        <TogglePill
          open={inspectorOpen}
          onClick={onToggleInspector}
          icon={<Helix size={12} />}
          label="inspect"
          tip={inspectorOpen ? "Hide inspector" : "Show inspector"}
        />
      </div>
      <DataStrip trust={trust} ai={ai} onOpenPanel={onOpenPanel} />
    </div>
  );
}

function TogglePill({
  open,
  onClick,
  icon,
  label,
  tip,
}: {
  open: boolean;
  onClick: () => void;
  icon: ReactNode;
  label: string;
  tip: string;
}) {
  return (
    <button
      type="button"
      className={styles.pill}
      data-open={open ? "" : undefined}
      onClick={onClick}
      data-tip={tip}
      aria-label={tip}
      aria-expanded={open}
    >
      {open && <span className="iris-ring" style={{ borderRadius: 7 }} />}
      <span className={styles.pillIcon}>{icon}</span>
      <span style={{ position: "relative" }}>{label}</span>
      <span className={styles.pillChevron}>{open ? "v" : ">"}</span>
    </button>
  );
}

// ---- data strip ----
interface Cell {
  key: string;
  value: string;
  tone?: "ok" | "warn" | "bad" | "muted" | "irid";
  dot?: string;
  pulse?: boolean;
  title?: string;
  panel: PanelId;
}

function DataStrip({
  trust,
  ai,
  onOpenPanel,
}: {
  trust: TrustState;
  ai: AiState;
  onOpenPanel: (panel: PanelId) => void;
}) {
  const t = FU_TRUST[trust];
  const a = FU_AI[ai];
  const cells: Cell[] = [
    {
      key: "trust",
      value: t.label.toLowerCase().replace(" ", "_"),
      tone: t.tone,
      dot: toneVar(t.tone),
      pulse: true,
      title: t.sub,
      panel: "trust",
    },
    {
      key: "ai",
      value: ai,
      tone: ai === "granted" ? "irid" : a.tone === "bad" ? "bad" : "muted",
      title: a.sub,
      panel: "ai",
    },
    { key: "peers", value: "4", title: "Participants in this room", panel: "peers" },
    { key: "room", value: "epoch_42", title: "Membership and room epoch safety", panel: "group" },
    {
      key: "encryption",
      value: "end-to-end",
      title: "Messages are encrypted on your device. The server never sees plaintext.",
      panel: "encryption",
    },
    {
      key: "bootstrap",
      value: "ACCEPTED",
      tone: "ok",
      dot: toneVar("ok"),
      title: "Client is past the bootstrap gate.",
      panel: "bootstrap",
    },
    {
      key: "policy",
      value: "attested",
      tone: "ok",
      dot: toneVar("ok"),
      title: "Explain the active mercury-core and Helix policy gates.",
      panel: "policy",
    },
  ];

  return (
    <div className={`${styles.dataStrip} mono`}>
      {cells.map((cell) => (
        <DataCell key={cell.key} cell={cell} pulseKey={cell.key === "trust" ? trust : undefined} onClick={() => onOpenPanel(cell.panel)} />
      ))}
    </div>
  );
}

function DataCell({
  cell,
  pulseKey,
  onClick,
}: {
  cell: Cell;
  pulseKey?: string;
  onClick: () => void;
}) {
  const isIrid = cell.tone === "irid";
  const valColor =
    cell.tone === "ok"
      ? toneVar("ok")
      : cell.tone === "warn"
        ? toneVar("warn")
        : cell.tone === "bad"
          ? toneVar("bad")
          : cell.tone === "muted"
            ? toneVar("muted")
            : toneVar("ink");

  return (
    <button
      type="button"
      className={styles.cell}
      data-irid={isIrid ? "" : undefined}
      onClick={onClick}
      data-tip={cell.title}
    >
      {isIrid && <span className="iris-ring" style={{ borderRadius: 7 }} />}
      {cell.dot && (
        <span
          key={pulseKey}
          className={cell.pulse ? "pulse-dot" : undefined}
          style={{ width: 5, height: 5, borderRadius: 999, background: cell.dot, position: "relative" }}
        />
      )}
      <span className={styles.cellKey}>{cell.key}</span>
      <span className={styles.cellEq}>=</span>
      {isIrid ? (
        <span className="iris-text" style={{ fontWeight: 600, position: "relative" }}>
          {cell.value}
        </span>
      ) : (
        <span style={{ color: valColor, fontWeight: 600, position: "relative" } as CSSProperties}>
          {cell.value}
        </span>
      )}
      <span className={styles.cellArrow}>{">"}</span>
    </button>
  );
}

// ---- day divider + empty ----
function DayDivider({ ts }: { ts: number }) {
  return (
    <div className={styles.dayDivider}>
      <span className={styles.rule} />
      <span className={styles.dayLabel}>{fmtDay(ts)}</span>
      <span className={styles.rule} />
    </div>
  );
}

function EmptyThread() {
  return (
    <div className={styles.empty}>
      <div className={styles.emptyIcon}>
        <span className="iris-ring" style={{ borderRadius: "50%" }} />
        <RefreshIcon size={22} color="var(--mc-ai)" className={styles.emptyGlyph} />
      </div>
      <div className={styles.emptyTitle}>No messages yet</div>
      <div className={styles.emptyBody}>
        This room is end-to-end encrypted and gated by mercury-core. Your first message will start
        the thread.
      </div>
    </div>
  );
}
