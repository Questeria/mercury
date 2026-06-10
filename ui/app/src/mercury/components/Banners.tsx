import { Spinner } from "../icons";
import { REASON_CODE } from "../simulator";
import type { ReasonLabel } from "../types";
import styles from "./Banners.module.css";

interface GapBannerProps {
  /** Handoff or real-core label (e.g. ORDERING_GAP). */
  reason?: string;
  code?: number;
  text?: string;
}

/** Ordering-gap banner — amber, spinner, surfaces requires_client_retry. */
export function GapBanner({ reason = "ORDERING_GAP", code, text }: GapBannerProps) {
  const rc = code ?? REASON_CODE[reason as ReasonLabel] ?? 10;
  return (
    <div className={`${styles.gap} rise`}>
      <Spinner color="var(--mc-warn)" />
      <div style={{ flex: 1 }}>
        <div style={{ fontWeight: 600 }}>
          {text ?? "Ordering gap — fetching missing messages"}
        </div>
        <div className={`${styles.diag} mono`}>
          client_receive · {reason} · rc={rc} · requires_client_retry
        </div>
      </div>
    </div>
  );
}
