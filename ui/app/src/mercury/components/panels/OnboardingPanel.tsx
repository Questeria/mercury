// Start: real documentation of what Mercury can actually do today, plus one honest "not yet"
// line. Every feature listed here is shipped and backend-enforced; the Open buttons jump to the
// matching panel. Nothing simulated.

import type { PanelId } from "../../types";
import { PanelShell } from "./PanelShell";
import { GhostButton, IridButton, Section } from "./primitives";
import styles from "./panels.module.css";

interface OnboardingPanelProps {
  mobile: boolean;
  onClose: () => void;
  openPanel: (panel: PanelId) => void;
}

const FEATURES: { title: string; detail: string; panel?: PanelId }[] = [
  {
    title: "Connect your way",
    detail: "Add people by account id, invite link, QR code, one-shot pairing code, or @username (checked against the key-transparency directory).",
  },
  {
    title: "Verify who you're talking to",
    detail: "Compare the safety number out-of-band, then mark the conversation verified — the ✓ shows in the thread header.",
    panel: "trust",
  },
  {
    title: "Disappearing messages",
    detail: "Per conversation (1h / 1d / 1w), auto-deleting on both sides. Set it from the conversation's ⋯ menu.",
  },
  {
    title: "Mute, block, delete",
    detail: "Silence notifications, refuse + drop a contact's messages, or delete a conversation — for you or for everyone.",
    panel: "notifications",
  },
  {
    title: "Group chats",
    detail: "Create a group from your contacts (up to 16) with real MLS encryption, creator-administered. Classical, not-yet-post-quantum — shown honestly in the group panel.",
  },
  {
    title: "Attachments up to 4 MiB",
    detail: "Files ride the same end-to-end encryption as text (larger ones are chunked into sealed frames); images preview inline. Both sides need 0.1.26+.",
    panel: "attachments",
  },
  {
    title: "Encrypted backup",
    detail: "Export your whole account as one passphrase-sealed file; restore it on the desktop app.",
    panel: "recovery",
  },
  {
    title: "Auto-update & tray delivery",
    detail: "Signed updates install themselves on Windows (Linux re-downloads new versions); closing the window hides to the tray, which keeps receiving (notifications show sender + count, never text).",
    panel: "updates",
  },
];

export function OnboardingPanel({ mobile, onClose, openPanel }: OnboardingPanelProps) {
  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Start"
      title="What Mercury does today"
      footer={<GhostButton onClick={onClose}>Close</GhostButton>}
    >
      <Section kicker="Working now — all end-to-end encrypted">
        <div className={styles.colGap}>
          {FEATURES.map((f) => (
            <div className={styles.attachmentRow} key={f.title}>
              <span className={styles.dot6} style={{ background: "var(--mc-ok)", alignSelf: "center" }} />
              <div style={{ minWidth: 0, flex: 1 }}>
                <div className={styles.attachmentName}>{f.title}</div>
                <div className={styles.attachmentMeta} style={{ whiteSpace: "normal", lineHeight: 1.45 }}>
                  {f.detail}
                </div>
              </div>
              {f.panel && <IridButton onClick={() => openPanel(f.panel as PanelId)}>Open</IridButton>}
            </div>
          ))}
        </div>
      </Section>

      <Section kicker="Not yet">
        <div className={styles.card} style={{ color: "var(--mc-ink2)", fontSize: 11.5, lineHeight: 1.55 }}>
          <span style={{ color: "var(--mc-warn)", fontWeight: 600 }}>Honestly missing:</span> true
          push while the app is fully quit, post-quantum protection for groups (1:1 is already
          hybrid post-quantum), and iOS / Android / macOS builds. If a screen here claims otherwise,
          it carries a preview banner.
        </div>
      </Section>

      <Section kicker="The trust model in one line">
        <div className={styles.card} style={{ color: "var(--mc-ink2)", fontSize: 11.5, lineHeight: 1.55 }}>
          Keys never leave your device, the relay only ever sees ciphertext, and the one thing that
          proves you&apos;re talking to the real person is comparing the safety number over a
          channel you trust.
        </div>
      </Section>
    </PanelShell>
  );
}
