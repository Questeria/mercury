import { SearchIcon } from "../icons";
import { MERCURY_PEOPLE } from "../simulator";
import { toneVar } from "../strings";
import type { Tone } from "../types";
import { Avatar } from "./Avatar";
import styles from "./Rail.module.css";

interface Room {
  id: string;
  n: string;
  name: string;
  sub: string;
  active?: boolean;
  tone: Tone;
}

const ROOMS: Room[] = [
  { id: "core", n: "01", name: "mercury-core", sub: "4 · just now", active: true, tone: "ok" },
  { id: "sec", n: "02", name: "security-review", sub: "6 · 12m", tone: "ok" },
  { id: "eng", n: "03", name: "eng-leads", sub: "11 · 1h", tone: "warn" },
  { id: "ai", n: "04", name: "ai-policy", sub: "3 · 2h", tone: "ok" },
  { id: "me", n: "05", name: "You · notes", sub: "yesterday", tone: "ok" },
];

interface RailProps {
  onClose?: () => void;
  onOpenProfile: (who: string) => void;
}

export function Rail({ onClose, onOpenProfile }: RailProps) {
  return (
    <div className={styles.rail}>
      <div className={styles.header}>
        <div className={styles.wordmark}>
          <span className="iris-text">Mercury</span>
        </div>
        <span className={`${styles.version} mono`}>v0.1.12</span>
        {onClose && (
          <button type="button" className={styles.close} onClick={onClose} data-tip="Hide rooms" aria-label="Hide rooms">
            ✕
          </button>
        )}
      </div>
      <div className={styles.searchWrap}>
        <div className={styles.search}>
          <SearchIcon size={12} />
          Search
          <span className={`${styles.kbd} mono`}>⌘K</span>
        </div>
      </div>
      <div className={`${styles.kicker} mono`}>rooms</div>
      <div className={styles.rooms}>
        {ROOMS.map((r) => (
          <div key={r.id} className={styles.room} data-active={r.active ? "" : undefined}>
            <span className={`${styles.roomNum} mono`}>{r.n}</span>
            <span className={styles.roomDot} style={{ background: toneVar(r.tone) }} />
            <div style={{ minWidth: 0, flex: 1 }}>
              <div className={styles.roomName} data-active={r.active ? "" : undefined}>
                {r.name}
              </div>
              <div className={`${styles.roomSub} mono`}>{r.sub}</div>
            </div>
          </div>
        ))}
      </div>
      <button type="button" className={styles.footer} onClick={() => onOpenProfile("me")} data-tip="Your profile">
        <Avatar person={MERCURY_PEOPLE.me} size={26} />
        <div style={{ minWidth: 0, fontSize: 12 }}>
          <div style={{ fontWeight: 500 }}>You</div>
          <div className={`${styles.footerSub} mono`}>device a1f3 · trusted</div>
        </div>
        <span className={styles.footerArrow}>›</span>
      </button>
    </div>
  );
}
