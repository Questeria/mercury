// Faithful TypeScript port of the design-handoff `useMercuryThread` hook.
//
// This hook is THE integration surface. It takes scenario inputs (dev
// simulation knobs) and returns the rendered message list, the three live
// `PlatformDecisionView`s, and the send/trust actions. In production the
// scenario inputs disappear and the views come straight from the
// `MercuryBinding` (see `binding.ts`) -- but the RETURNED shape
// (`MercuryThreadState`) never changes, so the UI never changes.
//
// Integration rules honored here:
//   - The message array is EPHEMERAL render state (rule 4: never persist
//     plaintext durably). Treat it as a render of accepted decisions.
//   - The UI branches on `reason_label` + capability flags, never on
//     `reason_code` (rule 2).
//   - A rejected decision is never silently converted to accepted (rule 3):
//     a blocked send becomes a visible `send_blocked` banner.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { createSimulatorBinding, type MercuryBinding } from "./binding";
import { aiReplyFor, nextInbound, now, seedThread } from "./simulator";
import type {
  Message,
  MercuryThreadInputs,
  MercuryThreadState,
  PlatformDecisionView,
} from "./types";

const AI_MENTION = /(^|\s)@ai\b/i;

const INBOUND_INTERVAL_MS = 11_000;
const GAP_RESOLVE_MS = 3_000;
const DELIVERY_MS = 900;
const AI_REPLY_MS = 1_800;
const MANUAL_RETRY_MS = 1_200;

