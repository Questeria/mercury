// mercury-variant-fusion.jsx
// "Mercury" — fluid modern with iridescent identity.
// Clean sans body (Geist). Modern chat bubbles. Outgoing messages, the send
// button, and the AI mark all carry a slow-shifting opal-iridescent gradient.
// A pastel rainbow drifts subtly behind every surface. Diagnostic mono is
// present but quiet (small footer lines, decision codes), not dominant.
// AI replies still stream in character-by-character.
//
// Exports (window): FusionVariant

const FUSION_FONT_CSS = `
@import url('https://fonts.googleapis.com/css2?family=Geist:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');
.mv-fuse { font-family: 'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace; font-feature-settings: 'ss01', 'cv11'; -webkit-font-smoothing: antialiased; }
.mv-fuse .display { font-family: 'Geist', -apple-system, BlinkMacSystemFont, system-ui, sans-serif; }

@keyframes fuse-irid       { 0% { background-position: 0% 50%; } 100% { background-position: 300% 50%; } }
@keyframes fuse-iris       { to { transform: rotate(360deg); } }
@keyframes fuse-rise       { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: translateY(0); } }
@keyframes fuse-pulse      { 0%,100% { transform: scale(1); } 50% { transform: scale(1.4); } }
@keyframes fuse-blink      { 0%,49% { opacity: 1; } 50%,100% { opacity: 0; } }
@keyframes fuse-bg         {
  0%   { transform: translate3d(-6%, -4%, 0) rotate(0deg); }
  50%  { transform: translate3d(6%,  6%, 0) rotate(8deg); }
  100% { transform: translate3d(-6%, -4%, 0) rotate(0deg); }
}
@keyframes fuse-card-in    { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: translateY(0); } }

/* @property lets us animate the conic gradient's start angle without
   rotating the element itself — so the rainbow drifts but the ring stays put. */
@property --mc-iris-angle { syntax: '<angle>'; initial-value: 0deg; inherits: false; }
@keyframes fuse-iris-angle { to { --mc-iris-angle: 360deg; } }

.mv-fuse .rise        { animation: fuse-rise .26s ease-out both; }
.mv-fuse .pulse-dot   { animation: fuse-pulse .55s ease-out; }
.mv-fuse .blink       { animation: fuse-blink 1.05s steps(1) infinite; }
.mv-fuse .card-in     { animation: fuse-card-in .22s ease-out both; }

/* Opal iridescent fill — for outgoing bubbles and the send button.
   Slow background-position shift gives a hue-drift without changing CSS vars. */
.mv-fuse .irid-fill {
  background-image: var(--mc-irid);
  background-size: 300% 100%;
  animation: fuse-irid 14s linear infinite;
}

/* Iridescent border via masked conic gradient — used on AI bubble/pill.
   Animates the gradient's start angle (via @property), so the colors drift
   around the ring while the ring's geometry stays exactly in place. */
.mv-fuse .iris-ring {
  position: absolute; inset: 0; border-radius: inherit; padding: 1.25px;
  background: conic-gradient(from var(--mc-iris-angle, 0deg),
    #C2A2FF, #93D7F5, #5BE1A8, #FFE49B, #FF99D6, #C2A2FF);
  -webkit-mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0);
  -webkit-mask-composite: xor; mask-composite: exclude;
  animation: fuse-iris-angle 11s linear infinite; pointer-events: none;
}
.mv-fuse .iris-text {
  background: var(--mc-iris-text);
  background-size: 250% 100%; animation: fuse-irid 12s linear infinite;
  -webkit-background-clip: text; background-clip: text;
  -webkit-text-fill-color: transparent; color: transparent;
}

/* Drifting rainbow blobs behind every surface — pastel, blurred, slow. */
.mv-fuse .shimmer-bg {
  position: absolute; inset: -30%; pointer-events: none;
  opacity: var(--mc-shim-opacity, .42);
  background:
    radial-gradient(28% 28% at 22% 18%, var(--mc-shim-a), transparent 65%),
    radial-gradient(26% 26% at 78% 28%, var(--mc-shim-b), transparent 65%),
    radial-gradient(30% 30% at 30% 82%, var(--mc-shim-c), transparent 65%),
    radial-gradient(28% 28% at 80% 80%, var(--mc-shim-d), transparent 65%);
  filter: blur(60px);
  animation: fuse-bg 26s ease-in-out infinite;
}

.mv-fuse textarea, .mv-fuse input { font-family: inherit; }
.mv-fuse textarea::placeholder, .mv-fuse input::placeholder { color: var(--mc-mute); }
.mv-fuse ::-webkit-scrollbar { width: 8px; height: 8px; }
.mv-fuse ::-webkit-scrollbar-thumb { background: var(--mc-scroll); border-radius: 8px; }
.mv-fuse ::-webkit-scrollbar-track { background: transparent; }

/* Keyboard focus ring — visible only on keyboard navigation (focus-visible).
   Uses a 2px iridescent-ish offset outline so it works on any background. */
.mv-fuse :focus { outline: none; }
.mv-fuse button:focus-visible,
.mv-fuse a:focus-visible,
.mv-fuse [tabindex]:focus-visible,
.mv-fuse textarea:focus-visible,
.mv-fuse input:focus-visible {
  outline: 2px solid var(--mc-focus);
  outline-offset: 2px;
  border-radius: inherit;
}

/* Themed hover tooltip — attaches to any element with [data-tip].
   Positioned BELOW the trigger because the header is at the top of the device
   frame; pseudo-elements can't escape the mv-fuse overflow:hidden so above-
   the-trigger placement would clip. Bubble + arrow matched to the rest of the
   surface system. */
.mv-fuse [data-tip]:hover,
.mv-fuse [data-tip]:focus-visible { z-index: 50; }
.mv-fuse [data-tip]:hover::after,
.mv-fuse [data-tip]:focus-visible::after {
  content: attr(data-tip);
  position: absolute;
  top: calc(100% + 9px); left: 50%;
  transform: translateX(-50%);
  padding: 6px 10px;
  background: var(--mc-tip-bg);
  color: var(--mc-tip-ink);
  border: 1px solid var(--mc-tip-border);
  border-radius: 8px;
  font-size: 11px; line-height: 1.4; font-weight: 500;
  max-width: 220px; width: max-content;
  white-space: normal; text-align: center;
  pointer-events: none; z-index: 100;
  box-shadow: 0 8px 24px rgba(0,0,0,.12), 0 1px 0 var(--mc-tip-hi) inset;
  animation: fuse-tip-in .14s ease-out both;
}
.mv-fuse [data-tip]:hover::before,
.mv-fuse [data-tip]:focus-visible::before {
  content: '';
  position: absolute;
  top: calc(100% + 4.5px); left: 50%;
  transform: translateX(-50%) rotate(45deg);
  width: 8px; height: 8px;
  background: var(--mc-tip-bg);
  border-top: 1px solid var(--mc-tip-border);
  border-left: 1px solid var(--mc-tip-border);
  pointer-events: none; z-index: 100;
}
@keyframes fuse-tip-in { from { opacity: 0; transform: translate(-50%, -4px); } to { opacity: 1; transform: translate(-50%, 0); } }
`;

const FUSION_LIGHT = {
  bg: '#FAFAFC', surface: '#FFFFFF', surfaceWarm: '#F4F5F8', surfaceMute: '#EEEFF3',
  ink: '#0B0E16', ink2: '#3F4554', ink3: '#5B6373', muted: '#828A99', dim: '#B0B6C2',
  border: 'rgba(15,18,28,.07)', borderStrong: 'rgba(15,18,28,.14)',
  // iridescent stops — pastel, gentle saturation, drift over 300% background
  irid: 'linear-gradient(110deg, #FFD0E4 0%, #FFC9A8 18%, #FFE9A8 36%, #BDF0CE 54%, #B5D7FF 72%, #D5BBFF 90%, #FFD0E4 100%)',
  // iridescent TEXT — saturated/darker so it stays legible on light surfaces
  iridText: 'linear-gradient(95deg, #6B3FE0, #1D88C7, #1E8F5A, #C13B7C, #6B3FE0)',
  shimA: 'rgba(255,180,220,.55)', shimB: 'rgba(255,210,170,.45)',
  shimC: 'rgba(180,235,210,.45)', shimD: 'rgba(195,220,255,.55)',
  scroll: 'rgba(0,0,0,.10)',
  focus: '#7B5CF0',
  // tooltip palette
  tipBg: '#FFFFFF', tipInk: '#0B0E16', tipBorder: 'rgba(15,18,28,.18)',
  tipHi: 'rgba(255,255,255,.7)',
  accent: '#7B5CF0', accentInk: '#FFFFFF',
  ai: '#7B5CF0', aiSoft: 'rgba(123,92,240,.08)',
  ok: '#1F8A5B', okSoft: 'rgba(31,138,91,.10)',
  warn: '#B26314', warnSoft: 'rgba(178,99,20,.08)', warnTint: 'rgba(178,99,20,.18)',
  bad: '#B8281C', badSoft: 'rgba(184,40,28,.08)',
  shimOpacity: 0.55,
};
const FUSION_DARK = {
  bg: '#0A0C13', surface: '#13171F', surfaceWarm: '#181C25', surfaceMute: '#1E232E',
  ink: '#EBEDF2', ink2: '#BFC4D0', ink3: '#8B92A0', muted: '#666E80', dim: '#404654',
  border: 'rgba(235,237,242,.08)', borderStrong: 'rgba(235,237,242,.18)',
  // dark mode iridescence: same hue family but jewel-toned + still on light text
  irid: 'linear-gradient(110deg, #FFAFD0 0%, #FFB58C 18%, #FFE08C 36%, #9CE6B5 54%, #9CC8FF 72%, #C49AFF 90%, #FFAFD0 100%)',
  // iridescent TEXT — brighter pastels that pop against dark surfaces
  iridText: 'linear-gradient(95deg, #C2A2FF, #93D7F5, #5BE1A8, #FF99D6, #C2A2FF)',
  shimA: 'rgba(160,90,180,.40)', shimB: 'rgba(220,120,100,.30)',
  shimC: 'rgba(80,160,140,.32)', shimD: 'rgba(80,130,200,.40)',
  scroll: 'rgba(255,255,255,.10)',
  focus: '#A89BFF',
  // tooltip palette
  tipBg: '#1A1E27', tipInk: '#EBEDF2', tipBorder: 'rgba(235,237,242,.18)',
  tipHi: 'rgba(255,255,255,.06)',
  accent: '#A89BFF', accentInk: '#0E0905',
  ai: '#A89BFF', aiSoft: 'rgba(168,155,255,.10)',
  ok: '#5DCE96', okSoft: 'rgba(93,206,150,.12)',
  warn: '#F3B564', warnSoft: 'rgba(243,181,100,.10)', warnTint: 'rgba(243,181,100,.20)',
  bad: '#F37A6B', badSoft: 'rgba(243,122,107,.10)',
  shimOpacity: 0.35,
};

