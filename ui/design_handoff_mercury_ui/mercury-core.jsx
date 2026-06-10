// mercury-core.jsx
// Shared state + PlatformDecisionView simulator for the Mercury thread.
// All three visual variants consume this same hook so behavior stays consistent.
//
// Exports (window): useMercuryThread, MERCURY_PEOPLE, REASON_DICT

const MERCURY_PEOPLE = {
  me:    { id: 'me',    name: 'You',          short: 'YOU', hue: 220 },
  rin:   { id: 'rin',   name: 'Rin Hadley',   short: 'RH',  hue: 16  },
  jules: { id: 'jules', name: 'Jules Okafor', short: 'JO',  hue: 152 },
  ai:    { id: 'ai',    name: 'Mercury AI',   short: 'AI',  hue: 268, isAi: true },
};

// reason_label -> human gloss + severity
const REASON_DICT = {
  ACCEPTED:                 { gloss: 'Accepted',                          tone: 'ok'   },
  ORDERING_GAP:             { gloss: 'Ordering gap — fetching missing',   tone: 'warn' },
  SYNC_INCOMPLETE:          { gloss: 'Sync incomplete',                   tone: 'warn' },
  RECOVERY_REQUIRED:        { gloss: 'Recovery required',                 tone: 'bad'  },
  RECIPIENT_TRUST_REJECTED: { gloss: 'Recipient trust rejected',          tone: 'bad'  },
  TOFU_PENDING:             { gloss: 'First-use verification needed',     tone: 'warn' },
  KEY_STALE:                { gloss: 'Recipient key stale',               tone: 'bad'  },
  AI_GRANT_ABSENT:          { gloss: 'No AI grant in this room',          tone: 'bad'  },
  AI_GRANT_REVOKED:         { gloss: 'AI grant revoked',                  tone: 'bad'  },
  AI_GRANT_EXPIRED:         { gloss: 'AI grant expired',                  tone: 'bad'  },
  SENDER_TRUST_REJECTED:    { gloss: 'Sender trust rejected',             tone: 'bad'  },
  REPLAY_DETECTED:          { gloss: 'Replay detected — dropped',         tone: 'bad'  },
};

// Reason codes — short integers mercury-core uses on the wire. UI must not
// rely on the numbers (the report says use reason_label), but we surface them
// in the Operator variant for diagnostic flavor.
const REASON_CODE = {
  ACCEPTED: 0, ORDERING_GAP: 10, REPLAY_DETECTED: 11, SENDER_TRUST_REJECTED: 12,
  RECIPIENT_TRUST_REJECTED: 20, TOFU_PENDING: 21, KEY_STALE: 22,
  AI_GRANT_ABSENT: 30, AI_GRANT_REVOKED: 31, AI_GRANT_EXPIRED: 32,
  SYNC_INCOMPLETE: 18, RECOVERY_REQUIRED: 19,
};

// -------- decision builders (mirror PlatformDecisionView) ---------
function decisionBootstrap(bootstrap) {
  // bootstrap: 'accepted' | 'sync_incomplete' | 'recovery_required'
  if (bootstrap === 'accepted') {
    return base('client_bootstrap', true, 'ACCEPTED', {
      can_open_message_ui: true, can_start_sync: true,
    });
  }
  if (bootstrap === 'sync_incomplete') {
    return base('client_bootstrap', false, 'SYNC_INCOMPLETE', {
      can_start_sync: true, requires_sync: true,
    });
  }
  return base('client_bootstrap', false, 'RECOVERY_REQUIRED', {
    requires_recovery: true,
  });
}

function decisionOutbound({ trust, ai, draftMentionsAi }) {
  // trust: 'trusted' | 'sendable_not_full' | 'unverified' | 'rejected' | 'stale'
  // ai: 'absent' | 'granted' | 'revoked' | 'expired'
  if (trust === 'rejected') {
    return base('outbound_send', false, 'RECIPIENT_TRUST_REJECTED', {});
  }
  if (trust === 'stale') {
    return base('outbound_send', false, 'KEY_STALE', { requires_user_action: true });
  }
  if (draftMentionsAi) {
    if (ai === 'absent')  return base('outbound_send', false, 'AI_GRANT_ABSENT',  {});
    if (ai === 'revoked') return base('outbound_send', false, 'AI_GRANT_REVOKED', {});
    if (ai === 'expired') return base('outbound_send', false, 'AI_GRANT_EXPIRED', {});
  }
  if (trust === 'unverified') {
    return base('outbound_send', true, 'TOFU_PENDING', {
      can_send: true, can_persist_ciphertext: true, requires_user_action: true,
    });
  }
  return base('outbound_send', true, 'ACCEPTED', {
    can_send: true, can_persist_ciphertext: true,
  });
}

