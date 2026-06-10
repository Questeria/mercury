// AI access: LIVE mode (a `controller` prop is present) is honest about the REAL model — in
// Mercury an AI participant is an ordinary end-to-end-encrypted contact you explicitly add; the
// only real, enforced controls are block/unblock (framed as revoking/restoring its ability to
// message you), driven against the backend for the active conversation. The old simulated grant
// lifecycle survives only in DEMO mode (no `controller` — the simulator app), unchanged.

import { useState } from "react";

import { MERCURY_PEOPLE } from "../../simulator";
import type { LiveConversations } from "../../liveConversations";
import type { AiState, Person } from "../../types";
import { Avatar } from "../Avatar";
import { PanelShell } from "./PanelShell";
import { GhostButton, HistoryRow, IridButton, ScopeTile, Section, Segment, StatusRow, ToggleRow } from "./primitives";
import styles from "./panels.module.css";

interface AiPanelProps {
  mobile: boolean;
  onClose: () => void;
  /** DEMO (simulator) props — used only when `controller` is absent. */
  ai?: AiState;
  onChangeAi?: (v: AiState) => void;
  /** LIVE props — presence of `controller` selects the real panel. */
  controller?: LiveConversations;
  activePeer?: { peer: string; person: Person } | null;
  blocked?: boolean;
}

export function AiPanel(props: AiPanelProps) {
  if (props.controller) {
    return (
      <LiveAiPanel
        mobile={props.mobile}
        onClose={props.onClose}
        controller={props.controller}
        activePeer={props.activePeer ?? null}
        blocked={props.blocked ?? false}
      />
    );
  }
  return (
    <DemoAiPanel
      mobile={props.mobile}
      onClose={props.onClose}
      ai={props.ai ?? "absent"}
      onChangeAi={props.onChangeAi ?? (() => {})}
    />
  );
}

// ---------------------------------------------------------------- live (honest model)

