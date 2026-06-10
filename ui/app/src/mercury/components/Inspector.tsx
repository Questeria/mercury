import { useEffect, useRef, useState } from "react";
import type { CSSProperties } from "react";

import { fmtClkSec } from "../format";
import { Helix } from "../icons";
import {
  attestationForDecisionSource,
  componentAttestations,
  type PolicyName,
} from "../policyAttestation.generated";
import { MERCURY_PEOPLE } from "../simulator";
import { CAPABILITY_FLAGS, explainDecision, FLAG_LABEL, toneVar } from "../strings";
import type { AiState, DecisionSource, MercuryThreadState, PlatformDecisionView } from "../types";
import { Avatar } from "./Avatar";
import styles from "./Inspector.module.css";

interface InspectorProps {
  thread: MercuryThreadState;
  ai: AiState;
  onClose?: () => void;
  fullWidth?: boolean;
  onOpenProfile: (who: string) => void;
}

export function Inspector({ thread, ai, onClose, fullWidth, onOpenProfile }: InspectorProps) {
  return (
    <div className={styles.inspector} data-full={fullWidth ? "" : undefined}>
      <div className={styles.head}>
        <Helix size={14} color="var(--mc-ai)" />
        <span className={styles.headTitle}>Inspector</span>
        <span className={styles.headSub}>· binding view</span>
        <span className={styles.live}>● live</span>
        {onClose && (
          <button type="button" className={styles.close} onClick={onClose} data-tip="Hide inspector" aria-label="Hide inspector">
            ✕
          </button>
        )}
      </div>
      <div className={styles.scroll}>
        <DecisionCard title="Bootstrap" source="client_bootstrap" view={thread.bootstrapView} />
        <DecisionCard title="Outbound send" source="outbound_send" view={thread.outboundView} />
        <DecisionCard title="Last receive" source="client_receive" view={thread.receiveView} />
        <div>
          <div className={styles.kicker}>Participants</div>
          {(["rin", "jules", "ai"] as const).map((id) => {
            const p = MERCURY_PEOPLE[id];
            return (
              <button key={id} type="button" className={styles.participant} onClick={() => onOpenProfile(id)}>
                <Avatar person={p} size={22} />
                <div style={{ minWidth: 0, flex: 1 }}>
                  <div className={p.isAi ? "iris-text" : styles.pName}>{p.name}</div>
                  <div className={styles.pSub}>{p.isAi ? `ai · ${ai}` : "2 devices · verified"}</div>
                </div>
              </button>
            );
          })}
        </div>
      </div>
      <EventTail thread={thread} />
    </div>
  );
}

