import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { getConnectionPrefs, setConnectionPref } from "./connectionPrefs";

// connectionPrefs guards on `typeof window`; provide a minimal window+localStorage in the node env
// (no jsdom dependency) so the persistence path is exercised.
const store = new Map<string, string>();

beforeEach(() => {
  store.clear();
  vi.stubGlobal("window", {
    localStorage: {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => void store.set(k, v),
      removeItem: (k: string) => void store.delete(k),
    },
  });
});

afterEach(() => vi.unstubAllGlobals());

const ALL_ON = {
  qr: true,
  link: true,
  safetyVerify: true,
  pairing: true,
  username: true,
};

describe("connection prefs", () => {
  it("defaults to all methods on", () => {
    expect(getConnectionPrefs()).toEqual(ALL_ON);
  });

  it("persists a single toggle without disturbing the others", () => {
    const next = setConnectionPref("qr", false);
    expect(next.qr).toBe(false);
    expect(getConnectionPrefs()).toEqual({ ...ALL_ON, qr: false });
  });

  it("persists the new pairing + username toggles independently", () => {
    setConnectionPref("pairing", false);
    expect(getConnectionPrefs()).toEqual({ ...ALL_ON, pairing: false });
    setConnectionPref("username", false);
    expect(getConnectionPrefs()).toEqual({ ...ALL_ON, pairing: false, username: false });
  });

  it("survives a corrupt stored value (falls back to defaults)", () => {
    store.set("mercury.connectionPrefs", "{not json");
    expect(getConnectionPrefs()).toEqual(ALL_ON);
  });
});