function LiveAiPanel({
  mobile,
  onClose,
  controller,
  activePeer,
  blocked,
}: {
  mobile: boolean;
  onClose: () => void;
  controller: LiveConversations;
  activePeer: { peer: string; person: Person } | null;
  blocked: boolean;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toggleBlock = () => {
    if (!activePeer || busy) return;
    setBusy(true);
    setError(null);
    void (async () => {
      try {
        if (blocked) await controller.unblock(activePeer.peer);
        else await controller.block(activePeer.peer);
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
      eyebrow="AI access"
      title="AI in Mercury: just another contact"
      footer={<GhostButton onClick={onClose}>Close</GhostButton>}
    >
      <Section kicker="The real model">
        <div className={styles.card} style={{ fontSize: 12, color: "var(--mc-ink2)", lineHeight: 1.6 }}>
          An AI participant in Mercury is an <strong>ordinary end-to-end-encrypted contact</strong>{" "}
          that you explicitly add by its account id — exactly the same crypto, the same safety
          number, the same mute/block/delete controls as a human peer. Nothing is hidden: an AI
          sees only what you send to <em>its</em> conversation, and Mercury gives no AI any
          background access to your other conversations. There is no built-in AI account; if you
          haven&apos;t added one, none exists in your contact list.
        </div>
      </Section>

      {activePeer ? (
        <Section kicker="Active conversation">
          <StatusRow
            tone={blocked ? "bad" : "ok"}
            label={activePeer.person.name}
            sub={blocked ? "blocked — cannot message you" : "can message you"}
          />
          <div className={styles.card} style={{ marginTop: 8 }}>
            <div style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55, marginBottom: 10 }}>
              {blocked
                ? "This contact is blocked: your sends are refused, and anything they send is received then discarded on your device — never shown or notified. Unblocking restores both directions."
                : "Revoking access blocks this contact: your sends are refused, and anything they send is received then discarded on your device — never shown or notified. This is the real, enforced control — it works the same whether the peer is an AI or a human."}
            </div>
            {blocked ? (
              <IridButton disabled={busy} onClick={toggleBlock}>
                {busy ? "Working…" : "Restore access (unblock)"}
              </IridButton>
            ) : (
              <GhostButton danger onClick={toggleBlock}>
                {busy ? "Working…" : "Revoke access (block)"}
              </GhostButton>
            )}
            {error && (
              <div style={{ marginTop: 8, fontSize: 11.5, color: "var(--mc-warn)", lineHeight: 1.45 }}>
                {error}
              </div>
            )}
          </div>
        </Section>
      ) : (
        <Section kicker="Active conversation">
          <div className={styles.card} style={{ color: "var(--mc-muted)", fontSize: 12, lineHeight: 1.5 }}>
            No conversation is open. Open one to block or unblock that contact from here.
          </div>
        </Section>
      )}

      <Section kicker="Future (not built)">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          <span style={{ color: "var(--mc-warn)", fontWeight: 600 }}>Design intent, not a shipped
          feature:</span>{" "}
          scoped, time-boxed AI grants (read-only windows, tool approval, automatic context expiry)
          are a future design. Today the honest description is the one above — an AI is a contact,
          and block/unblock is the enforcement you actually have.
        </div>
      </Section>
    </PanelShell>
  );
}

// ---------------------------------------------------------------- demo (simulator mockup)

type GrantMode = "summarize" | "draft" | "search";
type GrantWindow = "8h" | "24h" | "session";

function DemoAiPanel({
  mobile,
  onClose,
  ai,
  onChangeAi,
}: {
  mobile: boolean;
  onClose: () => void;
  ai: AiState;
  onChangeAi: (v: AiState) => void;
}) {
  const granted = ai === "granted";
  const [mode, setMode] = useState<GrantMode>("summarize");
  const [window, setWindow] = useState<GrantWindow>("24h");
  const [localOnly, setLocalOnly] = useState(true);
  const [toolUse, setToolUse] = useState(false);
  const [retain, setRetain] = useState(false);
  const title = granted ? "Mercury AI scoped" : ai === "absent" ? "No AI access in this room" : `AI access ${ai}`;

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="AI access"
      title={title}
      footer={
        <>
          <GhostButton onClick={onClose}>Close</GhostButton>
          {granted ? (
            <GhostButton danger onClick={() => onChangeAi("revoked")}>
              Revoke now
            </GhostButton>
          ) : (
            <IridButton onClick={() => onChangeAi("granted")}>Request grant</IridButton>
          )}
        </>
      }
    >
      <div style={{ position: "relative", marginBottom: 16 }}>
        <div style={{ position: "relative", padding: "14px 16px", background: "var(--mc-surface)", borderRadius: 12, overflow: "hidden" }}>
          {granted && <span className="iris-ring" style={{ borderRadius: 12 }} />}
          <div style={{ position: "relative" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 8 }}>
              <Avatar person={MERCURY_PEOPLE.ai} size={28} />
              <div style={{ minWidth: 0, flex: 1 }}>
                <div className={granted ? "iris-text" : undefined} style={{ fontSize: 13.5, fontWeight: 600 }}>
                  {granted ? "Mercury AI" : "AI participant"}
                </div>
                <div style={{ fontSize: 11, color: "var(--mc-muted)", marginTop: 1 }}>
                  {granted ? `scoped to this room / ${window} grant` : ai === "absent" ? "no grant active" : `grant ${ai}`}
                </div>
              </div>
              {granted && <span className={styles.tag} style={{ color: "var(--mc-ok)" }}>active</span>}
            </div>
            <div style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
              {granted &&
                "AI has temporary, scoped access to decrypted content in this room only. No other rooms are visible to it."}
              {ai === "absent" &&
                "@ai mentions in this room are held at the outbound gate. Request a grant to give Mercury AI scoped, read-only access."}
              {ai === "revoked" &&
                "A previous grant was revoked. To resume, request a new one. Old context is not retained."}
              {ai === "expired" &&
                "The previous grant lapsed. Mercury AI has no current access. Request a new grant to resume."}
            </div>
          </div>
        </div>
      </div>

      <Section kicker="Grant mode">
        <Segment<GrantMode>
          value={mode}
          onChange={setMode}
          options={[
            { value: "summarize", label: "Summarize", sub: "read-only" },
            { value: "draft", label: "Draft", sub: "needs send" },
            { value: "search", label: "Search", sub: "selected room" },
          ]}
        />
      </Section>

      <Section kicker="Window">
        <Segment<GrantWindow>
          value={window}
          onChange={setWindow}
          options={[
            { value: "8h", label: "8h" },
            { value: "24h", label: "24h" },
            { value: "session", label: "Session" },
          ]}
        />
      </Section>

      <Section kicker="Limits">
        <div className={styles.card} style={{ padding: 0, overflow: "hidden" }}>
          <ToggleRow label="Local model only" sub="Remote providers stay blocked for sensitive rooms." value={localOnly} onChange={setLocalOnly} />
          <ToggleRow label="Allow tools" sub="Tool calls are denied unless every human member approves." value={toolUse} onChange={setToolUse} divider />
          <ToggleRow label="Retain context" sub="Off clears decrypted context when the grant expires." value={retain} onChange={setRetain} divider />
        </div>
      </Section>

      {granted && (
        <Section kicker="Scope">
          <div className={styles.scopeGrid}>
            <ScopeTile label="Room" value="mercury-core" note="this room only" />
            <ScopeTile label="Provider" value={localOnly ? "local" : "remote blocked"} note="policy gated" />
            <ScopeTile label="Read" value="allowed" tone="ok" note="decrypted in-room" />
            <ScopeTile label="Send" value="not allowed" tone="bad" note="drafts only" />
            <ScopeTile label="Tools" value={toolUse ? "pending" : "off"} tone={toolUse ? "warn" : "bad"} note="member vote required" />
            <ScopeTile label="Retain" value={retain ? "pending" : "off"} tone={retain ? "warn" : "ok"} note="clears at expiry" />
          </div>
        </Section>
      )}

      <Section kicker="Grant history">
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <HistoryRow when="today, 09:12" by="you" event={`grant requested / ${window} / ${mode}`} tone="ok" />
          <HistoryRow when="today, 09:12" by="binding" event="grant accepted" tone="ok" />
          <HistoryRow when="3d ago, 14:02" by="you" event="previous grant revoked" tone="bad" />
          <HistoryRow when="3d ago, 09:30" by="you" event="grant requested / 8h scoped" tone="muted" />
        </div>
      </Section>
    </PanelShell>
  );
}
