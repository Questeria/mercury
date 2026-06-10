// Decision-contract tests.
//
// Covers the design handoff's "Minimum UI Test Scenarios" (30_UI_INTEGRATION_
// REPORT.md) plus the non-negotiable integration rules. Because the UI only
// ever RENDERS a PlatformDecisionView, asserting the decisions the binding
// returns is asserting the UI's security behavior.

import { describe, expect, it } from "vitest";

import { createSimulatorBinding } from "./binding";
import { REASON_CODE } from "./simulator";
import type { PlatformDecisionView, ReasonLabel } from "./types";

const binding = createSimulatorBinding();

const ALL_FIELDS: (keyof PlatformDecisionView)[] = [
  "source",
  "accepted",
  "reason_code",
  "reason_label",
  "can_open_message_ui",
  "can_start_sync",
  "can_send",
  "can_receive",
  "can_persist_ciphertext",
  "requires_sync",
  "requires_recovery",
  "requires_client_retry",
  "requires_user_action",
];

/** Every view is fully populated and its code matches its label (rule 2: the
 *  UI branches on the label; the code is only ever diagnostic flavor). */
function expectWellFormed(v: PlatformDecisionView) {
  for (const f of ALL_FIELDS) expect(v[f]).toBeDefined();
  // Simulator views carry the strict handoff vocabulary.
  expect(v.reason_code).toBe(REASON_CODE[v.reason_label as ReasonLabel]);
}

describe("bootstrap gate", () => {
  it("accepted opens the message shell", () => {
    const v = binding.bootstrapStatus("accepted");
    expectWellFormed(v);
    expect(v.accepted).toBe(true);
    expect(v.can_open_message_ui).toBe(true);
    expect(v.can_start_sync).toBe(true);
    expect(v.reason_label).toBe("ACCEPTED");
  });

  it("sync incomplete keeps the shell closed but offers sync", () => {
    const v = binding.bootstrapStatus("sync_incomplete");
    expectWellFormed(v);
    expect(v.accepted).toBe(false);
    expect(v.can_open_message_ui).toBe(false);
    expect(v.can_start_sync).toBe(true);
    expect(v.requires_sync).toBe(true);
    expect(v.reason_label).toBe("SYNC_INCOMPLETE");
  });

  it("recovery required keeps the shell closed and offers recovery", () => {
    const v = binding.bootstrapStatus("recovery_required");
    expectWellFormed(v);
    expect(v.accepted).toBe(false);
    expect(v.can_open_message_ui).toBe(false);
    expect(v.requires_recovery).toBe(true);
    expect(v.reason_label).toBe("RECOVERY_REQUIRED");
  });
});

