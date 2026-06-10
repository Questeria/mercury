import type { MercuryThreadState, PanelId } from "../types";
import type { ThemeChoice } from "../MercuryApp";
import type { Scenario } from "./DevControls";
import { AiPanel } from "./panels/AiPanel";
import { AttachmentsPanel } from "./panels/AttachmentsPanel";
import { BootstrapPanel } from "./panels/BootstrapPanel";
import { EncryptionPanel } from "./panels/EncryptionPanel";
import { GroupSafetyPanel } from "./panels/GroupSafetyPanel";
import { NotificationsPanel } from "./panels/NotificationsPanel";
import { OnboardingPanel } from "./panels/OnboardingPanel";
import { PeersPanel } from "./panels/PeersPanel";
import { PolicyPanel } from "./panels/PolicyPanel";
import { ProfilePanel } from "./panels/ProfilePanel";
import { RecoveryPanel } from "./panels/RecoveryPanel";
import { SettingsPanel } from "./panels/SettingsPanel";
import { SyncPanel } from "./panels/SyncPanel";
import { TrustPanel } from "./panels/TrustPanel";
import { UpdatesPanel } from "./panels/UpdatesPanel";

interface PanelHostProps {
  panel: PanelId | null;
  mobile: boolean;
  onClose: () => void;
  thread: MercuryThreadState;
  scenario: Scenario;
  setScenario: (next: Partial<Scenario>) => void;
  theme: ThemeChoice;
  setTheme: (t: ThemeChoice) => void;
  openPanel: (panel: PanelId) => void;
  dark: boolean;
}

/**
 * Routes the active panel id to its surface. Modal on desktop, bottom sheet on
 * mobile (each panel renders its own PanelShell). The interactive panels feed
 * straight back into the scenario inputs -- "Mark all verified" flips trust,
 * "Revoke"/"Request" flips the AI grant, recovery/sync re-open the message UI
 * -- so the decision boundary updates live everywhere (data strip, inspector,
 * composer).
 */
export function PanelHost({
  panel,
  mobile,
  onClose,
  thread,
  scenario,
  setScenario,
  theme,
  setTheme,
  openPanel,
  dark,
}: PanelHostProps) {
  if (!panel) return null;

  if (panel === "trust") {
    return (
      <TrustPanel
        mobile={mobile}
        onClose={onClose}
        trust={scenario.trust}
        onMarkVerified={() => {
          setScenario({ trust: "trusted" });
          onClose();
        }}
      />
    );
  }
  if (panel === "ai") {
    return <AiPanel mobile={mobile} onClose={onClose} ai={scenario.ai} onChangeAi={(v) => setScenario({ ai: v })} />;
  }
  if (panel === "peers") {
    return (
      <PeersPanel
        mobile={mobile}
        onClose={onClose}
        trust={scenario.trust}
        ai={scenario.ai}
        onOpenTrust={() => openPanel("trust")}
        onOpenProfile={(w) => openPanel(`profile:${w}` as PanelId)}
      />
    );
  }
  if (panel === "encryption") {
    return <EncryptionPanel mobile={mobile} onClose={onClose} />;
  }
  if (panel === "attachments") {
    return <AttachmentsPanel mobile={mobile} onClose={onClose} />;
  }
  if (panel === "group") {
    return (
      <GroupSafetyPanel
        mobile={mobile}
        onClose={onClose}
        trust={scenario.trust}
        ai={scenario.ai}
        onOpenTrust={() => openPanel("trust")}
        onRotateEpoch={() => setScenario({ trust: "trusted", ai: scenario.ai === "revoked" ? "absent" : scenario.ai })}
      />
    );
  }
  if (panel === "policy") {
    return <PolicyPanel mobile={mobile} onClose={onClose} thread={thread} />;
  }
  if (panel === "bootstrap") {
    return <BootstrapPanel mobile={mobile} onClose={onClose} view={thread.bootstrapView} />;
  }
  if (panel === "recovery") {
    return (
      <RecoveryPanel mobile={mobile} onClose={onClose} onComplete={() => setScenario({ bootstrap: "accepted" })} />
    );
  }
  if (panel === "sync") {
    return <SyncPanel mobile={mobile} onClose={onClose} onComplete={() => setScenario({ bootstrap: "accepted" })} />;
  }
  if (panel === "notifications") {
    return <NotificationsPanel mobile={mobile} onClose={onClose} />;
  }
  if (panel === "updates") {
    return <UpdatesPanel mobile={mobile} onClose={onClose} />;
  }
  if (panel === "onboarding") {
    return <OnboardingPanel mobile={mobile} onClose={onClose} openPanel={openPanel} />;
  }
  if (panel === "settings") {
    return (
      <SettingsPanel
        mobile={mobile}
        onClose={onClose}
        dark={dark}
        theme={theme}
        setTheme={setTheme}
        onOpenProfile={(w) => openPanel(`profile:${w}` as PanelId)}
        onOpenNotifications={() => openPanel("notifications")}
        onOpenRecovery={() => openPanel("recovery")}
        onOpenTrust={() => openPanel("trust")}
        onOpenEncryption={() => openPanel("encryption")}
        onOpenAttachments={() => openPanel("attachments")}
        onOpenGroup={() => openPanel("group")}
        onOpenPolicy={() => openPanel("policy")}
        onOpenUpdates={() => openPanel("updates")}
        onOpenOnboarding={() => openPanel("onboarding")}
      />
    );
  }
  if (panel.startsWith("profile:")) {
    const who = panel.slice("profile:".length);
    return (
      <ProfilePanel
        mobile={mobile}
        onClose={onClose}
        who={who}
        trust={scenario.trust}
        ai={scenario.ai}
        onOpenTrust={() => openPanel("trust")}
        onOpenAi={() => openPanel("ai")}
        onOpenNotifications={() => openPanel("notifications")}
      />
    );
  }
  return null;
}
