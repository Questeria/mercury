import { MERCURY_PEOPLE } from "../../simulator";
import type { ThemeChoice } from "../../MercuryApp";
import { Avatar } from "../Avatar";
import { PanelShell } from "./PanelShell";
import { GhostButton, Kv, NavRow, Section, Segment } from "./primitives";
import styles from "./panels.module.css";

interface SettingsPanelProps {
  mobile: boolean;
  onClose: () => void;
  dark: boolean;
  theme: ThemeChoice;
  setTheme: (t: ThemeChoice) => void;
  onOpenProfile: (who: string) => void;
  onOpenNotifications: () => void;
  onOpenRecovery: () => void;
  onOpenTrust: () => void;
  onOpenEncryption: () => void;
  onOpenAttachments: () => void;
  onOpenGroup: () => void;
  onOpenPolicy: () => void;
  onOpenUpdates: () => void;
  onOpenOnboarding: () => void;
}

export function SettingsPanel({
  mobile,
  onClose,
  dark,
  theme,
  setTheme,
  onOpenProfile,
  onOpenNotifications,
  onOpenRecovery,
  onOpenTrust,
  onOpenEncryption,
  onOpenAttachments,
  onOpenGroup,
  onOpenPolicy,
  onOpenUpdates,
  onOpenOnboarding,
}: SettingsPanelProps) {
  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Mercury"
      title="Settings"
      footer={<GhostButton onClick={onClose}>Done</GhostButton>}
    >
      <button
        type="button"
        onClick={() => onOpenProfile("me")}
        className={styles.navRow}
        style={{ borderRadius: 12, gap: 12, marginBottom: 18 }}
      >
        <Avatar person={MERCURY_PEOPLE.me} size={40} />
        <div style={{ minWidth: 0, flex: 1 }}>
          <div style={{ fontSize: 14, fontWeight: 700, color: "var(--mc-ink)", marginBottom: 1 }}>You</div>
          <div style={{ fontSize: 11, color: "var(--mc-muted)" }}>device a1f3 / 3 devices / trusted</div>
        </div>
        <span className={styles.chev}>{">"}</span>
      </button>

      <Section kicker="Appearance">
        <Segment<ThemeChoice>
          value={theme}
          onChange={setTheme}
          options={[
            { value: "light", label: "Light" },
            { value: "dark", label: "Dark" },
            { value: "auto", label: "Auto", sub: `device / ${dark ? "dark" : "light"}` },
          ]}
        />
      </Section>

      <Section kicker="Security">
        <NavRow label="Verify devices" sub="Compare safety numbers and review the device list." onClick={onOpenTrust} />
        <NavRow label="Encryption" sub="What protects this room, and what does not." divider onClick={onOpenEncryption} />
        <NavRow label="Recovery" sub="Recovery phrase / designated contacts." divider onClick={onOpenRecovery} />
        <NavRow label="Room safety" sub="Membership, epoch rotation, and AI revoke state." divider onClick={onOpenGroup} />
        <NavRow label="Policy explanation" sub="Helix attestations and current gate decisions." divider onClick={onOpenPolicy} />
      </Section>

      <Section kicker="Notifications">
        <NavRow label="Lock-screen previews" sub="Previews / sound / AI alerts." onClick={onOpenNotifications} />
      </Section>

      <Section kicker="Files and releases">
        <NavRow label="Encrypted attachments" sub="Stage files and review relay-visible metadata." onClick={onOpenAttachments} />
        <NavRow label="Signed updates" sub="Manual downloads, checksums, and auto-update status." divider onClick={onOpenUpdates} />
        <NavRow label="Launch checklist" sub="The quick setup pass for a new device or room." divider onClick={onOpenOnboarding} />
      </Section>

      <Section kicker="About">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.6 }}>
          <Kv k="Build" v="mercury 0.1.12 / client 2026.06.06" />
          <Kv k="Core" v="mercury-core 0.1.12 / binding 0.1.12" />
          <Kv k="Audit" v="Key Transparency log / live" />
          <Kv k="Region" v="eu-west / scoped to org" />
        </div>
      </Section>
    </PanelShell>
  );
}
