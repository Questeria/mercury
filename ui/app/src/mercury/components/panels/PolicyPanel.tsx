import { useMemo, useState } from "react";

import {
  HELIX_POLICY_BOUNDARY,
  attestationForDecisionSource,
  componentAttestations,
  type PolicyName,
} from "../../policyAttestation.generated";
import { CAPABILITY_FLAGS, FLAG_LABEL, explainDecision, labelGloss, toneVar } from "../../strings";
import type { DecisionSource, MercuryThreadState, PlatformDecisionView } from "../../types";
import { PanelShell } from "./PanelShell";
import { GhostButton, Kv, ScopeTile, Section, Segment, StatusRow } from "./primitives";
import styles from "./panels.module.css";

interface PolicyPanelProps {
  mobile: boolean;
  onClose: () => void;
  thread: MercuryThreadState;
}

type ViewKey = "bootstrap" | "outbound" | "receive";

export function PolicyPanel({ mobile, onClose, thread }: PolicyPanelProps) {
  const [selected, setSelected] = useState<ViewKey>("outbound");
  const views = useMemo(
    () => ({
      bootstrap: thread.bootstrapView,
      outbound: thread.outboundView,
      receive: thread.receiveView,
    }),
    [thread.bootstrapView, thread.outboundView, thread.receiveView],
  );
  const view = views[selected];
  const gloss = labelGloss(view.reason_label);
  const attestation = attestationForDecisionSource(view.source);

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Policy"
      title="Decision explanation"
      footer={<GhostButton onClick={onClose}>Done</GhostButton>}
    >
      <StatusRow tone={view.accepted ? "ok" : gloss.tone} label={view.reason_label} sub={explainDecision(view)} />

      <Section kicker="Gate">
        <Segment<ViewKey>
          value={selected}
          onChange={setSelected}
          options={[
            { value: "bootstrap", label: "Start", sub: "client" },
            { value: "outbound", label: "Send", sub: "message" },
            { value: "receive", label: "Receive", sub: "message" },
          ]}
        />
      </Section>

      <Section kicker="Decision view">
        <DecisionCard view={view} />
      </Section>

      <Section kicker="Capabilities">
        <div className={styles.flagGrid}>
          {CAPABILITY_FLAGS.map((flag) => {
            const on = Boolean(view[flag]);
            return (
              <div key={flag} className={styles.flag} data-on={on ? "" : undefined}>
                <span className={styles.dot5} style={{ background: on ? toneVar("ok") : "var(--mc-border-strong)", marginRight: 6 }} />
                {FLAG_LABEL[flag]}
              </div>
            );
          })}
        </div>
      </Section>

      {attestation && (
        <Section kicker="Helix attestation">
          <AttestationBlock source={view.source} policy={attestation.name} />
        </Section>
      )}

      <Section kicker="Boundary">
        <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.55 }}>
          {HELIX_POLICY_BOUNDARY}
        </div>
      </Section>
    </PanelShell>
  );
}

function DecisionCard({ view }: { view: PlatformDecisionView }) {
  return (
    <div className={styles.decisionCard}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
        <span className={styles.dot8} style={{ background: view.accepted ? toneVar("ok") : toneVar("bad") }} />
        <span style={{ color: "var(--mc-ink)", fontWeight: 700, fontSize: 13 }}>{view.source}</span>
        <span style={{ marginLeft: "auto", color: "var(--mc-muted)", fontSize: 11 }} className="mono">
          rc={view.reason_code}
        </span>
      </div>
      <div style={{ color: "var(--mc-ink2)", fontSize: 12, lineHeight: 1.5 }}>
        {explainDecision(view)}
      </div>
    </div>
  );
}

function AttestationBlock({ source, policy }: { source: DecisionSource; policy: PolicyName }) {
  const attestation = attestationForDecisionSource(source);
  if (!attestation) return null;
  const components = componentAttestations(policy);
  return (
    <div className={styles.colGap}>
      <div className={styles.card} style={{ fontSize: 11.5, lineHeight: 1.6 }}>
        <Kv k="Policy" v={attestation.displayName} />
        <Kv k="Role" v={attestation.role} />
        <Kv k="Runtime" v={attestation.productionRuntime} />
        <Kv k="Source" v={<span className="mono">{attestation.sourceHashShort}</span>} />
        <Kv k="Vectors" v={`${attestation.vectorFileCount} files / ${attestation.vectorHashShort}`} />
      </div>
      {components.length > 0 && (
        <div className={styles.featureGrid}>
          {components.map((component) => (
            <ScopeTile
              key={component.name}
              label={component.displayName}
              value={component.sourceHashShort}
              note={component.role}
            />
          ))}
        </div>
      )}
    </div>
  );
}
