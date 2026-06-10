import { PanelShell } from "./PanelShell";
import { Bullet, GhostButton, Kv, Section } from "./primitives";
import styles from "./panels.module.css";

interface EncryptionPanelProps {
  mobile: boolean;
  onClose: () => void;
}

export function EncryptionPanel({ mobile, onClose }: EncryptionPanelProps) {
  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Encryption"
      title="End-to-end · scoped to this room"
      footer={<GhostButton onClick={onClose}>Close</GhostButton>}
    >
      <div
        style={{
          padding: "12px 14px",
          marginBottom: 16,
          background: "var(--mc-ok-soft)",
          border: "1px solid color-mix(in srgb, var(--mc-ok) 27%, transparent)",
          borderRadius: 10,
          display: "flex",
          alignItems: "center",
          gap: 10,
        }}
      >
        <span className={styles.dot8} style={{ background: "var(--mc-ok)" }} />
        <div style={{ flex: 1, fontSize: 12.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          Messages are encrypted on your device. The server stores only ciphertext. Mercury never sees
          the plaintext of any message in this room.
        </div>
      </div>

      <Section kicker="What this means">
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <Bullet tone="ok" text="Only the people in this room can read what's said here." />
          <Bullet tone="ok" text="The server can't read messages, even with a subpoena." />
          <Bullet tone="ok" text="If a device key changes, the next send is held until you verify." />
          <Bullet tone="muted" text="Metadata (room id, peer list, timestamps) is still server-visible." />
        </div>
      </Section>

      <Section kicker="Algorithm">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.6 }}>
          <Kv k="Cipher" v="XChaCha20-Poly1305" />
          <Kv k="Ratchet" v="Double Ratchet · per-message forward secrecy" />
          <Kv k="Identity" v="Ed25519 long-term · X25519 ephemeral" />
          <Kv k="Group" v="Sender keys + per-recipient envelope" />
          <Kv k="Audit" v="Key Transparency log · binding-enforced" />
        </div>
      </Section>

      <Section kicker="Room fingerprint">
        <div
          className={styles.card}
          style={{ fontSize: 13, color: "var(--mc-ink)", letterSpacing: 0.8, textAlign: "center", fontWeight: 600, padding: "12px 14px" }}
        >
          a1f3 · 9c20 · 8e4b · 7d12 · 6b8e · 0a35
        </div>
        <div style={{ fontSize: 10.5, color: "var(--mc-muted)", marginTop: 6 }}>
          Open Trust to compare the full safety number with your correspondents.
        </div>
      </Section>
    </PanelShell>
  );
}