const FU_TRUST = {
  trusted:           { tone: 'ok',   label: 'Verified',          sub: 'safety number matched' },
  sendable_not_full: { tone: 'warn', label: 'Sendable',          sub: 'one key not fully verified' },
  unverified:        { tone: 'warn', label: 'New device',        sub: 'first-use pending' },
  rejected:          { tone: 'bad',  label: 'Key change',        sub: 'unconfirmed — send blocked' },
  stale:             { tone: 'bad',  label: 'Key stale',         sub: 'rehandshake required' },
};
const FU_AI = {
  granted: { label: 'Mercury AI', sub: 'scoped · read-only · 24h', tone: 'ai'    },
  absent:  { label: 'AI off',     sub: 'no grant in this room',    tone: 'muted' },
  revoked: { label: 'AI revoked', sub: 'access removed',           tone: 'bad'   },
  expired: { label: 'AI expired', sub: 'grant lapsed',             tone: 'muted' },
};

function FusionVariant({ mode = 'mobile', dark = false, trust, ai, thread, onChangeTrust, onChangeAi, onSetTheme, themeChoice }) {
  const c = dark ? FUSION_DARK : FUSION_LIGHT;
  // Both panels open by default on desktop, closed on mobile.
  const [inspectorOpen, setInspectorOpen] = React.useState(mode === 'desktop');
  const [railOpen,      setRailOpen]      = React.useState(mode === 'desktop');
  // Detail panel: null | 'trust' | 'ai' | 'recovery' | 'sync'
  const [activePanel,   setActivePanel]   = React.useState(null);
  const closePanel = React.useCallback(() => setActivePanel(null), []);
  // If the binding starts requiring action, nudge inspector open so the user sees why.
  React.useEffect(() => {
    if (mode !== 'mobile') return;
    const v = thread.outboundView;
    if (v && (v.requires_user_action || (!v.accepted && v.reason_label !== 'ACCEPTED'))) {
      // don't auto-open; just leave a visual nudge via the toggle's tone
    }
  }, [thread.outboundView, mode]);

  const cssVars = {
    '--mc-irid':       c.irid,
    '--mc-iris-text':  c.iridText,
    '--mc-mute':       c.muted,
    '--mc-scroll':     c.scroll,
    '--mc-focus':      c.focus,
    '--mc-tip-bg':     c.tipBg,
    '--mc-tip-ink':    c.tipInk,
    '--mc-tip-border': c.tipBorder,
    '--mc-tip-hi':     c.tipHi,
    '--mc-shim-a':     c.shimA, '--mc-shim-b': c.shimB,
    '--mc-shim-c':     c.shimC, '--mc-shim-d': c.shimD,
    '--mc-shim-opacity': c.shimOpacity,
  };
  return (
    <React.Fragment>
      <style>{FUSION_FONT_CSS}</style>
      <div className="mv-fuse" style={{
        width: '100%', height: '100%', display: 'flex', position: 'relative',
        background: c.bg, color: c.ink, overflow: 'hidden',
        transition: 'background .25s ease',
        ...cssVars,
      }}>
        <div className="shimmer-bg" />
        {mode === 'desktop' && railOpen && <FuseRail c={c}
          onOpenProfile={(who) => setActivePanel('profile:' + who)} />}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, position: 'relative', zIndex: 1 }}>
          <FuseThread c={c} mode={mode} trust={trust} ai={ai} thread={thread}
            inspectorOpen={inspectorOpen} onToggleInspector={() => setInspectorOpen(o => !o)}
            railOpen={railOpen} onToggleRail={() => setRailOpen(o => !o)}
            onOpenPanel={setActivePanel}
            onOpenProfile={(who) => setActivePanel('profile:' + who)} />
        </div>
        {mode === 'desktop' && inspectorOpen && <FuseInspector c={c} thread={thread} trust={trust} ai={ai}
          onClose={() => setInspectorOpen(false)}
          onOpenProfile={(who) => setActivePanel('profile:' + who)} />}
        {mode === 'mobile' && (
          <FuseMobileInspectorOverlay c={c} open={inspectorOpen}
            onClose={() => setInspectorOpen(false)}
            thread={thread} trust={trust} ai={ai}
            onOpenProfile={(who) => { setInspectorOpen(false); setActivePanel('profile:' + who); }} />
        )}
        {mode === 'mobile' && (
          <FuseMobileRoomsOverlay c={c} open={railOpen}
            onClose={() => setRailOpen(false)}
            onOpenProfile={(who) => { setRailOpen(false); setActivePanel('profile:' + who); }} />
        )}
        {/* Detail panels — modal/sheet overlays for trust verification,
            AI grant management, recovery, sync. */}
        <FuseTrustPanel    c={c} mode={mode} open={activePanel === 'trust'}    onClose={closePanel}
          trust={trust} onMarkVerified={() => { onChangeTrust && onChangeTrust('trusted'); closePanel(); }} />
        <FuseAiPanel       c={c} mode={mode} open={activePanel === 'ai'}       onClose={closePanel}
          ai={ai} onChangeAi={(v) => { onChangeAi && onChangeAi(v); }} />
        <FusePeersPanel    c={c} mode={mode} open={activePanel === 'peers'}    onClose={closePanel}
          trust={trust} ai={ai}
          onOpenTrust={() => setActivePanel('trust')}
          onOpenProfile={(who) => setActivePanel('profile:' + who)} />
        <FuseEncryptionPanel c={c} mode={mode} open={activePanel === 'encryption'} onClose={closePanel} />
        <FuseBootstrapPanel  c={c} mode={mode} open={activePanel === 'bootstrap'}  onClose={closePanel}
          view={thread.bootstrapView} />
        <FuseRecoveryPanel c={c} mode={mode} open={activePanel === 'recovery'} onClose={closePanel} />
        <FuseSyncPanel     c={c} mode={mode} open={activePanel === 'sync'}     onClose={closePanel} />
        <FuseProfilePanel  c={c} mode={mode}
          open={typeof activePanel === 'string' && activePanel.startsWith('profile:')}
          who={typeof activePanel === 'string' && activePanel.startsWith('profile:') ? activePanel.slice(8) : null}
          onClose={closePanel} trust={trust} ai={ai}
          onOpenTrust={() => setActivePanel('trust')}
          onOpenAi={() => setActivePanel('ai')}
          onOpenNotifications={() => setActivePanel('notifications')} />
        <FuseNotificationsPanel c={c} mode={mode}
          open={activePanel === 'notifications'} onClose={closePanel} />
        <FuseSettingsPanel c={c} mode={mode}
          open={activePanel === 'settings'} onClose={closePanel}
          dark={dark} themeChoice={themeChoice} onSetTheme={onSetTheme}
          onOpenProfile={(who) => setActivePanel('profile:' + who)}
          onOpenNotifications={() => setActivePanel('notifications')}
          onOpenRecovery={() => setActivePanel('recovery')}
          onOpenTrust={() => setActivePanel('trust')}
          onOpenEncryption={() => setActivePanel('encryption')} />
      </div>
    </React.Fragment>
  );
}

