// Per-conversation settings overlay: the real safety number (out-of-band MITM verification),
// disappearing timer, mute/block, clear-history, and the delete actions. Self-contained — it
// drives the live controller directly and fetches the safety number on open.
//
// GROUP mode (a `group` prop is present): only the items that apply to a group are shown —
// Pin/Unpin, Mute/Unmute, Clear history, Disappearing timer (this-device-only for groups in v1),
// and Leave/Close group. Safety number, Block, and the Delete items are 1:1 concepts and are
// omitted. Pin/Unpin applies to BOTH 1:1 and groups (a persisted rail-ordering preference).

import { useEffect, useState, type CSSProperties } from "react";

import type { LiveConversations } from "../liveConversations";
import type { GroupInfo } from "../messaging";
import type { Person } from "../types";

interface ConversationMenuProps {
  controller: LiveConversations;
  peer: string;
  person: Person;
  muted: boolean;
  blocked: boolean;
  /** This conversation is pinned to the top of the rail (1:1 and groups alike). */
  pinned: boolean;
  /** Toggle pin — called with the DESIRED state; the parent persists it via the controller. */
  onPin: (on: boolean) => void;
  disappearingSecs: number;
  onClose: () => void;
  /** GROUP mode: the open conversation's group metadata (null/absent for 1:1). */
  group?: GroupInfo | null;
  /** True when YOU created the group — leaving then closes it for everyone (v1). */
  isCreator?: boolean;
}

export function ConversationMenu({
  controller,
  peer,
  person,
  muted,
  blocked,
  pinned,
  onPin,
  disappearingSecs,
  onClose,
  group,
  isCreator,
}: ConversationMenuProps) {
  const isGroup = group != null;
  const [grouped, setGrouped] = useState<string | null>(null);
  const [snError, setSnError] = useState(false);
  const [confirmMode, setConfirmMode] = useState<"" | "me" | "everyone" | "leave">("");
  const [timer, setTimer] = useState(disappearingSecs);

  useEffect(() => {
    if (isGroup) return; // safety numbers are a 1:1 concept — no fetch for groups
    let cancelled = false;
    void (async () => {
      const sn = await controller.safetyNumber(peer);
      if (cancelled) return;
      if (sn) setGrouped(sn.grouped);
      else setSnError(true);
    })();
    return () => {
      cancelled = true;
    };
  }, [controller, peer, isGroup]);

  return (
    <div role="dialog" aria-modal="true" aria-label="Conversation settings" style={overlay} onClick={onClose}>
      <div style={card} onClick={(e) => e.stopPropagation()}>
        <div style={head}>
          <span style={{ fontWeight: 700, fontSize: 14 }}>
            {isGroup ? "Group" : "Conversation"} · {person.name}
          </span>
          <button type="button" style={closeBtn} onClick={onClose} aria-label="Close">
            ✕
          </button>
        </div>

        {!isGroup && (
          <section style={section}>
            <div style={kicker}>Safety number</div>
            <div style={snBox} className="mono">
              {grouped ?? (snError ? "Unavailable — add this contact first." : "Computing…")}
            </div>
            <div style={note}>
              Compare these digits with {person.name} <strong>in person</strong> or over a call you
              recognize. If they match on both devices, no one is in the middle — this is the backstop
              that makes any way of connecting safe, even if a code was shared over an insecure channel.
            </div>
          </section>
        )}

        <section style={section}>
          <div style={kicker}>
            Disappearing messages{isGroup ? " · this device only" : ""}
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            {(
              [
                ["Off", 0],
                ["1h", 3600],
                ["1d", 86400],
                ["1w", 604800],
              ] as [string, number][]
            ).map(([label, secs]) => {
              const on = timer === secs;
              return (
                <button
                  key={secs}
                  type="button"
                  onClick={() => {
                    setTimer(secs);
                    void controller.setDisappearing(peer, secs);
                  }}
                  style={{
                    flex: 1,
                    border: "1px solid var(--mc-border)",
                    borderRadius: 8,
                    padding: "7px 0",
                    fontSize: 12,
                    cursor: "pointer",
                    background: on ? "var(--mc-accent)" : "var(--mc-surface-warm)",
                    color: on ? "#fff" : "var(--mc-ink)",
                  }}
                >
                  {label}
                </button>
              );
            })}
          </div>
          <div style={note}>
            {isGroup ? (
              <>
                For groups (v1) the timer applies on <strong>this device only</strong> — it is not
                yet propagated to the other members. Timer propagation is a 1:1 feature for now.
              </>
            ) : (
              <>
                When on, messages auto-delete on <strong>both</strong> sides after the chosen time
                (applies going forward). The peer is told the timer over the encrypted channel.
              </>
            )}
          </div>
        </section>

        <section style={section}>
          <div style={kicker}>Manage</div>
          <button
            type="button"
            style={actionBtn}
            onClick={() => {
              onPin(!pinned);
              onClose();
            }}
          >
            {pinned ? "Unpin conversation" : "Pin conversation"}{" "}
            <span style={subtle}>
              · {pinned ? "back to normal order" : "keep it at the top of the list"}
            </span>
          </button>
          <button
            type="button"
            style={actionBtn}
            onClick={() => {
              void (muted ? controller.unmute(peer) : controller.mute(peer));
              onClose();
            }}
          >
            {muted ? "Unmute conversation" : "Mute conversation"}{" "}
            <span style={subtle}>· {muted ? "notifications on" : "silence notifications"}</span>
          </button>
          <button
            type="button"
            style={actionBtn}
            onClick={() => {
              void controller.clearHistory(peer);
              onClose();
            }}
          >
            Clear chat history{" "}
            <span style={subtle}>· {isGroup ? "on this device · keeps the group" : "keeps the contact"}</span>
          </button>
          {!isGroup && (
            <button
              type="button"
              style={blocked ? actionBtn : dangerBtn}
              onClick={() => {
                void (blocked ? controller.unblock(peer) : controller.block(peer));
                onClose();
              }}
            >
              {blocked ? "Unblock contact" : "Block contact"}{" "}
              <span style={subtle}>
                · {blocked ? "restore send + receive" : "refuse + drop their messages"}
              </span>
            </button>
          )}

          {isGroup ? (
            confirmMode === "" ? (
              <button type="button" style={dangerBtn} onClick={() => setConfirmMode("leave")}>
                {group.ended ? "Remove group" : isCreator ? "Close group" : "Leave group"}{" "}
                <span style={subtle}>
                  ·{" "}
                  {group.ended
                    ? "it is already closed; drop it from your list"
                    : isCreator
                      ? "you created it — closing ends it for everyone"
                      : "stop receiving; others keep the group"}
                </span>
              </button>
            ) : (
              <div style={{ display: "flex", gap: 8 }}>
                <button
                  type="button"
                  style={dangerBtnFilled}
                  onClick={() => {
                    void controller.leaveGroup(peer);
                    onClose();
                  }}
                >
                  {group.ended
                    ? "Remove group — confirm"
                    : isCreator
                      ? "Close for everyone — confirm"
                      : "Leave group — confirm"}
                </button>
                <button type="button" style={actionBtn} onClick={() => setConfirmMode("")}>
                  Cancel
                </button>
              </div>
            )
          ) : confirmMode === "" ? (
            <>
              <button type="button" style={dangerBtn} onClick={() => setConfirmMode("me")}>
                Delete conversation <span style={subtle}>· for me, on this device</span>
              </button>
              <button type="button" style={dangerBtn} onClick={() => setConfirmMode("everyone")}>
                Delete for everyone <span style={subtle}>· wipes it on their device too</span>
              </button>
            </>
          ) : (
            <div style={{ display: "flex", gap: 8 }}>
              <button
                type="button"
                style={dangerBtnFilled}
                onClick={() => {
                  void (confirmMode === "everyone"
                    ? controller.deleteForEveryone(peer)
                    : controller.deleteConversation(peer));
                  onClose();
                }}
              >
                {confirmMode === "everyone"
                  ? "Delete for everyone — confirm"
                  : "Delete for me — confirm"}
              </button>
              <button type="button" style={actionBtn} onClick={() => setConfirmMode("")}>
                Cancel
              </button>
            </div>
          )}
        </section>
      </div>
    </div>
  );
}

