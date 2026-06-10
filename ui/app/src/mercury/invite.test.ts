import { describe, expect, it } from "vitest";

import { inviteActionId, inviteDeepLink, inviteLink, isAccountId, parseInviteOrId } from "./invite";

const ID = "aa".repeat(32); // 64 lowercase hex
const ID2 = "bb".repeat(32);

describe("parseInviteOrId", () => {
  it("accepts a bare account id", () => {
    expect(parseInviteOrId(ID)).toBe(ID);
  });

  it("lowercases an uppercase id", () => {
    expect(parseInviteOrId("AA".repeat(32))).toBe(ID);
  });

  it("trims surrounding whitespace", () => {
    expect(parseInviteOrId(`  ${ID}\n`)).toBe(ID);
  });

  it("extracts the id from a mercury:// deep link", () => {
    expect(parseInviteOrId(`mercury://add/${ID}`)).toBe(ID);
  });

  it("extracts the id from a web invite link (hash form)", () => {
    expect(parseInviteOrId(`https://mercury-messaging.com/add/#${ID}`)).toBe(ID);
  });

  it("extracts the id from a web invite link (path form)", () => {
    expect(parseInviteOrId(`https://mercury-messaging.com/add/${ID}`)).toBe(ID);
  });

  it("extracts the id from a link with a trailing query", () => {
    expect(parseInviteOrId(`https://mercury-messaging.com/add/#${ID}?ref=x`)).toBe(ID);
  });

  it("rejects a too-short id (63 hex)", () => {
    expect(parseInviteOrId("a".repeat(63))).toBeNull();
  });

  it("rejects an ambiguous too-long hex run (65 hex)", () => {
    expect(parseInviteOrId("a".repeat(65))).toBeNull();
  });

  it("rejects non-hex input", () => {
    expect(parseInviteOrId("not-an-id")).toBeNull();
    expect(parseInviteOrId(`${"g".repeat(64)}`)).toBeNull();
    expect(parseInviteOrId("")).toBeNull();
  });

  it("returns the FIRST id when two separated ids are present (intentional first-wins)", () => {
    expect(parseInviteOrId(`${ID} ${ID2}`)).toBe(ID);
  });
});

describe("inviteActionId (strict, deep-link)", () => {
  it("accepts an explicit add deep link", () => {
    expect(inviteActionId(`mercury://add/${ID}`)).toBe(ID);
  });
  it("accepts an explicit add web link", () => {
    expect(inviteActionId(`https://mercury-messaging.com/add/#${ID}`)).toBe(ID);
  });
  it("rejects a non-add verb even with a valid id (no silent add)", () => {
    expect(inviteActionId(`mercury://evil/${ID}`)).toBeNull();
    expect(inviteActionId(`mercury://join/${ID}`)).toBeNull();
  });
  it("rejects a bare id (it is not an opened add-invite URL)", () => {
    expect(inviteActionId(ID)).toBeNull();
  });
  it("rejects an add URL with no valid id", () => {
    expect(inviteActionId("mercury://add/not-hex")).toBeNull();
  });
});

describe("isAccountId", () => {
  it("accepts exactly 64 lowercase hex", () => {
    expect(isAccountId(ID)).toBe(true);
  });
  it("rejects wrong length / case / chars", () => {
    expect(isAccountId("AA".repeat(32))).toBe(false); // uppercase not pre-normalized
    expect(isAccountId("a".repeat(63))).toBe(false);
    expect(isAccountId(`mercury://add/${ID}`)).toBe(false);
  });
});

describe("invite link builders", () => {
  it("builds a hash-fragment web link (id never sent to the server)", () => {
    expect(inviteLink(ID)).toBe(`https://mercury-messaging.com/add/#${ID}`);
    // round-trips back through the parser
    expect(parseInviteOrId(inviteLink(ID))).toBe(ID);
  });
  it("builds a deep link that round-trips", () => {
    expect(inviteDeepLink(ID)).toBe(`mercury://add/${ID}`);
    expect(parseInviteOrId(inviteDeepLink(ID))).toBe(ID);
  });
});