// ───────── rail ─────────
function FuseRail({ c, onClose, onOpenProfile }) {
  const rooms = [
    { id: 'core', n: '01', name: 'mercury-core',    sub: '4 · just now', active: true, tone: 'ok'   },
    { id: 'sec',  n: '02', name: 'security-review', sub: '6 · 12m',                    tone: 'ok'   },
    { id: 'eng',  n: '03', name: 'eng-leads',       sub: '11 · 1h',                    tone: 'warn' },
    { id: 'ai',   n: '04', name: 'ai-policy',       sub: '3 · 2h',                     tone: 'ok'   },
    { id: 'me',   n: '05', name: 'You · notes',     sub: 'yesterday',                  tone: 'ok'   },
  ];
  return (
    <div style={{
      width: 232, flexShrink: 0, borderRight: `1px solid ${c.border}`,
      background: c.surfaceWarm + 'cc', backdropFilter: 'blur(20px)',
      WebkitBackdropFilter: 'blur(20px)', position: 'relative', zIndex: 2,
      display: 'flex', flexDirection: 'column',
    }}>
      <div style={{ padding: '14px 16px 8px', display: 'flex', alignItems: 'baseline', gap: 8 }}>
        <div style={{ fontSize: 16, fontWeight: 600, letterSpacing: '-0.014em' }}>
          <span className="iris-text">Mercury</span>
        </div>
        <span className="mono" style={{ fontSize: 10.5, color: c.muted, marginLeft: 'auto' }}>v0.30</span>
        {onClose && (
          <button onClick={onClose} aria-label="Hide rooms" data-tip="Hide rooms"
            style={{
              background: 'transparent', border: 'none', cursor: 'pointer',
              padding: '2px 6px', color: c.muted, fontFamily: 'inherit', fontSize: 13,
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              lineHeight: 1, marginLeft: 4, borderRadius: 6,
            }}>✕</button>
        )}
      </div>
      <div style={{ padding: '0 12px 6px' }}>
        <div style={{
          display: 'flex', alignItems: 'center', gap: 8, padding: '6px 10px',
          background: c.surface, borderRadius: 10, fontSize: 12.5, color: c.muted,
          border: `1px solid ${c.border}`,
        }}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg>
          Search
          <span className="mono" style={{ marginLeft: 'auto', fontSize: 10, opacity: 0.7 }}>⌘K</span>
        </div>
      </div>
      <div className="mono" style={{ padding: '8px 16px 4px', fontSize: 9.5, letterSpacing: 1.2, textTransform: 'uppercase', color: c.muted, fontWeight: 600 }}>
        rooms
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: '0 8px' }}>
        {rooms.map(r => (
          <div key={r.id} style={{
            padding: '7px 10px', borderRadius: 10, marginBottom: 1,
            background: r.active ? c.surface : 'transparent',
            boxShadow: r.active ? '0 0 0 1px ' + c.border : 'none',
            cursor: 'pointer', display: 'flex', alignItems: 'center', gap: 10,
            transition: 'background .15s',
          }}>
            <span className="mono" style={{
              fontSize: 9.5, color: r.active ? c.ink2 : c.dim, fontWeight: 600,
              minWidth: 18, letterSpacing: 0.5,
            }}>{r.n}</span>
            <span style={{
              width: 6, height: 6, borderRadius: 999, flexShrink: 0,
              background: c[r.tone],
            }} />
            <div style={{ minWidth: 0, flex: 1 }}>
              <div style={{ fontSize: 13, fontWeight: r.active ? 600 : 500 }}>{r.name}</div>
              <div className="mono" style={{ fontSize: 10, color: c.muted, marginTop: 1 }}>{r.sub}</div>
            </div>
          </div>
        ))}
      </div>
      <button onClick={() => onOpenProfile && onOpenProfile('me')}
        data-tip="Your profile" style={{
        padding: '8px 14px 12px', borderTop: `1px solid ${c.border}`,
        display: 'flex', gap: 9, alignItems: 'center', width: '100%',
        background: 'transparent', border: 'none', cursor: 'pointer',
        color: 'inherit', fontFamily: 'inherit',
        textAlign: 'left', position: 'relative',
        transition: 'background .15s ease',
      }}>
        <Avatar c={c} p={MERCURY_PEOPLE.me} sz={26} />
        <div style={{ minWidth: 0, fontSize: 12 }}>
          <div style={{ fontWeight: 500 }}>You</div>
          <div className="mono" style={{ fontSize: 10, color: c.muted }}>device a1f3 · trusted</div>
        </div>
        <span style={{ marginLeft: 'auto', color: c.muted, fontSize: 11 }}>›</span>
      </button>
    </div>
  );
}

// ───────── thread ─────────
function FuseThread({ c, mode, trust, ai, thread, inspectorOpen, onToggleInspector, railOpen, onToggleRail, onOpenPanel, onOpenProfile }) {
  const ref = React.useRef(null);
  // Track whether the user is near the bottom — only auto-scroll if so, so we
  // don't snatch the viewport while they're reading history.
  const stickyRef = React.useRef(true);
  const onScroll = () => {
    const el = ref.current;
    if (!el) return;
    stickyRef.current = (el.scrollHeight - el.scrollTop - el.clientHeight) < 80;
  };
  React.useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (stickyRef.current) {
      el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' });
    }
  }, [thread.messages.length, thread.pendingTofu]);

  if (!thread.bootstrapView.can_open_message_ui) {
    return <FuseBootstrapLock c={c} view={thread.bootstrapView} onOpenPanel={onOpenPanel} />;
  }

  return (
    <React.Fragment>
      <FuseHeader c={c} mode={mode} trust={trust} ai={ai}
        inspectorOpen={inspectorOpen} onToggleInspector={onToggleInspector}
        railOpen={railOpen} onToggleRail={onToggleRail}
        onOpenPanel={onOpenPanel}
        onOpenProfile={onOpenProfile} />
      <div ref={ref} onScroll={onScroll} style={{
        flex: 1, overflowY: 'auto', overflowX: 'hidden',
        scrollBehavior: 'smooth', overscrollBehavior: 'contain',
        padding: mode === 'mobile' ? '12px 14px 6px' : '16px 24px 8px',
        display: 'flex', flexDirection: 'column', gap: 6,
      }}>
        {thread.messages.length === 0 && (
          <FuseEmptyThread c={c} />
        )}
        {thread.messages.map((m, i) => {
          const prev = thread.messages[i-1];
          const needsDay = !prev || !sameDay(prev.ts, m.ts);
          return (
            <React.Fragment key={m.id}>
              {needsDay && <FuseDayDivider c={c} ts={m.ts} />}
              <FuseMessage c={c} m={m} mode={mode} prev={prev} onOpenProfile={onOpenProfile} />
            </React.Fragment>
          );
        })}
        {thread.retryingGap && <FuseGapBanner c={c} />}
      </div>
      {thread.pendingTofu && <FuseTofu c={c} pending={thread.pendingTofu}
        onAccept={thread.acceptTofu} onCancel={thread.cancelTofu} mode={mode} />}
      <FuseComposer c={c} thread={thread} mode={mode} />
    </React.Fragment>
  );
}

