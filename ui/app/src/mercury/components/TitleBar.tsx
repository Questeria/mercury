// Custom themed window title bar for the desktop (Tauri) build. Replaces the native OS chrome
// (decorations:false) with a draggable bar that matches the app theme and carries the Mercury mark +
// minimize / maximize-restore / close controls. Rendered only inside Tauri.

import { useEffect, useState } from "react";
import { MercuryLogo } from "./MercuryLogo";
import styles from "./TitleBar.module.css";

async function appWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

export function TitleBar({
  menuOpen,
  onToggleMenu,
}: {
  /** Open state of the app menu (drives the ⋯ button highlight). */
  menuOpen?: boolean;
  /** Toggle the app menu. When provided, a ⋯ menu button is shown left of the window controls so the
   *  menu lives in the fixed title bar — above every panel, never floating over the inspector. */
  onToggleMenu?: () => void;
} = {}) {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let alive = true;
    void (async () => {
      try {
        const w = await appWindow();
        if (alive) setMaximized(await w.isMaximized());
        unlisten = await w.onResized(async () => {
          if (alive) setMaximized(await w.isMaximized());
        });
      } catch {
        /* not in Tauri / window API unavailable */
      }
    })();
    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  const minimize = () => void appWindow().then((w) => w.minimize());
  const toggleMax = () => void appWindow().then((w) => w.toggleMaximize());
  const close = () => void appWindow().then((w) => w.close());

  return (
    <div className={styles.bar} data-tauri-drag-region>
      <div className={styles.brand} data-tauri-drag-region>
        <MercuryLogo size={18} className={styles.logo} />
        <span className={`${styles.title} iris-text`}>Mercury</span>
      </div>
      <div className={styles.controls}>
        {onToggleMenu && (
          <button
            className={`${styles.ctl} ${styles.menuCtl}`}
            type="button"
            onClick={onToggleMenu}
            aria-label="Menu"
            aria-haspopup="menu"
            aria-expanded={menuOpen ?? false}
            data-active={menuOpen ? "" : undefined}
            title="Menu"
          >
            ⋯
          </button>
        )}
        <button className={styles.ctl} type="button" onClick={minimize} aria-label="Minimize" title="Minimize">
          <svg width="11" height="11" viewBox="0 0 11 11" aria-hidden>
            <rect x="1.5" y="5" width="8" height="1.2" fill="currentColor" />
          </svg>
        </button>
        <button className={styles.ctl} type="button" onClick={toggleMax} aria-label={maximized ? "Restore" : "Maximize"} title={maximized ? "Restore" : "Maximize"}>
          {maximized ? (
            <svg width="11" height="11" viewBox="0 0 11 11" aria-hidden>
              <rect x="2.4" y="1.4" width="6" height="6" rx="1" fill="none" stroke="currentColor" strokeWidth="1.1" />
              <rect x="1.4" y="3.2" width="6" height="6" rx="1" fill="var(--mc-surface)" stroke="currentColor" strokeWidth="1.1" />
            </svg>
          ) : (
            <svg width="11" height="11" viewBox="0 0 11 11" aria-hidden>
              <rect x="1.6" y="1.6" width="7.8" height="7.8" rx="1" fill="none" stroke="currentColor" strokeWidth="1.1" />
            </svg>
          )}
        </button>
        <button className={`${styles.ctl} ${styles.close}`} type="button" onClick={close} aria-label="Close" title="Close">
          <svg width="11" height="11" viewBox="0 0 11 11" aria-hidden>
            <path d="M2 2 L9 9 M9 2 L2 9" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </button>
      </div>
    </div>
  );
}
