// Automatic update notice. On launch (and every 6h for long-running tray sessions) the app checks
// the SIGNED update manifest; if a newer version exists it surfaces a small banner. Detection is
// automatic; INSTALL stays an explicit one-click action (we never silently replace the binary), and
// it's a no-op in the browser. Reuses the audited updater seam in ../updater.ts.
import { useEffect, useRef, useState } from "react";

import { inTauri } from "../messaging";
import { checkForUpdates, downloadAndInstallUpdate } from "../updater";
import styles from "../LiveMercuryApp.module.css";

type Phase = "hidden" | "available" | "installing" | "installed" | "failed" | "checkfailed";

const RECHECK_MS = 6 * 60 * 60 * 1000;

export function UpdateBanner() {
  const [phase, setPhase] = useState<Phase>("hidden");
  const [version, setVersion] = useState<string | undefined>();
  const [detail, setDetail] = useState<string | undefined>();
  // Persists across the periodic re-check so a "Later"/"Dismiss" isn't re-nagged this session.
  const dismissed = useRef(false);

  useEffect(() => {
    if (!inTauri()) return;
    let cancelled = false;
    const check = async () => {
      if (dismissed.current) return;
      const r = await checkForUpdates(); // never throws
      if (cancelled) return;
      if (r.state === "available") {
        setVersion(r.version);
        setDetail(undefined);
        // Only raise the banner from a resting state — don't interrupt an in-progress install.
        setPhase((p) => (p === "hidden" ? "available" : p));
      } else if (r.state === "dormant" || r.state === "error") {
        // Surface a swallowed check failure (unreachable manifest, blocked request, …) instead of
        // silently looking "up to date" — a stuck auto-updater should be visible, not invisible.
        setDetail(r.detail);
        setPhase((p) => (p === "hidden" ? "checkfailed" : p));
      }
    };
    void check();
    const id = window.setInterval(() => void check(), RECHECK_MS);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, []);

  if (phase === "hidden") return null;

  const install = async () => {
    setPhase("installing");
    const r = await downloadAndInstallUpdate();
    // downloadAndInstallUpdate() returns state "available" (title "Update installed") on success.
    setPhase(r.state === "available" ? "installed" : "failed");
  };
  const dismiss = () => {
    dismissed.current = true;
    setPhase("hidden");
  };

  return (
    <div className={styles.updateBar} role="status">
      {phase === "available" && (
        <>
          <span className={styles.updateBarText}>
            Mercury <strong>v{version}</strong> is available.
          </span>
          <button className={styles.updateBarInstall} type="button" onClick={install}>
            Install update
          </button>
          <button className={styles.updateBarLater} type="button" onClick={dismiss}>
            Later
          </button>
        </>
      )}
      {phase === "installing" && <span className={styles.updateBarText}>Downloading update…</span>}
      {phase === "installed" && (
        <>
          <span className={styles.updateBarText}>Update installed — restart Mercury to finish.</span>
          <button className={styles.updateBarLater} type="button" onClick={dismiss}>
            Dismiss
          </button>
        </>
      )}
      {phase === "failed" && (
        <>
          <span className={styles.updateBarText}>
            Update couldn&apos;t install — try Settings → Updates.
          </span>
          <button className={styles.updateBarLater} type="button" onClick={dismiss}>
            Dismiss
          </button>
        </>
      )}
      {phase === "checkfailed" && (
        <>
          <span className={styles.updateBarText}>
            Couldn&apos;t check for updates{detail ? ` — ${detail}` : "."}
          </span>
          <button className={styles.updateBarLater} type="button" onClick={dismiss}>
            Dismiss
          </button>
        </>
      )}
    </div>
  );
}
