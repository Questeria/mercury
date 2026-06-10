// Connecting: the interactive "add someone" + "share your handle" flows, plus the per-method
// preferences (each user-togglable, each with a hover tooltip stating ITS security drawback and an
// always-visible inline warning). The pairing-code and username flows drive the REAL backend
// (relay broker + key-transparency registry); a username add is only accepted after its inclusion
// proof verifies. These are convenience surfaces — the trust model is unchanged: the safety number
// is always the real backstop.

import { useState } from "react";

import {
  getConnectionPrefs,
  setConnectionPref,
  type ConnectionPrefs,
} from "../../connectionPrefs";
import type { LiveConversations } from "../../liveConversations";
import { PanelShell } from "./PanelShell";
import { Section } from "./primitives";
import styles from "./panels.module.css";

interface ConnectionPanelProps {
  mobile: boolean;
  onClose: () => void;
  controller: LiveConversations;
}

interface Method {
  key: keyof ConnectionPrefs;
  name: string;
  how: string;
  drawback: string;
}

const METHODS: Method[] = [
  {
    key: "qr",
    name: "QR code (in person)",
    how: "Scan their code face-to-face.",
    drawback: "Strongest — but needs you both in the same place.",
  },
  {
    key: "link",
    name: "Invite link / ID",
    how: "Share a link or your 64-char id.",
    drawback:
      "Texting an id/link over an insecure channel risks an id-swap MITM — verify the safety number after connecting.",
  },
  {
    key: "safetyVerify",
    name: "Safety-number verification",
    how: "Compare a short number after connecting.",
    drawback:
      "The backstop that makes ANY method safe — but only if you actually compare it (in person or over a recognizable call).",
  },
  {
    key: "pairing",
    name: "Pairing code",
    how: "A short code brokered by the relay, valid for ~10 minutes and usable once.",
    drawback:
      "Easier to read aloud than 64 hex; if someone intercepts the code in its short window they could connect as you — still verify the safety number.",
  },
  {
    key: "username",
    name: "Username",
    how: "Add someone by their handle.",
    drawback:
      "Most convenient. For the built-in default relay, the directory's key is baked into the signed app, so there's no trust-on-first-use gap. For a custom relay, the first add trusts the key it serves (trust-on-first-use) and a single directory can still equivocate. Either way, verify the safety number.",
  },
];

type Feedback = { kind: "ok" | "warn"; text: string } | null;

const inputStyle: React.CSSProperties = {
  flex: 1,
  minWidth: 0,
  border: "1px solid var(--mc-border)",
  borderRadius: 8,
  background: "var(--mc-surface)",
  color: "var(--mc-ink)",
  padding: "7px 9px",
  fontSize: 12.5,
};

const btnStyle: React.CSSProperties = {
  border: "1px solid var(--mc-border)",
  borderRadius: 8,
  background: "var(--mc-accent)",
  color: "#fff",
  padding: "7px 12px",
  fontSize: 12.5,
  fontWeight: 600,
  cursor: "pointer",
  flexShrink: 0,
};

