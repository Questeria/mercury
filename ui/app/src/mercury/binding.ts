// The mercury-core binding boundary.
//
// `MercuryBinding` is the narrow surface the UI depends on. Every method
// returns a `PlatformDecisionView` -- the UI renders the decision and never
// recomputes policy (integration rules 1-3).
//
// Two implementations live behind this one interface, per the handoff:
//
//   - `createSimulatorBinding()` -- the in-app dev simulator (default). Wraps
//     the faithful TS port of the mercury-core decision builders.
//   - `createFfiBinding()` -- the production swap point. Will call the real
//     generated FFI (`mercury_bootstrap_status`, `mercury_prepare_send`,
//     `mercury_accept_received_ciphertext`, `mercury_policy_status`) exposed by
//     the `mercury-bindings` / `mercury-ffi` crates. Throws until wired so the
//     swap is explicit and un-fakeable.
//
// Because both sit behind `MercuryBinding`, `useMercuryThread` -- and therefore
// every component -- is identical in dev and production.

import {
  decisionBootstrap,
  decisionOutbound,
  decisionReceive,
  outboundForce,
} from "./simulator";
import type {
  AiState,
  BootstrapState,
  ForceSendOutcome,
  PlatformDecisionView,
  ReceiveMode,
  SenderTrust,
  TrustState,
} from "./types";

export interface PrepareSendArgs {
  trust: TrustState;
  ai: AiState;
  draftMentionsAi: boolean;
  /** Dev-only demo override; absent/`"auto"` in production. */
  forceSendOutcome?: ForceSendOutcome;
}

export interface AcceptReceivedArgs {
  receiveMode: ReceiveMode;
  senderTrust: SenderTrust;
}

/** The single dependency the UI has on mercury-core. */
export interface MercuryBinding {
  /** `mercury_bootstrap_status(...)` */
  bootstrapStatus(bootstrap: BootstrapState): PlatformDecisionView;
  /** `mercury_prepare_send(...)` */
  prepareSend(args: PrepareSendArgs): PlatformDecisionView;
  /** `mercury_accept_received_ciphertext(...)` */
  acceptReceivedCiphertext(args: AcceptReceivedArgs): PlatformDecisionView;
  /** `mercury_policy_status(...)` -- reserved for the policy-pipeline source. */
  policyStatus?(): PlatformDecisionView;
}

/** Dev default: decisions computed by the in-app simulator. */
export function createSimulatorBinding(): MercuryBinding {
  return {
    bootstrapStatus: (bootstrap) => decisionBootstrap(bootstrap),
    prepareSend: ({ trust, ai, draftMentionsAi, forceSendOutcome }) => {
      if (forceSendOutcome && forceSendOutcome !== "auto") {
        return outboundForce(forceSendOutcome);
      }
      return decisionOutbound({ trust, ai, draftMentionsAi });
    },
    acceptReceivedCiphertext: ({ receiveMode, senderTrust }) =>
      decisionReceive({ receiveMode, senderTrust }),
  };
}

/**
 * Production swap point. Wraps the real mercury-core FFI once the
 * `mercury-bindings` bridge is generated. Until then it fails loudly rather
 * than silently degrading to simulated decisions.
 */
export function createFfiBinding(): MercuryBinding {
  const notWired = (fn: string) => (): never => {
    throw new Error(
      `mercury-core FFI not wired yet: ${fn}. Generate the binding from ` +
        `mercury-bindings/mercury-ffi and replace createFfiBinding(), or use ` +
        `createSimulatorBinding() for dev.`,
    );
  };
  return {
    bootstrapStatus: notWired("mercury_bootstrap_status"),
    prepareSend: notWired("mercury_prepare_send"),
    acceptReceivedCiphertext: notWired("mercury_accept_received_ciphertext"),
    policyStatus: notWired("mercury_policy_status"),
  };
}