function FuseHeader({ c, mode, trust, ai, inspectorOpen, onToggleInspector, railOpen, onToggleRail, onOpenPanel, onOpenProfile }) {
  const t = FU_TRUST[trust];
  const a = FU_AI[ai];
  const tColor = c[t.tone];
  const [bump, setBump] = React.useState(0);
  React.useEffect(() => { setBump(b => b + 1); }, [trust]);

  return (
    <div style={{
      padding: mode === 'mobile' ? '12px 14px 10px' : '14px 28px 12px',
      borderBottom: `1px solid ${c.border}`, background: c.surface + 'd9',
      backdropFilter: 'blur(20px)', WebkitBackdropFilter: 'blur(20px)',
      display: 'flex', flexDirection: 'column', gap: 10, position: 'relative',
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <FuseRoomsToggle c={c} open={railOpen} onClick={onToggleRail} />
        <span style={{ fontSize: 10.5, color: c.muted, letterSpacing: 0.4 }}>#</span>
        <span style={{
          fontSize: mode === 'mobile' ? 16 : 17, fontWeight: 600,
          letterSpacing: '-0.014em',
        }}>mercury-core</span>
        <div style={{ flex: 1 }} />
        {/* Your avatar — always available, taps open your profile. The rail
            footer also opens this, but most users don't need rooms to view
            their own profile. */}
        <Avatar c={c} p={MERCURY_PEOPLE.me} sz={26}
          onClick={() => onOpenProfile && onOpenProfile('me')} />
        <FuseSettingsToggle c={c} onClick={() => onOpenPanel && onOpenPanel('settings')} />
        <FuseInspectToggle c={c} open={inspectorOpen} onClick={onToggleInspector} />
      </div>
      <FuseDataStrip c={c} trust={trust} ai={ai} bump={bump} t={t} a={a} tColor={tColor} mode={mode} onOpenPanel={onOpenPanel} />
    </div>
  );
}

// Sidebar-panel glyph for the rooms toggle.
function RailIcon({ size = 12, color = 'currentColor' }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
      stroke={color} strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round"
      style={{ flexShrink: 0 }} aria-hidden="true">
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M9 4v16" />
    </svg>
  );
}

function FuseRoomsToggle({ c, open, onClick }) {
  return (
    <button onClick={onClick} data-tip={open ? 'Hide rooms' : 'Show rooms'}
      aria-label={open ? 'Hide rooms panel' : 'Show rooms panel'}
      aria-expanded={open}
      style={{
        position: 'relative',
        background: c.surfaceWarm, border: open ? 'none' : `1px solid ${c.border}`,
        padding: '4px 9px', borderRadius: 7, cursor: 'pointer',
        fontFamily: 'inherit', fontSize: 10.5, color: c.ink2, fontWeight: 600,
        display: 'inline-flex', alignItems: 'center', gap: 6, lineHeight: 1.2,
      }}>
      {open && <span className="iris-ring" style={{ borderRadius: 7 }} />}
      <span style={{ position: 'relative', display: 'flex', alignItems: 'center', color: open ? c.ink : c.ink2 }}>
        <RailIcon size={12} />
      </span>
      <span style={{ position: 'relative' }}>rooms</span>
      <span style={{
        position: 'relative', color: c.muted, fontSize: 9,
        opacity: 0.8, marginLeft: 2,
      }}>{open ? '▾' : '▸'}</span>
    </button>
  );
}

// Inspector toggle — reads like a terminal command pill: `$ inspect`.
// Active state gets the iridescent outline.
function FuseInspectToggle({ c, open, onClick }) {
  return (
    <button onClick={onClick} data-tip={open ? 'Hide inspector' : 'Show inspector'}
      aria-label={open ? 'Hide inspector panel' : 'Show inspector panel'}
      aria-expanded={open}
      style={{
        position: 'relative',
        background: c.surfaceWarm, border: open ? 'none' : `1px solid ${c.border}`,
        padding: '4px 9px', borderRadius: 7, cursor: 'pointer',
        fontFamily: 'inherit', fontSize: 10.5, color: c.ink2, fontWeight: 600,
        display: 'inline-flex', alignItems: 'center', gap: 6, lineHeight: 1.2,
      }}>
      {open && <span className="iris-ring" style={{ borderRadius: 7 }} />}
      <span style={{ position: 'relative', display: 'flex', alignItems: 'center', color: open ? c.ink : c.ink2 }}>
        <Helix size={12} />
      </span>
      <span style={{ position: 'relative' }}>inspect</span>
      <span style={{
        position: 'relative', color: c.muted, fontSize: 9,
        opacity: 0.8, marginLeft: 2,
      }}>{open ? '▾' : '▸'}</span>
    </button>
  );
}

// Gear glyph + Settings toggle pill (between avatar and inspect).
function GearIcon({ size = 13, color = 'currentColor' }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
      stroke={color} strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round"
      style={{ flexShrink: 0 }} aria-hidden="true">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06A2 2 0 1 1 4.27 16.96l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06A2 2 0 1 1 7.04 4.27l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

function FuseSettingsToggle({ c, onClick }) {
  return (
    <button onClick={onClick} data-tip="Settings" aria-label="Open settings"
      style={{
        background: c.surfaceWarm, border: `1px solid ${c.border}`,
        width: 26, height: 26, borderRadius: 7, cursor: 'pointer',
        display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
        color: c.ink2, padding: 0, fontFamily: 'inherit',
      }}>
      <GearIcon size={13} />
    </button>
  );
}

// Structured key=value data strip — a mono-labelled chip rail.
// Brings terminal organization into the modern chrome: each cell is a discrete
// label/value unit; iridescent chips mark the live AI grant. Wraps on mobile.
function FuseDataStrip({ c, trust, ai, bump, t, a, tColor, mode, onOpenPanel }) {
  const cells = [
    { key: 'trust',      value: t.label.toLowerCase().replace(' ','_'), tone: t.tone, dot: tColor, pulse: true, title: t.sub, panel: 'trust' },
    { key: 'ai',         value: ai,                                      tone: ai === 'granted' ? 'irid' : (a.tone === 'bad' ? 'bad' : 'muted'), title: a.sub, panel: 'ai' },
    { key: 'peers',      value: '4', title: 'Participants in this room', panel: 'peers' },
    { key: 'encryption', value: 'end-to-end', title: 'Messages are encrypted on your device. The server never sees plaintext.', panel: 'encryption' },
    { key: 'bootstrap',  value: 'ACCEPTED', tone: 'ok', dot: c.ok, title: 'Client is past the bootstrap gate.', panel: 'bootstrap' },
  ];
  return (
    <div className="mono" style={{
      display: 'flex', gap: 6, flexWrap: 'wrap',
      fontSize: 10.5,
    }}>
      {cells.map((cell, i) => (
        <FuseDataCell key={cell.key} c={c} cell={cell} bump={bump}
          onClick={cell.panel && onOpenPanel ? () => onOpenPanel(cell.panel) : undefined} />
      ))}
    </div>
  );
}

function FuseDataCell({ c, cell, bump, onClick }) {
  const isIrid = cell.tone === 'irid';
  const interactive = !!onClick;
  const valColor = cell.tone === 'ok'    ? c.ok
                : cell.tone === 'warn'  ? c.warn
                : cell.tone === 'bad'   ? c.bad
                : cell.tone === 'muted' ? c.muted
                : c.ink;
  const Tag = interactive ? 'button' : 'span';
  return (
    <Tag onClick={onClick}
      {...(cell.title ? { 'data-tip': cell.title } : {})} style={{
      position: 'relative',
      display: 'inline-flex', alignItems: 'center', gap: 5,
      padding: '4px 9px', borderRadius: 7,
      background: isIrid ? c.surface : c.surfaceWarm,
      border: isIrid ? 'none' : `1px solid ${c.border}`,
      lineHeight: 1.2,
      cursor: interactive ? 'pointer' : (cell.title ? 'help' : 'default'),
      fontFamily: 'inherit', fontSize: 'inherit',
      color: 'inherit',
      transition: 'background .15s ease',
    }}>
      {isIrid && <span className="iris-ring" style={{ borderRadius: 7 }} />}
      {cell.dot && (
        <span key={cell.key === 'trust' ? 't'+bump : undefined}
              className={cell.pulse ? 'pulse-dot' : undefined}
              style={{ width: 5, height: 5, borderRadius: 999, background: cell.dot, position: 'relative' }} />
      )}
      <span style={{ color: c.muted, position: 'relative' }}>{cell.key}</span>
      <span style={{ color: c.muted, opacity: 0.5, position: 'relative' }}>=</span>
      {isIrid
        ? <span className="iris-text" style={{ fontWeight: 600, position: 'relative' }}>{cell.value}</span>
        : <span style={{ color: valColor, fontWeight: 600, position: 'relative' }}>{cell.value}</span>}
      {interactive && (
        <span style={{ color: c.muted, fontSize: 8.5, opacity: 0.7, marginLeft: 1, position: 'relative' }}>›</span>
      )}
    </Tag>
  );
}

// ───────── messages ─────────
// Day divider — "Today" / "Yesterday" / "Mar 15" rule rendered between
// messages from different calendar days. Styled like the system kicker.
// Empty-state — shown when the thread has no messages yet (new room or cleared).
function FuseEmptyThread({ c }) {
  return (
    <div style={{
      flex: 1, display: 'flex', flexDirection: 'column',
      alignItems: 'center', justifyContent: 'center',
      padding: '40px 24px', textAlign: 'center', minHeight: 200,
    }}>
      <div style={{
        position: 'relative', overflow: 'hidden',
        width: 52, height: 52, borderRadius: '50%',
        background: c.surface, marginBottom: 14,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        color: c.ai,
      }}>
        <span className="iris-ring" style={{ borderRadius: '50%' }} />
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" style={{ position: 'relative' }}>
          <path d="M21 12a8 8 0 1 1-3.13-6.31"/>
          <path d="M21 4v6h-6"/>
        </svg>
      </div>
      <div style={{ fontSize: 14.5, fontWeight: 600, color: c.ink, marginBottom: 4 }}>
        No messages yet
      </div>
      <div style={{ fontSize: 12, color: c.muted, lineHeight: 1.5, maxWidth: 280 }}>
        This room is end-to-end encrypted and gated by mercury-core. Your first message
        will start the thread.
      </div>
    </div>
  );
}

function FuseDayDivider({ c, ts }) {
  return (
    <div style={{
      alignSelf: 'stretch', display: 'flex', alignItems: 'center', gap: 10,
      padding: '6px 0', color: c.muted,
    }}>
      <span style={{ flex: 1, height: 1, background: c.border }} />
      <span style={{ fontSize: 10, letterSpacing: 1.1, textTransform: 'uppercase', fontWeight: 700 }}>
        {fmtDay(ts)}
      </span>
      <span style={{ flex: 1, height: 1, background: c.border }} />
    </div>
  );
}

function FuseMessage({ c, m, mode, prev, onOpenProfile }) {
  if (m.kind === 'system') {
    return (
      <div className="rise" style={{
        alignSelf: 'stretch', display: 'flex', alignItems: 'center', gap: 10,
        padding: '4px 0', color: c.muted,
      }}>
        <span style={{ flex: 1, height: 1, background: c.border }} />
        <span style={{
          fontSize: 10, letterSpacing: 1.1, textTransform: 'uppercase', fontWeight: 600,
        }}>{m.text}</span>
        <span style={{ flex: 1, height: 1, background: c.border }} />
      </div>
    );
  }
  if (m.kind === 'gap')              return <FuseGapBanner c={c} reason={m.decision?.reason_label} text={m.text} />;
  if (m.kind === 'send_blocked')     return <FuseSendBlocked c={c} m={m} />;
  if (m.kind === 'rejected_inbound') return <FuseInboundBlocked c={c} m={m} />;

  const p = MERCURY_PEOPLE[m.author] || MERCURY_PEOPLE.rin;
  const isMe = m.author === 'me';
  const isAi = p.isAi;
  const sameAuthor = prev && prev.author === m.author && (prev.kind === 'incoming' || prev.kind === 'outgoing');

  if (isMe) return <FuseOutgoingBubble c={c} m={m} mode={mode} />;
  return <FuseIncomingBubble c={c} m={m} mode={mode} p={p} isAi={isAi} grouped={sameAuthor} onOpenProfile={onOpenProfile} />;
}

// Per-author tint — derived from the participant's hue so each member of a
// group chat has a unique, subtle bubble color + name color.
function authorTint(c, p) {
  const dark = (c === FUSION_DARK);
  return {
    bg:     `hsl(${p.hue} ${dark ? 35 : 70}% ${dark ? 14 : 95}%)`,
    border: `hsl(${p.hue} ${dark ? 40 : 55}% ${dark ? 28 : 82}%)`,
    name:   `hsl(${p.hue} ${dark ? 60 : 60}% ${dark ? 70 : 38}%)`,
  };
}

function FuseIncomingBubble({ c, m, mode, p, isAi, grouped, onOpenProfile }) {
  const tint = authorTint(c, p);
  const aiBg = c === FUSION_DARK ? 'rgba(168,155,255,.08)' : 'rgba(123,92,240,.06)';
  return (
    <div className="rise" style={{
      display: 'flex', gap: 8, alignSelf: 'flex-start',
      maxWidth: mode === 'mobile' ? '92%' : '78%',
    }}>
      {grouped
        ? <div style={{ width: 28, flexShrink: 0 }} />
        : <Avatar c={c} p={p} isAi={isAi} sz={28} tint={tint}
            onClick={onOpenProfile ? () => onOpenProfile(m.author) : undefined} />}
      <div style={{ minWidth: 0, flex: 1 }}>
        {!grouped && (
          <div style={{
            display: 'flex', gap: 8, alignItems: 'baseline', marginBottom: 3,
            fontSize: 11,
          }}>
            {isAi ? (
              <span className="iris-text" style={{ fontWeight: 600 }}>{p.name}</span>
            ) : (
              <span style={{ color: tint.name, fontWeight: 600 }}>{p.name}</span>
            )}
            {isAi && <span style={{ fontSize: 9.5, color: c.ai, letterSpacing: 0.4, textTransform: 'uppercase' }}>scoped · read-only</span>}
          </div>
        )}
        <div style={{
          position: 'relative',
          padding: '8px 12px', borderRadius: 12, borderBottomLeftRadius: 4,
          background: isAi ? aiBg : tint.bg,
          border: isAi ? 'none' : `1px solid ${tint.border}`,
          fontSize: 12.5, lineHeight: 1.5, color: c.ink,
          overflow: 'hidden',
        }}>
          {isAi && <span className="iris-ring" style={{ borderRadius: 12 }} />}
          <span style={{ position: 'relative' }}>
            {isAi ? <FuseStreamingText id={m.id} text={m.text} /> : m.text}
          </span>
        </div>
        {/* timestamp always visible — stays put for the entire conversation,
            and carries the date prefix when the message isn't from today. */}
        <div style={{ fontSize: 10.5, color: c.muted, marginTop: 3 }}>
          {fmtMsgTime(m.ts)}
        </div>
      </div>
    </div>
  );
}

function FuseOutgoingBubble({ c, m, mode }) {
  return (
    <div className="rise" style={{
      display: 'flex', flexDirection: 'column', alignItems: 'flex-end',
      gap: 3, marginLeft: 'auto', maxWidth: mode === 'mobile' ? '88%' : '74%',
    }}>
      <div style={{
        position: 'relative',
        padding: '8px 14px', borderRadius: 12, borderBottomRightRadius: 4,
        background: c.surface,
        fontSize: 12.5, lineHeight: 1.5, color: c.ink, fontWeight: 500,
        overflow: 'hidden',
        boxShadow: '0 1px 2px rgba(0,0,0,.04), 0 6px 18px rgba(0,0,0,.04)',
      }}>
        <span className="iris-ring" style={{ borderRadius: 12 }} />
        <span style={{ position: 'relative' }}>{m.text}</span>
      </div>
      <div style={{
        fontSize: 10.5, color: c.muted, textAlign: 'right',
        display: 'flex', gap: 8,
      }}>
        <span style={{ color: m.status === 'delivered' ? c.ok : c.muted }}>
          {m.status === 'sending' ? '○ sending' : '● delivered'}
        </span>
        <span>· {fmtMsgTime(m.ts)}</span>
        {m.decision?.reason_label && m.decision.reason_label !== 'ACCEPTED' && (
          <span style={{ color: c.warn }}>· {m.decision.reason_label}</span>
        )}
      </div>
    </div>
  );
}

function Avatar({ c, p, isAi, sz = 28, tint, onClick }) {
  const Tag = onClick ? 'button' : 'div';
  const commonBtn = onClick ? {
    cursor: 'pointer', padding: 0, fontFamily: 'inherit',
  } : {};
  if (isAi) {
    const aiBg = c === FUSION_DARK ? 'rgba(168,155,255,.10)' : 'rgba(123,92,240,.08)';
    return (
      <Tag onClick={onClick}
        {...(onClick ? { 'data-tip': 'Open ' + p.name } : {})}
        style={{
        width: sz, height: sz, borderRadius: '50%', flexShrink: 0,
        background: aiBg, position: 'relative', overflow: 'hidden',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        fontSize: 10, fontWeight: 700, letterSpacing: 0.4,
        border: 'none',
        ...commonBtn,
      }}>
        <span className="iris-ring" style={{ borderRadius: '50%' }} />
        <span className="iris-text" style={{ position: 'relative' }}>{p.short}</span>
      </Tag>
    );
  }
  const t = tint || authorTint(c, p);
  return (
    <Tag onClick={onClick}
      {...(onClick ? { 'data-tip': 'Open ' + p.name } : {})}
      style={{
      width: sz, height: sz, borderRadius: '50%', flexShrink: 0,
      background: t.bg, border: `1px solid ${t.border}`,
      color: t.name,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      fontSize: 10, fontWeight: 700, letterSpacing: 0.4,
      ...commonBtn,
    }}>
      {p.short}
    </Tag>
  );
}

// streaming AI text — persist completion across re-renders
const __streamedIds = new Set();
function FuseStreamingText({ id, text }) {
  const [shown, setShown] = React.useState(() => __streamedIds.has(id) ? text.length : 0);
  React.useEffect(() => {
    if (__streamedIds.has(id)) return;
    let i = shown;
    const handle = setInterval(() => {
      i += Math.max(1, Math.round(text.length / 80));
      if (i >= text.length) {
        i = text.length;
        clearInterval(handle);
        __streamedIds.add(id);
      }
      setShown(i);
    }, 18);
    return () => clearInterval(handle);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);
  return (
    <span>
      {text.slice(0, shown)}
      {shown < text.length && (
        <span style={{
          display: 'inline-block', width: 7, height: 13,
          marginLeft: 2, background: 'currentColor', verticalAlign: '-2px',
        }} className="blink" />
      )}
    </span>
  );
}

// ───────── banners ─────────
function FuseGapBanner({ c, reason = 'ORDERING_GAP', text }) {
  return (
    <div className="rise" style={{
      alignSelf: 'stretch', display: 'flex', gap: 10, alignItems: 'flex-start',
      padding: '10px 14px', borderRadius: 12,
      background: c.warnSoft, border: `1px solid ${c.warn}33`, color: c.warn,
      fontSize: 12.5,
    }}>
      <Spinner color={c.warn} />
      <div style={{ flex: 1 }}>
        <div style={{ fontWeight: 600 }}>{text || 'Ordering gap — fetching missing messages'}</div>
        <div className="mono" style={{ fontSize: 10.5, marginTop: 2, opacity: 0.85 }}>
          client_receive · {reason} · rc={REASON_CODE[reason] ?? 10} · requires_client_retry
        </div>
      </div>
    </div>
  );
}

function FuseSendBlocked({ c, m }) {
  const r = REASON_DICT[m.decision.reason_label] || { gloss: m.decision.reason_label };
  return (
    <div className="rise" style={{
      alignSelf: 'flex-end', maxWidth: '78%',
      padding: '10px 14px', borderRadius: 14,
      background: c.badSoft, border: `1px solid ${c.bad}40`, color: c.bad,
      fontSize: 12.5, borderBottomRightRadius: 6,
    }}>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 4 }}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2"><circle cx="12" cy="12" r="10"/><path d="M4.9 4.9 19.1 19.1"/></svg>
        <span style={{ fontWeight: 600 }}>Send withheld · {r.gloss}</span>
      </div>
      <div style={{ fontStyle: 'italic', color: c.ink2, lineHeight: 1.45 }}>"{m.text}"</div>
      <div className="mono" style={{ fontSize: 10.5, marginTop: 6, opacity: 0.85 }}>
        outbound_send · {m.decision.reason_label} · rc={m.decision.reason_code} · ciphertext not persisted
      </div>
    </div>
  );
}

function FuseInboundBlocked({ c, m }) {
  return (
    <div className="rise" style={{
      alignSelf: 'flex-start', maxWidth: '78%',
      padding: '10px 14px', borderRadius: 14,
      background: c.surface, border: `1px dashed ${c.borderStrong}`, color: c.muted,
      fontSize: 12.5,
    }}>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 4 }}>
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2 4 5v6c0 5 3.5 9 8 11 4.5-2 8-6 8-11V5l-8-3Z"/></svg>
        <span style={{ fontWeight: 600, color: c.ink2 }}>Message withheld</span>
        <span className="mono" style={{ fontSize: 10, color: c.muted, marginLeft: 'auto' }}>requires_user_action</span>
      </div>
      <div style={{ fontStyle: 'italic' }}>{m.text}</div>
      <div className="mono" style={{ fontSize: 10.5, marginTop: 6, color: c.muted }}>
        client_receive · SENDER_TRUST_REJECTED · rc={REASON_CODE.SENDER_TRUST_REJECTED}
      </div>
    </div>
  );
}

