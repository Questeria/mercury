import type { CSSProperties, ReactNode } from "react";

import { toneVar, type ToneKey } from "../../strings";
import styles from "./panels.module.css";

type PanelTone = ToneKey | "irid";

function toneColor(tone?: PanelTone): string {
  if (tone === "irid") return "var(--mc-ai)";
  return toneVar((tone ?? "ink") as ToneKey);
}

export function GhostButton({
  onClick,
  danger,
  children,
}: {
  onClick: () => void;
  danger?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      className={`${styles.btnGhost} ${danger ? styles.btnDanger : ""}`}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

export function IridButton({
  onClick,
  disabled,
  children,
}: {
  onClick: () => void;
  disabled?: boolean;
  children: ReactNode;
}) {
  return (
    <button type="button" className={styles.btnIrid} onClick={onClick} disabled={disabled}>
      <span className="iris-ring" style={{ borderRadius: 8 }} />
      <span style={{ position: "relative" }}>{children}</span>
    </button>
  );
}

export function Section({ kicker, children }: { kicker: string; children: ReactNode }) {
  return (
    <div className={styles.section}>
      <div className={styles.kicker}>{kicker}</div>
      {children}
    </div>
  );
}

export function StatusRow({ tone, label, sub }: { tone: PanelTone; label: string; sub: string }) {
  return (
    <div className={styles.statusRow}>
      <span className={styles.dot8} style={{ background: toneColor(tone) }} />
      <span className={styles.statusLabel}>{label}</span>
      <span className={styles.statusSub}>/ {sub}</span>
    </div>
  );
}

export function ScopeTile({
  label,
  value,
  note,
  tone,
}: {
  label: string;
  value: string;
  note?: string;
  tone?: PanelTone;
}) {
  return (
    <div className={styles.scopeTile}>
      <div className={styles.scopeLabel}>{label}</div>
      <div className={styles.scopeValue} style={{ color: toneColor(tone) }}>
        {value}
      </div>
      {note && <div className={styles.scopeNote}>{note}</div>}
    </div>
  );
}

export function HistoryRow({
  when,
  by,
  event,
  tone,
}: {
  when: string;
  by: string;
  event: string;
  tone?: PanelTone;
}) {
  return (
    <div className={styles.historyRow}>
      <span className={styles.dot4} style={{ background: toneColor(tone ?? "muted"), alignSelf: "center" }} />
      <span className={styles.histWhen}>{when}</span>
      <span className={styles.histBy}>{by}</span>
      <span className={styles.histEvent}>{event}</span>
    </div>
  );
}

export function ProgressBar({
  value,
  label,
  tone = "ok",
}: {
  value: number;
  label: string;
  tone?: PanelTone;
}) {
  const v = Math.max(0, Math.min(100, value));
  return (
    <div>
      <div className={styles.progLabel}>
        <span style={{ color: "var(--mc-ink2)" }}>{label}</span>
        <span style={{ color: "var(--mc-muted)" }}>{Math.round(v)}%</span>
      </div>
      <div className={styles.progTrack}>
        <div className={styles.progFill} style={{ width: `${v}%`, background: toneColor(tone) }} />
      </div>
    </div>
  );
}

export function Stepper({ step, steps }: { step: number; steps: string[] }) {
  return (
    <div className={styles.stepper}>
      {steps.map((s, i) => {
        const n = i + 1;
        const active = n === step;
        const done = n < step;
        const badgeStyle: CSSProperties = {
          background: done ? "var(--mc-ok)" : active ? "var(--mc-surface)" : "var(--mc-surface-warm)",
          color: done ? "#fff" : active ? "var(--mc-ink)" : "var(--mc-muted)",
          border: active ? "1px solid var(--mc-ink2)" : "1px solid var(--mc-border)",
        };
        return (
          <span key={s} style={{ display: "contents" }}>
            <div className={styles.stepItem}>
              <span className={styles.stepBadge} style={badgeStyle}>
                {done ? "ok" : n}
              </span>
              <span
                className={styles.stepLabel}
                style={{ fontWeight: active ? 600 : 500, color: active ? "var(--mc-ink)" : "var(--mc-muted)" }}
              >
                {s}
              </span>
            </div>
            {i < steps.length - 1 && <span className={styles.stepLine} />}
          </span>
        );
      })}
    </div>
  );
}

export interface SegmentOption<T extends string> {
  value: T;
  label: string;
  sub?: string;
}

export function Segment<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: SegmentOption<T>[];
  onChange: (v: T) => void;
}) {
  return (
    <div
      className={styles.segment}
      role="radiogroup"
      style={{ gridTemplateColumns: `repeat(${options.length}, 1fr)` }}
    >
      {options.map((opt) => {
        const active = opt.value === value;
        return (
          <button
            key={opt.value}
            type="button"
            role="radio"
            aria-checked={active}
            data-on={active ? "" : undefined}
            className={styles.seg}
            onClick={() => onChange(opt.value)}
          >
            {active && <span className="iris-ring" style={{ borderRadius: 10 }} />}
            <div className={styles.segLabel}>{opt.label}</div>
            {opt.sub && <div className={styles.segSub}>{opt.sub}</div>}
          </button>
        );
      })}
    </div>
  );
}

