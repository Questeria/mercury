import { useContext, type ReactNode } from "react";

import { EmbeddedContext, PreviewContext } from "./panelContext";
import styles from "./panels.module.css";

function PreviewBanner() {
  const preview = useContext(PreviewContext);
  if (!preview) return null;
  return (
    <div className={styles.previewBanner} role="note">
      <strong>Preview.</strong> This screen is a UI mockup — it is not yet wired to the encrypted backend
      and enforces nothing. Don’t rely on anything it shows.
    </div>
  );
}

interface PanelShellProps {
  mobile: boolean;
  onClose: () => void;
  eyebrow?: string;
  title: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
}

/** Modal on desktop, bottom sheet on mobile. Mounted only while active, so the
 *  enter animation plays on mount; Esc is handled by the app shell. */
export function PanelShell({ mobile, onClose, eyebrow, title, children, footer }: PanelShellProps) {
  const embedded = useContext(EmbeddedContext);
  if (embedded) {
    // Inline inside the Settings hub: section header + body + footer, no overlay / no close.
    return (
      <div className={styles.embedded}>
        {(eyebrow || title) && (
          <div className={styles.embeddedHead}>
            {eyebrow && <div className={styles.eyebrow}>{eyebrow}</div>}
            <div className={`display ${styles.title}`}>{title}</div>
          </div>
        )}
        <div className={styles.body}>
          <PreviewBanner />
          {children}
        </div>
        {footer && <div className={styles.footer}>{footer}</div>}
      </div>
    );
  }
  return (
    <>
      <div className={styles.backdrop} onClick={onClose} aria-hidden />
      <div
        className={`${styles.shell} ${mobile ? styles.sheet : styles.modal}`}
        role="dialog"
        aria-modal="true"
      >
        {mobile && (
          <div className={styles.grab}>
            <span className={styles.grabPill} />
          </div>
        )}
        <div className={styles.head}>
          <div style={{ flex: 1, minWidth: 0 }}>
            {eyebrow && <div className={styles.eyebrow}>{eyebrow}</div>}
            <div className={`display ${styles.title}`}>{title}</div>
          </div>
          <button
            type="button"
            className={styles.close}
            onClick={onClose}
            data-tip="Close (Esc)"
            aria-label="Close panel"
          >
            ✕
          </button>
        </div>
        <div className={styles.body}>
          <PreviewBanner />
          {children}
        </div>
        {footer && <div className={styles.footer}>{footer}</div>}
      </div>
    </>
  );
}