// ───────── TOFU ─────────
function FuseTofu({ c, pending, onAccept, onCancel, mode }) {
  return (
    <div className="rise" style={{
      margin: mode === 'mobile' ? '0 14px 8px' : '0 28px 12px',
      padding: '14px 16px', position: 'relative',
      background: c.warnSoft, border: `1px solid ${c.warn}55`, borderRadius: 14,
    }}>
      <div style={{ display: 'flex', gap: 10, alignItems: 'flex-start', marginBottom: 10 }}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={c.warn} strokeWidth="2">
          <path d="M12 2 4 5v6c0 5 3.5 9 8 11 4.5-2 8-6 8-11V5l-8-3Z"/>
          <path d="M9 12l2 2 4-4"/>
        </svg>
        <div style={{ flex: 1, fontSize: 13, lineHeight: 1.5 }}>
          <div style={{ fontWeight: 600, color: c.warn, marginBottom: 2 }}>First-use verification</div>
          <span style={{ color: c.ink2 }}>
            A correspondent has a new device key. Confirm to release the dispatch, or compare safety numbers first.
          </span>
          <div className="mono" style={{
            fontSize: 11, color: c.ink2, marginTop: 8,
            padding: '6px 10px', background: c.surface, borderRadius: 8,
            letterSpacing: 0.3,
          }}>SN · a1f3 · 9c20 · 8e4b · 7d12 · 6b8e · 0a35</div>
        </div>
      </div>
      <div style={{ display: 'flex', gap: 8 }}>
        <button onClick={onCancel} style={fuseBtn(c, 'ghost')}>Cancel</button>
        <button onClick={onAccept} style={fuseBtn(c, 'irid')}>
          <span className="iris-ring" style={{ borderRadius: 8 }} />
          <span style={{ position: 'relative' }}>Confirm &amp; send</span>
        </button>
      </div>
      <div className="mono" style={{ fontSize: 10.5, color: c.muted, marginTop: 10 }}>
        outbound_send · TOFU_PENDING · rc={REASON_CODE.TOFU_PENDING} · requires_user_action
      </div>
    </div>
  );
}