// ---------------------------------------------------------------------------
// HTTP binding -- real mercury-core decisions via the `mercury-ui-bridge` dev
// shim (see core/rust/mercury-ui-bridge). The bridge is fixture-only today, so
// this maps the UI's scenario inputs to the nearest of the core's 10 canonical
// fixtures. Decisions are prefetched into a module cache so the synchronous
// MercuryBinding surface (and the hook) stays unchanged; until the cache is
// warm -- or for an input with no matching fixture -- it falls back to the
// simulator.
//
// NOTE: the real core's reason_label vocabulary is richer/different from the
// design-handoff simulator's (e.g. MESSAGE_POLICY_REJECTED, AI_GRANT_REJECT,
// SENDER_DEVICE_TRUST_REJECTED, and AI grants surface on the policy_pipeline
// source, not outbound_send). The UI surfaces whatever label the core emits;
// reconciling glosses/tones for the full real vocabulary is a follow-up.
// ---------------------------------------------------------------------------

const httpCache = new Map<string, PlatformDecisionView>();
let prefetchInFlight: Promise<number> | null = null;

const BOOTSTRAP_FIXTURE: Record<BootstrapState, string> = {
  accepted: "bootstrap_accepted",
  sync_incomplete: "bootstrap_sync_incomplete",
  recovery_required: "bootstrap_recovery_required",
};

function sendFixture({ trust, ai, draftMentionsAi }: PrepareSendArgs): string {
  if (draftMentionsAi && ai !== "granted") {
    // AI grants surface on the policy_pipeline source in the real core.
    return ai === "expired" ? "policy_ai_lifecycle_expired" : "policy_ai_grant_rejected";
  }
  // Recipient trust/key problems -> the real DEVICE_TRUST_REJECTED (bridge extra),
  // which fits the UI's "rejected"/"stale" knobs better than MESSAGE_POLICY_REJECTED.
  if (trust === "rejected" || trust === "stale") return "outbound_device_trust_rejected";
  return "outbound_send_accepted";
}

function receiveFixture({ receiveMode, senderTrust }: AcceptReceivedArgs): string {
  // Real SENDER_DEVICE_TRUST_REJECTED (bridge extra) -- a genuine withhold, vs
  // the sender_trust_action fixture which is actually Accepted+requires_user_action.
  if (senderTrust === "rejected") return "receive_sender_device_trust_rejected";
  if (receiveMode === "ordering_gap") return "client_receive_ordering_gap";
  return "client_receive_accepted";
}

/** Warm the module cache from the bridge. Idempotent; safe to call repeatedly. */
export async function prefetchDecisions(baseUrl: string): Promise<number> {
  if (prefetchInFlight) return prefetchInFlight;
  prefetchInFlight = (async () => {
    const listRes = await fetch(`${baseUrl}/decisions`);
    if (!listRes.ok) throw new Error(`bridge /decisions -> ${listRes.status}`);
    const { decisions } = (await listRes.json()) as { decisions: string[] };
    await Promise.all(
      decisions.map(async (name) => {
        const res = await fetch(`${baseUrl}/decision/${name}`);
        if (res.ok) httpCache.set(name, (await res.json()) as PlatformDecisionView);
      }),
    );
    return httpCache.size;
  })();
  try {
    return await prefetchInFlight;
  } finally {
    prefetchInFlight = null;
  }
}

export function decisionsLoaded(): number {
  return httpCache.size;
}

/**
 * Real-core binding. Synchronous facade over the prefetched cache, with a
 * simulator fallback for cache misses (cold cache or unmapped input).
 */
export function createHttpBinding(baseUrl: string): MercuryBinding {
  const fallback = createSimulatorBinding();
  const fromCacheOr = (name: string, fb: () => PlatformDecisionView): PlatformDecisionView =>
    httpCache.get(name) ?? fb();
  void baseUrl; // baseUrl is used by prefetchDecisions(); kept for symmetry/clarity.
  return {
    bootstrapStatus: (bootstrap) =>
      fromCacheOr(BOOTSTRAP_FIXTURE[bootstrap], () => fallback.bootstrapStatus(bootstrap)),
    prepareSend: (args) => fromCacheOr(sendFixture(args), () => fallback.prepareSend(args)),
    acceptReceivedCiphertext: (args) =>
      fromCacheOr(receiveFixture(args), () => fallback.acceptReceivedCiphertext(args)),
  };
}