const overlay: CSSProperties = {
  position: "fixed",
  inset: 0,
  zIndex: 60,
  background: "rgba(0,0,0,0.45)",
  backdropFilter: "blur(2px)",
  WebkitBackdropFilter: "blur(2px)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  padding: 20,
};

const card: CSSProperties = {
  width: "min(440px, 92vw)",
  maxHeight: "86vh",
  overflowY: "auto",
  background: "var(--mc-surface)",
  color: "var(--mc-ink)",
  border: "1px solid var(--mc-border)",
  borderRadius: 16,
  padding: 18,
  boxShadow: "0 24px 64px rgba(0,0,0,0.4)",
};

const head: CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  marginBottom: 14,
};

const closeBtn: CSSProperties = {
  border: "1px solid var(--mc-border)",
  background: "var(--mc-surface-warm)",
  color: "var(--mc-ink2)",
  borderRadius: 8,
  width: 28,
  height: 28,
  cursor: "pointer",
};

const section: CSSProperties = { marginBottom: 16 };

const kicker: CSSProperties = {
  fontSize: 10.5,
  letterSpacing: 1,
  textTransform: "uppercase",
  color: "var(--mc-muted)",
  marginBottom: 8,
};

const snBox: CSSProperties = {
  background: "var(--mc-surface-warm)",
  border: "1px solid var(--mc-border)",
  borderRadius: 10,
  padding: "12px 14px",
  fontSize: 14,
  letterSpacing: 1.5,
  lineHeight: 1.7,
  wordBreak: "break-word",
  color: "var(--mc-ink)",
};

const note: CSSProperties = {
  fontSize: 11.5,
  color: "var(--mc-muted)",
  marginTop: 9,
  lineHeight: 1.5,
};

const actionBtn: CSSProperties = {
  display: "block",
  width: "100%",
  textAlign: "left",
  border: "1px solid var(--mc-border)",
  background: "var(--mc-surface-warm)",
  color: "var(--mc-ink)",
  borderRadius: 10,
  padding: "10px 12px",
  fontSize: 13,
  cursor: "pointer",
  marginBottom: 8,
};

const dangerBtn: CSSProperties = {
  ...actionBtn,
  color: "var(--mc-bad)",
  borderColor: "color-mix(in srgb, var(--mc-bad) 35%, transparent)",
};

const dangerBtnFilled: CSSProperties = {
  flex: 1,
  border: "none",
  background: "var(--mc-bad)",
  color: "#fff",
  borderRadius: 10,
  padding: "10px 12px",
  fontSize: 13,
  fontWeight: 600,
  cursor: "pointer",
};

const subtle: CSSProperties = { color: "var(--mc-muted)", fontWeight: 400, fontSize: 11.5 };
