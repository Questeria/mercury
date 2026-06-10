import { MERCURY_PEOPLE } from "../../simulator";
import { toneVar } from "../../strings";
import type { AiState, TrustState } from "../../types";
import { Avatar } from "../Avatar";
import { PanelShell } from "./PanelShell";
import { GhostButton, IridButton } from "./primitives";
import styles from "./panels.module.css";

interface PeersPanelProps {
  mobile: boolean;
  onClose: () => void;
  trust: TrustState;
  ai: AiState;
  onOpenTrust: () => void;
  onOpenProfile: (who: string) => void;
}

export function PeersPanel({ mobile, onClose, trust, ai, onOpenTrust, onOpenProfile }: PeersPanelProps) {
  const peers = [
    { id: "me", devices: 1, role: "admin · this device", tone: "ok" as const, added: "today" },
    { id: "rin", devices: 2, role: "verified · iPhone + MacBook", tone: "ok" as const, added: "12d ago" },
    {
      id: "jules",
      devices: 2,
      role: trust === "unverified" ? "verified · 1 device pending" : "verified · 2 devices",
      tone: (trust === "unverified" ? "warn" : "ok") as "warn" | "ok",
      added: "32d ago",
    },
    {
      id: "ai",
      devices: 0,
      role: ai === "granted" ? "AI · scoped read-only" : `AI · ${ai}`,
      tone: (ai === "granted" ? "irid" : "muted") as "irid" | "muted",
      added: "today",
    },
  ];

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Participants"
      title="People in mercury-core"
      footer={
        <>
          <GhostButton onClick={onClose}>Close</GhostButton>
          <IridButton
            onClick={() => {
              onClose();
              onOpenTrust();
            }}
          >
            Verify devices
          </IridButton>
        </>
      }
    >
      <div className={styles.colGap}>
        {peers.map((pe) => {
          const p = MERCURY_PEOPLE[pe.id];
          const isAi = !!p.isAi;
          const toneColor =
            pe.tone === "ok"
              ? toneVar("ok")
              : pe.tone === "warn"
                ? toneVar("warn")
                : pe.tone === "irid"
                  ? "var(--mc-ai)"
                  : toneVar("muted");
          return (
            <button
              key={pe.id}
              type="button"
              onClick={() => onOpenProfile(pe.id)}
              className={styles.navRow}
              style={{ position: "relative", overflow: "hidden", gap: 12 }}
            >
              {pe.tone === "irid" && <span className="iris-ring" style={{ borderRadius: 10 }} />}
              <Avatar person={p} size={32} />
              <div style={{ minWidth: 0, flex: 1, position: "relative" }}>
                <div className={isAi ? "iris-text" : undefined} style={{ fontSize: 13, fontWeight: 600, color: "var(--mc-ink)" }}>
                  {p.name}
                </div>
                <div style={{ fontSize: 11, color: "var(--mc-muted)", marginTop: 1 }}>{pe.role}</div>
              </div>
              <div style={{ textAlign: "right", position: "relative" }}>
                <div style={{ fontSize: 11, color: toneColor, fontWeight: 600 }}>
                  {pe.devices === 0 ? "—" : `${pe.devices} device${pe.devices > 1 ? "s" : ""}`}
                </div>
                <div style={{ fontSize: 10, color: "var(--mc-muted)", marginTop: 1 }}>added {pe.added}</div>
              </div>
              <span className={styles.chev} style={{ position: "relative", fontSize: 12, marginLeft: 4 }}>
                ›
              </span>
            </button>
          );
        })}
      </div>
    </PanelShell>
  );
}
