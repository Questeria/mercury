// Notifications: LIVE mode (a `controller` prop is present) is fully real — the master toggle
// gates the actual OS-notification path (notify.ts, persisted), the start-at-login toggle drives
// the Tauri autostart plugin (tray delivery), the read-receipt toggle drives the backend's real
// persisted privacy setting, and the muted list is the backend's real per-conversation mute state
// with working Unmute. DEMO mode (no `controller` — the simulator app) keeps the original mockup
// content unchanged.

import { useEffect, useState } from "react";

import { isAutostartEnabled, setAutostartEnabled } from "../../autostart";
import type { LiveConversations } from "../../liveConversations";
import { inTauri } from "../../messaging";
import { notificationsEnabled, setNotificationsEnabled } from "../../notify";
import { PanelShell } from "./PanelShell";
import { GhostButton, LockNotification, ScopeTile, Section, ToggleRow, type LockSample } from "./primitives";
import styles from "./panels.module.css";

interface NotificationsPanelProps {
  mobile: boolean;
  onClose: () => void;
  /** LIVE props — presence of `controller` selects the real, backend-wired panel. */
  controller?: LiveConversations;
  muted?: string[];
  aliases?: Record<string, string>;
  /** The account's persisted read-receipt privacy setting (live path). */
  readReceipts?: boolean;
}

export function NotificationsPanel({
  mobile,
  onClose,
  controller,
  muted,
  aliases,
  readReceipts,
}: NotificationsPanelProps) {
  if (controller) {
    return (
      <LiveNotificationsPanel
        mobile={mobile}
        onClose={onClose}
        controller={controller}
        muted={muted ?? []}
        aliases={aliases ?? {}}
        readReceipts={readReceipts ?? true}
      />
    );
  }
  return <DemoNotificationsPanel mobile={mobile} onClose={onClose} />;
}

// ---------------------------------------------------------------- live (real backend)

function shortHex(hex: string): string {
  return hex.length > 12 ? `${hex.slice(0, 6)}…${hex.slice(-4)}` : hex;
}

function LiveNotificationsPanel({
  mobile,
  onClose,
  controller,
  muted,
  aliases,
  readReceipts,
}: {
  mobile: boolean;
  onClose: () => void;
  controller: LiveConversations;
  muted: string[];
  aliases: Record<string, string>;
  readReceipts: boolean;
}) {
  const tauri = inTauri();
  const [enabled, setEnabled] = useState(notificationsEnabled);
  // null = still reading the OS state; the toggle renders once we know the truth.
  const [autoOn, setAutoOn] = useState<boolean | null>(tauri ? null : false);
  const [autoBusy, setAutoBusy] = useState(false);

  useEffect(() => {
    if (!tauri) return;
    let cancelled = false;
    void isAutostartEnabled().then((on) => {
      if (!cancelled) setAutoOn(on);
    });
    return () => {
      cancelled = true;
    };
  }, [tauri]);

  const toggleMaster = (on: boolean) => {
    setNotificationsEnabled(on);
    setEnabled(on);
  };

  const toggleAutostart = (on: boolean) => {
    if (autoBusy || autoOn === null) return;
    setAutoBusy(true);
    void (async () => {
      const result = await setAutostartEnabled(on); // returns the REAL resulting OS state
      setAutoOn(result);
      setAutoBusy(false);
    })();
  };

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Notifications"
      title="Notifications & background delivery"
      footer={<GhostButton onClick={onClose}>Done</GhostButton>}
    >
      <Section kicker="Settings">
        <div className={styles.card} style={{ padding: 0, overflow: "hidden" }}>
          <ToggleRow
            label="OS notifications"
            sub="Master switch for desktop notifications about new encrypted messages."
            value={enabled}
            onChange={toggleMaster}
          />
          {tauri ? (
            <ToggleRow
              label="Start Mercury at login (receives messages in the tray)"
              sub={
                autoOn === null
                  ? "Checking the current login setting…"
                  : autoBusy
                    ? "Applying…"
                    : "The OS launches Mercury minimized to the tray at sign-in, so messages keep arriving."
              }
              value={autoOn === true}
              onChange={toggleAutostart}
              divider
            />
          ) : (
            <div
              style={{
                padding: "11px 14px",
                borderTop: "1px solid var(--mc-border)",
                fontSize: 11.5,
                color: "var(--mc-muted)",
                lineHeight: 1.5,
              }}
            >
              <span style={{ color: "var(--mc-ink2)", fontWeight: 600 }}>Start Mercury at login</span> — desktop
              app only. The browser dev build has no tray and cannot register a login launch.
            </div>
          )}
        </div>
      </Section>

      <Section kicker="Privacy">
        <div className={styles.card} style={{ padding: 0, overflow: "hidden" }}>
          {/* The REAL persisted backend setting: the toggle reflects state.readReceipts and flips
              once the backend confirms. Honest scope: it controls only the receipts YOU emit. */}
          <ToggleRow
            label="Send read receipts"
            sub="When on, people you chat with see when you've read their 1:1 messages. Delivery confirmations aren't covered by this toggle — they ride the protocol best-effort and can't be switched off. Read receipts are mutual courtesy — turning this off only stops YOURS."
            value={readReceipts}
            onChange={(on) => void controller.setReadReceipts(on)}
          />
        </div>
      </Section>

      <Section kicker="Muted conversations">
        {muted.length === 0 ? (
          <div className={styles.card} style={{ color: "var(--mc-muted)", fontSize: 12, lineHeight: 1.5 }}>
            Nothing is muted. Mute a conversation from its ⋯ menu to silence its notifications
            (delivery is unaffected).
          </div>
        ) : (
          <div className={styles.colGap}>
            {muted.map((peer) => (
              <div className={styles.attachmentRow} key={peer}>
                <span className={styles.dot6} style={{ background: "var(--mc-warn)" }} />
                <div style={{ minWidth: 0, flex: 1 }}>
                  <div className={styles.attachmentName}>
                    {aliases[peer] ? `@${aliases[peer]}` : shortHex(peer)}
                  </div>
                  <div className={styles.attachmentMeta}>notifications silenced · still receiving</div>
                </div>
                <GhostButton onClick={() => void controller.unmute(peer)}>Unmute</GhostButton>
              </div>
            ))}
          </div>
        )}
      </Section>

      <Section kicker="What notifications contain">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          A Mercury notification carries only the (shortened) sender and a count — never message
          text. &quot;Delivery while closed&quot; means the close button hides the window to the
          tray, where the app keeps running and receiving. True OS push with the app fully quit
          does not exist yet — if you quit Mercury from the tray, messages wait at the relay until
          you start it again.
        </div>
      </Section>
    </PanelShell>
  );
}