function DecisionCard({
  title,
  source,
  view,
}: {
  title: string;
  source: DecisionSource;
  view: PlatformDecisionView;
}) {
  const ok = view.accepted;
  const tone = ok ? toneVar("ok") : toneVar("bad");
  const liveFlags = CAPABILITY_FLAGS.filter((f) => view[f] === true);
  const [showRaw, setShowRaw] = useState(false);
  const key = `${view.source}-${view.reason_label}-${view.accepted}`;
  const attestation = attestationForDecisionSource(view.source) ?? attestationForDecisionSource(source);
  const components = attestation ? componentAttestations(attestation.name as PolicyName) : [];

  return (
    <div>
      <div className={styles.kicker}>{title}</div>
      <div key={key} className={`${styles.card} card-in`}>
        <div className={styles.cardHead} style={{ borderLeft: `3px solid ${tone}` }}>
          <span
            className={styles.verdict}
            style={
              {
                background: ok ? "var(--mc-ok-soft)" : "var(--mc-bad-soft)",
                color: tone,
              } as CSSProperties
            }
          >
            {ok ? "ACCEPT" : "REJECT"}
          </span>
          <span className={styles.cardLabel}>{view.reason_label}</span>
          <span className={styles.cardRc}>rc={view.reason_code}</span>
        </div>
        <div className={styles.explain}>{explainDecision(view, source)}</div>
        {attestation && <PolicyProvenance attestation={attestation} components={components} />}
        {liveFlags.length > 0 && (
          <div className={styles.flags}>
            {liveFlags.map((f) => {
              const isReq = String(f).startsWith("requires_");
              return (
                <div key={String(f)} className={styles.flagRow}>
                  <span
                    className={styles.flagDot}
                    style={{ background: isReq ? toneVar("warn") : toneVar("ok") }}
                  />
                  <span style={{ color: "var(--mc-ink)" }}>{FLAG_LABEL[String(f)] ?? String(f)}</span>
                </div>
              );
            })}
          </div>
        )}
        <button type="button" className={styles.rawToggle} onClick={() => setShowRaw((r) => !r)}>
          <span style={{ fontSize: 9 }}>{showRaw ? "▾" : "▸"}</span>
          raw fields
        </button>
        {showRaw && (
          <div className={styles.raw}>
            <span className={styles.rawKey}>source</span>
            <span className={styles.rawVal}>{view.source}</span>
            <span className={styles.rawKey}>accepted</span>
            <span className={styles.rawVal} style={{ color: tone, fontWeight: 600 }}>
              {String(ok)}
            </span>
            {CAPABILITY_FLAGS.map((f) => {
              const v = view[f];
              if (typeof v !== "boolean") return null;
              const isReq = String(f).startsWith("requires_");
              return (
                <span key={String(f)} style={{ display: "contents" }}>
                  <span className={styles.rawKey}>{String(f)}</span>
                  <span
                    className={styles.rawVal}
                    style={{
                      color: v ? (isReq ? toneVar("warn") : toneVar("ok")) : "var(--mc-dim)",
                      fontWeight: v ? 600 : 400,
                    }}
                  >
                    {String(v)}
                  </span>
                </span>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

type PolicyAttestation = NonNullable<ReturnType<typeof attestationForDecisionSource>>;

function PolicyProvenance({
  attestation,
  components,
}: {
  attestation: PolicyAttestation;
  components: readonly PolicyAttestation[];
}) {
  return (
    <div className={styles.provenance}>
      <div className={styles.provHead}>
        <Helix size={11} color="var(--mc-ai)" />
        <span className={styles.provName}>{attestation.displayName}</span>
        <span className={styles.provHash}>hx {attestation.sourceHashShort}</span>
      </div>
      <div className={styles.provGrid}>
        <span>vectors</span>
        <span>
          {attestation.vectorFileCount} / {attestation.vectorHashShort}
        </span>
        <span>runtime</span>
        <span>{formatRuntime(attestation.productionRuntime)}</span>
        <span>gates</span>
        <span>hash + proof + ELF 42</span>
      </div>
      {components.length > 0 && (
        <div className={styles.provComponents}>
          components: {components.map((c) => `${c.name}:${c.sourceHashShort}`).join(" / ")}
        </div>
      )}
    </div>
  );
}

function formatRuntime(runtime: string) {
  return runtime === "rust_mirror" ? "Rust mirror" : runtime;
}

interface TailEntry {
  id: string;
  when: number;
  source: string;
  view: PlatformDecisionView;
}

function EventTail({ thread }: { thread: MercuryThreadState }) {
  const [log, setLog] = useState<TailEntry[]>([]);
  const lastBoot = useRef("");
  const lastOut = useRef("");
  const lastRcv = useRef("");
  const scrollRef = useRef<HTMLDivElement>(null);

  const push = (source: string, view: PlatformDecisionView) => {
    setLog((L) =>
      [...L, { id: Math.random().toString(36).slice(2), when: Date.now(), source, view }].slice(-40),
    );
  };

  useEffect(() => {
    const v = thread.bootstrapView;
    const k = `${v.accepted}-${v.reason_label}`;
    if (k !== lastBoot.current) {
      lastBoot.current = k;
      push("bootstrap", v);
    }
  }, [thread.bootstrapView]);
  useEffect(() => {
    const v = thread.outboundView;
    const k = `${v.accepted}-${v.reason_label}`;
    if (k !== lastOut.current) {
      lastOut.current = k;
      push("outbound", v);
    }
  }, [thread.outboundView]);
  useEffect(() => {
    const v = thread.receiveView;
    const k = `${v.accepted}-${v.reason_label}`;
    if (k !== lastRcv.current) {
      lastRcv.current = k;
      push("receive", v);
    }
  }, [thread.receiveView]);

  useEffect(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
  }, [log.length]);

  return (
    <div className={styles.tail}>
      <div className={styles.tailHead}>
        <Helix size={11} color="var(--mc-ai)" />
        <span style={{ color: "var(--mc-ink)" }}>events</span>
        <span style={{ color: "var(--mc-muted)" }}>· most recent below</span>
        <span style={{ marginLeft: "auto", fontSize: 10 }}>{log.length}</span>
      </div>
      <div ref={scrollRef} className={styles.tailScroll}>
        {log.map((e) => (
          <TailRow key={e.id} entry={e} />
        ))}
      </div>
    </div>
  );
}

function TailRow({ entry }: { entry: TailEntry }) {
  const v = entry.view;
  const ok = v.accepted;
  const tone = ok ? toneVar("ok") : toneVar("bad");
  const reqs = (["requires_sync", "requires_recovery", "requires_client_retry", "requires_user_action"] as const).filter(
    (k) => v[k],
  );
  return (
    <div className={`${styles.tailRow} card-in`}>
      <div className={styles.tailLine}>
        <span className={styles.tailClk}>{fmtClkSec(entry.when)}</span>
        <span style={{ color: "var(--mc-ai)" }}>{entry.source}</span>
        <span style={{ color: "var(--mc-muted)" }}>→</span>
        <span style={{ color: tone, fontWeight: 600 }}>{ok ? "ACCEPT" : "REJECT"}</span>
        <span className={styles.tailRc}>rc={v.reason_code}</span>
      </div>
      <div className={styles.tailLabel} style={{ color: ok ? "var(--mc-ink2)" : "var(--mc-bad)" }}>
        <span style={{ color: "var(--mc-muted)" }}>label=</span>
        {v.reason_label}
      </div>
      {reqs.length > 0 && (
        <div className={styles.tailReqs}>
          <span style={{ color: "var(--mc-muted)" }}>&gt; </span>
          {reqs.join(" · ")}
        </div>
      )}
    </div>
  );
}
