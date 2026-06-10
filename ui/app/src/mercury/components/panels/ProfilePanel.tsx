import { MERCURY_PEOPLE } from "../../simulator";
import { toneVar } from "../../strings";
import type { AiState, TrustState } from "../../types";
import { Avatar } from "../Avatar";
import { PanelShell } from "./PanelShell";
import {
  DeviceRow,
  GhostButton,
  IridButton,
  Kv,
  NavRow,
  ScopeTile,
  Section,
  SharedRoom,
} from "./primitives";
import styles from "./panels.module.css";

interface ProfilePanelProps {
  mobile: boolean;
  onClose: () => void;
  who: string;
  trust: TrustState;
  ai: AiState;
  onOpenTrust: () => void;
  onOpenAi: () => void;
  onOpenNotifications: () => void;
}

export function ProfilePanel({
  mobile,
  onClose,
  who,
  trust,
  ai,
  onOpenTrust,
  onOpenAi,
  onOpenNotifications,
}: ProfilePanelProps) {
  const p = MERCURY_PEOPLE[who];
  if (!p) return null;
  const isMe = who === "me";
  const isAi = !!p.isAi;

  let statusTone = "ok";
  let statusLabel = "Verified";
  let statusSub = "safety number matched";
  if (isMe) {
    statusLabel = "This device";
    statusSub = "device a1f3 · trusted";
  } else if (isAi) {
    if (ai === "granted") {
      statusLabel = "Scoped grant active";
      statusSub = "24h · read-only";
    } else if (ai === "revoked") {
      statusLabel = "Access revoked";
      statusSub = "no current grant";
      statusTone = "bad";
    } else if (ai === "expired") {
      statusLabel = "Grant expired";
      statusSub = "request a new one";
      statusTone = "muted";
    } else {
      statusLabel = "No grant";
      statusSub = "@ai mentions held";
      statusTone = "muted";
    }
  } else if (who === "jules" && trust === "unverified") {
    statusLabel = "New device";
    statusSub = "first-use verification pending";
    statusTone = "warn";
  }

  const footer = (
    <>
      <GhostButton onClick={onClose}>Close</GhostButton>
      {isMe && (
        <GhostButton danger onClick={onClose}>
          Sign out this device
        </GhostButton>
      )}
      {!isMe && !isAi && (
        <IridButton
          onClick={() => {
            onClose();
            onOpenTrust();
          }}
        >
          Verify in person
        </IridButton>
      )}
      {isAi && (
        <IridButton
          onClick={() => {
            onClose();
            onOpenAi();
          }}
        >
          Manage grant
        </IridButton>
      )}
    </>
  );

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow={isMe ? "Your profile" : isAi ? "AI participant" : "Profile"}
      title={isMe ? "You" : p.name}
      footer={footer}
    >
      <div className={styles.heroBox}>
        {isAi && <span className="iris-ring" style={{ borderRadius: 12 }} />}
        <Avatar person={p} size={56} />
        <div style={{ minWidth: 0, flex: 1, position: "relative" }}>
          <div
            className={isAi ? "iris-text display" : "display"}
            style={{ fontSize: 17, fontWeight: 700, letterSpacing: "-0.014em", color: isAi ? undefined : "var(--mc-ink)", marginBottom: 2 }}
          >
            {isMe ? "You" : p.name}
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 11.5, color: "var(--mc-ink2)" }}>
            <span className={styles.dot6} style={{ background: toneVar(statusTone as "ok") }} />
            <span style={{ fontWeight: 600 }}>{statusLabel}</span>
            <span style={{ color: "var(--mc-muted)" }}>· {statusSub}</span>
          </div>
        </div>
      </div>

      {isMe && (
        <>
          <Section kicker="Your devices">
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              <DeviceRow name="iPhone · this device" id="a1f3" added="this device" tone="ok" current />
              <DeviceRow name="MacBook" id="8e4b" added="12d ago" tone="ok" action="Sign out" />
              <DeviceRow name="iPad (Air)" id="3b29" added="32d ago" tone="ok" action="Sign out" />
            </div>
          </Section>
          <Section kicker="Identity">
            <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.6 }}>
              <Kv k="Display" v="You" />
              <Kv k="Joined" v="32 days ago" />
              <Kv k="Pubkey" v="a1f3 · 9c20 · 8e4b" />
              <Kv k="Rooms" v="4 active" />
            </div>
          </Section>
          <Section kicker="Recovery">
            <div className={styles.card} style={{ fontSize: 12, color: "var(--mc-ink2)", lineHeight: 1.55, padding: "12px 14px" }}>
              <div style={{ marginBottom: 8 }}>
                <span style={{ color: "var(--mc-ok)", fontWeight: 600 }}>● Recovery phrase set</span>
                <span style={{ color: "var(--mc-muted)", marginLeft: 8, fontSize: 11 }}>last verified 12d ago</span>
              </div>
              <div>
                <span style={{ color: "var(--mc-warn)", fontWeight: 600 }}>▲ 1 of 2 recovery contacts</span>
                <span style={{ color: "var(--mc-muted)", marginLeft: 8, fontSize: 11 }}>
                  add a second to enable social recovery
                </span>
              </div>
            </div>
          </Section>
          <Section kicker="Notifications">
            <NavRow label="Lock-screen previews" sub="Manage previews · sound · AI alerts" onClick={onOpenNotifications} />
          </Section>
        </>
      )}

      {!isMe && !isAi && (
        <>
          <Section kicker="Devices">
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              <DeviceRow name="Phone" id="b2e7" added="32d ago" tone="ok" />
              <DeviceRow
                name="Desktop"
                id="c4d9"
                added="2h ago"
                tone={trust === "unverified" ? "warn" : "ok"}
                badge={trust === "unverified" ? "new" : undefined}
              />
            </div>
          </Section>
          <Section kicker="Shared rooms">
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              <SharedRoom name="mercury-core" last="just now" />
              <SharedRoom name="security-review" last="12m ago" />
              <SharedRoom name="eng-leads" last="1h ago" />
            </div>
          </Section>
          <Section kicker="Activity">
            <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.6 }}>
              <Kv k="Joined" v="32 days ago" />
              <Kv k="Last seen" v="2 minutes ago" />
              <Kv k="Pubkey" v={`${p.short.toLowerCase()}3 · ${p.short.toLowerCase()}9 · b2e7`} />
            </div>
          </Section>
        </>
      )}

      {isAi && (
        <>
          <Section kicker="What this is">
            <div className={styles.card} style={{ fontSize: 12, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
              Mercury AI is a participant that joins specific rooms under a scoped, time-bound grant. It
              reads decrypted content in-room only, never writes to durable storage, and cannot send unless
              the grant explicitly allows it.
            </div>
          </Section>
          {ai === "granted" && (
            <Section kicker="Current scope">
              <div className={styles.scopeGrid}>
                <ScopeTile label="Room" value="mercury-core" note="this room only" />
                <ScopeTile label="Mode" value="local" note="runs on device" />
                <ScopeTile label="Read" value="allowed" tone="ok" note="decrypted in-room" />
                <ScopeTile label="Send" value="not allowed" tone="bad" note="can't post messages" />
                <ScopeTile label="Tools" value="summarize · reply" note="2 of 12 enabled" />
                <ScopeTile label="Expires" value="23h 41m" note="auto-revokes" />
              </div>
            </Section>
          )}
        </>
      )}
    </PanelShell>
  );
}
