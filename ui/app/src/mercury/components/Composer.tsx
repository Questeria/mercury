import { useRef, useState, type KeyboardEvent } from "react";

import { fileToBase64 } from "../b64file";
import { LockIcon, PaperclipIcon, SendIcon, SmileIcon } from "../icons";
import { labelGloss, toneVar } from "../strings";
import type { MercuryThreadState, PanelId } from "../types";
import styles from "./Composer.module.css";

const MAX_ATTACH_BYTES = 4 * 1024 * 1024;

const EMOJIS = ["👍","❤️","😂","🎉","🔥","🙏","👀","✅","💯","😊","🤔","😍","😎","🚀","💪","✨","👏","🙌","😢","😮","🥳","🤝","😅","🤙"];

interface ComposerProps {
  thread: MercuryThreadState;
  mobile: boolean;
  onOpenPanel: (panel: PanelId) => void;
  /** LIVE attachment hook: when present, the paperclip becomes a REAL file picker — ≤4 MiB,
   *  read as base64 and handed to the caller (who sends it over the sealed channel; the backend
   *  chunks anything over 512 KiB transparently). When absent (demo app), the paperclip keeps
   *  opening the attachments panel, unchanged. */
  onAttach?: (file: { name: string; mime: string; dataB64: string }) => Promise<void>;
}

export function Composer({ thread, mobile, onOpenPanel, onAttach }: ComposerProps) {
  const view = thread.outboundView;
  const canSend = view.can_send;
  const reason = labelGloss(view.reason_label);

  const fileRef = useRef<HTMLInputElement>(null);
  const [attaching, setAttaching] = useState(false);
  const [attachErr, setAttachErr] = useState<string | null>(null);
  const [showEmoji, setShowEmoji] = useState(false);

  const placeholder = !canSend
    ? `Withheld / ${reason.gloss}`
    : view.requires_user_action
      ? "Message / verification will be requested"
      : thread.draftMentionsAi
        ? "Message / @ai will receive scoped context"
        : "Message";

  const statusColor = !canSend
    ? toneVar("bad")
    : view.requires_user_action
      ? toneVar("warn")
      : thread.draftMentionsAi
        ? toneVar("ai")
        : toneVar("muted");

  const onKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      thread.send();
    }
  };

  const onFilePicked = async (file: File | undefined) => {
    if (!file || !onAttach) return;
    setAttachErr(null);
    if (file.size > MAX_ATTACH_BYTES) {
      const size =
        file.size >= 1024 * 1024
          ? `${(file.size / (1024 * 1024)).toFixed(1)} MiB`
          : `${Math.ceil(file.size / 1024)} KiB`;
      setAttachErr(`"${file.name}" is ${size} — files up to 4 MiB are supported for now.`);
      return;
    }
    setAttaching(true);
    try {
      const dataB64 = await fileToBase64(file);
      await onAttach({ name: file.name, mime: file.type || "application/octet-stream", dataB64 });
    } catch (e) {
      setAttachErr(e instanceof Error ? e.message : String(e));
    } finally {
      setAttaching(false);
    }
  };

  return (
    <div className={styles.wrap} data-mobile={mobile ? "" : undefined}>
      <div className={styles.inputRow} data-blocked={canSend ? undefined : ""}>
        <div className={`${styles.prefix} mono`}>
          <span className={styles.prefixPath}>~/mercury-core</span>
          <span className={styles.prefixCaret}>{">"}</span>
        </div>
        <textarea
          className={styles.textarea}
          value={thread.draft}
          onChange={(e) => thread.setDraft(e.target.value)}
          onKeyDown={onKeyDown}
          rows={1}
          placeholder={placeholder}
          aria-label="Message"
        />
        {!thread.draft && canSend && <span className={`${styles.idleCursor} blink`} />}
        <div className={styles.emojiCell}>
          <div className={styles.emojiWrap}>
            <button
              type="button"
              className={styles.attachBtn}
              onClick={() => setShowEmoji((v) => !v)}
              disabled={!canSend}
              data-tip="Emoji"
              aria-label="Insert emoji"
              aria-expanded={showEmoji}
            >
              <SmileIcon size={16} className={styles.icon} />
            </button>
            {showEmoji && (
              <>
                <div className={styles.emojiBackdrop} onClick={() => setShowEmoji(false)} />
                <div className={styles.emojiPop} role="menu" aria-label="Emoji">
                  {EMOJIS.map((e) => (
                    <button
                      key={e}
                      type="button"
                      className={styles.emojiItem}
                      role="menuitem"
                      onClick={() => {
                        thread.setDraft(thread.draft + e);
                        setShowEmoji(false);
                      }}
                    >
                      {e}
                    </button>
                  ))}
                </div>
              </>
            )}
          </div>
        </div>
        <div className={styles.attachCell}>
          {onAttach ? (
            <>
              <input
                ref={fileRef}
                type="file"
                style={{ display: "none" }}
                aria-hidden
                tabIndex={-1}
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  e.target.value = "";
                  void onFilePicked(f);
                }}
              />
              <button
                type="button"
                className={styles.attachBtn}
                onClick={() => fileRef.current?.click()}
                disabled={attaching || !canSend}
                data-tip="Attach a file (≤4 MiB, end-to-end encrypted)"
                aria-label="Attach a file (up to 4 MiB, end-to-end encrypted)"
              >
                <PaperclipIcon size={15} className={styles.icon} />
              </button>
            </>
          ) : (
            <button
              type="button"
              className={styles.attachBtn}
              onClick={() => onOpenPanel("attachments")}
              data-tip="Encrypted attachments"
              aria-label="Open encrypted attachments"
            >
              <PaperclipIcon size={15} className={styles.icon} />
            </button>
          )}
        </div>
        <div className={styles.sendCell}>
          <button
            type="button"
            className={styles.sendBtn}
            onClick={() => thread.send()}
            disabled={!canSend}
            data-cansend={canSend ? "" : undefined}
            title={canSend ? "Send (Enter)" : reason.gloss}
            aria-label={canSend ? "Send message" : "Sending blocked"}
          >
            {canSend && <span className="iris-ring" style={{ borderRadius: 10 }} />}
            {canSend ? (
              <SendIcon size={15} className={styles.icon} />
            ) : (
              <LockIcon size={14} className={styles.icon} />
            )}
          </button>
        </div>
      </div>
      <div className={`${styles.diag} mono`} style={{ color: statusColor }}>
        <span>
          outbound_send / {view.reason_label} / rc={view.reason_code}
        </span>
        {view.requires_user_action && (
          <span style={{ color: toneVar("warn") }}>requires_user_action</span>
        )}
        {!view.can_persist_ciphertext && !view.accepted && (
          <span style={{ color: toneVar("bad") }}>no persistence</span>
        )}
        {thread.draftMentionsAi && <span style={{ color: toneVar("ai") }}>@ai to scoped context</span>}
        {/* Persistent live region so a screen reader announces attach progress + a rejection
            (e.g. "file is 6.2 MiB…") instead of the attach silently failing. Polite: queues behind
            the user's own typing. The region must exist before its content changes to announce. */}
        <span role="status" aria-live="polite">
          {attaching && <span>attaching…</span>}
          {attachErr && <span style={{ color: toneVar("bad") }}>{attachErr}</span>}
        </span>
        <span className={styles.sendHint}>Enter send</span>
      </div>
    </div>
  );
}
