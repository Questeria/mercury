import { useEffect, useState } from "react";

import {
  DOWNLOAD_URL,
  UPDATE_MANIFEST_URL,
  checkForUpdates,
  downloadAndInstallUpdate,
  probeUpdateManifest,
  type UpdateCheckResult,
  type UpdateManifestProbe,
} from "../../updater";
import { inTauri } from "../../messaging";
import { isAutostartEnabled, setAutostartEnabled } from "../../autostart";
import { getAutoUpdate, setAutoUpdate } from "../../updatePrefs";
import { PanelShell } from "./PanelShell";
import { GhostButton, IridButton, Kv, ScopeTile, Section, StatusRow } from "./primitives";
import styles from "./panels.module.css";

interface UpdatesPanelProps {
  mobile: boolean;
  onClose: () => void;
}

export function UpdatesPanel({ mobile, onClose }: UpdatesPanelProps) {
  const [probe, setProbe] = useState<UpdateManifestProbe | null>(null);
  const [check, setCheck] = useState<UpdateCheckResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [autostart, setAutostart] = useState<boolean | null>(null);
  const [autoUpdate, setAuto] = useState(getAutoUpdate);

  useEffect(() => {
    let cancelled = false;
    probeUpdateManifest().then((next) => {
      if (!cancelled) setProbe(next);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!inTauri()) return;
    let cancelled = false;
    isAutostartEnabled().then((v) => {
      if (!cancelled) setAutostart(v);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const status = check ?? probe;
  const tone = status?.state === "available" || status?.state === "current" ? "ok" : status?.state === "dormant" ? "warn" : "bad";

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Updates"
      title="Signed update channel"
      footer={
        <>
          <GhostButton onClick={onClose}>Close</GhostButton>
          {check?.state === "available" && check.version && (
            <IridButton
              disabled={busy}
              onClick={async () => {
                setBusy(true);
                try {
                  setCheck(await downloadAndInstallUpdate());
                } finally {
                  setBusy(false);
                }
              }}
            >
              {busy ? "Installing…" : `Install v${check.version} & restart`}
            </IridButton>
          )}
          <IridButton
            disabled={busy}
            onClick={async () => {
              setBusy(true);
              try {
                setCheck(await checkForUpdates());
              } finally {
                setBusy(false);
              }
            }}
          >
            {busy ? "Checking…" : "Check now"}
          </IridButton>
        </>
      }
    >
      <StatusRow
        tone={tone}
        label={status?.title ?? "Checking update channel"}
        sub={status?.detail ?? "Probing latest.json"}
      />

      <Section kicker="Release path">
        <div className={styles.featureGrid}>
          <ScopeTile label="Installer" value="Authenticode" tone="ok" note="Azure Artifact Signing" />
          <ScopeTile label="Checksum" value="SHA-256" tone="ok" note="published beside installer" />
          <ScopeTile label="Updater" value={probe?.state === "available" ? "live" : "dormant"} tone={probe?.state === "available" ? "ok" : "warn"} note="Tauri signed manifest" />
          <ScopeTile label="Apply" value="next launch" note="desktop app only" />
        </div>
      </Section>

      {inTauri() && (
        <Section kicker="Startup">
          <div className={styles.colGap}>
            <div
              className={styles.card}
              style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12 }}
            >
              <div style={{ fontSize: 12, lineHeight: 1.5 }}>
                <div>
                  <strong>Update automatically at launch</strong>
                </div>
                <div style={{ color: "var(--mc-ink2)" }}>
                  Install the latest signed build when Mercury starts (applies on the next restart). The
                  signature is always verified first — this only changes whether you have to click.
                </div>
              </div>
              <GhostButton onClick={() => setAuto(setAutoUpdate(!autoUpdate))}>
                {autoUpdate ? "On" : "Off"}
              </GhostButton>
            </div>
            <div
              className={styles.card}
              style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12 }}
            >
              <div style={{ fontSize: 12, lineHeight: 1.5 }}>
                <div>
                  <strong>Launch at login</strong>
                </div>
                <div style={{ color: "var(--mc-ink2)" }}>
                  Start Mercury minimized in the tray at sign-in, so messages arrive without opening it.
                </div>
              </div>
              <GhostButton
                onClick={async () => {
                  if (busy || autostart === null) return;
                  setBusy(true);
                  try {
                    setAutostart(await setAutostartEnabled(!autostart));
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                {autostart === null ? "…" : autostart ? "On" : "Off"}
              </GhostButton>
            </div>
          </div>
        </Section>
      )}

      <Section kicker="Endpoints">
        <div className={styles.card} style={{ fontSize: 11.5, lineHeight: 1.6 }}>
          <Kv k="Download" v={DOWNLOAD_URL} />
          <Kv k="Manifest" v={UPDATE_MANIFEST_URL} />
          <Kv k="Version" v={status?.version ?? "not advertised"} />
        </div>
      </Section>

      <Section kicker="What Mercury verifies">
        <div className={styles.colGap}>
          <div className={styles.monoCard}>1. Windows verifies the installer publisher and timestamp.</div>
          <div className={styles.monoCard}>2. The website publishes a SHA-256 checksum for manual downloads.</div>
          <div className={styles.monoCard}>3. In-app updates require a Tauri signature over the final installer bytes.</div>
        </div>
      </Section>
    </PanelShell>
  );
}