function decisionReceive({ receiveMode, senderTrust }) {
  // receiveMode: 'accepted' | 'ordering_gap'
  if (senderTrust === 'rejected') {
    return base('client_receive', false, 'SENDER_TRUST_REJECTED', { requires_user_action: true });
  }
  if (receiveMode === 'ordering_gap') {
    return base('client_receive', false, 'ORDERING_GAP', { requires_client_retry: true });
  }
  return base('client_receive', true, 'ACCEPTED', {
    can_receive: true, can_open_message_ui: true,
  });
}

function base(source, accepted, reason_label, over) {
  return {
    source, accepted, reason_label,
    reason_code: REASON_CODE[reason_label] ?? 0,
    can_open_message_ui:    false,
    can_start_sync:         false,
    can_send:               false,
    can_receive:            false,
    can_persist_ciphertext: false,
    requires_sync:          false,
    requires_recovery:      false,
    requires_client_retry:  false,
    requires_user_action:   false,
    ...over,
  };
}

// -------- the hook ---------
function useMercuryThread({ bootstrap, trust, ai, receiveMode, senderTrust, forceSendOutcome }) {
  const { useState, useEffect, useRef, useMemo, useCallback } = React;

  // seed conversation (already-accepted received messages)
  const [messages, setMessages] = useState(() => seedThread());
  const [draft, setDraft] = useState('');
  const [pendingTofu, setPendingTofu] = useState(null); // a draft awaiting user confirm
  const [retryingGap, setRetryingGap] = useState(false);
  const [unlocked, setUnlocked] = useState(false); // for terminal variant composer gate
  const idRef = useRef(100);

  // detect @ai mention in draft
  const draftMentionsAi = /(^|\s)@ai\b/i.test(draft);

  // decision views
  const bootstrapView = useMemo(() => decisionBootstrap(bootstrap), [bootstrap]);
  const outboundView  = useMemo(
    () => {
      if (forceSendOutcome && forceSendOutcome !== 'auto') {
        // Tweak override — force a specific outbound outcome regardless of trust state.
        const map = {
          accepted:  base('outbound_send', true,  'ACCEPTED',                 { can_send: true, can_persist_ciphertext: true }),
          tofu:      base('outbound_send', true,  'TOFU_PENDING',             { can_send: true, can_persist_ciphertext: true, requires_user_action: true }),
          rejected:  base('outbound_send', false, 'RECIPIENT_TRUST_REJECTED', {}),
          stale:     base('outbound_send', false, 'KEY_STALE',                { requires_user_action: true }),
        };
        return map[forceSendOutcome];
      }
      return decisionOutbound({ trust, ai, draftMentionsAi });
    },
    [trust, ai, draftMentionsAi, forceSendOutcome]
  );
  const receiveView   = useMemo(() => decisionReceive({ receiveMode, senderTrust }), [receiveMode, senderTrust]);

  // ----- inbound message scheduler -----
  useEffect(() => {
    if (!bootstrapView.can_open_message_ui) return;
    let stopped = false;
    const tick = () => {
      if (stopped) return;
      const decision = decisionReceive({ receiveMode, senderTrust });
      const inbound = nextInbound();
      const id = ++idRef.current;
      if (decision.accepted) {
        setMessages(m => [...m, {
          id, kind: 'incoming', author: inbound.author, text: inbound.text,
          ts: now(), decision,
        }]);
      } else if (decision.reason_label === 'ORDERING_GAP') {
        setMessages(m => [...m, {
          id, kind: 'gap', text: 'Ordering gap — fetching missing messages',
          ts: now(), decision,
        }]);
        // simulate retry+resolve after 3s
        setRetryingGap(true);
        setTimeout(() => {
          setRetryingGap(false);
          setMessages(m => m.map(x => x.id === id
            ? { ...x, kind: 'incoming', text: inbound.text, author: inbound.author, decision: decisionReceive({ receiveMode: 'accepted', senderTrust }) }
            : x));
        }, 3000);
      } else if (decision.reason_label === 'SENDER_TRUST_REJECTED') {
        setMessages(m => [...m, {
          id, kind: 'rejected_inbound', author: inbound.author,
          text: '[plaintext withheld — sender trust rejected]',
          ts: now(), decision,
        }]);
      }
    };
    const handle = setInterval(tick, 11000);
    return () => { stopped = true; clearInterval(handle); };
  }, [bootstrapView.can_open_message_ui, receiveMode, senderTrust]);

  // ----- send action -----
  const send = useCallback((overrideText) => {
    const text = (overrideText ?? draft).trim();
    if (!text) return;
    if (!outboundView.can_send) {
      // surface the rejection as a system entry
      setMessages(m => [...m, {
        id: ++idRef.current, kind: 'send_blocked',
        text, decision: outboundView, ts: now(),
      }]);
      setDraft('');
      return;
    }
    if (outboundView.requires_user_action && !pendingTofu) {
      // park as TOFU-pending until user confirms
      setPendingTofu({ text, decision: outboundView });
      return;
    }
    // accepted send
    const id = ++idRef.current;
    setMessages(m => [...m, {
      id, kind: 'outgoing', author: 'me', text,
      ts: now(), decision: outboundView, status: 'sending',
    }]);
    setDraft('');
    setPendingTofu(null);
    // simulate delivery
    setTimeout(() => {
      setMessages(m => m.map(x => x.id === id ? { ...x, status: 'delivered' } : x));
    }, 900);
    // if @ai and granted, schedule an AI reply
    if (draftMentionsAi && ai === 'granted') {
      setTimeout(() => {
        setMessages(m => [...m, {
          id: ++idRef.current, kind: 'incoming', author: 'ai',
          text: aiReplyFor(text), ts: now(),
          decision: decisionReceive({ receiveMode: 'accepted', senderTrust: 'trusted' }),
        }]);
      }, 1800);
    }
  }, [draft, outboundView, draftMentionsAi, ai, pendingTofu]);

  const acceptTofu = useCallback(() => {
    if (!pendingTofu) return;
    send(pendingTofu.text);
  }, [pendingTofu, send]);

  const cancelTofu = useCallback(() => setPendingTofu(null), []);

  // manual retry for gap state
  const retryGap = useCallback(() => {
    if (retryingGap) return;
    setRetryingGap(true);
    setTimeout(() => setRetryingGap(false), 1200);
  }, [retryingGap]);

  return {
    messages, draft, setDraft,
    bootstrapView, outboundView, receiveView,
    pendingTofu, acceptTofu, cancelTofu,
    retryGap, retryingGap,
    send,
    unlocked, setUnlocked,
    draftMentionsAi,
  };
}

