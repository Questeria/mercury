import { useEffect, useState } from "react";

import { toneVar } from "../../strings";
import { PanelShell } from "./PanelShell";
import { GhostButton, IridButton, ProgressBar, Section } from "./primitives";
import styles from "./panels.module.css";

interface SyncPanelProps {
  mobile: boolean;
  onClose: () => void;
  onComplete: () => void;
}

interface Room {
  id: string;
  name: string;
  value: number;
  priority: number;
  note: string;
}

function initRooms(): Room[] {
  return [
    { id: "core", name: "mercury-core", value: 12, priority: 1.4, note: "8 messages behind" },
    { id: "sec", name: "security-review", value: 4, priority: 1.1, note: "23 messages behind" },
    { id: "eng", name: "eng-leads", value: 0, priority: 0.9, note: "142 messages behind" },
    { id: "ai", name: "ai-policy", value: 32, priority: 1.6, note: "2 messages behind" },
  ];
}

export function SyncPanel({ mobile, onClose, onComplete }: SyncPanelProps) {
  const [rooms, setRooms] = useState<Room[]>(() => initRooms());

  useEffect(() => {
    const handle = window.setInterval(() => {
      setRooms((R) =>
        R.map((r) => {
          if (r.value >= 100) return r;
          const inc = r.priority * (Math.random() * 6 + 1);
          return { ...r, value: Math.min(100, r.value + inc) };
        }),
      );
    }, 280);
    return () => window.clearInterval(handle);
  }, []);

  const allDone = rooms.every((r) => r.value >= 100);
  useEffect(() => {
    if (!allDone) return;
    const handle = window.setTimeout(() => onComplete(), 600);
    return () => window.clearTimeout(handle);
  }, [allDone, onComplete]);

  const total = Math.round(rooms.reduce((s, r) => s + r.value, 0) / rooms.length);

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Sync"
      title={allDone ? "Sync complete" : "Finishing sync"}
      footer={
        <>
          <GhostButton onClick={onClose}>Continue in background</GhostButton>
          {allDone && (
            <IridButton
              onClick={() => {
                onComplete();
                onClose();
              }}
            >
              Open Mercury
            </IridButton>
          )}
        </>
      }
    >
      <div style={{ fontSize: 12.5, color: "var(--mc-ink2)", lineHeight: 1.55, marginBottom: 14 }}>
        Catching up missed messages and replay checkpoints for each room. The message UI stays closed
        until the bootstrap gate accepts — no stale timelines.
      </div>
      <ProgressBar value={total} label={`Total · ${rooms.length} rooms`} />
      <div style={{ height: 16 }} />
      <Section kicker="Per-room progress">
        <div className={styles.colGap}>
          {rooms.map((r) => {
            const done = r.value >= 100;
            return (
              <div key={r.id} className={styles.card}>
                <div style={{ display: "flex", alignItems: "baseline", marginBottom: 6, fontSize: 12 }}>
                  <span style={{ color: "var(--mc-ink)", fontWeight: 600 }}># {r.name}</span>
                  <span style={{ color: "var(--mc-muted)", marginLeft: 8, fontSize: 10.5 }}>
                    {done ? "caught up" : `${Math.round(r.value)}% · ${r.note}`}
                  </span>
                  <span style={{ marginLeft: "auto", fontSize: 10.5, color: done ? toneVar("ok") : "var(--mc-muted)" }}>
                    {done ? "✓" : ""}
                  </span>
                </div>
                <div style={{ height: 4, background: "var(--mc-surface-mute)", borderRadius: 3, overflow: "hidden" }}>
                  <div
                    style={{
                      width: `${r.value}%`,
                      height: "100%",
                      background: done ? toneVar("ok") : toneVar("warn"),
                      transition: "width 0.25s ease",
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </Section>
    </PanelShell>
  );
}