export function useMercuryThread(
  inputs: MercuryThreadInputs,
  bindingProp?: MercuryBinding,
): MercuryThreadState {
  const { bootstrap, trust, ai, receiveMode, senderTrust, forceSendOutcome } = inputs;

  // Stable binding: default to the simulator, memoized so the inbound interval
  // and decision memos don't tear down/recompute on every render.
  const binding = useMemo(() => bindingProp ?? createSimulatorBinding(), [bindingProp]);

  const [messages, setMessages] = useState<Message[]>(() => seedThread());
  const [draft, setDraft] = useState("");
  const [pendingTofu, setPendingTofu] = useState<MercuryThreadState["pendingTofu"]>(null);
  const [retryingGap, setRetryingGap] = useState(false);
  const idRef = useRef(100);

  // Live @ai detection in the draft (drives the composer hint + outbound gate).
  const draftMentionsAi = AI_MENTION.test(draft);

  // ---- the three live decision views (memoized off their inputs) ----
  const bootstrapView = useMemo<PlatformDecisionView>(
    () => binding.bootstrapStatus(bootstrap),
    [binding, bootstrap],
  );

  const outboundView = useMemo<PlatformDecisionView>(
    () => binding.prepareSend({ trust, ai, draftMentionsAi, forceSendOutcome }),
    [binding, trust, ai, draftMentionsAi, forceSendOutcome],
  );

  const receiveView = useMemo<PlatformDecisionView>(
    () => binding.acceptReceivedCiphertext({ receiveMode, senderTrust }),
    [binding, receiveMode, senderTrust],
  );

  // ---- inbound scheduler: a scripted message arrives ~every 11s, routed
  //      through the receive gate. Accepted renders; ORDERING_GAP shows a gap
  //      banner that resolves after ~3s; SENDER_TRUST_REJECTED withholds. ----
  useEffect(() => {
    if (!bootstrapView.can_open_message_ui) return;
    let stopped = false;

    const tick = () => {
      if (stopped) return;
      const decision = binding.acceptReceivedCiphertext({ receiveMode, senderTrust });
      const inbound = nextInbound();
      const id = ++idRef.current;

      if (decision.accepted) {
        setMessages((m) => [
          ...m,
          { id, kind: "incoming", author: inbound.author, text: inbound.text, ts: now(), decision },
        ]);
        return;
      }

      // Branch on the capability flag, not the label string, so this is robust
      // to either vocabulary (simulator ORDERING_GAP or real-core view).
      if (decision.requires_client_retry) {
        setMessages((m) => [
          ...m,
          { id, kind: "gap", text: "Ordering gap — fetching missing messages", ts: now(), decision },
        ]);
        // simulate retry + resolve
        setRetryingGap(true);
        window.setTimeout(() => {
          if (stopped) return;
          setRetryingGap(false);
          const resolved = binding.acceptReceivedCiphertext({ receiveMode: "accepted", senderTrust });
          setMessages((m) =>
            m.map((x) =>
              x.id === id
                ? { ...x, kind: "incoming", text: inbound.text, author: inbound.author, decision: resolved }
                : x,
            ),
          );
        }, GAP_RESOLVE_MS);
        return;
      }

      // Any other non-accepted inbound: withhold plaintext (rule: render nothing
      // a gate hasn't accepted). Covers SENDER_TRUST_REJECTED (sim) and the
      // real core's SENDER_DEVICE_TRUST_REJECTED / MESSAGE_POLICY_REJECTED / etc.
      setMessages((m) => [
        ...m,
        {
          id,
          kind: "rejected_inbound",
          author: inbound.author,
          text: "[plaintext withheld — receive gate rejected]",
          ts: now(),
          decision,
        },
      ]);
    };

    const handle = window.setInterval(tick, INBOUND_INTERVAL_MS);
    return () => {
      stopped = true;
      window.clearInterval(handle);
    };
  }, [binding, bootstrapView.can_open_message_ui, receiveMode, senderTrust]);

  // ---- send action ----
  const send = useCallback(
    (overrideText?: string) => {
      const text = (overrideText ?? draft).trim();
      if (!text) return;

      // rule 3: a rejected outbound is surfaced, never silently accepted.
      if (!outboundView.can_send) {
        setMessages((m) => [
          ...m,
          { id: ++idRef.current, kind: "send_blocked", text, decision: outboundView, ts: now() },
        ]);
        setDraft("");
        return;
      }

      // first-use verification: park the draft until the user confirms.
      if (outboundView.requires_user_action && !pendingTofu) {
        setPendingTofu({ text, decision: outboundView });
        return;
      }

      const id = ++idRef.current;
      setMessages((m) => [
        ...m,
        { id, kind: "outgoing", author: "me", text, ts: now(), decision: outboundView, status: "sending" },
      ]);
      setDraft("");
      setPendingTofu(null);

      // simulate delivery
      window.setTimeout(() => {
        setMessages((m) => m.map((x) => (x.id === id ? { ...x, status: "delivered" } : x)));
      }, DELIVERY_MS);

      // @ai + granted: schedule a scoped, streamed AI reply
      if (draftMentionsAi && ai === "granted") {
        window.setTimeout(() => {
          setMessages((m) => [
            ...m,
            {
              id: ++idRef.current,
              kind: "incoming",
              author: "ai",
              text: aiReplyFor(text),
              ts: now(),
              decision: binding.acceptReceivedCiphertext({ receiveMode: "accepted", senderTrust: "trusted" }),
            },
          ]);
        }, AI_REPLY_MS);
      }
    },
    [binding, draft, outboundView, draftMentionsAi, ai, pendingTofu],
  );

  const acceptTofu = useCallback(() => {
    if (!pendingTofu) return;
    send(pendingTofu.text);
  }, [pendingTofu, send]);

  const cancelTofu = useCallback(() => setPendingTofu(null), []);

  const retryGap = useCallback(() => {
    setRetryingGap((cur) => {
      if (cur) return cur;
      window.setTimeout(() => setRetryingGap(false), MANUAL_RETRY_MS);
      return true;
    });
  }, []);

  return {
    messages,
    draft,
    setDraft,
    bootstrapView,
    outboundView,
    receiveView,
    pendingTofu,
    acceptTofu,
    cancelTofu,
    retryGap,
    retryingGap,
    send,
    draftMentionsAi,
  };
}