// ───────── composer ─────────
function FuseComposer({ c, thread, mode }) {
  const view = thread.outboundView;
  const canSend = view.can_send;
  const reason = REASON_DICT[view.reason_label];
  const placeholder = !canSend
    ? `Withheld · ${reason?.gloss || view.reason_label}`
    : view.requires_user_action
      ? 'Message · verification will be requested'
      : thread.draftMentionsAi
        ? 'Message · @ai will receive scoped context'
        : 'Message';

  const statusColor = !canSend ? c.bad
    : view.requires_user_action ? c.warn
    : thread.draftMentionsAi ? c.ai : c.muted;

  return (
    <div style={{
      padding: mode === 'mobile' ? '10px 14px 14px' : '12px 28px 16px',
      borderTop: `1px solid ${c.border}`,
      background: c.surface + 'd9',
      backdropFilter: 'blur(20px)', WebkitBackdropFilter: 'blur(20px)',
      position: 'relative',
    }}>
      <div style={{
        display: 'flex', alignItems: 'stretch', gap: 0,
        background: canSend ? c.surfaceWarm : c.badSoft,
        border: `1px solid ${canSend ? c.border : c.bad + '55'}`,
        borderRadius: 14, overflow: 'hidden',
        transition: 'background .2s ease, border-color .2s ease',
      }}>
        {/* structured mono prefix block — terminal-organized addressing */}
        <div className="mono" style={{
          padding: '0 10px', display: 'flex', alignItems: 'center', gap: 6,
          fontSize: 11, color: c.muted, flexShrink: 0,
          borderRight: `1px solid ${c.border}`, background: c.surface,
        }}>
          <span style={{ color: c.ink3 }}>~/mercury-core</span>
          <span style={{ color: c.ink }}>›</span>
        </div>
        <textarea
          value={thread.draft}
          onChange={e => thread.setDraft(e.target.value)}
          onKeyDown={e => {
            if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); thread.send(); }
          }}
          rows={1}
          placeholder={placeholder}
          style={{
            flex: 1, resize: 'none', border: 'none', outline: 'none',
            background: 'transparent', color: c.ink, fontSize: 14.5,
            lineHeight: 1.5, padding: '10px 12px', minHeight: 22, maxHeight: 120,
            fontFamily: 'inherit',
          }}
        />
        {/* blinking cursor when input is empty — terminal idle state */}
        {!thread.draft && canSend && (
          <span className="blink" style={{
            display: 'inline-block', width: 8, height: 14,
            background: c.ink, alignSelf: 'center', marginRight: 4, marginLeft: -6,
          }} />
        )}
        <div style={{ padding: 5, display: 'flex', alignItems: 'flex-end' }}>
          <button
            onClick={() => thread.send()}
            disabled={!canSend && !view.requires_user_action}
            style={{
              border: 'none', cursor: (canSend || view.requires_user_action) ? 'pointer' : 'not-allowed',
              width: 36, height: 36, borderRadius: 10, position: 'relative',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              color: canSend ? c.ink : c.muted,
              background: c.surface,
              overflow: 'hidden', transition: 'background .2s ease',
            }}
            title={canSend ? 'Send (⏎)' : (reason?.gloss || 'Blocked')}
          >
            {canSend && <span className="iris-ring" style={{ borderRadius: 10 }} />}
            {canSend ? (
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" style={{ position: 'relative' }}>
                <path d="M12 19V5"/><path d="m5 12 7-7 7 7"/>
              </svg>
            ) : (
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round">
                <rect x="5" y="11" width="14" height="9" rx="2"/>
                <path d="M8 11V8a4 4 0 0 1 8 0v3"/>
              </svg>
            )}
          </button>
        </div>
      </div>
      <div className="mono" style={{
        fontSize: 10.5, marginTop: 6, paddingLeft: 4,
        color: statusColor, display: 'flex', gap: 10, flexWrap: 'wrap',
        transition: 'color .2s ease',
      }}>
        <span>outbound_send · {view.reason_label} · rc={view.reason_code}</span>
        {view.requires_user_action && <span style={{ color: c.warn }}>requires_user_action</span>}
        {!view.can_persist_ciphertext && !view.accepted && <span style={{ color: c.bad }}>no persistence</span>}
        {thread.draftMentionsAi && <span style={{ color: c.ai }}>@ai → scoped context</span>}
        <span style={{ marginLeft: 'auto', color: c.muted }}>⏎ send</span>
      </div>
    </div>
  );
}

// ───────── mobile rooms overlay ─────────
// Slides in from the left over the thread. Backdrop closes on tap.
function FuseMobileRoomsOverlay({ c, open, onClose, onOpenProfile }) {
  return (
    <React.Fragment>
      <div onClick={onClose} style={{
        position: 'absolute', inset: 0, zIndex: 4,
        background: 'rgba(0,0,0,.32)',
        opacity: open ? 1 : 0,
        pointerEvents: open ? 'auto' : 'none',
        transition: 'opacity .22s ease',
      }} />
      <div style={{
        position: 'absolute', top: 0, left: 0, bottom: 0, zIndex: 5,
        width: '76%', maxWidth: 252,
        background: c.surfaceWarm,
        borderRight: `1px solid ${c.border}`,
        boxShadow: '12px 0 32px rgba(0,0,0,.18)',
        transform: open ? 'translateX(0)' : 'translateX(-105%)',
        transition: 'transform .26s cubic-bezier(.2,.7,.2,1)',
        display: 'flex', flexDirection: 'column', overflow: 'hidden',
      }}>
        <FuseRail c={c} onClose={onClose} onOpenProfile={onOpenProfile} />
      </div>
    </React.Fragment>
  );
}

// ───────── mobile inspector overlay ─────────
// Slides in from the right over the thread. Backdrop closes on tap.
function FuseMobileInspectorOverlay({ c, open, onClose, thread, trust, ai, onOpenProfile }) {
  return (
    <React.Fragment>
      <div onClick={onClose} style={{
        position: 'absolute', inset: 0, zIndex: 4,
        background: 'rgba(0,0,0,.32)',
        opacity: open ? 1 : 0,
        pointerEvents: open ? 'auto' : 'none',
        transition: 'opacity .22s ease',
      }} />
      <div style={{
        position: 'absolute', top: 0, right: 0, bottom: 0, zIndex: 5,
        width: '88%', maxWidth: 340,
        background: c.surfaceWarm,
        borderLeft: `1px solid ${c.border}`,
        boxShadow: '-12px 0 32px rgba(0,0,0,.18)',
        transform: open ? 'translateX(0)' : 'translateX(105%)',
        transition: 'transform .26s cubic-bezier(.2,.7,.2,1)',
        display: 'flex', flexDirection: 'column', overflow: 'hidden',
      }}>
        <FuseInspector c={c} thread={thread} trust={trust} ai={ai} onClose={onClose} fullWidth onOpenProfile={onOpenProfile} />
      </div>
    </React.Fragment>
  );
}

