import { useState } from "react";

import type {
  AiState,
  BootstrapState,
  ForceSendOutcome,
  ReceiveMode,
  SenderTrust,
  TrustState,
} from "../types";
import type { BindingMode, ThemeChoice } from "../MercuryApp";
import styles from "./DevControls.module.css";

export interface Scenario {
  bootstrap: BootstrapState;
  trust: TrustState;
  ai: AiState;
  receiveMode: ReceiveMode;
  senderTrust: SenderTrust;
  forceSendOutcome: ForceSendOutcome;
}

interface DevControlsProps {
  scenario: Scenario;
  setScenario: (next: Partial<Scenario>) => void;
  theme: ThemeChoice;
  setTheme: (t: ThemeChoice) => void;
  mobile: boolean;
  setMobile: (m: boolean) => void;
  bindingMode: BindingMode;
  setBindingMode: (m: BindingMode) => void;
  /** 0 = not loaded, >0 = fixtures cached, -1 = bridge unreachable. */
  coreCount: number;
}

function Segmented<T extends string>({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: T;
  options: readonly { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className={styles.field}>
      <div className={styles.fieldLabel}>{label}</div>
      <div className={styles.segments} role="radiogroup" aria-label={label}>
        {options.map((o) => (
          <button
            key={o.value}
            type="button"
            role="radio"
            aria-checked={value === o.value}
            className={styles.segment}
            data-on={value === o.value ? "" : undefined}
            onClick={() => onChange(o.value)}
          >
            {o.label}
          </button>
        ))}
      </div>
    </div>
  );
}

export function DevControls({
  scenario,
  setScenario,
  theme,
  setTheme,
  mobile,
  setMobile,
  bindingMode,
  setBindingMode,
  coreCount,
}: DevControlsProps) {
  const [open, setOpen] = useState(false);

  if (!open) {
    return (
      <button type="button" className={styles.fab} onClick={() => setOpen(true)} aria-label="Open dev scenario controls">
        scenario
      </button>
    );
  }

  return (
    <div className={styles.panel}>
      <div className={styles.head}>
        <span className={styles.headTitle}>scenario</span>
        <span className={styles.headSub}>dev simulation knobs</span>
        <button type="button" className={styles.close} onClick={() => setOpen(false)} aria-label="Hide scenario controls">
          ✕
        </button>
      </div>
      <div className={styles.body}>
        <Segmented
          label="decisions"
          value={bindingMode}
          onChange={setBindingMode}
          options={[
            { value: "sim", label: "simulator" },
            { value: "core", label: "core (live)" },
          ]}
        />
        {bindingMode === "core" && (
          <div className={styles.note}>
            {coreCount > 0
              ? `mercury-core · ${coreCount} fixtures`
              : coreCount < 0
                ? "bridge offline — run: cargo run -p mercury-ui-bridge"
                : "loading…"}
          </div>
        )}
        <div className={styles.divider} />
        <Segmented
          label="bootstrap"
          value={scenario.bootstrap}
          onChange={(v) => setScenario({ bootstrap: v })}
          options={[
            { value: "accepted", label: "accepted" },
            { value: "sync_incomplete", label: "sync" },
            { value: "recovery_required", label: "recovery" },
          ]}
        />
        <Segmented
          label="trust"
          value={scenario.trust}
          onChange={(v) => setScenario({ trust: v })}
          options={[
            { value: "trusted", label: "trusted" },
            { value: "sendable_not_full", label: "sendable" },
            { value: "unverified", label: "new" },
            { value: "stale", label: "stale" },
            { value: "rejected", label: "rejected" },
          ]}
        />
        <Segmented
          label="ai grant"
          value={scenario.ai}
          onChange={(v) => setScenario({ ai: v })}
          options={[
            { value: "granted", label: "granted" },
            { value: "absent", label: "absent" },
            { value: "revoked", label: "revoked" },
            { value: "expired", label: "expired" },
          ]}
        />
        <Segmented
          label="receive mode"
          value={scenario.receiveMode}
          onChange={(v) => setScenario({ receiveMode: v })}
          options={[
            { value: "accepted", label: "accepted" },
            { value: "ordering_gap", label: "ordering gap" },
          ]}
        />
        <Segmented
          label="sender trust"
          value={scenario.senderTrust}
          onChange={(v) => setScenario({ senderTrust: v })}
          options={[
            { value: "trusted", label: "trusted" },
            { value: "rejected", label: "rejected" },
          ]}
        />
        <Segmented
          label="force send"
          value={scenario.forceSendOutcome}
          onChange={(v) => setScenario({ forceSendOutcome: v })}
          options={[
            { value: "auto", label: "auto" },
            { value: "accepted", label: "accept" },
            { value: "tofu", label: "tofu" },
            { value: "rejected", label: "reject" },
            { value: "stale", label: "stale" },
          ]}
        />
        <div className={styles.divider} />
        <Segmented
          label="theme"
          value={theme}
          onChange={setTheme}
          options={[
            { value: "light", label: "light" },
            { value: "dark", label: "dark" },
            { value: "auto", label: "auto" },
          ]}
        />
        <Segmented
          label="layout"
          value={mobile ? "mobile" : "desktop"}
          onChange={(v) => setMobile(v === "mobile")}
          options={[
            { value: "desktop", label: "desktop" },
            { value: "mobile", label: "mobile" },
          ]}
        />
      </div>
    </div>
  );
}
