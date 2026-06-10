// mercury-variant-fusion-panels.jsx
// Detail panels for Mercury — trust verification, AI grant management,
// recovery, and sync remediation. Each renders as a modal/sheet via
// FusePanelShell, which adapts to mobile (bottom sheet) and desktop
// (centered modal). Triggered from the data strip chips and the bootstrap
// lock CTAs.
//
// Exports (window): FusePanelShell, FuseTrustPanel, FuseAiPanel,
//                   FuseRecoveryPanel, FuseSyncPanel

// ─────────────────────────────────────────────────────────────
// Shell — backdrop + animated panel container.
// ─────────────────────────────────────────────────────────────
function FusePanelShell({ c, open, onClose, mode, title, eyebrow, children, footer }) {
  // Esc closes any open panel.
  React.useEffect(() => {
    if (!open) return;
    const onKey = (e) => { if (e.key === 'Escape') onClose && onClose(); };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  // mobile: slide-up bottom sheet (with rounded top)
  // desktop: centered modal that scales in
  const isMobile = mode === 'mobile';
  return (
    <React.Fragment>
      <div onClick={onClose} style={{
        position: 'absolute', inset: 0, zIndex: 9,
        background: 'rgba(0,0,0,.42)',
        opacity: open ? 1 : 0,
        pointerEvents: open ? 'auto' : 'none',
        transition: 'opacity .22s ease',
      }} />
      <div style={{
        position: 'absolute', zIndex: 10,
        ...(isMobile
          ? {
              left: 0, right: 0, bottom: 0,
              maxHeight: '88%',
              borderTopLeftRadius: 18, borderTopRightRadius: 18,
              transform: open ? 'translateY(0)' : 'translateY(110%)',
            }
          : {
              top: '50%', left: '50%',
              width: 'min(480px, 92%)', maxHeight: '86%',
              borderRadius: 16,
              transform: `translate(-50%, -50%) scale(${open ? 1 : 0.96})`,
              opacity: open ? 1 : 0,
            }
        ),
        background: c.surface,
        border: `1px solid ${c.border}`,
        boxShadow: '0 24px 64px rgba(0,0,0,.32), 0 1px 0 var(--mc-tip-hi, rgba(255,255,255,.5)) inset',
        transition: 'transform .26s cubic-bezier(.2,.7,.2,1), opacity .22s ease',
        display: 'flex', flexDirection: 'column', overflow: 'hidden',
        pointerEvents: open ? 'auto' : 'none',
      }}>
        {/* mobile grab handle */}
        {isMobile && (
          <div style={{
            display: 'flex', justifyContent: 'center', paddingTop: 8, paddingBottom: 2,
          }}>
            <span style={{ width: 36, height: 4, borderRadius: 999, background: c.border }} />
          </div>
        )}
        {/* header */}
        <div style={{
          padding: isMobile ? '8px 18px 12px' : '14px 20px 12px',
          borderBottom: `1px solid ${c.border}`,
          display: 'flex', alignItems: 'flex-end', gap: 10, flexShrink: 0,
        }}>
          <div style={{ flex: 1, minWidth: 0 }}>
            {eyebrow && (
              <div style={{
                fontSize: 10, color: c.muted, letterSpacing: 1.1,
                textTransform: 'uppercase', fontWeight: 600, marginBottom: 3,
              }}>{eyebrow}</div>
            )}
            <div className="display" style={{
              fontSize: 18, fontWeight: 600, letterSpacing: '-0.012em', lineHeight: 1.2,
            }}>{title}</div>
          </div>
          <button onClick={onClose} aria-label="Close panel" data-tip="Close (Esc)"
            style={{
              background: 'transparent', border: 'none', cursor: 'pointer',
              padding: '4px 8px', color: c.muted, fontFamily: 'inherit', fontSize: 14,
              lineHeight: 1, borderRadius: 6,
            }}>✕</button>
        </div>
        {/* body */}
        <div style={{
          flex: 1, minHeight: 0, overflow: 'auto',
          padding: '14px 18px 18px',
        }}>{children}</div>
        {/* footer */}
        {footer && (
          <div style={{
            padding: '10px 18px 14px', borderTop: `1px solid ${c.border}`,
            display: 'flex', gap: 10, justifyContent: 'flex-end', flexShrink: 0,
            background: c.surface,
          }}>{footer}</div>
        )}
      </div>
    </React.Fragment>
  );
}

// ─────────────────────────────────────────────────────────────
// Trust & verification.
// ─────────────────────────────────────────────────────────────
function FuseTrustPanel({ c, mode, open, onClose, trust, onMarkVerified }) {
  const t = FU_TRUST[trust];
  const tColor = c[t.tone];
  // safety number — fixed for demo
  const SN = ['a1f3', '9c20', '8e4b', '7d12', '6b8e', '0a35',
              'c7f1', '3b29', 'e8da', '4501', '9f63', '2b77'];
  // device list per participant — fixed for demo
  const devices = [
    { who: 'rin',   name: 'Rin Hadley',   list: [
        { dev: 'a1f3', added: '12d ago', state: 'verified' },
        { dev: '8e4b', added: '4d ago',  state: 'verified' },
      ]
    },
    { who: 'jules', name: 'Jules Okafor', list: [
        { dev: 'b2e7', added: '32d ago', state: 'verified' },
        { dev: 'c4d9', added: '2h ago',  state: trust === 'unverified' ? 'unverified' : 'verified', fresh: trust === 'unverified' },
      ]
    },
    { who: 'me',    name: 'You',          list: [
        { dev: 'a1f3', added: 'this device', state: 'verified' },
      ]
    },
  ];
  const hasUnverified = devices.some(d => d.list.some(x => x.state === 'unverified'));
  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="Trust"
      title="Verify the people in this room"
      footer={
        <React.Fragment>
          <button onClick={onClose} style={fuseBtn(c, 'ghost')}>Done</button>
          {hasUnverified && (
            <button onClick={onMarkVerified} style={fuseBtn(c, 'irid')}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Mark all verified</span>
            </button>
          )}
        </React.Fragment>
      }>
      {/* status */}
      <FuseStatusRow c={c} dotColor={tColor} label={t.label} sub={t.sub} />

      {/* safety number */}
      <FuseSection c={c} kicker="Room safety number">
        <div style={{
          padding: '12px 14px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10,
        }}>
          <div style={{
            display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px 12px',
            fontSize: 13, letterSpacing: 1, color: c.ink, fontWeight: 600,
          }}>
            {SN.map((g, i) => (
              <span key={i} style={{ textAlign: 'center' }}>{g}</span>
            ))}
          </div>
          <div style={{ fontSize: 11, color: c.muted, marginTop: 10, lineHeight: 1.5 }}>
            Compare these groups in person, by voice, or by QR code. If they match on
            both sides, the conversation is verified end-to-end.
          </div>
        </div>
      </FuseSection>

      {/* devices */}
      <FuseSection c={c} kicker="Devices in this room">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {devices.map(group => {
            const p = MERCURY_PEOPLE[group.who];
            return (
              <div key={group.who} style={{
                padding: '10px 12px', background: c.surfaceWarm,
                border: `1px solid ${c.border}`, borderRadius: 10,
              }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 8 }}>
                  <Avatar c={c} p={p} isAi={p.isAi} sz={24} />
                  <span style={{ fontSize: 12.5, fontWeight: 600, color: c.ink }}>{group.name}</span>
                  <span style={{ marginLeft: 'auto', fontSize: 10.5, color: c.muted }}>
                    {group.list.length} device{group.list.length === 1 ? '' : 's'}
                  </span>
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  {group.list.map(d => (
                    <div key={d.dev} style={{
                      display: 'flex', alignItems: 'center', gap: 8,
                      padding: '6px 8px', borderRadius: 6,
                      background: c.surface,
                      fontSize: 11.5,
                    }}>
                      <span style={{
                        width: 6, height: 6, borderRadius: 999,
                        background: d.state === 'verified' ? c.ok : c.warn,
                      }} />
                      <span style={{ color: c.ink2, fontWeight: 600 }}>device {d.dev}</span>
                      <span style={{ color: c.muted, marginLeft: 'auto' }}>{d.added}</span>
                      {d.fresh && <span style={{ color: c.warn, fontWeight: 600 }}>new</span>}
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </FuseSection>

      {/* key transparency */}
      <FuseSection c={c} kicker="Key transparency">
        <div style={{
          padding: '10px 12px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10,
          fontSize: 12, color: c.ink2, lineHeight: 1.5,
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
            <span style={{ width: 6, height: 6, borderRadius: 999, background: c.ok }} />
            <span style={{ color: c.ink, fontWeight: 600 }}>All keys consistent</span>
            <span style={{ marginLeft: 'auto', color: c.muted, fontSize: 11 }}>checked 12s ago</span>
          </div>
          Mercury checks every device key against the global transparency log.
          A key change without a matching log entry will block sends until you confirm.
        </div>
      </FuseSection>
    </FusePanelShell>
  );
}

// ─────────────────────────────────────────────────────────────
// AI grant management.
// ─────────────────────────────────────────────────────────────
function FuseAiPanel({ c, mode, open, onClose, ai, onChangeAi }) {
  const granted = ai === 'granted';
  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="AI access"
      title={granted ? 'Mercury AI · scoped' : ai === 'absent' ? 'No AI access in this room' : `AI access · ${ai}`}
      footer={
        <React.Fragment>
          <button onClick={onClose} style={fuseBtn(c, 'ghost')}>Close</button>
          {granted && (
            <button onClick={() => { onChangeAi('revoked'); }} style={{
              ...fuseBtn(c, 'ghost'), color: c.bad, borderColor: c.bad + '55',
            }}>Revoke now</button>
          )}
          {!granted && (
            <button onClick={() => { onChangeAi('granted'); }} style={fuseBtn(c, 'irid')}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Request grant</span>
            </button>
          )}
        </React.Fragment>
      }>
      {/* status card */}
      <div style={{ position: 'relative', padding: 0, marginBottom: 16 }}>
        <div style={{
          position: 'relative', padding: '14px 16px',
          background: c.surface, borderRadius: 12, overflow: 'hidden',
        }}>
          {granted && <span className="iris-ring" style={{ borderRadius: 12 }} />}
          <div style={{ position: 'relative' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 8 }}>
              <Avatar c={c} p={MERCURY_PEOPLE.ai} isAi sz={28} />
              <div style={{ minWidth: 0, flex: 1 }}>
                <div style={{ fontSize: 13.5, fontWeight: 600 }}
                     className={granted ? 'iris-text' : undefined}>{granted ? 'Mercury AI' : 'AI participant'}</div>
                <div style={{ fontSize: 11, color: c.muted, marginTop: 1 }}>
                  {granted ? 'scoped to this room · 24h grant' : ai === 'absent' ? 'no grant active' : `grant ${ai}`}
                </div>
              </div>
              {granted && (
                <span style={{
                  fontSize: 10, fontWeight: 700, color: c.ok, padding: '2px 7px',
                  background: c.okSoft, borderRadius: 4, letterSpacing: 0.6, textTransform: 'uppercase',
                }}>active</span>
              )}
            </div>
            {granted && (
              <div style={{ fontSize: 11.5, color: c.ink2, lineHeight: 1.55 }}>
                AI has temporary, scoped access to decrypted content in this room only.
                Nothing is stored past expiry and no other rooms are visible to it.
              </div>
            )}
            {!granted && (
              <div style={{ fontSize: 11.5, color: c.ink2, lineHeight: 1.55 }}>
                {ai === 'absent'  && '@ai mentions in this room are held at the outbound gate. Request a grant to give Mercury AI scoped, read-only access.'}
                {ai === 'revoked' && 'A previous grant was revoked. To resume, request a new one — old context is not retained.'}
                {ai === 'expired' && 'The previous grant lapsed. Mercury AI has no current access. Request a new grant to resume.'}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* scope grid — only when granted */}
      {granted && (
        <FuseSection c={c} kicker="Scope">
          <div style={{
            display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8,
          }}>
            <FuseScopeTile c={c} label="Room"      value="mercury-core"     note="this room only" />
            <FuseScopeTile c={c} label="Mode"      value="local"            note="runs on device" />
            <FuseScopeTile c={c} label="Read"      value="allowed"          tone="ok"   note="decrypted in-room" />
            <FuseScopeTile c={c} label="Send"      value="not allowed"      tone="bad"  note="can't post messages" />
            <FuseScopeTile c={c} label="Tools"     value="summarize · reply" note="2 of 12 enabled" />
            <FuseScopeTile c={c} label="Expires"   value="23h 41m"          note="auto-revokes" />
          </div>
        </FuseSection>
      )}

      {/* history */}
      <FuseSection c={c} kicker="Grant history">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <FuseHistoryRow c={c} when="today, 09:12"   by="you"     event="grant requested · 24h scoped" tone="ok"  />
          <FuseHistoryRow c={c} when="today, 09:12"   by="binding" event="grant accepted"               tone="ok"  />
          <FuseHistoryRow c={c} when="3d ago, 14:02"  by="you"     event="previous grant revoked"       tone="bad" />
          <FuseHistoryRow c={c} when="3d ago, 09:30"  by="you"     event="grant requested · 8h scoped"  tone="muted" />
        </div>
      </FuseSection>
    </FusePanelShell>
  );
}

function FuseScopeTile({ c, label, value, note, tone }) {
  const valColor = tone === 'ok' ? c.ok : tone === 'bad' ? c.bad : c.ink;
  return (
    <div style={{
      padding: '8px 10px', background: c.surfaceWarm,
      border: `1px solid ${c.border}`, borderRadius: 8,
    }}>
      <div style={{
        fontSize: 9.5, color: c.muted, letterSpacing: 0.8,
        textTransform: 'uppercase', fontWeight: 600, marginBottom: 3,
      }}>{label}</div>
      <div style={{ fontSize: 12.5, color: valColor, fontWeight: 600 }}>{value}</div>
      {note && <div style={{ fontSize: 10.5, color: c.muted, marginTop: 1 }}>{note}</div>}
    </div>
  );
}

function FuseHistoryRow({ c, when, by, event, tone }) {
  const color = tone === 'ok' ? c.ok : tone === 'bad' ? c.bad : c.muted;
  return (
    <div style={{
      display: 'flex', alignItems: 'baseline', gap: 8,
      padding: '6px 10px', background: c.surfaceWarm,
      border: `1px solid ${c.border}`, borderRadius: 6,
      fontSize: 11.5,
    }}>
      <span style={{ width: 4, height: 4, borderRadius: 999, background: color, alignSelf: 'center', flexShrink: 0 }} />
      <span style={{ color: c.muted, fontSize: 10.5, minWidth: 90 }}>{when}</span>
      <span style={{ color: c.ink2, fontWeight: 600, minWidth: 56 }}>{by}</span>
      <span style={{ color: c.ink, flex: 1 }}>{event}</span>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Recovery flow.
// ─────────────────────────────────────────────────────────────
function FuseRecoveryPanel({ c, mode, open, onClose, onComplete }) {
  const [step, setStep] = React.useState(1);
  const [phrase, setPhrase] = React.useState('');
  const [progress, setProgress] = React.useState(0);
  React.useEffect(() => {
    if (step !== 2) return;
    setProgress(0);
    const handle = setInterval(() => {
      setProgress(p => {
        const next = p + Math.random() * 14;
        if (next >= 100) { clearInterval(handle); setTimeout(() => setStep(3), 350); return 100; }
        return next;
      });
    }, 220);
    return () => clearInterval(handle);
  }, [step]);
  // reset when reopened
  React.useEffect(() => { if (open) { setStep(1); setPhrase(''); setProgress(0); } }, [open]);

  const phraseOk = phrase.trim().split(/\s+/).filter(Boolean).length >= 6;
  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="Recovery"
      title="Recover your account"
      footer={
        step === 1 ? (
          <React.Fragment>
            <button onClick={onClose} style={fuseBtn(c, 'ghost')}>Cancel</button>
            <button onClick={() => phraseOk && setStep(2)} disabled={!phraseOk} style={{
              ...fuseBtn(c, 'irid'),
              opacity: phraseOk ? 1 : 0.5, cursor: phraseOk ? 'pointer' : 'not-allowed',
            }}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Begin recovery</span>
            </button>
          </React.Fragment>
        ) : step === 3 ? (
          <React.Fragment>
            <button onClick={() => { onComplete && onComplete(); onClose(); }} style={fuseBtn(c, 'irid')}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Open Mercury</span>
            </button>
          </React.Fragment>
        ) : null
      }>
      <FuseStepperRow c={c} step={step} steps={['Phrase', 'Restore', 'Reverify']} />

      {step === 1 && (
        <React.Fragment>
          <div style={{
            fontSize: 12.5, color: c.ink2, lineHeight: 1.55, marginBottom: 14,
          }}>
            Enter the 12-word recovery phrase you generated when you set up Mercury.
            The phrase decrypts your account key on this device. Nothing is sent off-device.
          </div>
          <textarea
            value={phrase}
            onChange={e => setPhrase(e.target.value)}
            rows={4}
            placeholder="adjective abstract balcony …"
            style={{
              width: '100%', resize: 'none', boxSizing: 'border-box',
              padding: '10px 12px', borderRadius: 10,
              background: c.surfaceWarm, color: c.ink,
              border: `1px solid ${c.border}`,
              fontFamily: 'inherit', fontSize: 13.5, lineHeight: 1.6,
              letterSpacing: 0.3, outline: 'none',
            }}
          />
          <div style={{ fontSize: 10.5, color: c.muted, marginTop: 6 }}>
            {phrase.trim().split(/\s+/).filter(Boolean).length} / 12 words
          </div>
          <div style={{
            marginTop: 14, padding: '10px 12px',
            background: c.warnSoft, border: `1px solid ${c.warn}55`, borderRadius: 10,
            fontSize: 11.5, color: c.ink2, lineHeight: 1.55,
          }}>
            <span style={{ color: c.warn, fontWeight: 600 }}>Lost the phrase?</span>{' '}
            Contact a designated recovery contact, or use a hardware recovery key.
            Mercury cannot recover an account without one of those.
          </div>
        </React.Fragment>
      )}

      {step === 2 && (
        <React.Fragment>
          <div style={{
            fontSize: 12.5, color: c.ink2, lineHeight: 1.55, marginBottom: 16,
          }}>
            Restoring your account secrets. This decrypts on-device only — no
            ciphertext leaves the room.
          </div>
          <FuseProgressBar c={c} value={progress} label="Decrypting account key" />
          <div style={{ height: 10 }} />
          <FuseProgressBar c={c} value={Math.min(100, progress * 1.1)} label="Re-establishing device keys" />
          <div style={{ height: 10 }} />
          <FuseProgressBar c={c} value={Math.min(100, progress * 0.95)} label="Verifying transparency proofs" />
        </React.Fragment>
      )}

      {step === 3 && (
        <React.Fragment>
          <div style={{
            padding: 14, background: c.okSoft,
            border: `1px solid ${c.ok}44`, borderRadius: 10, marginBottom: 14,
            color: c.ok, fontSize: 12.5, fontWeight: 600,
            display: 'flex', alignItems: 'center', gap: 8,
          }}>
            <span style={{ width: 8, height: 8, borderRadius: 999, background: c.ok }} />
            Account recovered. New device key issued.
          </div>
          <div style={{ fontSize: 12.5, color: c.ink2, lineHeight: 1.55 }}>
            Your contacts will see a key-change notice. They'll be prompted to verify
            the new device the next time you exchange messages — Mercury holds sends
            in TOFU until they confirm.
          </div>
        </React.Fragment>
      )}
    </FusePanelShell>
  );
}

function FuseProgressBar({ c, value, label }) {
  const v = Math.max(0, Math.min(100, value));
  return (
    <div>
      <div style={{
        display: 'flex', justifyContent: 'space-between',
        fontSize: 11.5, marginBottom: 4,
      }}>
        <span style={{ color: c.ink2 }}>{label}</span>
        <span style={{ color: c.muted }}>{Math.round(v)}%</span>
      </div>
      <div style={{
        height: 6, background: c.surfaceMute, borderRadius: 4, overflow: 'hidden',
        border: `1px solid ${c.border}`,
      }}>
        <div style={{
          width: `${v}%`, height: '100%',
          background: c.ok, transition: 'width .2s ease',
        }} />
      </div>
    </div>
  );
}

function FuseStepperRow({ c, step, steps }) {
  return (
    <div style={{
      display: 'flex', gap: 8, marginBottom: 16, alignItems: 'center',
    }}>
      {steps.map((s, i) => {
        const n = i + 1;
        const active = n === step;
        const done = n < step;
        return (
          <React.Fragment key={s}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <span style={{
                width: 18, height: 18, borderRadius: 999,
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                fontSize: 10, fontWeight: 700,
                background: done ? c.ok : active ? c.surface : c.surfaceWarm,
                color: done ? '#fff' : active ? c.ink : c.muted,
                border: active ? `1px solid ${c.ink2}` : `1px solid ${c.border}`,
              }}>{done ? '✓' : n}</span>
              <span style={{
                fontSize: 11.5, fontWeight: active ? 600 : 500,
                color: active ? c.ink : c.muted,
              }}>{s}</span>
            </div>
            {i < steps.length - 1 && (
              <span style={{ flex: 1, height: 1, background: c.border }} />
            )}
          </React.Fragment>
        );
      })}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Sync remediation.
// ─────────────────────────────────────────────────────────────
function FuseSyncPanel({ c, mode, open, onClose, onComplete }) {
  // animate per-room sync progress
  const [rooms, setRooms] = React.useState(() => initRooms());
  React.useEffect(() => {
    if (!open) return;
    setRooms(initRooms());
    const handle = setInterval(() => {
      setRooms(R => R.map(r => {
        if (r.value >= 100) return r;
        const inc = r.priority * (Math.random() * 6 + 1);
        return { ...r, value: Math.min(100, r.value + inc) };
      }));
    }, 280);
    return () => clearInterval(handle);
  }, [open]);
  const allDone = rooms.every(r => r.value >= 100);
  React.useEffect(() => {
    if (allDone && open) {
      const handle = setTimeout(() => { onComplete && onComplete(); }, 600);
      return () => clearTimeout(handle);
    }
  }, [allDone, open, onComplete]);

  const total = Math.round(rooms.reduce((s, r) => s + r.value, 0) / rooms.length);

  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="Sync"
      title={allDone ? 'Sync complete' : 'Finishing sync'}
      footer={
        <React.Fragment>
          <button onClick={onClose} style={fuseBtn(c, 'ghost')}>Continue in background</button>
          {allDone && (
            <button onClick={() => { onComplete && onComplete(); onClose(); }} style={fuseBtn(c, 'irid')}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Open Mercury</span>
            </button>
          )}
        </React.Fragment>
      }>
      <div style={{
        fontSize: 12.5, color: c.ink2, lineHeight: 1.55, marginBottom: 14,
      }}>
        Catching up missed messages and replay checkpoints for each room. The message
        UI stays closed until the bootstrap gate accepts — no stale timelines.
      </div>
      <FuseProgressBar c={c} value={total} label={`Total · ${rooms.length} rooms`} />
      <div style={{ height: 16 }} />
      <FuseSection c={c} kicker="Per-room progress">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {rooms.map(r => (
            <div key={r.id} style={{
              padding: '8px 12px', background: c.surfaceWarm,
              border: `1px solid ${c.border}`, borderRadius: 10,
            }}>
              <div style={{
                display: 'flex', alignItems: 'baseline', marginBottom: 6,
                fontSize: 12,
              }}>
                <span style={{ color: c.ink, fontWeight: 600 }}># {r.name}</span>
                <span style={{ color: c.muted, marginLeft: 8, fontSize: 10.5 }}>
                  {r.value >= 100 ? 'caught up' : `${Math.round(r.value)}% · ${r.note}`}
                </span>
                <span style={{ marginLeft: 'auto', fontSize: 10.5, color: r.value >= 100 ? c.ok : c.muted }}>
                  {r.value >= 100 ? '✓' : ''}
                </span>
              </div>
              <div style={{
                height: 4, background: c.surfaceMute, borderRadius: 3, overflow: 'hidden',
              }}>
                <div style={{
                  width: `${r.value}%`, height: '100%',
                  background: r.value >= 100 ? c.ok : c.warn,
                  transition: 'width .25s ease',
                }} />
              </div>
            </div>
          ))}
        </div>
      </FuseSection>
    </FusePanelShell>
  );
}

function initRooms() {
  return [
    { id: 'core', name: 'mercury-core',    value: 12, priority: 1.4, note: '8 messages behind' },
    { id: 'sec',  name: 'security-review', value: 4,  priority: 1.1, note: '23 messages behind' },
    { id: 'eng',  name: 'eng-leads',       value: 0,  priority: 0.9, note: '142 messages behind' },
    { id: 'ai',   name: 'ai-policy',       value: 32, priority: 1.6, note: '2 messages behind' },
  ];
}

// ─────────────────────────────────────────────────────────────
// Shared bits.
// ─────────────────────────────────────────────────────────────
function FuseSection({ c, kicker, children }) {
  return (
    <div style={{ marginBottom: 16 }}>
      <div style={{
        fontSize: 9.5, color: c.muted, letterSpacing: 1.1,
        textTransform: 'uppercase', fontWeight: 600, marginBottom: 8,
      }}>{kicker}</div>
      {children}
    </div>
  );
}

function FuseStatusRow({ c, dotColor, label, sub }) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 10, marginBottom: 18,
      padding: '10px 12px', background: c.surfaceWarm,
      border: `1px solid ${c.border}`, borderRadius: 10,
    }}>
      <span style={{
        width: 8, height: 8, borderRadius: 999, background: dotColor, flexShrink: 0,
      }} />
      <span style={{ fontSize: 13, fontWeight: 600, color: c.ink }}>{label}</span>
      <span style={{ fontSize: 11.5, color: c.muted }}>· {sub}</span>
    </div>
  );
}

Object.assign(window, {
  FusePanelShell, FuseTrustPanel, FuseAiPanel,
  FuseRecoveryPanel, FuseSyncPanel,
  FusePeersPanel, FuseEncryptionPanel, FuseBootstrapPanel,
  FuseProfilePanel, FuseNotificationsPanel, FuseSettingsPanel,
  FuseSegment,
});

// Segmented control — a row of options with the active one wearing the
// iridescent outline. Used for theme choice (light/dark/auto), but reusable.
function FuseSegment({ c, value, options, onChange }) {
  return (
    <div role="radiogroup"
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${options.length}, 1fr)`,
        gap: 6,
      }}>
      {options.map(opt => {
        const active = opt.value === value;
        return (
          <button key={opt.value}
            role="radio" aria-checked={active}
            onClick={() => onChange && onChange(opt.value)}
            style={{
              position: 'relative', overflow: 'hidden',
              padding: '10px 8px', background: c.surfaceWarm,
              border: active ? 'none' : `1px solid ${c.border}`,
              borderRadius: 10, cursor: 'pointer',
              fontFamily: 'inherit', color: 'inherit',
              textAlign: 'center',
              transition: 'background .15s ease',
            }}>
            {active && <span className="iris-ring" style={{ borderRadius: 10 }} />}
            <div style={{
              position: 'relative',
              fontSize: 12, fontWeight: active ? 700 : 500,
              color: active ? c.ink : c.ink2, marginBottom: opt.sub ? 2 : 0,
            }}>{opt.label}</div>
            {opt.sub && (
              <div style={{
                position: 'relative', fontSize: 10, color: c.muted,
              }}>{opt.sub}</div>
            )}
          </button>
        );
      })}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Settings hub — account-level, not room-specific. Acts as a
// navigation surface to the existing detail panels plus a couple
// of preference toggles.
// ─────────────────────────────────────────────────────────────
function FuseSettingsPanel({ c, mode, open, onClose, dark, themeChoice, onSetTheme,
                             onOpenProfile, onOpenNotifications, onOpenRecovery,
                             onOpenTrust, onOpenEncryption }) {
  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="Mercury"
      title="Settings"
      footer={<button onClick={onClose} style={fuseBtn(c, 'ghost')}>Done</button>}>

      {/* identity strip */}
      <button onClick={() => { onClose(); setTimeout(() => onOpenProfile && onOpenProfile('me'), 50); }}
        style={{
          width: '100%', textAlign: 'left',
          padding: '12px 14px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 12,
          cursor: 'pointer', fontFamily: 'inherit', color: 'inherit',
          display: 'flex', alignItems: 'center', gap: 12, marginBottom: 18,
        }}>
        <Avatar c={c} p={MERCURY_PEOPLE.me} sz={40} />
        <div style={{ minWidth: 0, flex: 1 }}>
          <div style={{ fontSize: 14, fontWeight: 700, color: c.ink, marginBottom: 1 }}>You</div>
          <div style={{ fontSize: 11, color: c.muted }}>device a1f3 · 3 devices · trusted</div>
        </div>
        <span style={{ color: c.muted, fontSize: 14 }}>›</span>
      </button>

      {/* appearance */}
      <FuseSection c={c} kicker="Appearance">
        <FuseSegment c={c}
          value={themeChoice || (dark ? 'dark' : 'light')}
          onChange={(v) => onSetTheme && onSetTheme(v)}
          options={[
            { value: 'light', label: 'Light' },
            { value: 'dark',  label: 'Dark'  },
            { value: 'auto',  label: 'Auto', sub: `device · ${dark ? 'dark' : 'light'}` },
          ]} />
      </FuseSection>

      {/* security */}
      <FuseSection c={c} kicker="Security">
        <FuseSettingsRow c={c} label="Verify devices"
          sub="Compare safety numbers and review the device list."
          onClick={() => { onClose(); setTimeout(() => onOpenTrust && onOpenTrust(), 50); }} />
        <FuseSettingsRow c={c} label="Encryption"
          sub="What protects this room, and what doesn't."
          divider
          onClick={() => { onClose(); setTimeout(() => onOpenEncryption && onOpenEncryption(), 50); }} />
        <FuseSettingsRow c={c} label="Recovery"
          sub="Recovery phrase · designated contacts."
          divider
          onClick={() => { onClose(); setTimeout(() => onOpenRecovery && onOpenRecovery(), 50); }} />
      </FuseSection>

      {/* notifications */}
      <FuseSection c={c} kicker="Notifications">
        <FuseSettingsRow c={c} label="Lock-screen previews"
          sub="Previews · sound · AI alerts."
          onClick={() => { onClose(); setTimeout(() => onOpenNotifications && onOpenNotifications(), 50); }} />
      </FuseSection>

      {/* about */}
      <FuseSection c={c} kicker="About">
        <div style={{
          padding: '10px 12px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10,
          fontSize: 11.5, color: c.ink2, lineHeight: 1.6,
        }}>
          <FuseKv c={c} k="Build"    v="mercury v0.30 · client 2026.05.28" />
          <FuseKv c={c} k="Core"     v="mercury-core 0.30 · binding 0.30" />
          <FuseKv c={c} k="Audit"    v="Key Transparency log · live" />
          <FuseKv c={c} k="Region"   v="eu-west · scoped to org" />
        </div>
      </FuseSection>
    </FusePanelShell>
  );
}

// Row used inside the Settings hub. Like a list item with a chevron.
function FuseSettingsRow({ c, label, sub, onClick, divider }) {
  return (
    <button onClick={onClick} style={{
      width: '100%', textAlign: 'left',
      padding: '10px 12px', background: c.surfaceWarm,
      border: `1px solid ${c.border}`,
      borderRadius: 10, marginTop: divider ? 6 : 0,
      cursor: 'pointer', fontFamily: 'inherit', color: 'inherit',
      display: 'flex', alignItems: 'center', gap: 10,
    }}>
      <div style={{ minWidth: 0, flex: 1 }}>
        <div style={{ fontSize: 12.5, fontWeight: 600, color: c.ink, marginBottom: 1 }}>{label}</div>
        <div style={{ fontSize: 11, color: c.muted, lineHeight: 1.45 }}>{sub}</div>
      </div>
      <span style={{ color: c.muted, fontSize: 14 }}>›</span>
    </button>
  );
}

// ─────────────────────────────────────────────────────────────
// Notification preview discipline. Demonstrates that lock-screen
// previews respect the gate: when content can't be safely shown
// (TOFU pending, sender trust rejected), notifications fall back
// to a generic "New message" line and never leak plaintext.
// ─────────────────────────────────────────────────────────────
function FuseNotificationsPanel({ c, mode, open, onClose }) {
  const [previews, setPreviews] = React.useState(true);
  const [sound, setSound]       = React.useState(true);
  const [aiAlerts, setAiAlerts] = React.useState(false);

  // sample lockscreen notifications — each carries a "kind" that determines
  // whether the preview is allowed.
  const samples = [
    {
      kind: 'accepted',
      who: 'Rin Hadley · mercury-core',
      preview: 'Pushed the receive-gate rewrite. Ordering check is now in core, not the binding.',
      tone: 'ok', when: 'now',
    },
    {
      kind: 'tofu',
      who: 'Jules Okafor · mercury-core',
      preview: '[withheld] first-use verification needed before preview',
      tone: 'warn', when: '2m',
    },
    {
      kind: 'rejected',
      who: 'Unknown · mercury-core',
      preview: '[withheld] sender trust rejected',
      tone: 'bad', when: '5m',
    },
  ];

  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="Notifications"
      title="Lock-screen previews"
      footer={<button onClick={onClose} style={fuseBtn(c, 'ghost')}>Done</button>}>

      <FuseSection c={c} kicker="Settings">
        <div style={{
          padding: 0, background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10, overflow: 'hidden',
        }}>
          <FuseToggleRow c={c} label="Show message previews"
            sub="Off shows only the room name. Sensitive content is always withheld regardless."
            value={previews} onChange={setPreviews} />
          <FuseToggleRow c={c} label="Sound"
            sub="Audible alert when a new message arrives."
            value={sound} onChange={setSound} divider />
          <FuseToggleRow c={c} label="AI activity alerts"
            sub="Notify when Mercury AI is mentioned, replies, or a grant changes."
            value={aiAlerts} onChange={setAiAlerts} divider />
        </div>
      </FuseSection>

      <FuseSection c={c} kicker="Preview · this is what the lock screen will show">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {samples.map((s, i) => (
            <FuseLockNotification key={i} c={c} s={s} previews={previews} />
          ))}
        </div>
        <div style={{
          fontSize: 11, color: c.muted, marginTop: 10, lineHeight: 1.5,
        }}>
          <span style={{ color: c.ok }}>●</span> accepted &nbsp;·&nbsp;
          <span style={{ color: c.warn }}>●</span> tofu pending &nbsp;·&nbsp;
          <span style={{ color: c.bad }}>●</span> sender rejected
        </div>
      </FuseSection>

      <FuseSection c={c} kicker="How this works">
        <div style={{
          padding: '10px 12px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10,
          fontSize: 11.5, color: c.ink2, lineHeight: 1.55,
        }}>
          Mercury composes the notification text after the gate decision.
          Plaintext never reaches the OS notification surface for a message
          that the binding hasn't accepted as displayable — even if previews
          are enabled.
        </div>
      </FuseSection>
    </FusePanelShell>
  );
}

function FuseToggleRow({ c, label, sub, value, onChange, divider }) {
  return (
    <button
      onClick={() => onChange(!value)}
      role="switch" aria-checked={value} aria-label={label}
      style={{
        display: 'flex', alignItems: 'center', gap: 12,
        padding: '11px 12px', background: 'transparent',
        border: 'none', borderTop: divider ? `1px solid ${c.border}` : 'none',
        cursor: 'pointer', width: '100%', textAlign: 'left',
        fontFamily: 'inherit', color: 'inherit',
      }}>
      <div style={{ minWidth: 0, flex: 1 }}>
        <div style={{ fontSize: 12.5, fontWeight: 600, color: c.ink, marginBottom: 1 }}>{label}</div>
        {sub && <div style={{ fontSize: 11, color: c.muted, lineHeight: 1.5 }}>{sub}</div>}
      </div>
      <span style={{
        width: 32, height: 18, borderRadius: 999, position: 'relative',
        background: value ? c.ok : c.surfaceMute,
        border: `1px solid ${value ? c.ok : c.borderStrong}`,
        transition: 'background .15s ease, border-color .15s ease',
        flexShrink: 0,
      }}>
        <span style={{
          position: 'absolute', top: 1, left: value ? 15 : 1,
          width: 14, height: 14, borderRadius: '50%',
          background: '#fff', boxShadow: '0 1px 2px rgba(0,0,0,.2)',
          transition: 'left .15s ease',
        }} />
      </span>
    </button>
  );
}

function FuseLockNotification({ c, s, previews }) {
  const dot = s.tone === 'ok' ? c.ok : s.tone === 'warn' ? c.warn : c.bad;
  // Decide what the OS will show. If previews are off, hide content
  // entirely. If on but the message is held at the gate, fall back to
  // "[withheld] reason" — never the raw plaintext.
  const showsContent = previews;
  const isHeld = s.kind !== 'accepted';
  return (
    <div style={{
      padding: '10px 12px', background: c.surfaceWarm,
      border: `1px solid ${c.border}`, borderRadius: 10,
      display: 'flex', gap: 10,
    }}>
      <div style={{
        width: 24, height: 24, borderRadius: 6,
        background: c.surfaceMute,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        flexShrink: 0, position: 'relative', overflow: 'hidden',
      }}>
        <span className="iris-ring" style={{ borderRadius: 6 }} />
        <span style={{
          width: 6, height: 6, borderRadius: 999, background: dot, position: 'relative',
        }} />
      </div>
      <div style={{ minWidth: 0, flex: 1 }}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
          <span style={{ fontSize: 11.5, fontWeight: 600, color: c.ink }}>Mercury</span>
          <span style={{ fontSize: 10.5, color: c.muted }}>· {s.who}</span>
          <span style={{ marginLeft: 'auto', fontSize: 10.5, color: c.muted }}>{s.when}</span>
        </div>
        <div style={{
          fontSize: 12, color: isHeld ? c.muted : c.ink2,
          marginTop: 2, lineHeight: 1.4,
          fontStyle: isHeld ? 'italic' : 'normal',
        }}>
          {showsContent ? s.preview : 'New message'}
        </div>
      </div>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Profile — adapts to self (You), a person (Rin/Jules), or AI.
// Triggered from rail "You" card, peers list, inspector participants.
// ─────────────────────────────────────────────────────────────
function FuseProfilePanel({ c, mode, open, onClose, who, trust, ai, onChangeAi, onOpenTrust, onOpenAi, onOpenNotifications }) {
  if (!who) return null;
  const p = MERCURY_PEOPLE[who];
  if (!p) return null;
  const isMe = who === 'me';
  const isAi = p.isAi;
  const tint = isAi ? null : authorTint(c, p);

  // status line per kind
  let statusTone = c.ok;
  let statusLabel = 'Verified';
  let statusSub   = 'safety number matched';
  if (isMe) {
    statusLabel = 'This device';   statusSub = `device a1f3 · trusted`;
  } else if (isAi) {
    if (ai === 'granted') { statusLabel = 'Scoped grant active'; statusSub = '24h · read-only'; }
    else if (ai === 'revoked') { statusLabel = 'Access revoked'; statusSub = 'no current grant';  statusTone = c.bad; }
    else if (ai === 'expired') { statusLabel = 'Grant expired';  statusSub = 'request a new one'; statusTone = c.muted; }
    else                        { statusLabel = 'No grant';      statusSub = '@ai mentions held'; statusTone = c.muted; }
  } else {
    // a peer
    if (who === 'jules' && trust === 'unverified') {
      statusLabel = 'New device'; statusSub = 'first-use verification pending'; statusTone = c.warn;
    }
  }

  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow={isMe ? 'Your profile' : isAi ? 'AI participant' : 'Profile'}
      title={isMe ? 'You' : p.name}
      footer={
        <React.Fragment>
          <button onClick={onClose} style={fuseBtn(c, 'ghost')}>Close</button>
          {isMe && (
            <button onClick={onClose} style={{
              ...fuseBtn(c, 'ghost'), color: c.bad, borderColor: c.bad + '55',
            }}>Sign out this device</button>
          )}
          {!isMe && !isAi && (
            <button onClick={() => { onClose(); onOpenTrust && onOpenTrust(); }} style={fuseBtn(c, 'irid')}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Verify in person</span>
            </button>
          )}
          {isAi && (
            <button onClick={() => { onClose(); onOpenAi && onOpenAi(); }} style={fuseBtn(c, 'irid')}>
              <span className="iris-ring" style={{ borderRadius: 8 }} />
              <span style={{ position: 'relative' }}>Manage grant</span>
            </button>
          )}
        </React.Fragment>
      }>

      {/* hero — big avatar + name + status */}
      <div style={{
        display: 'flex', gap: 14, alignItems: 'center', marginBottom: 18,
        padding: '12px 14px', background: c.surfaceWarm,
        border: `1px solid ${c.border}`, borderRadius: 12,
        position: 'relative', overflow: 'hidden',
      }}>
        {isAi && <span className="iris-ring" style={{ borderRadius: 12 }} />}
        <Avatar c={c} p={p} isAi={isAi} sz={56} tint={tint} />
        <div style={{ minWidth: 0, flex: 1, position: 'relative' }}>
          <div className="display" style={{
            fontSize: 17, fontWeight: 700, letterSpacing: '-0.014em',
            color: isAi ? undefined : c.ink, marginBottom: 2,
          }} {...(isAi ? { className: 'iris-text display' } : {})}>
            {isMe ? 'You' : p.name}
          </div>
          <div style={{
            display: 'flex', alignItems: 'center', gap: 6,
            fontSize: 11.5, color: c.ink2,
          }}>
            <span style={{ width: 6, height: 6, borderRadius: 999, background: statusTone }} />
            <span style={{ fontWeight: 600 }}>{statusLabel}</span>
            <span style={{ color: c.muted }}>· {statusSub}</span>
          </div>
        </div>
      </div>

      {/* SELF: devices */}
      {isMe && (
        <React.Fragment>
          <FuseSection c={c} kicker="Your devices">
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <FuseDeviceRow c={c} name="iPhone · this device" id="a1f3" added="this device" tone="ok" current />
              <FuseDeviceRow c={c} name="MacBook"               id="8e4b" added="12d ago" tone="ok"   action="Sign out" />
              <FuseDeviceRow c={c} name="iPad (Air)"            id="3b29" added="32d ago" tone="ok"   action="Sign out" />
            </div>
          </FuseSection>

          <FuseSection c={c} kicker="Identity">
            <div style={{
              padding: '10px 12px', background: c.surfaceWarm,
              border: `1px solid ${c.border}`, borderRadius: 10,
              fontSize: 11.5, color: c.ink2, lineHeight: 1.6,
            }}>
              <FuseKv c={c} k="Display"   v="You" />
              <FuseKv c={c} k="Joined"    v="32 days ago" />
              <FuseKv c={c} k="Pubkey"    v="a1f3 · 9c20 · 8e4b" />
              <FuseKv c={c} k="Rooms"     v="4 active" />
            </div>
          </FuseSection>

          <FuseSection c={c} kicker="Recovery">
            <div style={{
              padding: '12px 14px', background: c.surfaceWarm,
              border: `1px solid ${c.border}`, borderRadius: 10,
              fontSize: 12, color: c.ink2, lineHeight: 1.55,
            }}>
              <div style={{ marginBottom: 8 }}>
                <span style={{ color: c.ok, fontWeight: 600 }}>● Recovery phrase set</span>
                <span style={{ color: c.muted, marginLeft: 8, fontSize: 11 }}>last verified 12d ago</span>
              </div>
              <div style={{ marginBottom: 8 }}>
                <span style={{ color: c.warn, fontWeight: 600 }}>▲ 1 of 2 recovery contacts</span>
                <span style={{ color: c.muted, marginLeft: 8, fontSize: 11 }}>add a second to enable social recovery</span>
              </div>
              <button style={{ ...fuseBtn(c, 'ghost'), fontSize: 11.5, padding: '6px 12px', marginTop: 4 }}>
                Manage recovery
              </button>
            </div>
          </FuseSection>

          <FuseSection c={c} kicker="Notifications">
            <button
              onClick={() => { onClose && onClose(); setTimeout(() => onOpenNotifications && onOpenNotifications(), 50); }}
              style={{
                width: '100%', textAlign: 'left',
                padding: '10px 12px', background: c.surfaceWarm,
                border: `1px solid ${c.border}`, borderRadius: 10,
                cursor: 'pointer', fontFamily: 'inherit', color: 'inherit',
                display: 'flex', alignItems: 'center', gap: 10,
              }}>
              <div style={{ minWidth: 0, flex: 1 }}>
                <div style={{ fontSize: 12.5, fontWeight: 600, color: c.ink, marginBottom: 1 }}>
                  Lock-screen previews
                </div>
                <div style={{ fontSize: 11, color: c.muted }}>
                  Manage previews · sound · AI alerts
                </div>
              </div>
              <span style={{ color: c.muted, fontSize: 14 }}>›</span>
            </button>
          </FuseSection>
        </React.Fragment>
      )}

      {/* PEER (non-AI): devices, shared rooms */}
      {!isMe && !isAi && (
        <React.Fragment>
          <FuseSection c={c} kicker="Devices">
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <FuseDeviceRow c={c} name="Phone"   id="b2e7" added="32d ago" tone="ok" />
              <FuseDeviceRow c={c} name="Desktop" id="c4d9" added="2h ago"
                tone={trust === 'unverified' ? 'warn' : 'ok'}
                badge={trust === 'unverified' ? 'new' : undefined} />
            </div>
          </FuseSection>

          <FuseSection c={c} kicker="Shared rooms">
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <FuseSharedRoom c={c} name="mercury-core"     last="just now" />
              <FuseSharedRoom c={c} name="security-review"  last="12m ago"  />
              <FuseSharedRoom c={c} name="eng-leads"        last="1h ago"   />
            </div>
          </FuseSection>

          <FuseSection c={c} kicker="Activity">
            <div style={{
              padding: '10px 12px', background: c.surfaceWarm,
              border: `1px solid ${c.border}`, borderRadius: 10,
              fontSize: 11.5, color: c.ink2, lineHeight: 1.6,
            }}>
              <FuseKv c={c} k="Joined"   v="32 days ago" />
              <FuseKv c={c} k="Last seen" v="2 minutes ago" />
              <FuseKv c={c} k="Pubkey"   v={`${p.short.toLowerCase()}3 · ${p.short.toLowerCase()}9 · b2e7`} />
            </div>
          </FuseSection>
        </React.Fragment>
      )}

      {/* AI */}
      {isAi && (
        <React.Fragment>
          <FuseSection c={c} kicker="What this is">
            <div style={{
              padding: '10px 12px', background: c.surfaceWarm,
              border: `1px solid ${c.border}`, borderRadius: 10,
              fontSize: 12, color: c.ink2, lineHeight: 1.55,
            }}>
              Mercury AI is a participant that joins specific rooms under a scoped, time-bound grant.
              It reads decrypted content in-room only, never writes to durable storage, and cannot send
              unless the grant explicitly allows it.
            </div>
          </FuseSection>

          {ai === 'granted' && (
            <FuseSection c={c} kicker="Current scope">
              <div style={{
                display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8,
              }}>
                <FuseScopeTile c={c} label="Room"    value="mercury-core" note="this room only" />
                <FuseScopeTile c={c} label="Mode"    value="local"        note="runs on device" />
                <FuseScopeTile c={c} label="Read"    value="allowed"      tone="ok"  note="decrypted in-room" />
                <FuseScopeTile c={c} label="Send"    value="not allowed"  tone="bad" note="can't post messages" />
                <FuseScopeTile c={c} label="Tools"   value="summarize · reply" note="2 of 12 enabled" />
                <FuseScopeTile c={c} label="Expires" value="23h 41m"      note="auto-revokes" />
              </div>
            </FuseSection>
          )}
        </React.Fragment>
      )}
    </FusePanelShell>
  );
}

function FuseDeviceRow({ c, name, id, added, tone, badge, current, action }) {
  const dot = tone === 'ok' ? c.ok : tone === 'warn' ? c.warn : c.bad;
  return (
    <div style={{
      padding: '8px 10px', background: c.surfaceWarm,
      border: `1px solid ${c.border}`, borderRadius: 8,
      display: 'flex', alignItems: 'center', gap: 8, fontSize: 12,
    }}>
      <span style={{ width: 6, height: 6, borderRadius: 999, background: dot, flexShrink: 0 }} />
      <div style={{ minWidth: 0, flex: 1 }}>
        <div style={{ color: c.ink, fontWeight: 600, fontSize: 12 }}>
          {name}
          {badge && <span style={{
            marginLeft: 8, fontSize: 9.5, color: c.warn, fontWeight: 700,
            letterSpacing: 0.6, textTransform: 'uppercase',
          }}>{badge}</span>}
          {current && <span style={{
            marginLeft: 8, fontSize: 9.5, color: c.ok, fontWeight: 700,
            letterSpacing: 0.6, textTransform: 'uppercase',
          }}>current</span>}
        </div>
        <div style={{ color: c.muted, fontSize: 10.5, marginTop: 1 }}>device {id} · {added}</div>
      </div>
      {action && !current && (
        <button style={{ ...fuseBtn(c, 'ghost'), fontSize: 10.5, padding: '4px 9px', color: c.muted }}>
          {action}
        </button>
      )}
    </div>
  );
}

function FuseSharedRoom({ c, name, last }) {
  return (
    <div style={{
      padding: '7px 10px', background: c.surfaceWarm,
      border: `1px solid ${c.border}`, borderRadius: 8,
      display: 'flex', alignItems: 'center', gap: 8, fontSize: 12,
    }}>
      <span style={{ color: c.muted, fontSize: 10.5 }}>#</span>
      <span style={{ color: c.ink, fontWeight: 600 }}>{name}</span>
      <span style={{ marginLeft: 'auto', color: c.muted, fontSize: 10.5 }}>last · {last}</span>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Peers — list of participants with avatars + device counts.
// ─────────────────────────────────────────────────────────────
function FusePeersPanel({ c, mode, open, onClose, trust, ai, onOpenTrust, onOpenProfile }) {
  const peers = [
    { id: 'me',    devices: 1, role: 'admin · this device',         tone: 'ok',   added: 'today' },
    { id: 'rin',   devices: 2, role: 'verified · iPhone + MacBook', tone: 'ok',   added: '12d ago' },
    { id: 'jules', devices: 2, role: trust === 'unverified' ? 'verified · 1 device pending' : 'verified · 2 devices',
                   tone: trust === 'unverified' ? 'warn' : 'ok',     added: '32d ago' },
    { id: 'ai',    devices: 0, role: ai === 'granted' ? 'AI · scoped read-only' : `AI · ${ai}`,
                   tone: ai === 'granted' ? 'irid' : 'muted',        added: 'today' },
  ];
  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="Participants"
      title="People in mercury-core"
      footer={
        <React.Fragment>
          <button onClick={onClose} style={fuseBtn(c, 'ghost')}>Close</button>
          <button onClick={() => { onClose(); onOpenTrust && onOpenTrust(); }} style={fuseBtn(c, 'irid')}>
            <span className="iris-ring" style={{ borderRadius: 8 }} />
            <span style={{ position: 'relative' }}>Verify devices</span>
          </button>
        </React.Fragment>
      }>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {peers.map(pe => {
          const p = MERCURY_PEOPLE[pe.id];
          const isAi = p.isAi;
          const toneColor = pe.tone === 'ok'    ? c.ok
                         : pe.tone === 'warn'  ? c.warn
                         : pe.tone === 'irid'  ? c.ai
                         : c.muted;
          return (
            <button key={pe.id}
              onClick={() => onOpenProfile && onOpenProfile(pe.id)}
              style={{
                position: 'relative',
                padding: '10px 12px', background: c.surfaceWarm,
                border: `1px solid ${c.border}`, borderRadius: 10,
                display: 'flex', alignItems: 'center', gap: 12,
                overflow: 'hidden', width: '100%',
                cursor: 'pointer', fontFamily: 'inherit', color: 'inherit',
                textAlign: 'left',
                transition: 'background .15s ease',
              }}>
              {pe.tone === 'irid' && <span className="iris-ring" style={{ borderRadius: 10 }} />}
              <Avatar c={c} p={p} isAi={isAi} sz={32} />
              <div style={{ minWidth: 0, flex: 1, position: 'relative' }}>
                <div style={{ fontSize: 13, fontWeight: 600, color: c.ink }}
                     className={isAi ? 'iris-text' : undefined}>{p.name}</div>
                <div style={{ fontSize: 11, color: c.muted, marginTop: 1 }}>{pe.role}</div>
              </div>
              <div style={{ textAlign: 'right', position: 'relative' }}>
                <div style={{ fontSize: 11, color: toneColor, fontWeight: 600 }}>
                  {pe.devices === 0 ? '—' : `${pe.devices} device${pe.devices > 1 ? 's' : ''}`}
                </div>
                <div style={{ fontSize: 10, color: c.muted, marginTop: 1 }}>added {pe.added}</div>
              </div>
              <span style={{ color: c.muted, fontSize: 12, position: 'relative', marginLeft: 4 }}>›</span>
            </button>
          );
        })}
      </div>
    </FusePanelShell>
  );
}

// ─────────────────────────────────────────────────────────────
// Encryption — explainer + algorithm spec + room fingerprint.
// ─────────────────────────────────────────────────────────────
function FuseEncryptionPanel({ c, mode, open, onClose }) {
  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="Encryption"
      title="End-to-end · scoped to this room"
      footer={<button onClick={onClose} style={fuseBtn(c, 'ghost')}>Close</button>}>

      {/* hero status */}
      <div style={{
        padding: '12px 14px', marginBottom: 16,
        background: c.okSoft, border: `1px solid ${c.ok}44`, borderRadius: 10,
        display: 'flex', alignItems: 'center', gap: 10,
      }}>
        <span style={{ width: 8, height: 8, borderRadius: 999, background: c.ok, flexShrink: 0 }} />
        <div style={{ flex: 1, fontSize: 12.5, color: c.ink2, lineHeight: 1.55 }}>
          Messages are encrypted on your device. The server stores only ciphertext.
          Mercury never sees the plaintext of any message in this room.
        </div>
      </div>

      <FuseSection c={c} kicker="What this means">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <FuseBullet c={c} tone="ok"   text="Only the people in this room can read what's said here." />
          <FuseBullet c={c} tone="ok"   text="The server can't read messages, even with a subpoena." />
          <FuseBullet c={c} tone="ok"   text="If a device key changes, the next send is held until you verify." />
          <FuseBullet c={c} tone="muted" text="Metadata (room id, peer list, timestamps) is still server-visible." />
        </div>
      </FuseSection>

      <FuseSection c={c} kicker="Algorithm">
        <div style={{
          padding: '10px 12px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10,
          fontSize: 11.5, color: c.ink2, lineHeight: 1.6,
        }}>
          <FuseKv c={c} k="Cipher"   v="XChaCha20-Poly1305" />
          <FuseKv c={c} k="Ratchet"  v="Double Ratchet · per-message forward secrecy" />
          <FuseKv c={c} k="Identity" v="Ed25519 long-term · X25519 ephemeral" />
          <FuseKv c={c} k="Group"    v="Sender keys + per-recipient envelope" />
          <FuseKv c={c} k="Audit"    v="Key Transparency log · binding-enforced" />
        </div>
      </FuseSection>

      <FuseSection c={c} kicker="Room fingerprint">
        <div style={{
          padding: '12px 14px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10,
          fontFamily: 'inherit', fontSize: 13, color: c.ink, letterSpacing: 0.8,
          textAlign: 'center', fontWeight: 600,
        }}>a1f3 · 9c20 · 8e4b · 7d12 · 6b8e · 0a35</div>
        <div style={{ fontSize: 10.5, color: c.muted, marginTop: 6 }}>
          Open Trust to compare the full safety number with your correspondents.
        </div>
      </FuseSection>
    </FusePanelShell>
  );
}

function FuseBullet({ c, tone, text }) {
  const color = tone === 'ok' ? c.ok : tone === 'warn' ? c.warn : tone === 'bad' ? c.bad : c.muted;
  return (
    <div style={{ display: 'flex', alignItems: 'flex-start', gap: 8, fontSize: 12, lineHeight: 1.5 }}>
      <span style={{ width: 4, height: 4, borderRadius: 999, background: color, flexShrink: 0, marginTop: 7 }} />
      <span style={{ color: c.ink }}>{text}</span>
    </div>
  );
}

function FuseKv({ c, k, v }) {
  return (
    <div style={{ display: 'flex', gap: 10, padding: '3px 0' }}>
      <span style={{ color: c.muted, minWidth: 72 }}>{k}</span>
      <span style={{ color: c.ink, flex: 1 }}>{v}</span>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Bootstrap status detail.
// ─────────────────────────────────────────────────────────────
function FuseBootstrapPanel({ c, mode, open, onClose, view }) {
  const explain = (REASON_EXPLAIN.client_bootstrap || {})[view.reason_label]
    || 'Decision from the bootstrap gate.';
  const ok = view.accepted;
  const flags = ['can_open_message_ui','can_start_sync','requires_sync','requires_recovery'];
  return (
    <FusePanelShell
      c={c} mode={mode} open={open} onClose={onClose}
      eyebrow="Bootstrap"
      title="Client bootstrap · gate decision"
      footer={
        <React.Fragment>
          <button onClick={onClose} style={fuseBtn(c, 'ghost')}>Close</button>
          <button onClick={onClose} style={fuseBtn(c, 'irid')}>
            <span className="iris-ring" style={{ borderRadius: 8 }} />
            <span style={{ position: 'relative' }}>Re-run bootstrap</span>
          </button>
        </React.Fragment>
      }>

      {/* status */}
      <div style={{
        padding: '12px 14px', marginBottom: 14,
        background: ok ? c.okSoft : c.badSoft,
        border: `1px solid ${ok ? c.ok + '44' : c.bad + '44'}`,
        borderRadius: 10,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
          <span style={{
            width: 8, height: 8, borderRadius: 999, background: ok ? c.ok : c.bad,
          }} />
          <span style={{ fontWeight: 700, fontSize: 11, letterSpacing: 1, textTransform: 'uppercase',
                          color: ok ? c.ok : c.bad }}>{ok ? 'Accepted' : 'Rejected'}</span>
          <span style={{ marginLeft: 'auto', fontSize: 10.5, color: c.muted }}>
            rc={view.reason_code}
          </span>
        </div>
        <div style={{ fontSize: 13, color: c.ink, fontWeight: 600, marginBottom: 2 }}>
          {view.reason_label}
        </div>
        <div style={{ fontSize: 12, color: c.ink2, lineHeight: 1.55 }}>{explain}</div>
      </div>

      <FuseSection c={c} kicker="Effects">
        <div style={{
          padding: '10px 12px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10,
        }}>
          {flags.map(f => {
            const v = view[f];
            if (typeof v !== 'boolean') return null;
            const color = v ? (f.startsWith('requires_') ? c.warn : c.ok) : c.dim;
            return (
              <div key={f} style={{
                display: 'flex', alignItems: 'center', gap: 8,
                fontSize: 11.5, padding: '3px 0',
              }}>
                <span style={{ width: 5, height: 5, borderRadius: 999, background: color }} />
                <span style={{ color: c.ink3 }}>{f}</span>
                <span style={{ color, fontWeight: v ? 600 : 400, marginLeft: 'auto' }}>{String(v)}</span>
              </div>
            );
          })}
        </div>
      </FuseSection>

      <FuseSection c={c} kicker="History">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <FuseHistoryRow c={c} when="today, 09:12"  by="binding" event="bootstrap accepted"  tone="ok" />
          <FuseHistoryRow c={c} when="today, 09:11"  by="client"  event="bootstrap started"   tone="muted" />
          <FuseHistoryRow c={c} when="3d ago, 14:02" by="binding" event="sync resumed · 23 messages caught up" tone="muted" />
        </div>
      </FuseSection>

      <FuseSection c={c} kicker="About this gate">
        <div style={{
          padding: '10px 12px', background: c.surfaceWarm,
          border: `1px solid ${c.border}`, borderRadius: 10,
          fontSize: 11.5, color: c.ink2, lineHeight: 1.55,
        }}>
          The bootstrap gate is mercury-core's first decision. The UI cannot open
          the message surface, decrypt local timelines, or render notifications until
          this view returns <span style={{ color: c.ok }}>can_open_message_ui = true</span>.
          The check is per-client, not per-room.
        </div>
      </FuseSection>
    </FusePanelShell>
  );
}
