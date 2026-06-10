// Multi-conversation controller tests against an in-memory fake backend: identities, opening a
// peer, sending, routing inbound by sender (incl. auto-creating a conversation for a first-time
// sender), and unread bookkeeping — all fail-closed.

import { describe, expect, it } from "vitest";

import { createLiveConversations } from "./liveConversations";
import {
  type ChatMessage,
  type DecisionViews,
  type MercuryMessaging,
  MessagingError,
} from "./messaging";
import type { PlatformDecisionView } from "./types";

/** Build a full 13-field view; the fake uses it to mirror real backend semantics. */
function fakeView(accepted: boolean, label: string): PlatformDecisionView {
  return {
    source: "outbound_send",
    accepted,
    reason_code: accepted ? 0 : 1,
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

class FakeMessaging implements MercuryMessaging {
  account = "aa".repeat(32);
  contacts = new Set<string>();
  logs = new Map<string, ChatMessage[]>();
  inbox: ChatMessage[] = [];
  failAddContact: string | null = null;
  failSend: string | null = null;
  failPoll: string | null = null;

  private push(peer: string, m: Omit<ChatMessage, "seq">) {
    const log = this.logs.get(peer) ?? [];
    log.push({ ...m, seq: log.length });
    this.logs.set(peer, log);
  }
  async accountId() {
    return this.account;
  }
  async publishSelf() {}
  async addContact(id: string) {
    if (this.failAddContact) throw new MessagingError(this.failAddContact);
    this.contacts.add(id);
  }
  async canSend(peer: string) {
    return this.contacts.has(peer);
  }
  async send(peer: string, text: string) {
    if (this.failSend) throw new MessagingError(this.failSend);
    this.push(peer, { direction: "outgoing", peer, text });
  }
  updatedPeers: string[] = [];
  async poll() {
    if (this.failPoll) throw new MessagingError(this.failPoll);
    const drained = this.inbox;
    this.inbox = [];
    for (const m of drained) this.push(m.peer, { direction: "incoming", peer: m.peer, text: m.text });
    const updated = this.updatedPeers;
    this.updatedPeers = [];
    return { messages: drained, updated };
  }
  async history(peer: string) {
    return [...(this.logs.get(peer) ?? [])];
  }
  async decisions(peer: string): Promise<DecisionViews> {
    // Mirrors the real controller: outbound accepts only a known contact; bootstrap/receive accept.
    const known = this.contacts.has(peer);
    return {
      bootstrap: fakeView(true, "ACCEPTED"),
      outbound: fakeView(known, known ? "ACCEPTED" : "DEVICE_TRUST_REJECTED"),
      receive: fakeView(true, "ACCEPTED"),
    };
  }
  async waitForDelivery() {
    return false;
  }
  async deleteConversation(peer: string) {
    this.contacts.delete(peer);
    this.logs.delete(peer);
  }
  async deleteForEveryone(peer: string) {
    this.contacts.delete(peer);
    this.logs.delete(peer);
  }
  async clearHistory(peer: string) {
    this.logs.set(peer, []);
  }
  async safetyNumber(_peer: string) {
    return {
      digits: "1".repeat(60),
      grouped: Array.from({ length: 12 }, () => "11111").join(" "),
    };
  }
  mutedSet = new Set<string>();
  blockedSet = new Set<string>();
  async mute(peer: string) {
    this.mutedSet.add(peer);
  }
  async unmute(peer: string) {
    this.mutedSet.delete(peer);
  }
  async block(peer: string) {
    this.blockedSet.add(peer);
  }
  async unblock(peer: string) {
    this.blockedSet.delete(peer);
  }
  disappearingMap = new Map<string, number>();
  verifiedSet = new Set<string>();
  aliasMap = new Map<string, string>();
  myHandle: string | null = null;
  groupMap = new Map<string, { name: string; creator: string; members: string[]; pending: string[]; ended: boolean }>();
  groupSends: Array<{ group: string; text: string }> = [];
  async conversationFlags() {
    return {
      muted: [...this.mutedSet],
      blocked: [...this.blockedSet],
      verified: [...this.verifiedSet],
      disappearing: Object.fromEntries(this.disappearingMap),
      aliases: Object.fromEntries(this.aliasMap),
      myUsername: this.myHandle,
      groups: Object.fromEntries(this.groupMap),
      pinned: [...this.pinnedSet],
      readReceipts: this.receiptsOn,
    };
  }
  pinnedSet = new Set<string>();
  receiptsOn = true;
  readMarks: string[] = [];
  async markRead(peer: string) {
    this.readMarks.push(peer);
  }
  async setReadReceipts(on: boolean) {
    this.receiptsOn = on;
  }
  async setPinned(peer: string, on: boolean) {
    if (on) this.pinnedSet.add(peer);
    else this.pinnedSet.delete(peer);
  }
  async createGroup(name: string, members: string[]) {
    const group = "dd".repeat(32);
    this.groupMap.set(group, {
      name,
      creator: this.account,
      members: [this.account, ...members],
      pending: [],
      ended: false,
    });
    this.push(group, { direction: "incoming", peer: group, text: `👥 Group “${name}” created — invitations sent.` });
    return { group };
  }
  async groupSend(group: string, text: string) {
    if (!this.groupMap.get(group)) throw new MessagingError("no such group");
    this.groupSends.push({ group, text });
    this.push(group, { direction: "outgoing", peer: group, text });
  }
  async leaveGroup(group: string) {
    this.groupMap.delete(group);
    this.logs.delete(group);
  }
  async setDisappearing(peer: string, secs: number) {
    if (secs > 0) this.disappearingMap.set(peer, secs);
    else this.disappearingMap.delete(peer);
  }
  async setVerified(peer: string) {
    this.verifiedSet.add(peer);
  }
  async unsetVerified(peer: string) {
    this.verifiedSet.delete(peer);
  }
  async sendAttachment(peer: string, name: string, _mime: string, _dataB64: string) {
    if (this.failSend) throw new MessagingError(this.failSend);
    this.push(peer, { direction: "outgoing", peer, text: `📎 ${name}` });
    return { pending: false };
  }
  async backupExport(passphrase: string) {
    if (passphrase.length < 9) throw new MessagingError("backup passphrase must be at least 9 characters");
    return { data_b64: "ZmFrZS1iYWNrdXA=" };
  }
  // A fake single-use pairing broker + a username registry the tests can pre-seed.
  pairingStore = new Map<string, string>(); // code -> account_id
  usernameStore = new Map<string, string>(); // handle -> account_id
  nextCode = "ABCDE-FGHIJ";
  async pairingCode() {
    this.pairingStore.set(this.nextCode, this.account);
    return { code: this.nextCode };
  }
  async addByPairing(code: string) {
    const id = this.pairingStore.get(code);
    if (!id) throw new MessagingError("no such contact"); // unknown/expired/used
    this.pairingStore.delete(code); // single-use
    this.contacts.add(id);
    return { account_id: id };
  }
  async claimUsername(username: string) {
    const handle = username.trim().toLowerCase();
    if (handle.length < 3) return { status: "rejected", username: handle };
    const existing = this.usernameStore.get(handle);
    if (existing === this.account) return { status: "already_owned", username: handle };
    if (existing) return { status: "taken", username: handle };
    this.usernameStore.set(handle, this.account);
    return { status: "created", username: handle };
  }
  async addByUsername(username: string) {
    const id = this.usernameStore.get(username.trim().toLowerCase());
    if (!id) throw new MessagingError("no such contact"); // unknown/unverifiable
    this.contacts.add(id);
    return { account_id: id, tofu: true };
  }
  /** test helper: queue an inbound message from `peer`. */
  inject(peer: string, text: string) {
    this.inbox.push({ direction: "incoming", peer, text, seq: 0 });
  }
  /** test helper: broker a code for `id` (as if another device minted it). */
  brokerFor(code: string, id: string) {
    this.pairingStore.set(code, id);
  }
  /** test helper: register a username binding (as if another account claimed it). */
  registerUsername(handle: string, id: string) {
    this.usernameStore.set(handle.toLowerCase(), id);
  }
}

const B = "bb".repeat(32);
const C = "cc".repeat(32);

describe("live conversations controller", () => {
  it("start loads the account id and is ready", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    expect(c.getState().status).toBe("ready");
    expect(c.getState().accountId).toBe(fake.account);
  });

  it("addPeer opens + activates a connected conversation", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    const s = c.getState();
    expect(s.activePeer).toBe(B);
    expect(s.conversations).toHaveLength(1);
    expect(s.conversations[0]).toMatchObject({ peer: B, connected: true });
  });

  it("addPeer refuses your own id", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(fake.account);
    expect(c.getState().conversations).toHaveLength(0);
    expect(c.getState().error).toMatch(/your own id/);
  });

  it("send goes to the active peer and shows from history", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await c.send("hi B");
    const conv = c.getState().conversations.find((x) => x.peer === B)!;
    expect(conv.messages.map((m) => m.text)).toContain("hi B");
    expect(conv.messages.find((m) => m.text === "hi B")?.direction).toBe("outgoing");
  });

  it("clearHistory empties messages but keeps the conversation; deleteConversation removes it", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await c.send("hi B");
    expect(c.getState().conversations.find((x) => x.peer === B)?.messages).toHaveLength(1);

    await c.clearHistory(B);
    const afterClear = c.getState().conversations.find((x) => x.peer === B);
    expect(afterClear).toBeDefined();
    expect(afterClear?.messages).toHaveLength(0);

    await c.deleteConversation(B);
    expect(c.getState().conversations.find((x) => x.peer === B)).toBeUndefined();
    expect(c.getState().activePeer).toBeNull();
  });

  it("safetyNumber returns the peer's digits", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    const sn = await c.safetyNumber(B);
    expect(sn?.digits).toHaveLength(60);
  });

  it("mute/block toggle controller state", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);

    await c.mute(B);
    expect(c.getState().muted).toContain(B);
    await c.unmute(B);
    expect(c.getState().muted).not.toContain(B);

    await c.block(B);
    expect(c.getState().blocked).toContain(B);
    await c.unblock(B);
    expect(c.getState().blocked).not.toContain(B);
  });

  it("deleteForEveryone removes the conversation from the list", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    expect(c.getState().conversations.find((x) => x.peer === B)).toBeDefined();
    await c.deleteForEveryone(B);
    expect(c.getState().conversations.find((x) => x.peer === B)).toBeUndefined();
  });

  it("setDisappearing updates the per-conversation timer state", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await c.setDisappearing(B, 86400);
    expect(c.getState().disappearing[B]).toBe(86400);
    await c.setDisappearing(B, 0);
    expect(c.getState().disappearing[B]).toBeUndefined();
  });

  it("poll routes inbound to the active conversation without unread", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    fake.inject(B, "yo from B");
    await c.poll();
    const conv = c.getState().conversations.find((x) => x.peer === B)!;
    expect(conv.messages.map((m) => m.text)).toContain("yo from B");
    expect(conv.unread).toBe(0);
  });

  it("poll auto-creates a conversation for a first-time sender and marks it unread", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B); // active = B
    fake.inject(C, "hello from a stranger");
    await c.poll();
    const conv = c.getState().conversations.find((x) => x.peer === C)!;
    expect(conv).toBeTruthy();
    expect(conv.messages.map((m) => m.text)).toContain("hello from a stranger");
    expect(conv.unread).toBe(1);
    expect(c.getState().activePeer).toBe(B); // unchanged
  });

  it("setActive clears unread", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    fake.inject(C, "ping");
    await c.poll();
    c.setActive(C);
    expect(c.getState().activePeer).toBe(C);
    expect(c.getState().conversations.find((x) => x.peer === C)!.unread).toBe(0);
  });

  it("logs real events as operations happen", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await c.send("hi");
    fake.inject(B, "yo");
    await c.poll();
    const kinds = c.getState().events.map((e) => e.kind);
    expect(kinds).toContain("ready");
    expect(kinds).toContain("open");
    expect(kinds).toContain("sent");
    expect(kinds).toContain("received");
    // newest-first, each with a timestamp
    expect(c.getState().events[0].t).toBeGreaterThan(0);
  });

  // Decisions fetch fire-and-forget after the action settles; flush the microtask/timer queue.
  const flush = () => new Promise((r) => setTimeout(r, 0));

  it("fetches the REAL decision views for the active peer (outbound reflects contact state)", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await flush();
    const d = c.getState().activeDecisions;
    expect(d).toBeTruthy();
    expect(d!.bootstrap.accepted).toBe(true);
    expect(d!.outbound.can_send).toBe(true); // B was added as a contact → real ACCEPT
  });

  it("clears decisions on peer switch and a non-contact peer yields a real REJECT (no fabricated accept)", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await flush();
    expect(c.getState().activeDecisions?.outbound.can_send).toBe(true);
    // A stranger C auto-creates a conversation we never added as a contact.
    fake.inject(C, "hi from stranger");
    await c.poll();
    c.setActive(C);
    // The prior peer's verdict is cleared in the SAME update as the switch (fail-closed null).
    expect(c.getState().activeDecisions).toBeNull();
    await flush();
    // C is not a contact → the REAL outbound view is a REJECT; the UI cannot show ACCEPT here.
    expect(c.getState().activeDecisions?.outbound.can_send).toBe(false);
    expect(c.getState().activeDecisions?.outbound.accepted).toBe(false);
  });

  it("a successful poll does NOT wipe a user send-failure banner (M4 provenance)", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    // A transient send failure raises an action-sourced banner the user must see.
    fake.failSend = "relay unreachable";
    await c.send("hi B");
    expect(c.getState().error).toMatch(/relay unreachable/);
    // A subsequent successful background poll must leave that banner intact.
    fake.failSend = null;
    await c.poll();
    expect(c.getState().error).toMatch(/relay unreachable/);
  });

  it("a successful poll clears a prior poll-sourced error (M4)", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    // A transient poll failure sets a poll-sourced banner.
    fake.failPoll = "transport failed";
    await c.poll();
    expect(c.getState().error).toMatch(/transport failed/);
    // The next successful poll clears its own stale error.
    fake.failPoll = null;
    await c.poll();
    expect(c.getState().error).toBeNull();
  });

  it("pairingCode mints a code; addByPairing opens the brokered contact once", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    // Another device brokered a code for peer B.
    fake.brokerFor("ZZZ-CODE", B);

    await c.addByPairing("ZZZ-CODE");
    const s = c.getState();
    expect(s.activePeer).toBe(B);
    expect(s.conversations.find((x) => x.peer === B)).toMatchObject({ connected: true });

    // Single-use: redeeming the same code again rejects (and adds nothing new).
    await expect(c.addByPairing("ZZZ-CODE")).rejects.toThrow();
  });

  it("addByPairing refuses an empty or own-account code", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await expect(c.addByPairing("   ")).rejects.toThrow(/Enter a pairing code/);
    // A code that brokers our OWN account is refused (you can't add yourself).
    const { code } = await c.pairingCode();
    await expect(c.addByPairing(code)).rejects.toThrow(/your own/);
  });

  it("claimUsername reports the registry outcome; addByUsername opens the resolved contact", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();

    expect((await c.claimUsername("Mine")).status).toBe("created");
    expect((await c.claimUsername("mine")).status).toBe("already_owned");
    expect((await c.claimUsername("ab")).status).toBe("rejected"); // too short

    // Another account owns "bob" -> resolve + open it.
    fake.registerUsername("bob", B);
    const { account_id, tofu } = await c.addByUsername("Bob");
    expect(account_id).toBe(B);
    expect(tofu).toBe(true);
    expect(c.getState().activePeer).toBe(B);

    // An unregistered handle fails closed (nothing added).
    await expect(c.addByUsername("ghost")).rejects.toThrow();
  });

  it("setVerified persists the flag and updates state both ways", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await c.setVerified(B, true);
    expect(c.getState().verified).toContain(B);
    expect([...fake.verifiedSet]).toContain(B);
    await c.setVerified(B, false);
    expect(c.getState().verified).not.toContain(B);
    expect(fake.verifiedSet.size).toBe(0);
  });

  it("poll refreshes silently-updated conversations without bumping unread", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await c.send("hi B");
    // Open a different conversation so B is NOT active (unread would normally bump).
    await c.addPeer(C);
    // The backend reports B's history changed (e.g. a delivery receipt) with NO new messages.
    fake.updatedPeers = [B];
    await c.poll();
    const conv = c.getState().conversations.find((x) => x.peer === B)!;
    expect(conv.unread).toBe(0); // silent refresh — never counted as a new arrival
    expect(conv.messages.map((m) => m.text)).toContain("hi B");
  });

  it("claimUsername records my handle; addByUsername records the peer alias", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.claimUsername("MyName");
    expect(c.getState().myUsername).toBe("myname");
    fake.registerUsername("bob", B);
    await c.addByUsername("Bob");
    expect(c.getState().aliases[B]).toBe("bob");
  });

  it("sendAttachment goes to the active peer and shows in history", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    await c.sendAttachment("photo.png", "image/png", "aGk=");
    const conv = c.getState().conversations.find((x) => x.peer === B)!;
    expect(conv.messages.some((m) => m.text.includes("photo.png"))).toBe(true);
  });

  it("createGroup opens the group conversation; send routes through groupSend", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await c.addPeer(B);
    const { group } = await c.createGroup("Team", [B]);
    const s = c.getState();
    expect(s.activePeer).toBe(group);
    expect(s.groups[group]?.name).toBe("Team");
    // The composer's send() must route group conversations through the MLS group path.
    await c.send("hello team");
    expect(fake.groupSends).toEqual([{ group, text: "hello team" }]);
    // Leaving drops the conversation + metadata.
    await c.leaveGroup(group);
    expect(c.getState().groups[group]).toBeUndefined();
    expect(c.getState().conversations.find((x) => x.peer === group)).toBeUndefined();
  });

  it("start hydrates existing group conversations from flags", async () => {
    const fake = new FakeMessaging();
    const gid = "ee".repeat(32);
    fake.groupMap.set(gid, { name: "Old Group", creator: fake.account, members: [fake.account], pending: [], ended: false });
    const c = createLiveConversations(fake);
    await c.start();
    expect(c.getState().groups[gid]?.name).toBe("Old Group");
    expect(c.getState().conversations.some((x) => x.peer === gid)).toBe(true);
  });

  it("backupExport enforces the passphrase minimum and returns the blob", async () => {
    const fake = new FakeMessaging();
    const c = createLiveConversations(fake);
    await c.start();
    await expect(c.backupExport("short")).rejects.toThrow(/at least 9/);
    expect(await c.backupExport("long enough passphrase")).toBe("ZmFrZS1iYWNrdXA=");
  });
});
