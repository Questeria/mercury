import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { useMercuryThread } from "./useMercuryThread";
import { createHttpBinding, createSimulatorBinding, prefetchDecisions } from "./binding";
import type { PanelId } from "./types";
import { DevControls, type Scenario } from "./components/DevControls";
import { Inspector } from "./components/Inspector";
import { Rail } from "./components/Rail";
import { Thread } from "./components/Thread";
import { PanelHost } from "./components/Panels";
import styles from "./MercuryApp.module.css";
import "./theme.css";

export type ThemeChoice = "light" | "dark" | "auto";

/** Where the UI's decisions come from: the in-app simulator, or the real
 *  mercury-core via the mercury-ui-bridge dev shim. */
export type BindingMode = "sim" | "core";

const BRIDGE_BASE = "/api/bridge";

const DEFAULT_SCENARIO: Scenario = {
  bootstrap: "accepted",
  trust: "trusted",
  ai: "granted",
  receiveMode: "accepted",
  senderTrust: "trusted",
  forceSendOutcome: "auto",
};

function useResolvedDark(choice: ThemeChoice): boolean {
  const [systemDark, setSystemDark] = useState(
    () => window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false,
  );
  useEffect(() => {
    if (choice !== "auto") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => setSystemDark(e.matches);
    mq.addEventListener("change", handler);
    setSystemDark(mq.matches);
    return () => mq.removeEventListener("change", handler);
  }, [choice]);
  return choice === "dark" || (choice === "auto" && systemDark);
}