// ----- seed / scripted content -----
function seedThread() {
  const t = Date.now();
  const day = 86_400_000;
  return [
    // ── yesterday ─────────────────────────────────────────
    { id: 0, kind: 'system', text: 'Room opened · 4 participants · end-to-end encrypted', ts: t - day - 8 * 3600_000 },
    { id: 1, kind: 'incoming', author: 'rin',
      text: 'Spec for the receive gate is up. Ordering check is moving into core next sprint.',
      ts: t - day - 6 * 3600_000, decision: { reason_label: 'ACCEPTED' } },
    { id: 2, kind: 'incoming', author: 'jules',
      text: 'Will the platform view still surface requires_client_retry separately, or will you fold it into requires_sync?',
      ts: t - day - 5.8 * 3600_000, decision: { reason_label: 'ACCEPTED' } },
    // ── today ─────────────────────────────────────────────
    { id: 3, kind: 'incoming', author: 'rin',
      text: 'Pushed the receive-gate rewrite. Ordering check is now in core, not the binding.',
      ts: t - 6800_000, decision: { reason_label: 'ACCEPTED' } },
    { id: 4, kind: 'incoming', author: 'rin',
      text: 'Separate. Retry is per-message; sync is per-room. UI needs both.',
      ts: t - 6650_000, decision: { reason_label: 'ACCEPTED' } },
    { id: 5, kind: 'outgoing', author: 'me',
      text: 'Good. I\u2019ll wire the thread shell against from_client_receive and use the reason_label, not the codes.',
      ts: t - 6200_000, status: 'delivered', decision: { reason_label: 'ACCEPTED' } },
    { id: 6, kind: 'incoming', author: 'jules',
      text: 'Per the report — never re-run policy from raw fields. Read the view and surface it.',
      ts: t - 5900_000, decision: { reason_label: 'ACCEPTED' } },
  ];
}

const INBOUND_SCRIPT = [
  { author: 'rin',   text: 'Quick check — what did the binding return for that last send? Was can_persist_ciphertext flipped?' },
  { author: 'jules', text: 'I\u2019m drafting the trust prompt copy. Going with "First-use verification needed" for TOFU.' },
  { author: 'rin',   text: 'Pushed a fixture for ordering_gap with reason_code 10. Test it against your retry path.' },
  { author: 'jules', text: 'AI grant lifecycle is in. Revoke flows through policy_pipeline now, not the binding.' },
  { author: 'rin',   text: 'Reminder: never store plaintext durably. The thread is a render of the binding\u2019s view.' },
];
let inboundIdx = 0;
function nextInbound() {
  const m = INBOUND_SCRIPT[inboundIdx % INBOUND_SCRIPT.length];
  inboundIdx++;
  return m;
}

function aiReplyFor(prompt) {
  const p = prompt.toLowerCase();
  if (p.includes('summarize') || p.includes('summary')) {
    return 'Summary (scoped to this grant): Rin moved the ordering check into core; the binding now reads from_client_receive; UI must surface requires_client_retry, not infer it.';
  }
  if (p.includes('reason') || p.includes('code')) {
    return 'reason_code is the wire-level integer; reason_label is the canonical string. Per integration rules, UI should display reason_label.';
  }
  return 'Acknowledged. I have read-only access to this room\u2019s decrypted content for the duration of this grant; nothing is persisted past expiry.';
}

const now = () => Date.now();

// expose
Object.assign(window, {
  useMercuryThread, MERCURY_PEOPLE, REASON_DICT, REASON_CODE,
});