// ───────── desktop inspector ─────────
function FuseInspector({ c, thread, trust, ai, onClose, fullWidth, onOpenProfile }) {
  return (
    <div style={{
      width: fullWidth ? '100%' : 290, flexShrink: 0,
      borderLeft: fullWidth ? 'none' : `1px solid ${c.border}`,
      background: c.surfaceWarm + 'cc', backdropFilter: 'blur(20px)',
      WebkitBackdropFilter: 'blur(20px)', position: 'relative', zIndex: 2,
      display: 'flex', flexDirection: 'column', overflow: 'hidden',
      height: '100%',
    }}>
      <div style={{
        padding: '14px 18px 12px', borderBottom: `1px solid ${c.border}`,
        display: 'flex', alignItems: 'center', gap: 8,
      }}>
        <Helix size={14} color={c.ai} />
        <span style={{ fontSize: 12.5, color: c.ink, fontWeight: 600 }}>Inspector</span>
        <span style={{ fontSize: 10.5, color: c.muted }}>· binding view</span>
        <span style={{ fontSize: 10, color: c.ok, marginLeft: 'auto' }}>● live</span>
        {onClose && (
          <button onClick={onClose} aria-label="Hide inspector" data-tip="Hide inspector"
            style={{
              background: 'transparent', border: 'none', cursor: 'pointer',
              padding: '2px 6px', color: c.muted, fontFamily: 'inherit', fontSize: 13,
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              lineHeight: 1, borderRadius: 6,
            }}>✕</button>
        )}
      </div>
      <div style={{ flex: '1 1 auto', minHeight: 0, padding: '14px 18px', display: 'flex', flexDirection: 'column', gap: 14, overflow: 'auto' }}>
        <DecisionCard c={c} title="Bootstrap"     source="client_bootstrap" view={thread.bootstrapView} />
        <DecisionCard c={c} title="Outbound send" source="outbound_send"    view={thread.outboundView}  />
        <DecisionCard c={c} title="Last receive"  source="client_receive"   view={thread.receiveView}   />
        <div>
          <div style={{
            fontSize: 10, letterSpacing: 1.3, textTransform: 'uppercase',
            color: c.muted, marginBottom: 8, fontWeight: 600,
          }}>Participants</div>
          {['rin','jules','ai'].map(id => {
            const p = MERCURY_PEOPLE[id];
            return (
              <button key={id}
                onClick={() => onOpenProfile && onOpenProfile(id)}
                style={{
                  display: 'flex', alignItems: 'center', gap: 10, padding: '3px 0',
                  background: 'transparent', border: 'none', cursor: 'pointer',
                  width: '100%', textAlign: 'left', fontFamily: 'inherit', color: 'inherit',
                }}>
                <Avatar c={c} p={p} isAi={p.isAi} sz={22} />
                <div style={{ minWidth: 0, flex: 1 }}>
                  <div style={{ fontSize: 11.5, fontWeight: 500 }} className={p.isAi ? 'iris-text' : undefined}>{p.name}</div>
                  <div style={{ fontSize: 10, color: c.muted }}>
                    {p.isAi ? `ai · ${ai}` : '2 devices · verified'}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      </div>
      <FuseEventTail c={c} thread={thread} />
    </div>
  );
}

// Live decision-event tail — `tail -f` on the binding's stream of views.
// New entries highlight then fade. Bottom-anchored, scrollable.
function FuseEventTail({ c, thread }) {
  const [log, setLog] = React.useState([]);
  const lastBoot = React.useRef('');
  const lastOut  = React.useRef('');
  const lastRcv  = React.useRef('');
  function push(source, view) {
    setLog(L => [...L, {
      id: Math.random().toString(36).slice(2),
      when: Date.now(), source, view,
    }].slice(-40));
  }
  React.useEffect(() => {
    const v = thread.bootstrapView; const k = `${v.accepted}-${v.reason_label}`;
    if (k !== lastBoot.current) { lastBoot.current = k; push('bootstrap', v); }
  }, [thread.bootstrapView]);
  React.useEffect(() => {
    const v = thread.outboundView; const k = `${v.accepted}-${v.reason_label}`;
    if (k !== lastOut.current)  { lastOut.current  = k; push('outbound',  v); }
  }, [thread.outboundView]);
  React.useEffect(() => {
    const v = thread.receiveView; const k = `${v.accepted}-${v.reason_label}`;
    if (k !== lastRcv.current)  { lastRcv.current  = k; push('receive',   v); }
  }, [thread.receiveView]);

  const ref = React.useRef(null);
  React.useEffect(() => {
    if (ref.current) ref.current.scrollTop = ref.current.scrollHeight;
  }, [log.length]);

  return (
    <div style={{
      flex: '0 0 220px', minHeight: 140, borderTop: `1px solid ${c.border}`,
      display: 'flex', flexDirection: 'column',
    }}>
      <div style={{
        padding: '8px 18px', display: 'flex', alignItems: 'center', gap: 6,
        fontSize: 11, color: c.muted, borderBottom: `1px solid ${c.border}`,
      }}>
        <Helix size={11} color={c.ai} />
        <span style={{ color: c.ink }}>events</span>
        <span style={{ color: c.muted }}>· most recent below</span>
        <span style={{ marginLeft: 'auto', fontSize: 10 }}>{log.length}</span>
      </div>
      <div ref={ref} style={{
        flex: 1, overflow: 'auto', padding: '6px 14px 10px',
        fontSize: 10.5, lineHeight: 1.5,
      }}>
        {log.map(e => <FuseTailRow key={e.id} c={c} entry={e} />)}
      </div>
    </div>
  );
}

function FuseTailRow({ c, entry }) {
  const v = entry.view;
  const ok = v.accepted;
  const tone = ok ? c.ok : c.bad;
  const reqs = ['requires_sync','requires_recovery','requires_client_retry','requires_user_action']
    .filter(k => v[k]);
  return (
    <div className="card-in" style={{ padding: '3px 4px', margin: '0 -4px', borderRadius: 3 }}>
      <div style={{ display: 'flex', gap: 6, alignItems: 'baseline' }}>
        <span style={{ color: c.dim, fontSize: 10 }}>{fmtClkSec(entry.when)}</span>
        <span style={{ color: c.ai }}>{entry.source}</span>
        <span style={{ color: c.muted }}>→</span>
        <span style={{ color: tone, fontWeight: 600 }}>{ok ? 'ACCEPT' : 'REJECT'}</span>
        <span style={{ color: c.muted, marginLeft: 'auto' }}>rc={v.reason_code}</span>
      </div>
      <div style={{ paddingLeft: 14, color: ok ? c.ink2 : c.bad }}>
        <span style={{ color: c.muted }}>label=</span>{v.reason_label}
      </div>
      {reqs.length > 0 && (
        <div style={{ paddingLeft: 14, color: c.warn, fontSize: 10 }}>
          <span style={{ color: c.muted }}>{'> '}</span>{reqs.join(' · ')}
        </div>
      )}
    </div>
  );
}

function DecisionCard({ c, title, source, view }) {
  const ok = view.accepted;
  const key = `${view.source}-${view.reason_label}-${view.accepted}`;
  const explain = (REASON_EXPLAIN[source || view.source] || {})[view.reason_label]
    || (ok ? 'Decision accepted.' : 'Decision rejected by the gate.');

  const flags = ['can_open_message_ui','can_start_sync','can_send','can_receive','can_persist_ciphertext',
                 'requires_sync','requires_recovery','requires_client_retry','requires_user_action'];
  const liveFlags = flags.filter(f => view[f] === true);
  const [showRaw, setShowRaw] = React.useState(false);

  return (
    <div>
      <div style={{
        fontSize: 10, letterSpacing: 1.3, textTransform: 'uppercase',
        color: c.muted, marginBottom: 8, fontWeight: 600,
      }}>{title}</div>
      <div key={key} className="card-in" style={{
        background: c.surface, border: `1px solid ${c.border}`, borderRadius: 12,
        overflow: 'hidden',
      }}>
        {/* status header */}
        <div style={{
          padding: '10px 12px 8px', display: 'flex', alignItems: 'center', gap: 8,
          borderLeft: `3px solid ${ok ? c.ok : c.bad}`,
        }}>
          <span style={{
            fontSize: 9.5, fontWeight: 700, letterSpacing: 1.2, padding: '2px 6px',
            background: ok ? c.okSoft : c.badSoft, color: ok ? c.ok : c.bad,
            borderRadius: 4,
          }}>{ok ? 'ACCEPT' : 'REJECT'}</span>
          <span style={{ color: c.ink, fontWeight: 600, fontSize: 12 }}>{view.reason_label}</span>
          <span style={{ color: c.muted, marginLeft: 'auto', fontSize: 10.5 }}>rc={view.reason_code}</span>
        </div>
        {/* plain-language explanation */}
        <div style={{
          padding: '0 12px 10px 18px', fontSize: 11.5, color: c.ink2, lineHeight: 1.5,
        }}>
          {explain}
        </div>
        {/* live flags as friendly bullets */}
        {liveFlags.length > 0 && (
          <div style={{
            padding: '10px 12px 10px 18px', borderTop: `1px solid ${c.border}`,
            display: 'flex', flexDirection: 'column', gap: 5,
          }}>
            {liveFlags.map(f => {
              const isReq = f.startsWith('requires_');
              const color = isReq ? c.warn : c.ok;
              return (
                <div key={f} style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 11.5 }}>
                  <span style={{ width: 4, height: 4, borderRadius: 999, background: color, flexShrink: 0 }} />
                  <span style={{ color: c.ink }}>{FLAG_LABEL[f] || f}</span>
                </div>
              );
            })}
          </div>
        )}
        {/* expandable raw flags */}
        <button onClick={() => setShowRaw(r => !r)} style={{
          width: '100%', textAlign: 'left',
          padding: '6px 12px 6px 18px',
          borderTop: `1px solid ${c.border}`, background: 'transparent',
          fontFamily: 'inherit', fontSize: 10, color: c.muted, cursor: 'pointer',
          letterSpacing: 0.6, textTransform: 'uppercase', fontWeight: 600,
          display: 'flex', alignItems: 'center', gap: 6,
        }}>
          <span style={{ fontSize: 9 }}>{showRaw ? '▾' : '▸'}</span>
          raw fields
        </button>
        {showRaw && (
          <div style={{
            fontSize: 10.5, color: c.muted, padding: '0 12px 10px 18px',
            display: 'grid', gridTemplateColumns: '1fr auto', gap: '2px 8px',
          }}>
            <span style={{ color: c.ink3 }}>source</span>
            <span style={{ color: c.ink2, textAlign: 'right' }}>{view.source}</span>
            <span style={{ color: c.ink3 }}>accepted</span>
            <span style={{ color: ok ? c.ok : c.bad, textAlign: 'right', fontWeight: 600 }}>{String(ok)}</span>
            {flags.map(f => {
              const v = view[f];
              if (typeof v !== 'boolean') return null;
              return (
                <React.Fragment key={f}>
                  <span style={{ color: c.ink3 }}>{f}</span>
                  <span style={{
                    color: v ? (f.startsWith('requires_') ? c.warn : c.ok) : c.dim,
                    fontWeight: v ? 600 : 400, textAlign: 'right',
                  }}>{String(v)}</span>
                </React.Fragment>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

// ───────── bootstrap lock ─────────
function FuseBootstrapLock({ c, view, onOpenPanel }) {
  return (
    <div style={{
      flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center',
      padding: 32, position: 'relative', zIndex: 1,
    }}>
      <div style={{ maxWidth: 380, textAlign: 'center' }}>
        <div style={{
          width: 64, height: 64, borderRadius: '50%', margin: '0 auto 20px',
          position: 'relative', overflow: 'hidden',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          background: c.surface, color: c.warn,
        }}>
          <span className="iris-ring" style={{ borderRadius: '50%' }} />
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ position: 'relative' }}>
            <rect x="4" y="10" width="16" height="11" rx="2"/>
            <path d="M8 10V7a4 4 0 0 1 8 0v3"/>
          </svg>
        </div>
        <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: '-0.014em', marginBottom: 8 }}>
          {view.requires_recovery ? 'Recovery required' : 'Finishing sync'}
        </div>
        <div style={{ fontSize: 14, color: c.ink2, lineHeight: 1.55, marginBottom: 18 }}>
          Message UI stays closed until <span className="mono" style={{ color: c.ink }}>client_bootstrap</span> accepts. The gate is core's decision, not the UI's.
        </div>
        <div className="mono" style={{
          fontSize: 11, color: c.muted, marginBottom: 20,
          padding: '8px 12px', background: c.surface, border: `1px solid ${c.border}`,
          borderRadius: 10, display: 'inline-block',
        }}>{view.reason_label} · rc={view.reason_code}</div>
        <div style={{ display: 'flex', gap: 8, justifyContent: 'center' }}>
          {view.can_start_sync && (
            <button onClick={() => onOpenPanel && onOpenPanel('sync')} style={fuseBtn(c, 'irid')}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Continue sync</span>
            </button>
          )}
          {view.requires_recovery && (
            <button onClick={() => onOpenPanel && onOpenPanel('recovery')} style={fuseBtn(c, 'irid')}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Open recovery</span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

// ───────── helpers ─────────
function Spinner({ color }) {
  return (
    <span style={{
      width: 12, height: 12, borderRadius: '50%',
      border: `1.5px solid ${color}33`, borderTopColor: color,
      animation: 'fuse-iris .9s linear infinite',
      flexShrink: 0, marginTop: 2,
    }} />
  );
}

// Helix glyph — Mercury's identity mark for the inspector.
function Helix({ size = 14, color = 'currentColor' }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
      stroke={color} strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"
      style={{ flexShrink: 0 }} aria-hidden="true">
      <path d="M5 3c0 4 14 4 14 9s-14 5-14 9" />
      <path d="M19 3c0 4-14 4-14 9s14 5 14 9" />
      <path d="M7 6h10" />
      <path d="M7 18h10" />
      <path d="M9 9h6" />
      <path d="M9 15h6" />
    </svg>
  );
}

// Plain-language gloss for each reason_label, scoped by decision source.
// Used inside the inspector so the panel reads like English, not a flag dump.
const REASON_EXPLAIN = {
  client_bootstrap: {
    ACCEPTED:          'Client is ready. Message UI may open.',
    SYNC_INCOMPLETE:   'Sync is not complete yet — message UI stays closed.',
    RECOVERY_REQUIRED: 'Account or device secrets need recovery before continuing.',
  },
  outbound_send: {
    ACCEPTED:                 'Outgoing message will be sent and ciphertext persisted.',
    TOFU_PENDING:             'First-use verification needed before this send goes out.',
    KEY_STALE:                'Recipient’s key is stale; re-handshake required.',
    RECIPIENT_TRUST_REJECTED: 'Recipient’s key change is unconfirmed — send is held.',
    AI_GRANT_ABSENT:          'No AI grant exists in this room — @ai mentions are held.',
    AI_GRANT_REVOKED:         'AI grant has been revoked — @ai mentions are held.',
    AI_GRANT_EXPIRED:         'AI grant has lapsed — @ai mentions are held.',
  },
  client_receive: {
    ACCEPTED:              'Incoming message accepted and handed to the renderer.',
    ORDERING_GAP:          'Out-of-order message — retrying to fill the gap.',
    SENDER_TRUST_REJECTED: 'Sender is not trusted — plaintext withheld.',
  },
};

// Friendly names for the boolean flags. Falsy flags are hidden in the
// summary view so the panel only shows what's actually live.
const FLAG_LABEL = {
  can_open_message_ui:    'Message UI may open',
  can_start_sync:         'Background sync allowed',
  can_send:               'Sending allowed',
  can_receive:            'Receiving allowed',
  can_persist_ciphertext: 'Ciphertext will persist',
  requires_sync:          'Needs sync',
  requires_recovery:      'Needs recovery',
  requires_client_retry:  'Needs retry',
  requires_user_action:   'Needs your confirmation',
};

function fuseBtn(c, variant) {
  if (variant === 'irid') {
    return {
      position: 'relative', overflow: 'hidden',
      background: c.surface, color: c.ink, border: 'none',
      padding: '8px 18px', borderRadius: 8, fontSize: 12, fontWeight: 600,
      cursor: 'pointer', fontFamily: 'inherit',
    };
  }
  return {
    background: 'transparent', color: c.ink, border: `1px solid ${c.border}`,
    padding: '8px 16px', borderRadius: 8, fontSize: 12, fontWeight: 500,
    cursor: 'pointer', fontFamily: 'inherit',
  };
}

// (the iridescent variant relies on the .iris-ring child for its outline)

function fmtClk(ts) {
  const d = new Date(ts);
  const p = n => String(n).padStart(2, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}`;
}
function fmtClkSec(ts) {
  const d = new Date(ts);
  const p = n => String(n).padStart(2, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

// Whether two timestamps fall on the same calendar day (local time).
function sameDay(a, b) {
  if (!a || !b) return false;
  const x = new Date(a), y = new Date(b);
  return x.getFullYear() === y.getFullYear()
      && x.getMonth()    === y.getMonth()
      && x.getDate()     === y.getDate();
}

// Human-friendly day label — "Today", "Yesterday", or e.g. "Mar 15" /
// "Mar 15 2024" depending on age.
function fmtDay(ts) {
  const d = new Date(ts), now = new Date();
  const y = new Date(now); y.setDate(y.getDate() - 1);
  if (sameDay(d, now)) return 'Today';
  if (sameDay(d, y))   return 'Yesterday';
  const sameYear = d.getFullYear() === now.getFullYear();
  const m = d.toLocaleString(undefined, { month: 'short' });
  return sameYear ? `${m} ${d.getDate()}` : `${m} ${d.getDate()} ${d.getFullYear()}`;
}

// Timestamp shown next to each message. Includes a date prefix whenever the
// message wasn't from today, so older history always carries its date.
function fmtMsgTime(ts) {
  const now = Date.now();
  if (sameDay(ts, now)) return fmtClkSec(ts);
  return `${fmtDay(ts)} · ${fmtClk(ts)}`;
}

Object.assign(window, {
  FusionVariant,
  // shared with mercury-variant-fusion-panels.jsx
  fuseBtn, Avatar, authorTint, FU_TRUST, FU_AI, FUSION_DARK, FUSION_LIGHT,
  REASON_EXPLAIN, FLAG_LABEL,
});