function useWindowWidth(): number {
  const [width, setWidth] = useState(() => window.innerWidth);
  useEffect(() => {
    const onResize = () => setWidth(window.innerWidth);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);
  return width;
}

// Responsive breakpoints. Below MOBILE the layout collapses to a single
// column with slide-in overlays; the side columns auto-hide as width tightens
// so the thread is never crushed.
const BP_MOBILE = 680;
const BP_RAIL = 880;
const BP_INSPECTOR = 1100;

export function MercuryApp() {
  const [scenario, setScenarioState] = useState<Scenario>(DEFAULT_SCENARIO);
  const [theme, setTheme] = useState<ThemeChoice>("light");
  const [forceMobile, setForceMobile] = useState<boolean | null>(null);
  const [inspectorOpen, setInspectorOpen] = useState(true);
  const [railOpen, setRailOpen] = useState(true);
  const [activePanel, setActivePanel] = useState<PanelId | null>(null);
  const [bindingMode, setBindingMode] = useState<BindingMode>("sim");
  const [bindingNonce, setBindingNonce] = useState(0);
  // 0 = not loaded, >0 = fixtures cached, -1 = bridge unreachable.
  const [coreCount, setCoreCount] = useState(0);

  const width = useWindowWidth();
  const mobile = forceMobile ?? width < BP_MOBILE;
  const dark = useResolvedDark(theme);

  // Decision source: in-app simulator (default) or real mercury-core via the
  // mercury-ui-bridge dev shim. The binding is recreated once the bridge cache
  // warms (bindingNonce bump) so the hook's memoized decisions pick up the
  // real-core views.
  const binding = useMemo(
    () => (bindingMode === "core" ? createHttpBinding(BRIDGE_BASE) : createSimulatorBinding()),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [bindingMode, bindingNonce],
  );
  useEffect(() => {
    if (bindingMode !== "core") return;
    let cancelled = false;
    prefetchDecisions(BRIDGE_BASE)
      .then((n) => {
        if (cancelled) return;
        setCoreCount(n);
        setBindingNonce((x) => x + 1);
      })
      .catch(() => {
        if (!cancelled) setCoreCount(-1);
      });
    return () => {
      cancelled = true;
    };
  }, [bindingMode]);

  const thread = useMercuryThread(scenario, binding);

  const setScenario = useCallback(
    (next: Partial<Scenario>) => setScenarioState((s) => ({ ...s, ...next })),
    [],
  );
  const closePanel = useCallback(() => setActivePanel(null), []);
  const openProfile = useCallback((who: string) => setActivePanel(`profile:${who}` as PanelId), []);

  // Esc closes any open panel.
  useEffect(() => {
    if (!activePanel) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") closePanel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [activePanel, closePanel]);

  // Auto-collapse the side columns as width tightens so the thread is never
  // crushed. These fire only when a breakpoint is crossed, so manual toggles
  // are respected between crossings.
  const wideRail = !mobile && width >= BP_RAIL;
  const wideInspector = !mobile && width >= BP_INSPECTOR;
  useEffect(() => {
    setRailOpen(wideRail);
  }, [wideRail]);
  useEffect(() => {
    setInspectorOpen(wideInspector);
  }, [wideInspector]);

  const threadEl = useMemo(
    () => (
      <Thread
        thread={thread}
        mobile={mobile}
        trust={scenario.trust}
        ai={scenario.ai}
        inspectorOpen={inspectorOpen}
        onToggleInspector={() => setInspectorOpen((o) => !o)}
        railOpen={railOpen}
        onToggleRail={() => setRailOpen((o) => !o)}
        onOpenPanel={setActivePanel}
        onOpenProfile={openProfile}
      />
    ),
    [thread, mobile, scenario.trust, scenario.ai, inspectorOpen, railOpen, openProfile],
  );

  return (
    <div className={styles.viewport}>
      <div
        className={`mercury ${styles.app} ${mobile ? styles.mobile : styles.desktop}`}
        data-theme={dark ? "dark" : "light"}
      >
        <div className="shimmer-bg" />

        {!mobile && railOpen && <Rail onOpenProfile={openProfile} />}

        <div className={styles.center}>{threadEl}</div>

        {!mobile && inspectorOpen && (
          <Inspector thread={thread} ai={scenario.ai} onClose={() => setInspectorOpen(false)} onOpenProfile={openProfile} />
        )}

        {/* mobile rooms overlay (slides from left) */}
        {mobile && (
          <Overlay side="left" open={railOpen} onClose={() => setRailOpen(false)}>
            <Rail onClose={() => setRailOpen(false)} onOpenProfile={(w) => { setRailOpen(false); openProfile(w); }} />
          </Overlay>
        )}

        {/* mobile inspector overlay (slides from right) */}
        {mobile && (
          <Overlay side="right" open={inspectorOpen} onClose={() => setInspectorOpen(false)}>
            <Inspector
              thread={thread}
              ai={scenario.ai}
              fullWidth
              onClose={() => setInspectorOpen(false)}
              onOpenProfile={(w) => { setInspectorOpen(false); openProfile(w); }}
            />
          </Overlay>
        )}

        <PanelHost
          panel={activePanel}
          mobile={mobile}
          onClose={closePanel}
          thread={thread}
          scenario={scenario}
          setScenario={setScenario}
          theme={theme}
          setTheme={setTheme}
          openPanel={setActivePanel}
          dark={dark}
        />
      </div>

      <DevControls
        scenario={scenario}
        setScenario={setScenario}
        theme={theme}
        setTheme={setTheme}
        mobile={mobile}
        setMobile={setForceMobile}
        bindingMode={bindingMode}
        setBindingMode={setBindingMode}
        coreCount={coreCount}
      />
    </div>
  );
}

function Overlay({
  side,
  open,
  onClose,
  children,
}: {
  side: "left" | "right";
  open: boolean;
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <>
      <div
        className={styles.scrim}
        data-open={open ? "" : undefined}
        onClick={onClose}
        aria-hidden
      />
      <div
        className={`${styles.sheet} ${side === "left" ? styles.sheetLeft : styles.sheetRight}`}
        data-open={open ? "" : undefined}
      >
        {children}
      </div>
    </>
  );
}
