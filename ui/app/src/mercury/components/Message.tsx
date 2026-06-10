import { useEffect, useRef, useState } from "react";
import type { CSSProperties } from "react";

import { fmtMsgTime } from "../format";
import { ShieldIcon, XCircleIcon } from "../icons";
import { MERCURY_PEOPLE, REASON_CODE } from "../simulator";
import { labelGloss } from "../strings";
import type { Message as Msg } from "../types";
import { Avatar } from "./Avatar";
import { GapBanner } from "./Banners";
import styles from "./Message.module.css";

interface MessageProps {
  m: Msg;
  prev?: Msg;
  mobile: boolean;
  onOpenProfile?: (who: string) => void;
}

export function Message({ m, prev, mobile, onOpenProfile }: MessageProps) {
  if (m.kind === "system") {
    return (
      <div className={`${styles.system} rise`}>
        <span className={styles.rule} />
        <span className={styles.systemText}>{m.text}</span>
        <span className={styles.rule} />
      </div>
    );
  }
  if (m.kind === "gap") {
    return <GapBanner reason={m.decision?.reason_label} code={m.decision?.reason_code} text={m.text} />;
  }
  if (m.kind === "send_blocked") return <SendBlocked m={m} />;
  if (m.kind === "rejected_inbound") return <InboundBlocked m={m} />;

  const person = MERCURY_PEOPLE[m.author ?? "rin"] ?? MERCURY_PEOPLE.rin;
  const isMe = m.author === "me";
  const isAi = !!person.isAi;
  const grouped =
    !!prev && prev.author === m.author && (prev.kind === "incoming" || prev.kind === "outgoing");

  if (isMe) return <OutgoingBubble m={m} mobile={mobile} />;
  return (
    <IncomingBubble
      m={m}
      mobile={mobile}
      isAi={isAi}
      grouped={grouped}
      onOpenProfile={onOpenProfile}
    />
  );
}

function IncomingBubble({
  m,
  mobile,
  isAi,
  grouped,
  onOpenProfile,
}: {
  m: Msg;
  mobile: boolean;
  isAi: boolean;
  grouped: boolean;
  onOpenProfile?: (who: string) => void;
}) {
  const person = MERCURY_PEOPLE[m.author ?? "rin"] ?? MERCURY_PEOPLE.rin;
  const rowStyle = {
    maxWidth: mobile ? "92%" : "78%",
    ["--hue" as string]: String(person.hue),
  } as CSSProperties;

  return (
    <div className={`${styles.incomingRow} rise`} style={rowStyle}>
      {grouped ? (
        <div className={styles.avatarSpacer} />
      ) : (
        <Avatar
          person={person}
          size={28}
          onClick={onOpenProfile ? () => onOpenProfile(m.author ?? "rin") : undefined}
        />
      )}
      <div className={styles.bubbleCol}>
        {!grouped && (
          <div className={styles.nameRow}>
            {isAi ? (
              <span className="iris-text" style={{ fontWeight: 600 }}>
                {person.name}
              </span>
            ) : (
              <span className={styles.authorName}>{person.name}</span>
            )}
            {isAi && <span className={styles.aiTag}>scoped · read-only</span>}
          </div>
        )}
        <div className={`${styles.bubble} ${isAi ? styles.aiBubble : styles.tintBubble}`}>
          {isAi && <span className="iris-ring" style={{ borderRadius: 12 }} />}
          <span style={{ position: "relative" }}>
            {isAi ? (m.streamed ? m.text : <StreamingText id={m.id} text={m.text} />) : m.text}
          </span>
        </div>
        <div className={styles.time}>{fmtMsgTime(m.ts)}</div>
      </div>
    </div>
  );
}

function OutgoingBubble({ m, mobile }: { m: Msg; mobile: boolean }) {
  const reasonLabel = m.decision?.reason_label;
  const rowStyle = {
    maxWidth: mobile ? "88%" : "74%",
    ["--hue" as string]: String(MERCURY_PEOPLE.me.hue),
  } as CSSProperties;
  return (
    <div className={`${styles.outgoingRow} rise`} style={rowStyle}>
      <div className={styles.outBubble}>
        <span className="iris-ring" style={{ borderRadius: 12 }} />
        <span style={{ position: "relative" }}>{m.text}</span>
      </div>
      <div className={styles.outFooter}>
        <span className={m.status === "delivered" ? styles.delivered : undefined}>
          {m.status === "sending"
            ? "○ sending"
            : m.status === "delivered"
              ? "● delivered"
              : "● sent"}
        </span>
        <span>· {fmtMsgTime(m.ts)}</span>
        {reasonLabel && reasonLabel !== "ACCEPTED" && (
          <span className={styles.warnLabel}>· {reasonLabel}</span>
        )}
      </div>
    </div>
  );
}

function SendBlocked({ m }: { m: Msg }) {
  const label = m.decision?.reason_label;
  const gloss = label ? labelGloss(label).gloss : "Blocked";
  return (
    <div className={`${styles.sendBlocked} rise`}>
      <div className={styles.blockHead}>
        <XCircleIcon size={14} />
        <span style={{ fontWeight: 600 }}>Send withheld · {gloss}</span>
      </div>
      <div className={styles.blockQuote}>"{m.text}"</div>
      <div className={`${styles.diag} mono`}>
        outbound_send · {label} · rc={m.decision?.reason_code} · ciphertext not persisted
      </div>
    </div>
  );
}

function InboundBlocked({ m }: { m: Msg }) {
  return (
    <div className={`${styles.inboundBlocked} rise`}>
      <div className={styles.blockHead}>
        <ShieldIcon size={13} />
        <span style={{ fontWeight: 600, color: "var(--mc-ink2)" }}>Message withheld</span>
        {m.decision?.requires_user_action && (
          <span className={`${styles.reqTag} mono`}>requires_user_action</span>
        )}
      </div>
      <div style={{ fontStyle: "italic" }}>{m.text}</div>
      <div className={`${styles.diagMute} mono`}>
        client_receive · {m.decision?.reason_label ?? "SENDER_TRUST_REJECTED"} · rc=
        {m.decision?.reason_code ?? REASON_CODE.SENDER_TRUST_REJECTED}
      </div>
    </div>
  );
}

// AI streaming text — persists completion across re-renders so a streamed
// message doesn't re-animate when the list updates.
const streamedIds = new Set<number>();

function StreamingText({ id, text }: { id: number; text: string }) {
  const [shown, setShown] = useState(() => (streamedIds.has(id) ? text.length : 0));
  const shownRef = useRef(shown);
  shownRef.current = shown;

  useEffect(() => {
    if (streamedIds.has(id)) return;
    let i = shownRef.current;
    const step = Math.max(1, Math.round(text.length / 80));
    const handle = window.setInterval(() => {
      i += step;
      if (i >= text.length) {
        i = text.length;
        window.clearInterval(handle);
        streamedIds.add(id);
      }
      setShown(i);
    }, 18);
    return () => window.clearInterval(handle);
  }, [id, text.length]);

  return (
    <span>
      {text.slice(0, shown)}
      {shown < text.length && <span className={`${styles.caret} blink`} />}
    </span>
  );
}
