import { explainDecision, toneVar } from "../../strings";
import type { PlatformDecisionView } from "../../types";
import { PanelShell } from "./PanelShell";
import { GhostButton, HistoryRow, IridButton, Section } from "./primitives";
import styles from "./panels.module.css";

interface BootstrapPanelProps {
  mobile: boolean;
  onClose: () => void;
  view: PlatformDecisionView;
}

const FLAGS: (keyof PlatformDecisionView)[] = [
  "can_open_message_ui",
  "can_start_sync",
  "requires_sync",
  "requires_recovery",
];

export function BootstrapPanel({ mobile, onClose, view }: BootstrapPanelProps) {
  const ok = view.accepted;
  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Bootstrap"
      title="Client bootstrap · gate decision"
      footer={
        <>
          <GhostButton onClick={onClose}>Close</GhostButton>
          <IridButton onClick={onClose}>Re-run bootstrap</IridButton>
        </>
      }
    >
      <div
        style={{
          padding: "12px 14px",
          marginBottom: 14,
          background: ok ? "var(--mc-ok-soft)" : "var(--mc-bad-soft)",
          border: `1px solid color-mix(in srgb, ${ok ? "var(--mc-ok)" : "var(--mc-bad)"} 27%, transparent)`,
          borderRadius: 10,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4 }}>
          <span className={styles.dot8} style={{ background: ok ? toneVar("ok") : toneVar("bad") }} />
          <span
            style={{
              fontWeight: 700,
              fontSize: 11,
              letterSpacing: 1,
              textTransform: "uppercase",
              color: ok ? toneVar("ok") : toneVar("bad"),
            }}
          >
            {ok ? "Accepted" : "Rejected"}
          </span>
          <span style={{ marginLeft: "auto", fontSize: 10.5, color: "var(--mc-muted)" }}>rc={view.reason_code}</span>
        </div>
        <div style={{ fontSize: 13, color: "var(--mc-ink)", fontWeight: 600, marginBottom: 2 }}>{view.reason_label}</div>
        <div style={{ fontSize: 12, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          {explainDecision(view, "client_bootstrap")}
        </div>
      </div>

      <Section kicker="Effects">
        <div className={styles.card}>
          {FLAGS.map((f) => {
            const v = view[f];
            if (typeof v !== "boolean") return null;
            const isReq = String(f).startsWith("requires_");
            const color = v ? (isReq ? toneVar("warn") : toneVar("ok")) : "var(--mc-dim)";
            return (
              <div key={String(f)} style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 11.5, padding: "3px 0" }}>
                <span className={styles.dot5} style={{ background: color }} />
                <span style={{ color: "var(--mc-ink3)" }}>{String(f)}</span>
                <span style={{ color, fontWeight: v ? 600 : 400, marginLeft: "auto" }}>{String(v)}</span>
              </div>
            );
          })}
        </div>
      </Section>

      <Section kicker="History">
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <HistoryRow when="today, 09:12" by="binding" event="bootstrap accepted" tone="ok" />
          <HistoryRow when="today, 09:11" by="client" event="bootstrap started" tone="muted" />
          <HistoryRow when="3d ago, 14:02" by="binding" event="sync resumed · 23 messages caught up" tone="muted" />
        </div>
      </Section>

      <Section kicker="About this gate">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          The bootstrap gate is mercury-core's first decision. The UI cannot open the message surface,
          decrypt local timelines, or render notifications until this view returns{" "}
          <span style={{ color: "var(--mc-ok)" }}>can_open_message_ui = true</span>. The check is
          per-client, not per-room.
        </div>
      </Section>
    </PanelShell>
  );
}