export function ConnectionPanel({ mobile, onClose, controller }: ConnectionPanelProps) {
  const [prefs, setPrefs] = useState<ConnectionPrefs>(() => getConnectionPrefs());
  const toggle = (k: keyof ConnectionPrefs) => setPrefs(setConnectionPref(k, !prefs[k]));

  const [enterCode, setEnterCode] = useState("");
  const [addName, setAddName] = useState("");
  const [claimName, setClaimName] = useState("");
  const [myCode, setMyCode] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<Feedback>(null);

  const run = async (tag: string, fn: () => Promise<Feedback>) => {
    setBusy(tag);
    setFeedback(null);
    try {
      const fb = await fn();
      setFeedback(fb);
    } catch (e) {
      setFeedback({ kind: "warn", text: e instanceof Error ? e.message : String(e) });
    } finally {
      setBusy(null);
    }
  };

  const showMyCode = () =>
    run("mycode", async () => {
      const { code } = await controller.pairingCode();
      setMyCode(code);
      return { kind: "ok", text: "Share this code — it works once, for about 10 minutes." };
    });

  const redeemCode = () =>
    run("redeem", async () => {
      await controller.addByPairing(enterCode);
      setEnterCode("");
      onClose(); // jump to the freshly-opened conversation
      return null;
    });

  const addByName = () =>
    run("addname", async () => {
      const { tofu } = await controller.addByUsername(addName);
      setAddName("");
      if (!tofu) {
        onClose(); // verified against the already-pinned directory key — jump to the chat
        return null;
      }
      // First-use trust: keep the panel open so the caveat is actually SEEN (the conversation is
      // already open behind it). The directory's key was trusted on first use, not yet pinned-honest.
      return {
        kind: "warn",
        text: "Added — but this trusted the directory's key on first use; verify the safety number to be sure.",
      };
    });

  const claim = () =>
    run("claim", async () => {
      const { status, username } = await controller.claimUsername(claimName);
      switch (status) {
        case "created":
          return { kind: "ok", text: `You now own @${username}.` };
        case "already_owned":
          return { kind: "ok", text: `You already own @${username}.` };
        case "taken":
          return { kind: "warn", text: `@${username} is already taken.` };
        case "rejected":
          return {
            kind: "warn",
            text: "Usernames are 3–20 chars: a–z, 0–9, _ and must start with a letter.",
          };
        default:
          return { kind: "warn", text: "The directory is busy — try again shortly." };
      }
    });

  return (
    <PanelShell mobile={mobile} onClose={onClose} eyebrow="Connecting" title="How you connect">
      {(prefs.pairing || prefs.username) && (
        <Section kicker="Add someone">
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {prefs.pairing && (
              <div style={{ display: "flex", gap: 6 }}>
                <input
                  style={inputStyle}
                  placeholder="Enter a pairing code"
                  value={enterCode}
                  onChange={(e) => setEnterCode(e.target.value)}
                  aria-label="Pairing code to redeem"
                />
                <button
                  type="button"
                  style={btnStyle}
                  disabled={busy !== null || !enterCode.trim()}
                  onClick={redeemCode}
                >
                  {busy === "redeem" ? "Adding…" : "Add"}
                </button>
              </div>
            )}
            {prefs.username && (
              <div style={{ display: "flex", gap: 6 }}>
                <input
                  style={inputStyle}
                  placeholder="Add by username"
                  value={addName}
                  onChange={(e) => setAddName(e.target.value)}
                  aria-label="Username to add"
                />
                <button
                  type="button"
                  style={btnStyle}
                  disabled={busy !== null || !addName.trim()}
                  onClick={addByName}
                >
                  {busy === "addname" ? "Adding…" : "Add"}
                </button>
              </div>
            )}
          </div>
        </Section>
      )}

      {(prefs.pairing || prefs.username) && (
        <Section kicker="Share your handle">
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {prefs.pairing && (
              <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                <button type="button" style={btnStyle} disabled={busy !== null} onClick={showMyCode}>
                  {busy === "mycode" ? "…" : "Show pairing code"}
                </button>
                {myCode && (
                  <code
                    style={{
                      fontSize: 16,
                      fontWeight: 700,
                      letterSpacing: 1,
                      color: "var(--mc-ink)",
                      fontFamily: "var(--mc-mono, monospace)",
                    }}
                  >
                    {myCode}
                  </code>
                )}
              </div>
            )}
            {prefs.username && (
              <div style={{ display: "flex", gap: 6 }}>
                <input
                  style={inputStyle}
                  placeholder="Claim a username"
                  value={claimName}
                  onChange={(e) => setClaimName(e.target.value)}
                  aria-label="Username to claim"
                />
                <button
                  type="button"
                  style={btnStyle}
                  disabled={busy !== null || !claimName.trim()}
                  onClick={claim}
                >
                  {busy === "claim" ? "Claiming…" : "Claim"}
                </button>
              </div>
            )}
          </div>
          {feedback && (
            <div
              style={{
                marginTop: 8,
                fontSize: 11.5,
                lineHeight: 1.45,
                color: feedback.kind === "ok" ? "var(--mc-accent)" : "var(--mc-warn)",
              }}
            >
              {feedback.text}
            </div>
          )}
        </Section>
      )}

      <Section kicker="Methods — hover for the trade-off">
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {METHODS.map((m) => {
            const on = prefs[m.key];
            return (
              <div
                key={m.key}
                className={styles.card}
                title={m.drawback}
                style={{ display: "flex", alignItems: "flex-start", gap: 10 }}
              >
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontWeight: 600, fontSize: 13, color: "var(--mc-ink)" }}>
                    {m.name}
                  </div>
                  <div
                    style={{ fontSize: 11, color: "var(--mc-muted)", marginTop: 3, lineHeight: 1.45 }}
                  >
                    {m.how}
                  </div>
                  <div
                    style={{ fontSize: 11, color: "var(--mc-warn)", marginTop: 4, lineHeight: 1.45 }}
                  >
                    {"⚠ "}
                    {m.drawback}
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => toggle(m.key)}
                  aria-pressed={on}
                  aria-label={`${on ? "Disable" : "Enable"} ${m.name}`}
                  style={{
                    border: "1px solid var(--mc-border)",
                    borderRadius: 999,
                    width: 44,
                    height: 24,
                    background: on ? "var(--mc-accent)" : "var(--mc-surface)",
                    position: "relative",
                    cursor: "pointer",
                    flexShrink: 0,
                    padding: 0,
                  }}
                >
                  <span
                    style={{
                      position: "absolute",
                      top: 2,
                      left: on ? 22 : 2,
                      width: 18,
                      height: 18,
                      borderRadius: "50%",
                      background: "#fff",
                      transition: "left .15s ease",
                    }}
                  />
                </button>
              </div>
            );
          })}
        </div>
      </Section>

      <Section kicker="The honest rule">
        <div
          className={styles.card}
          style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.5 }}
        >
          Sharing an id, link, pairing code, or username is convenient, but any of them can be
          intercepted on the way. None of it changes the trust model: the one thing that proves
          you're talking to the real person is comparing the <strong>safety number</strong> over a
          channel you trust. Turn methods on or off to match how careful you want to be.
        </div>
      </Section>
    </PanelShell>
  );
}