export function Kv({ k, v }: { k: string; v: ReactNode }) {
  return (
    <div className={styles.kv}>
      <span className={styles.kvK}>{k}</span>
      <span className={styles.kvV}>{v}</span>
    </div>
  );
}

export function Bullet({ tone, text }: { tone?: PanelTone; text: string }) {
  return (
    <div className={styles.bullet}>
      <span className={styles.bulletDot} style={{ background: toneColor(tone ?? "muted") }} />
      <span style={{ color: "var(--mc-ink)" }}>{text}</span>
    </div>
  );
}

export function ToggleRow({
  label,
  sub,
  value,
  onChange,
  divider,
}: {
  label: string;
  sub?: string;
  value: boolean;
  onChange: (v: boolean) => void;
  divider?: boolean;
}) {
  return (
    <button
      type="button"
      className={`${styles.toggleRow} ${divider ? styles.divider : ""}`}
      role="switch"
      aria-checked={value}
      aria-label={label}
      onClick={() => onChange(!value)}
    >
      <div style={{ minWidth: 0, flex: 1 }}>
        <div className={styles.rowLabel}>{label}</div>
        {sub && <div className={styles.rowSub}>{sub}</div>}
      </div>
      <span className={styles.switch} data-on={value ? "" : undefined}>
        <span className={styles.knob} />
      </span>
    </button>
  );
}

export function NavRow({
  label,
  sub,
  onClick,
  divider,
}: {
  label: string;
  sub: string;
  onClick: () => void;
  divider?: boolean;
}) {
  return (
    <button
      type="button"
      className={styles.navRow}
      style={divider ? { marginTop: 6 } : undefined}
      onClick={onClick}
    >
      <div style={{ minWidth: 0, flex: 1 }}>
        <div className={styles.rowLabel}>{label}</div>
        <div className={styles.rowSub}>{sub}</div>
      </div>
      <span className={styles.chev}>{">"}</span>
    </button>
  );
}

export function DeviceRow({
  name,
  id,
  added,
  tone = "ok",
  badge,
  current,
  action,
}: {
  name: string;
  id: string;
  added: string;
  tone?: PanelTone;
  badge?: string;
  current?: boolean;
  action?: string;
}) {
  return (
    <div className={styles.deviceRow}>
      <span className={styles.dot6} style={{ background: toneColor(tone) }} />
      <div style={{ minWidth: 0, flex: 1 }}>
        <div style={{ color: "var(--mc-ink)", fontWeight: 600, fontSize: 12 }}>
          {name}
          {badge && (
            <span className={styles.tag} style={{ color: "var(--mc-warn)" }}>
              {badge}
            </span>
          )}
          {current && (
            <span className={styles.tag} style={{ color: "var(--mc-ok)" }}>
              current
            </span>
          )}
        </div>
        <div style={{ color: "var(--mc-muted)", fontSize: 10.5, marginTop: 1 }}>
          device {id} / {added}
        </div>
      </div>
      {action && !current && (
        <button type="button" className={styles.btnGhost} style={{ fontSize: 10.5, padding: "4px 9px", color: "var(--mc-muted)" }}>
          {action}
        </button>
      )}
    </div>
  );
}

export function SharedRoom({ name, last }: { name: string; last: string }) {
  return (
    <div className={styles.sharedRoom}>
      <span style={{ color: "var(--mc-muted)", fontSize: 10.5 }}>#</span>
      <span style={{ color: "var(--mc-ink)", fontWeight: 600 }}>{name}</span>
      <span style={{ marginLeft: "auto", color: "var(--mc-muted)", fontSize: 10.5 }}>last / {last}</span>
    </div>
  );
}

export interface LockSample {
  kind: "accepted" | "tofu" | "rejected";
  who: string;
  preview: string;
  tone: PanelTone;
  when: string;
}

export function LockNotification({ s, previews }: { s: LockSample; previews: boolean }) {
  const isHeld = s.kind !== "accepted";
  return (
    <div className={styles.lockNotif}>
      <div className={styles.lockIcon}>
        <span className="iris-ring" style={{ borderRadius: 6 }} />
        <span className={styles.dot6} style={{ background: toneColor(s.tone), position: "relative" }} />
      </div>
      <div style={{ minWidth: 0, flex: 1 }}>
        <div style={{ display: "flex", alignItems: "baseline", gap: 8 }}>
          <span style={{ fontSize: 11.5, fontWeight: 600, color: "var(--mc-ink)" }}>Mercury</span>
          <span style={{ fontSize: 10.5, color: "var(--mc-muted)" }}>/ {s.who}</span>
          <span style={{ marginLeft: "auto", fontSize: 10.5, color: "var(--mc-muted)" }}>{s.when}</span>
        </div>
        <div
          style={{
            fontSize: 12,
            color: isHeld ? "var(--mc-muted)" : "var(--mc-ink2)",
            marginTop: 2,
            lineHeight: 1.4,
            fontStyle: isHeld ? "italic" : "normal",
          }}
        >
          {previews ? s.preview : "New message"}
        </div>
      </div>
    </div>
  );
}
