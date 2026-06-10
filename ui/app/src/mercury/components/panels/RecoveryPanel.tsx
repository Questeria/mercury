// Recovery: REAL passphrase-sealed account backup (export + restore). The old simulated
// recovery stepper is gone. Export calls the backend's `backupExport` (throws under 9 chars)
// and saves the sealed blob to Downloads via the Rust shell; restore reads a picked .mercbak,
// hands it to the `restore_backup` Tauri command, and reloads the app on success. Both actions
// are desktop-only; the browser dev build shows the controls disabled with an explanation.
// The demo simulator renders this same panel without a controller (everything disabled).

import { useState } from "react";

import { fileToBase64, saveToDownloads } from "../../b64file";
import type { LiveConversations } from "../../liveConversations";
import { inTauri } from "../../messaging";
import { PanelShell } from "./PanelShell";
import { GhostButton, Section, StatusRow } from "./primitives";
import styles from "./panels.module.css";

interface RecoveryPanelProps {
  mobile: boolean;
  onClose: () => void;
  /** Legacy demo prop (simulated stepper used to call it). Kept for compile compatibility; the
   *  real backup flow never invents a "recovered" state, so it is intentionally unused. */
  onComplete?: () => void;
  /** LIVE prop — the real backend controller. Without it (demo app) actions are disabled. */
  controller?: LiveConversations;
}

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

export function RecoveryPanel({ mobile, onClose, controller }: RecoveryPanelProps) {
  const tauri = inTauri();
  const canAct = tauri && !!controller;

  const [pass, setPass] = useState("");
  const [confirm, setConfirm] = useState("");
  const [exportFeedback, setExportFeedback] = useState<Feedback>(null);

  const [restoreFile, setRestoreFile] = useState<File | null>(null);
  const [restorePass, setRestorePass] = useState("");
  const [restoreFeedback, setRestoreFeedback] = useState<Feedback>(null);

  const [busy, setBusy] = useState<"export" | "restore" | null>(null);

  const passTooShort = pass.length > 0 && pass.length < 9;
  const mismatch = confirm.length > 0 && confirm !== pass;
  const exportReady = canAct && busy === null && pass.length >= 9 && confirm === pass;

  const doExport = () => {
    if (!exportReady || !controller) return;
    setBusy("export");
    setExportFeedback(null);
    void (async () => {
      try {
        const dataB64 = await controller.backupExport(pass);
        const date = new Date().toISOString().slice(0, 10);
        const path = await saveToDownloads(`mercury-backup-${date}.mercbak`, dataB64);
        setExportFeedback({ kind: "ok", text: `Saved: ${path}` });
        setPass("");
        setConfirm("");
      } catch (e) {
        setExportFeedback({ kind: "warn", text: e instanceof Error ? e.message : String(e) });
      } finally {
        setBusy(null);
      }
    })();
  };

  const doRestore = () => {
    if (!canAct || busy !== null || !restoreFile || !restorePass) return;
    setBusy("restore");
    setRestoreFeedback(null);
    void (async () => {
      try {
        const dataB64 = await fileToBase64(restoreFile);
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("restore_backup", { dataB64, passphrase: restorePass });
        setRestoreFeedback({ kind: "ok", text: "Backup restored — reloading…" });
        window.setTimeout(() => window.location.reload(), 900);
        // keep busy set: the app is about to reload; nothing else should run
      } catch (e) {
        // Wrong passphrase / damaged file: the shell rejected before touching the store — nothing
        // changed. Show the real error and let the user try again.
        setRestoreFeedback({ kind: "warn", text: e instanceof Error ? e.message : String(e) });
        setBusy(null);
      }
    })();
  };

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Recovery"
      title="Encrypted account backup"
      footer={<GhostButton onClick={onClose}>Close</GhostButton>}
    >
      <StatusRow
        tone={canAct ? "ok" : "warn"}
        label={canAct ? "Backup & restore available" : "Desktop app required"}
        sub={canAct ? "passphrase-sealed, saved locally" : "the browser build cannot write or replace the account store"}
      />

      <Section kicker="Export a backup">
        <div className={styles.card}>
          <div style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55, marginBottom: 10 }}>
            Seals your <strong>entire account</strong> — identity keys, contacts, sessions, and
            message history — into one file encrypted with the passphrase you type here. The
            passphrase <strong>is</strong> the security: it is never stored anywhere, and a lost
            passphrase makes the file permanently unreadable.
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <input
              style={inputStyle}
              type="password"
              placeholder="Backup passphrase (at least 9 characters)"
              value={pass}
              onChange={(e) => setPass(e.target.value)}
              aria-label="Backup passphrase"
              disabled={!canAct}
            />
            <input
              style={inputStyle}
              type="password"
              placeholder="Repeat the passphrase"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              aria-label="Repeat backup passphrase"
              disabled={!canAct}
            />
            <button type="button" style={btnStyle} disabled={!exportReady} onClick={doExport}>
              {busy === "export" ? "Exporting…" : "Export encrypted backup"}
            </button>
          </div>
          {(passTooShort || mismatch) && (
            <div style={{ marginTop: 8, fontSize: 11.5, color: "var(--mc-warn)", lineHeight: 1.45 }}>
              {passTooShort ? "The passphrase must be at least 9 characters." : "The passphrases don't match."}
            </div>
          )}
          {exportFeedback && (
            <div
              style={{
                marginTop: 8,
                fontSize: 11.5,
                lineHeight: 1.45,
                wordBreak: "break-all",
                color: exportFeedback.kind === "ok" ? "var(--mc-accent)" : "var(--mc-warn)",
              }}
            >
              {exportFeedback.text}
            </div>
          )}
        </div>
      </Section>

      <Section kicker="Restore from a backup">
        <div className={styles.card}>
          <div style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55, marginBottom: 10 }}>
            Replaces what this app currently holds with the backup&apos;s account. A wrong
            passphrase or a damaged file is rejected outright — nothing changes on failure.
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <input
              style={{ ...inputStyle, padding: "6px 9px" }}
              type="file"
              accept=".mercbak,application/octet-stream"
              aria-label="Backup file to restore"
              disabled={!canAct}
              onChange={(e) => {
                setRestoreFile(e.target.files?.[0] ?? null);
                setRestoreFeedback(null);
              }}
            />
            <input
              style={inputStyle}
              type="password"
              placeholder="Backup passphrase"
              value={restorePass}
              onChange={(e) => setRestorePass(e.target.value)}
              aria-label="Passphrase of the backup being restored"
              disabled={!canAct}
            />
            <button
              type="button"
              style={btnStyle}
              disabled={!canAct || busy !== null || !restoreFile || !restorePass}
              onClick={doRestore}
            >
              {busy === "restore" ? "Restoring…" : "Restore backup"}
            </button>
          </div>
          {restoreFeedback && (
            <div
              style={{
                marginTop: 8,
                fontSize: 11.5,
                lineHeight: 1.45,
                color: restoreFeedback.kind === "ok" ? "var(--mc-accent)" : "var(--mc-warn)",
              }}
            >
              {restoreFeedback.text}
            </div>
          )}
        </div>
      </Section>

      <Section kicker="Handle the file like your identity">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          The backup contains <strong>everything</strong>. Anyone holding the file{" "}
          <strong>and</strong> the passphrase IS you — they can read your history and message as
          you. Store it offline (a USB stick in a drawer beats a cloud folder), and prefer a long
          passphrase you don&apos;t use anywhere else.
        </div>
      </Section>
    </PanelShell>
  );
}
