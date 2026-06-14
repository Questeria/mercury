// Automatic update notice. The app checks the SIGNED update manifest on launch, every 6h, AND when
// the window is shown from the tray. That last trigger matters: with close-to-tray + single-instance
// the process rarely restarts, so the "on launch" check almost never re-fires — the window-show check
// is what actually catches a new build for a tray-resident app. If "auto-update" is ON (the default,
// see ../updatePrefs.ts) any trigger downloads + installs the signed build automatically (it applies
// on the next FULL restart — tray → Quit → reopen); the banner shows progress + the restart prompt,
// never silent. Every outcome is recorded via recordUpdateResult so the Updates panel shows exactly
// what happened. The signature is verified before applying, and it's a no-op in the browser.
import { useEffect, useRef, useState } from "react";

import { inTauri } from "../messaging";
import { checkForUpdates, downloadAndInstallUpdate } from "../updater";
import { getAutoUpdate, recordUpdateResult } from "../updatePrefs";
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
    let lastCheckAt = 0;

    const runCheck = async (kind: string) => {
      if (dismissed.current) return;
      const now = Date.now();
      // Throttle window-show checks so foregrounding repeatedly doesn't re-hit the network.
      if (kind === "focus" && now - lastCheckAt < 10 * 60 * 1000) return;
      lastCheckAt = now;

      const r = await checkForUpdates(); // never throws
      if (cancelled) return;
      recordUpdateResult({ at: now, kind, state: r.state, detail: r.detail, version: r.version });

      if (r.state === "available") {
        setVersion(r.version);
        setDetail(undefined);
        if (getAutoUpdate()) {
          // Auto-install on EVERY trigger — launch, the periodic re-check, AND when the window is
          // shown from the tray. A tray app's process rarely restarts (close-to-tray + single
          // instance), so the window-show check is the main chance to catch a newer build.
          setPhase((p) => (p === "hidden" || p === "available" ? "installing" : p));
          const ir = await downloadAndInstallUpdate();
          if (cancelled) return;
          recordUpdateResult({
            at: Date.now(),
            kind: `${kind}-install`,
            state: ir.state,
            detail: ir.detail,
            version: ir.version,
          });
          if (ir.state === "available") {
            setPhase((p) => (p === "installing" ? "installed" : p));
          } else {
            setDetail(ir.detail);
            setPhase((p) => (p === "installing" ? "failed" : p));
          }
        } else {
          // Auto-update off — just surface the one-click banner.
          setPhase((p) => (p === "hidden" ? "available" : p));
        }
      } else if (r.state === "dormant" || r.state === "error") {
        // Surface a swallowed check failure (unreachable manifest, blocked request, …) instead of
        // silently looking "up to date" — a stuck auto-updater should be visible, not invisible.
        setDetail(r.detail);
        setPhase((p) => (p === "hidden" ? "checkfailed" : p));
      }
    };

    void runCheck("launch");
    const id = window.setInterval(() => void runCheck("periodic"), RECHECK_MS);
    const onVisible = () => {
      if (document.visibilityState === "visible") void runCheck("focus");
    };
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      cancelled = true;
      window.clearInterval(id);
      document.removeEventListener("visibilitychange", onVisible);
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
          <span className={styles.updateBarText}>
            Update installed — fully quit Mercury (tray icon → Quit) and reopen to finish.
          </span>
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
