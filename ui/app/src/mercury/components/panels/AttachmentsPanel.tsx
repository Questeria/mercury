// Attachments: an honest explainer of the SHIPPED small-file feature. The actual feature lives
// in the composer (📎 picks a file, ≤4 MiB, sent over the same sealed channel as text — chunked
// transparently above 512 KiB) and in the message bubbles (inline image preview + Save). This
// panel only documents it — no controls, nothing simulated.

import { PaperclipIcon } from "../../icons";
import { PanelShell } from "./PanelShell";
import { Bullet, GhostButton, Section } from "./primitives";
import styles from "./panels.module.css";

interface AttachmentsPanelProps {
  mobile: boolean;
  onClose: () => void;
}

export function AttachmentsPanel({ mobile, onClose }: AttachmentsPanelProps) {
  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Attachments"
      title="Small files, same encryption"
      footer={<GhostButton onClick={onClose}>Close</GhostButton>}
    >
      <Section kicker="How it works">
        <div className={styles.card} style={{ fontSize: 12, color: "var(--mc-ink2)", lineHeight: 1.6 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
            <PaperclipIcon size={15} color="var(--mc-ai)" />
            <span style={{ color: "var(--mc-ink)", fontWeight: 700, fontSize: 13 }}>
              The 📎 in the composer sends a file to the open conversation.
            </span>
          </div>
          Files up to <strong>4 MiB</strong> ride the <strong>same end-to-end encryption as
          text</strong> — sealed on your device, ratcheted, and routed through the relay as opaque
          bytes. Files over 512 KiB are split into <strong>encrypted chunks</strong> and reassembled
          on the receiving device. There is no separate file server and no side channel: an
          attachment is just sealed messages with bytes in them.
        </div>
      </Section>

      <Section kicker="What you get">
        <div className={styles.colGap}>
          <Bullet tone="ok" text="Images preview inline in the conversation; every file shows its name with a Save button." />
          <Bullet tone="ok" text="Save writes to your Downloads folder (desktop app); the browser build offers a plain download." />
          <Bullet tone="ok" text="The relay sees only ciphertext — no plaintext filename, no content type." />
        </div>
      </Section>

      <Section kicker="Honest limits">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          <span style={{ color: "var(--mc-warn)", fontWeight: 600 }}>Attachments need both sides on
          0.1.26 or newer.</span>{" "}
          0.1.26 raised the sealed-frame size limit (and added chunking for files over 512 KiB), so a
          pre-0.1.26 receiver silently drops any attachment beyond a few KiB — no error on either
          side, the file just never appears. Only a tiny file reliably reaches an older contact;
          anything meaningful — a photo, a document — needs you both on 0.1.26+.
          <br />
          <span style={{ color: "var(--mc-warn)", fontWeight: 600 }}>Larger files are not supported
          yet.</span>{" "}
          4 MiB covers documents, screenshots, and most photos; video and full-size media still need
          more than the current design. The composer rejects oversized picks up front with a clear
          error rather than silently degrading them.
        </div>
      </Section>
    </PanelShell>
  );
}