// ---------------------------------------------------------------- demo (simulator mockup)

const SAMPLES: LockSample[] = [
  {
    kind: "accepted",
    who: "Rin Hadley / mercury-core",
    preview: "Pushed the receive-gate rewrite. Ordering check is now in core, not the binding.",
    tone: "ok",
    when: "now",
  },
  {
    kind: "tofu",
    who: "Jules Okafor / mercury-core",
    preview: "[withheld] first-use verification needed before preview",
    tone: "warn",
    when: "2m",
  },
  {
    kind: "rejected",
    who: "Unknown / mercury-core",
    preview: "[withheld] sender trust rejected",
    tone: "bad",
    when: "5m",
  },
];

function DemoNotificationsPanel({ mobile, onClose }: { mobile: boolean; onClose: () => void }) {
  const [previews, setPreviews] = useState(true);
  const [sound, setSound] = useState(true);
  const [aiAlerts, setAiAlerts] = useState(false);
  const [backgroundReceive, setBackgroundReceive] = useState(true);
  const [quiet, setQuiet] = useState(false);

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Notifications"
      title="Background receive"
      footer={<GhostButton onClick={onClose}>Done</GhostButton>}
    >
      <Section kicker="Settings">
        <div className={styles.card} style={{ padding: 0, overflow: "hidden" }}>
          <ToggleRow
            label="Background receive"
            sub="Poll encrypted envelopes and replay checkpoints while the app is minimized."
            value={backgroundReceive}
            onChange={setBackgroundReceive}
          />
          <ToggleRow
            label="Show message previews"
            sub="Off shows only the room name. Sensitive content is always withheld."
            value={previews}
            onChange={setPreviews}
            divider
          />
          <ToggleRow
            label="Sound"
            sub="Audible alert when a new accepted message arrives."
            value={sound}
            onChange={setSound}
            divider
          />
          <ToggleRow
            label="AI activity alerts"
            sub="Notify when Mercury AI is mentioned, replies, or a grant changes."
            value={aiAlerts}
            onChange={setAiAlerts}
            divider
          />
          <ToggleRow
            label="Quiet hours"
            sub="Suppress sound while still recording encrypted delivery events."
            value={quiet}
            onChange={setQuiet}
            divider
          />
        </div>
      </Section>

      <Section kicker="Receive gate">
        <div className={styles.featureGrid}>
          <ScopeTile label="Accepted" value={previews ? "preview" : "room only"} tone="ok" note="message UI may render" />
          <ScopeTile label="TOFU" value="withheld" tone="warn" note="verification first" />
          <ScopeTile label="Rejected" value="withheld" tone="bad" note="sender trust failed" />
          <ScopeTile label="Plaintext" value="local only" tone="ok" note="never in relay metadata" />
        </div>
      </Section>

      <Section kicker="Preview samples">
        <div className={styles.colGap}>
          {SAMPLES.map((s, i) => (
            <LockNotification key={i} s={s} previews={previews} />
          ))}
        </div>
        <div style={{ fontSize: 11, color: "var(--mc-muted)", marginTop: 10, lineHeight: 1.5 }}>
          Accepted, TOFU pending, and sender rejected messages take different notification paths.
        </div>
      </Section>

      <Section kicker="How this works">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          Mercury composes notification text after the gate decision. Plaintext never reaches the OS
          notification surface for a message that the binding has not accepted as displayable.
        </div>
      </Section>
    </PanelShell>
  );
}
