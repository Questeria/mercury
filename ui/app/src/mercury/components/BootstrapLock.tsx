import { LockBigIcon } from "../icons";
import type { PanelId, PlatformDecisionView } from "../types";
import styles from "./BootstrapLock.module.css";

interface BootstrapLockProps {
  view: PlatformDecisionView;
  onOpenPanel: (panel: PanelId) => void;
}

/** Replaces the thread while `can_open_message_ui` is false. The gate is
 *  core's decision, not the UI's -- we only render the reason + remediation. */
export function BootstrapLock({ view, onOpenPanel }: BootstrapLockProps) {
  return (
    <div className={styles.wrap}>
      <div className={styles.inner}>
        <div className={styles.icon}>
          <span className="iris-ring" style={{ borderRadius: "50%" }} />
          <LockBigIcon size={24} color="var(--mc-warn)" className={styles.glyph} />
        </div>
        <div className={styles.title}>
          {view.requires_recovery ? "Recovery required" : "Finishing sync"}
        </div>
        <div className={styles.body}>
          Message UI stays closed until <span className="mono" style={{ color: "var(--mc-ink)" }}>client_bootstrap</span> accepts.
          The gate is core's decision, not the UI's.
        </div>
        <div className={`${styles.chip} mono`}>
          {view.reason_label} · rc={view.reason_code}
        </div>
        <div className={styles.actions}>
          {view.can_start_sync && (
            <button type="button" className={styles.iridBtn} onClick={() => onOpenPanel("sync")}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: "relative" }}>Continue sync</span>
            </button>
          )}
          {view.requires_recovery && (
            <button type="button" className={styles.iridBtn} onClick={() => onOpenPanel("recovery")}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: "relative" }}>Open recovery</span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
