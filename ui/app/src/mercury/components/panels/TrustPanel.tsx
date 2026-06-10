// Trust: the per-conversation safety-number + verified-mark panel.
//
// LIVE mode (a `controller` prop is present — the live app): shows the REAL mutual safety number
// for the ACTIVE conversation (same backend call the conversation menu uses) and the REAL persisted
// verified flag; "Mark as verified" records YOUR out-of-band comparison via the backend. It grants
// nothing and gates nothing — it is an honest bookmark of an attestation you made.
//
// DEMO mode (no `controller` — the simulator app): the original mockup content, unchanged.

import { useEffect, useState } from "react";

import { MERCURY_PEOPLE } from "../../simulator";
import { FU_TRUST, toneVar } from "../../strings";
import type { LiveConversations } from "../../liveConversations";
import type { Person, TrustState } from "../../types";
import { Avatar } from "../Avatar";
import { PanelShell } from "./PanelShell";
import { GhostButton, HistoryRow, IridButton, ScopeTile, Section, StatusRow } from "./primitives";
import styles from "./panels.module.css";

interface TrustPanelProps {
  mobile: boolean;
  onClose: () => void;
  /** DEMO (simulator) props — used only when `controller` is absent. */
  trust?: TrustState;
  onMarkVerified?: () => void;
  /** LIVE props — presence of `controller` selects the live, backend-wired panel. */
  controller?: LiveConversations;
  activePeer?: { peer: string; person: Person } | null;
  verified?: boolean;
}

export function TrustPanel(props: TrustPanelProps) {
  if (props.controller) {
    return (
      <LiveTrustPanel
        mobile={props.mobile}
        onClose={props.onClose}
        controller={props.controller}
        activePeer={props.activePeer ?? null}
        verified={props.verified ?? false}
      />
    );
  }
  return (
    <DemoTrustPanel
      mobile={props.mobile}
      onClose={props.onClose}
      trust={props.trust ?? "trusted"}
      onMarkVerified={props.onMarkVerified ?? (() => {})}
    />
  );
}

// ---------------------------------------------------------------- live (real backend)

function LiveTrustPanel({
  mobile,
  onClose,
  controller,
  activePeer,
  verified,
}: {
  mobile: boolean;
  onClose: () => void;
  controller: LiveConversations;
  activePeer: { peer: string; person: Person } | null;
  verified: boolean;
}) {
  const [grouped, setGrouped] = useState<string | null>(null);
  const [snError, setSnError] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!activePeer) return;
    let cancelled = false;
    setGrouped(null);
    setSnError(false);
    void (async () => {
      const sn = await controller.safetyNumber(activePeer.peer);
      if (cancelled) return;
      if (sn) setGrouped(sn.grouped);
      else setSnError(true);
    })();
    return () => {
      cancelled = true;
    };
  }, [controller, activePeer]);

  if (!activePeer) {
    return (
      <PanelShell mobile={mobile} onClose={onClose} eyebrow="Trust" title="Verify a conversation">
        <div className={styles.card} style={{ fontSize: 12.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          No conversation is open. Open one, then come back here to compare its safety number and
          mark it verified.
        </div>
      </PanelShell>
    );
  }

  const peer = activePeer.peer;
  const setMark = (on: boolean) => {
    setBusy(true);
    setError(null);
    void (async () => {
      try {
        await controller.setVerified(peer, on);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setBusy(false);
      }
    })();
  };

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Trust"
      title={`Verify ${activePeer.person.name}`}
      footer={
        <>
          <GhostButton onClick={onClose}>Done</GhostButton>
          {verified ? (
            <GhostButton danger onClick={() => !busy && setMark(false)}>
              {busy ? "Working…" : "Remove verification"}
            </GhostButton>
          ) : (
            <IridButton disabled={busy || grouped === null} onClick={() => setMark(true)}>
              {busy ? "Working…" : "Mark as verified"}
            </IridButton>
          )}
        </>
      }
    >
      <StatusRow
        tone={verified ? "ok" : "warn"}
        label={verified ? "You marked this contact verified" : "Not verified yet"}
        sub={verified ? "your recorded attestation" : "compare the safety number first"}
      />

      <Section kicker="Safety number">
        <div className={styles.card}>
          <div
            className="mono"
            style={{
              fontSize: 14,
              letterSpacing: 1.5,
              lineHeight: 1.7,
              wordBreak: "break-word",
              color: "var(--mc-ink)",
              fontWeight: 600,
            }}
          >
            {grouped ?? (snError ? "Unavailable — the backend could not compute it for this contact." : "Computing…")}
          </div>
          <div style={{ fontSize: 11, color: "var(--mc-muted)", marginTop: 10, lineHeight: 1.5 }}>
            This number is derived from both sides&apos; keys. Compare it with{" "}
            {activePeer.person.name} <strong>in person</strong> or over a call you recognize. If it
            matches on both devices, no one is sitting between you.
          </div>
        </div>
      </Section>

      <Section kicker="What the verified mark means">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          Marking this contact verified records <strong>your own attestation</strong> that you
          compared the numbers over a channel you trust. It is saved by the backend and shown as a
          &quot;✓ verified&quot; mark on the conversation — but it grants nothing and blocks
          nothing. Encryption is identical either way; the mark only documents that you checked.
        </div>
        {error && (
          <div style={{ marginTop: 8, fontSize: 11.5, color: "var(--mc-warn)", lineHeight: 1.45 }}>
            {error}
          </div>
        )}
      </Section>
    </PanelShell>
  );
}

