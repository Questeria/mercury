// Messaging-binding tests. The binding talks to the `mercury-app-server` shim over
// `POST /command`; here `fetch` is mocked so we assert (a) each method forms the exact JSON
// command the Rust backend expects, (b) responses are parsed into typed results, and (c) every
// failure mode (in-band ok:false, non-2xx, network error, non-JSON) is fail-closed.

import { afterEach, describe, expect, it, vi } from "vitest";

import { type ChatMessage, createHttpMessaging, MessagingError } from "./messaging";

type FetchResult = { ok?: boolean; status?: number; json?: () => Promise<unknown>; reject?: boolean };

function mockFetch(r: FetchResult) {
  const fn = vi.fn();
  if (r.reject) {
    fn.mockRejectedValue(new Error("network down"));
  } else {
    fn.mockResolvedValue({
      ok: r.ok ?? true,
      status: r.status ?? 200,
      json: r.json ?? (async () => ({ ok: true, result: null })),
    });
  }
  global.fetch = fn as unknown as typeof fetch;
  return fn;
}

function bodyOf(fn: ReturnType<typeof vi.fn>, call = 0): unknown {
  const init = fn.mock.calls[call][1] as RequestInit;
  return JSON.parse(init.body as string);
}

/** A well-formed 13-field PlatformDecisionView as it arrives over the wire. */
function validView(accepted: boolean, label: string): Record<string, unknown> {
  return {
    source: "outbound_send",
    accepted,
    reason_code: accepted ? 0 : 7,
    reason_label: label,
    can_open_message_ui: accepted,
    can_start_sync: accepted,
    can_send: accepted,
    can_receive: accepted,
    can_persist_ciphertext: accepted,
    requires_sync: false,
    requires_recovery: false,
    requires_client_retry: false,
    requires_user_action: false,
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("http messaging binding", () => {
  it("accountId posts {cmd:account_id} to /command and returns the id", async () => {
    const id = "ab".repeat(32);
    const fn = mockFetch({ json: async () => ({ ok: true, result: { account_id: id } }) });
    const m = createHttpMessaging("http://127.0.0.1:7879/"); // trailing slash trimmed
    expect(await m.accountId()).toBe(id);
    expect(fn).toHaveBeenCalledTimes(1);
    expect(fn.mock.calls[0][0]).toBe("http://127.0.0.1:7879/command");
    expect((fn.mock.calls[0][1] as RequestInit).method).toBe("POST");
    expect(bodyOf(fn)).toEqual({ cmd: "account_id" });
  });

  it("send posts {cmd:send, peer, text}", async () => {
    const fn = mockFetch({ json: async () => ({ ok: true, result: null }) });
    const m = createHttpMessaging("http://x");
    await m.send("de".repeat(32), "hello");
    expect(bodyOf(fn)).toEqual({ cmd: "send", peer: "de".repeat(32), text: "hello" });
  });

  it("addContact and canSend form their commands and parse results", async () => {
    const fnAdd = mockFetch({ json: async () => ({ ok: true, result: null }) });
    await createHttpMessaging("http://x").addContact("aa".repeat(32));
    expect(bodyOf(fnAdd)).toEqual({ cmd: "add_contact", account_id: "aa".repeat(32) });

    mockFetch({ json: async () => ({ ok: true, result: { can_send: true } }) });
    expect(await createHttpMessaging("http://x").canSend("aa".repeat(32))).toBe(true);
  });

  it("poll returns typed ChatMessages plus silently-updated peers", async () => {
    const msgs: ChatMessage[] = [
      { direction: "incoming", peer: "aa".repeat(32), text: "hi", seq: 0 },
      { direction: "outgoing", peer: "aa".repeat(32), text: "yo", seq: 1 },
    ];
    mockFetch({ json: async () => ({ ok: true, result: { messages: msgs } }) });
    // `updated` is additive — an older backend that omits it yields [].
    expect(await createHttpMessaging("http://x").poll()).toEqual({ messages: msgs, updated: [] });
    mockFetch({
      json: async () => ({ ok: true, result: { messages: [], updated: ["bb".repeat(32)] } }),
    });
    expect(await createHttpMessaging("http://x").poll()).toEqual({
      messages: [],
      updated: ["bb".repeat(32)],
    });
  });

  it("rejects with MessagingError on an in-band ok:false", async () => {
    mockFetch({ json: async () => ({ ok: false, error: "no session or contact card for that peer" }) });
    const m = createHttpMessaging("http://x");
    await expect(m.send("00".repeat(32), "x")).rejects.toBeInstanceOf(MessagingError);
    await expect(m.send("00".repeat(32), "x")).rejects.toThrow(/no session or contact card/);
  });

  it("rejects on a non-2xx HTTP status", async () => {
    mockFetch({ ok: false, status: 400 });
    await expect(createHttpMessaging("http://x").accountId()).rejects.toBeInstanceOf(MessagingError);
  });

  it("rejects on a network failure", async () => {
    mockFetch({ reject: true });
    await expect(createHttpMessaging("http://x").accountId()).rejects.toBeInstanceOf(MessagingError);
  });

  it("rejects on a non-JSON response body", async () => {
    mockFetch({
      json: async () => {
        throw new Error("not json");
      },
    });
    await expect(createHttpMessaging("http://x").poll()).rejects.toBeInstanceOf(MessagingError);
  });

  it("decisions posts {cmd:decisions, peer} and parses the three real views", async () => {
    const result = {
      bootstrap: validView(true, "ACCEPTED"),
      outbound: validView(false, "DEVICE_TRUST_REJECTED"),
      receive: validView(true, "ACCEPTED"),
    };
    const fn = mockFetch({ json: async () => ({ ok: true, result }) });
    const d = await createHttpMessaging("http://x").decisions("aa".repeat(32));
    expect(bodyOf(fn)).toEqual({ cmd: "decisions", peer: "aa".repeat(32) });
    expect(d.bootstrap.accepted).toBe(true);
    // A real REJECT is preserved verbatim — never coerced to an accept.
    expect(d.outbound.accepted).toBe(false);
    expect(d.outbound.can_send).toBe(false);
    expect(d.outbound.reason_label).toBe("DEVICE_TRUST_REJECTED");
  });

  it("decisions fail-closed: a view with a non-boolean capability rejects (no partial view leaks)", async () => {
    // outbound.can_send arrives as a string — a malformed/hostile response must throw, never yield a
    // half-built view the UI could render as a granted capability.
    const result = {
      bootstrap: validView(true, "ACCEPTED"),
      outbound: { ...validView(true, "ACCEPTED"), can_send: "yes" },
      receive: validView(true, "ACCEPTED"),
    };
    mockFetch({ json: async () => ({ ok: true, result }) });
    await expect(
      createHttpMessaging("http://x").decisions("aa".repeat(32)),
    ).rejects.toBeInstanceOf(MessagingError);
  });

  it("decisions fail-closed: a missing view / non-object envelope rejects", async () => {
    mockFetch({ json: async () => ({ ok: true, result: { bootstrap: validView(true, "ACCEPTED") } }) });
    await expect(
      createHttpMessaging("http://x").decisions("aa".repeat(32)),
    ).rejects.toBeInstanceOf(MessagingError);
  });
});
