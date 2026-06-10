import { describe, expect, it } from "vitest";

import { resolveBackendUrl } from "./liveConfig";

describe("resolveBackendUrl", () => {
  it("prefers the ?backend= query param over the env var", () => {
    expect(resolveBackendUrl("?backend=http://127.0.0.1:7880", "http://env")).toBe(
      "http://127.0.0.1:7880",
    );
  });

  it("falls back to the env var when no query param", () => {
    expect(resolveBackendUrl("", "http://127.0.0.1:7879")).toBe("http://127.0.0.1:7879");
  });

  it("returns null when neither is set (demo mode)", () => {
    expect(resolveBackendUrl("", undefined)).toBeNull();
    expect(resolveBackendUrl("", "")).toBeNull();
    expect(resolveBackendUrl("?other=1", "")).toBeNull();
  });

  it("trims surrounding whitespace", () => {
    expect(resolveBackendUrl("", "  http://x  ")).toBe("http://x");
    expect(resolveBackendUrl("?backend=%20http://y%20", undefined)).toBe("http://y");
  });
});