describe("outbound send gate", () => {
  const send = (over: Parameters<typeof binding.prepareSend>[0]) => binding.prepareSend(over);

  it("trusted recipient enables final send and persistence", () => {
    const v = send({ trust: "trusted", ai: "granted", draftMentionsAi: false });
    expectWellFormed(v);
    expect(v.accepted).toBe(true);
    expect(v.can_send).toBe(true);
    expect(v.can_persist_ciphertext).toBe(true);
    expect(v.reason_label).toBe("ACCEPTED");
  });

  it("rejected recipient disables send and persistence (rule 3: no silent accept)", () => {
    const v = send({ trust: "rejected", ai: "granted", draftMentionsAi: false });
    expect(v.accepted).toBe(false);
    expect(v.can_send).toBe(false);
    expect(v.can_persist_ciphertext).toBe(false);
    expect(v.reason_label).toBe("RECIPIENT_TRUST_REJECTED");
  });

  it("stale key blocks send and asks for user action", () => {
    const v = send({ trust: "stale", ai: "granted", draftMentionsAi: false });
    expect(v.can_send).toBe(false);
    expect(v.requires_user_action).toBe(true);
    expect(v.reason_label).toBe("KEY_STALE");
  });

  it("unverified recipient sends under first-use verification (TOFU)", () => {
    const v = send({ trust: "unverified", ai: "granted", draftMentionsAi: false });
    expect(v.accepted).toBe(true);
    expect(v.can_send).toBe(true);
    expect(v.can_persist_ciphertext).toBe(true);
    expect(v.requires_user_action).toBe(true);
    expect(v.reason_label).toBe("TOFU_PENDING");
  });

  it("@ai with no grant blocks the send", () => {
    const v = send({ trust: "trusted", ai: "absent", draftMentionsAi: true });
    expect(v.can_send).toBe(false);
    expect(v.reason_label).toBe("AI_GRANT_ABSENT");
  });

  it("@ai with a revoked or expired grant removes AI access", () => {
    const revoked = send({ trust: "trusted", ai: "revoked", draftMentionsAi: true });
    expect(revoked.can_send).toBe(false);
    expect(revoked.reason_label).toBe("AI_GRANT_REVOKED");

    const expired = send({ trust: "trusted", ai: "expired", draftMentionsAi: true });
    expect(expired.can_send).toBe(false);
    expect(expired.reason_label).toBe("AI_GRANT_EXPIRED");
  });

  it("a missing grant only blocks when the draft actually mentions @ai", () => {
    const v = send({ trust: "trusted", ai: "absent", draftMentionsAi: false });
    expect(v.can_send).toBe(true);
    expect(v.reason_label).toBe("ACCEPTED");
  });

  it("trust rejection takes precedence over an @ai grant problem", () => {
    const v = send({ trust: "rejected", ai: "absent", draftMentionsAi: true });
    expect(v.reason_label).toBe("RECIPIENT_TRUST_REJECTED");
  });

  it("demo force-override yields the requested outcome regardless of trust", () => {
    expect(send({ trust: "rejected", ai: "absent", draftMentionsAi: true, forceSendOutcome: "accepted" }).can_send).toBe(true);
    expect(send({ trust: "trusted", ai: "granted", draftMentionsAi: false, forceSendOutcome: "tofu" }).reason_label).toBe("TOFU_PENDING");
    expect(send({ trust: "trusted", ai: "granted", draftMentionsAi: false, forceSendOutcome: "rejected" }).can_send).toBe(false);
    expect(send({ trust: "trusted", ai: "granted", draftMentionsAi: false, forceSendOutcome: "stale" }).reason_label).toBe("KEY_STALE");
    // "auto" defers to the real trust/ai inputs
    expect(send({ trust: "trusted", ai: "granted", draftMentionsAi: false, forceSendOutcome: "auto" }).reason_label).toBe("ACCEPTED");
  });

  it("never persists ciphertext on a blocked send", () => {
    for (const v of [
      send({ trust: "rejected", ai: "granted", draftMentionsAi: false }),
      send({ trust: "stale", ai: "granted", draftMentionsAi: false }),
      send({ trust: "trusted", ai: "absent", draftMentionsAi: true }),
    ]) {
      expect(v.can_send).toBe(false);
      expect(v.can_persist_ciphertext).toBe(false);
    }
  });
});

describe("client receive gate", () => {
  it("accepted hands the message to the renderer", () => {
    const v = binding.acceptReceivedCiphertext({ receiveMode: "accepted", senderTrust: "trusted" });
    expectWellFormed(v);
    expect(v.accepted).toBe(true);
    expect(v.can_receive).toBe(true);
    expect(v.can_open_message_ui).toBe(true);
  });

  it("ordering gap triggers retry and renders no plaintext", () => {
    const v = binding.acceptReceivedCiphertext({ receiveMode: "ordering_gap", senderTrust: "trusted" });
    expect(v.accepted).toBe(false);
    expect(v.can_receive).toBe(false);
    expect(v.requires_client_retry).toBe(true);
    expect(v.reason_label).toBe("ORDERING_GAP");
  });

  it("sender-trust rejection withholds plaintext and shows a user-action path", () => {
    const v = binding.acceptReceivedCiphertext({ receiveMode: "accepted", senderTrust: "rejected" });
    expect(v.accepted).toBe(false);
    expect(v.can_receive).toBe(false);
    expect(v.requires_user_action).toBe(true);
    expect(v.reason_label).toBe("SENDER_TRUST_REJECTED");
  });

  it("sender-trust rejection takes precedence over an ordering gap", () => {
    const v = binding.acceptReceivedCiphertext({ receiveMode: "ordering_gap", senderTrust: "rejected" });
    expect(v.reason_label).toBe("SENDER_TRUST_REJECTED");
  });
});
