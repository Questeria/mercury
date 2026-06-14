// Automatic update notice. The app checks the SIGNED update manifest on launch, every 6h, AND when
// the window is shown from the tray. That last trigger matters: with close-to-tray + single-instance
// the process rarely restarts, so the "on launch" check almost never re-fires — the window-show check
// is what actually catches a new build for a tray-resident app. If "auto-update" is ON (the default,
// see ../updatePrefs.ts) any trigger downloads + installs the signed build automatically, and on a
// user-present trigger (launch / window-shown) it then QUITS + RELAUNCHES to apply it — no manual
// quit needed (the 6h background re-check only stages it, to avoid yanking a window mid-action). A
// localStorage loop guard relaunches at most once per version: if the same version is still offered
// after a relaunch, it stops and shows a manual-install prompt instead of looping. Every outcome is
// recorded via recordUpdateResult (Updates → Recent activity). Signature verified; no-op in browser.
import { useEffect, useRef, useState } from "react";

import { inTauri } from "../messaging";
import { checkForUpdates, downloadAndInstallUpdate, relaunchApp } from "../updater";
import {
  getAutoUpdate,
  recordUpdateResult,
  setRelaunchAttempt,
  getRecentRelaunchVersion,
  clearRelaunchAttempt,
} from "../updatePrefs";
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
        if (!getAutoUpdate()) {
          // Auto-update off — just surface the one-click banner.
          setPhase((p) => (p === "hidden" ? "available" : p));
          return;
        }

        // Loop guard: if we already quit+relaunched for this exact version and it's STILL offered,
        // the install didn't apply — do NOT relaunch again (that would loop). Fall back to manual.
        if (r.version && getRecentRelaunchVersion() === r.version) {
          clearRelaunchAttempt();
          recordUpdateResult({ at: Date.now(), kind: `${kind}-relaunch-noop`, state: "error", detail: "relaunch did not apply the update", version: r.version });
          setDetail("Auto-update couldn't apply — please reinstall from mercury-messaging.com.");
          setPhase("failed");
          return;
        }

        // Download + install the signed build (verified). Runs on launch, the 6h re-check, AND when
        // the window is shown from the tray — the last is the main chance for a tray-resident app.
        setPhase((p) => (p === "hidden" || p === "available" ? "installing" : p));
        const ir = await downloadAndInstallUpdate();
        if (cancelled) return;
        recordUpdateResult({ at: Date.now(), kind: `${kind}-install`, state: ir.state, detail: ir.detail, version: ir.version });

        if (ir.state !== "available") {
          setDetail(ir.detail);
          setPhase((p) => (p === "installing" ? "failed" : p));
          return;
        }

        // Installed. On a user-present trigger (launch / window-shown) quit + relaunch so it applies
        // with no manual quit. On the background 6h re-check, just stage it (don't yank a window the
        // user may be mid-action in) — the next launch/show will relaunch.
        if (kind === "launch" || kind === "focus") {
          if (r.version) setRelaunchAttempt(r.version);
          recordUpdateResult({ at: Date.now(), kind: `${kind}-relaunch`, state: "available", detail: "quitting to apply update", version: r.version });
          setPhase("installing");
          await relaunchApp(); // quits + restarts into the new version
        } else {
          setPhase((p) => (p === "installing" ? "installed" : p));
        }
      } else if (r.state === "dormant" || r.state === "error") {
        // Surface a swallowed check failure (unreachable manifest, blocked request, …) instead of
        // silently looking "up to date" — a stuck auto-updater should be visible, not invisible.
        setDetail(r.detail);
        setPhase((p) => (p === "hidden" ? "checkfailed" : p));
      } else if (r.state === "current") {
        // Up to date — the relaunch (if any) applied, or there was nothing to do. Reset the guard.
        clearRelaunchAttempt();
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
      {phase === "installing" && (
        <span className={styles.updateBarText}>Updating Mercury — it will restart to finish…</span>
      )}
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