// ---------------------------------------------------------------- demo (simulator mockup)

const SN = [
  "a1f3", "9c20", "8e4b", "7d12", "6b8e", "0a35",
  "c7f1", "3b29", "e8da", "4501", "9f63", "2b77",
];

type DeviceState = "verified" | "unverified";

function DemoTrustPanel({
  mobile,
  onClose,
  trust,
  onMarkVerified,
}: {
  mobile: boolean;
  onClose: () => void;
  trust: TrustState;
  onMarkVerified: () => void;
}) {
  const t = FU_TRUST[trust];
  const groups = [
    {
      who: "rin",
      name: "Rin Hadley",
      list: [
        { dev: "a1f3", added: "12d ago", state: "verified" as DeviceState },
        { dev: "8e4b", added: "4d ago", state: "verified" as DeviceState },
      ],
    },
    {
      who: "jules",
      name: "Jules Okafor",
      list: [
        { dev: "b2e7", added: "32d ago", state: "verified" as DeviceState },
        {
          dev: "c4d9",
          added: "2h ago",
          state: (trust === "unverified" ? "unverified" : "verified") as DeviceState,
          fresh: trust === "unverified",
        },
      ],
    },
    { who: "me", name: "You", list: [{ dev: "a1f3", added: "this device", state: "verified" as DeviceState }] },
  ];
  const hasUnverified = groups.some((g) => g.list.some((x) => x.state === "unverified"));

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Trust"
      title="Verify the people in this room"
      footer={
        <>
          <GhostButton onClick={onClose}>Done</GhostButton>
          {hasUnverified && <IridButton onClick={onMarkVerified}>Mark all verified</IridButton>}
        </>
      }
    >
      <StatusRow tone={t.tone} label={t.label} sub={t.sub} />

      <Section kicker="Room safety number">
        <div className={styles.card}>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "6px 12px",
              fontSize: 13,
              letterSpacing: 1,
              color: "var(--mc-ink)",
              fontWeight: 600,
            }}
          >
            {SN.map((g, i) => (
              <span key={i} style={{ textAlign: "center" }}>
                {g}
              </span>
            ))}
          </div>
          <div style={{ fontSize: 11, color: "var(--mc-muted)", marginTop: 10, lineHeight: 1.5 }}>
            Compare these groups by voice, in person, or by scanning a QR code on the other device.
            If they match on both sides, the conversation is verified end-to-end.
          </div>
        </div>
      </Section>

      <Section kicker="QR compare">
        <div className={styles.featureGrid}>
          <QrTile label="Your code" />
          <div className={styles.card} style={{ display: "flex", flexDirection: "column", justifyContent: "center", gap: 8 }}>
            <ScopeTile label="Method" value="mutual scan" note="no server trust" />
            <ScopeTile label="Mismatch" value="block send" tone="bad" note="until confirmed" />
          </div>
        </div>
      </Section>

      <Section kicker="Devices in this room">
        <div className={styles.colGap}>
          {groups.map((group) => {
            const p = MERCURY_PEOPLE[group.who as keyof typeof MERCURY_PEOPLE];
            return (
              <div key={group.who} className={styles.card}>
                <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 8 }}>
                  <Avatar person={p} size={24} />
                  <span style={{ fontSize: 12.5, fontWeight: 600, color: "var(--mc-ink)" }}>{group.name}</span>
                  <span style={{ marginLeft: "auto", fontSize: 10.5, color: "var(--mc-muted)" }}>
                    {group.list.length} device{group.list.length === 1 ? "" : "s"}
                  </span>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                  {group.list.map((d) => (
                    <div
                      key={d.dev}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 8,
                        padding: "6px 8px",
                        borderRadius: 6,
                        background: "var(--mc-surface)",
                        fontSize: 11.5,
                      }}
                    >
                      <span
                        className={styles.dot6}
                        style={{ background: d.state === "verified" ? toneVar("ok") : toneVar("warn") }}
                      />
                      <span style={{ color: "var(--mc-ink2)", fontWeight: 600 }}>device {d.dev}</span>
                      <span style={{ color: "var(--mc-muted)", marginLeft: "auto" }}>{d.added}</span>
                      {"fresh" in d && d.fresh && (
                        <span style={{ color: "var(--mc-warn)", fontWeight: 600 }}>new</span>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </Section>

      <Section kicker="Key transparency">
        <div className={styles.colGap}>
          <div className={styles.card} style={{ fontSize: 12, color: "var(--mc-ink2)", lineHeight: 1.5 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 4 }}>
              <span className={styles.dot6} style={{ background: toneVar("ok") }} />
              <span style={{ color: "var(--mc-ink)", fontWeight: 600 }}>All keys consistent</span>
              <span style={{ marginLeft: "auto", color: "var(--mc-muted)", fontSize: 11 }}>checked 12s ago</span>
            </div>
            Mercury checks every device key against the global transparency log. A key change without a
            matching log entry blocks sends until you confirm.
          </div>
          <HistoryRow when="12s ago" by="kt" event="inclusion proof verified for Jules c4d9" tone="ok" />
          <HistoryRow when="2h ago" by="tofu" event="new device c4d9 entered first-use verification" tone="warn" />
          <HistoryRow when="32d ago" by="kt" event="Jules b2e7 checkpoint anchored" tone="ok" />
        </div>
      </Section>
    </PanelShell>
  );
}

function QrTile({ label }: { label: string }) {
  const cells = Array.from({ length: 49 }, (_, i) => (i * 17 + 11) % 5 !== 0);
  return (
    <div className={styles.card} style={{ display: "flex", alignItems: "center", gap: 12 }}>
      <div
        aria-label={label}
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(7, 8px)",
          gap: 2,
          padding: 8,
          borderRadius: 8,
          background: "var(--mc-surface)",
          border: "1px solid var(--mc-border)",
        }}
      >
        {cells.map((on, i) => (
          <span key={i} style={{ width: 8, height: 8, borderRadius: 2, background: on ? "var(--mc-ink)" : "transparent" }} />
        ))}
      </div>
      <div style={{ minWidth: 0 }}>
        <div style={{ color: "var(--mc-ink)", fontWeight: 700, fontSize: 12.5 }}>{label}</div>
        <div style={{ color: "var(--mc-muted)", fontSize: 11, lineHeight: 1.45, marginTop: 3 }}>
          Encodes the same safety number and current room epoch.
        </div>
      </div>
    </div>
  );
}
