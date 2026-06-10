import { ShieldCheckIcon } from "../icons";
import { REASON_CODE } from "../simulator";
import type { MercuryThreadState } from "../types";
import styles from "./Tofu.module.css";

interface TofuProps {
  pending: NonNullable<MercuryThreadState["pendingTofu"]>;
  onAccept: () => void;
  onCancel: () => void;
  mobile: boolean;
}

/** First-use verification sheet — parks a send until the user confirms. */
export function Tofu({ onAccept, onCancel, mobile }: TofuProps) {
  return (
    <div className={`${styles.sheet} rise`} data-mobile={mobile ? "" : undefined}>
      <div className={styles.body}>
        <ShieldCheckIcon size={18} color="var(--mc-warn)" />
        <div className={styles.copy}>
          <div className={styles.heading}>First-use verification</div>
          <span className={styles.text}>
            A correspondent has a new device key. Confirm to release the dispatch, or compare safety
            numbers first.
          </span>
          <div className={`${styles.sn} mono`}>SN · a1f3 · 9c20 · 8e4b · 7d12 · 6b8e · 0a35</div>
        </div>
      </div>
      <div className={styles.actions}>
        <button type="button" className={styles.ghostBtn} onClick={onCancel}>
          Cancel
        </button>
        <button type="button" className={styles.iridBtn} onClick={onAccept}>
          <span className="iris-ring" style={{ borderRadius: 8 }} />
          <span style={{ position: "relative" }}>Confirm &amp; send</span>
        </button>
      </div>
      <div className={`${styles.diag} mono`}>
        outbound_send · TOFU_PENDING · rc={REASON_CODE.TOFU_PENDING} · requires_user_action
      </div>
    </div>
  );
}
