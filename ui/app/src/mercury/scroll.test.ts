import { describe, expect, it } from "vitest";

import { isAtBottom, STICK_TO_BOTTOM_THRESHOLD_PX } from "./scroll";

describe("isAtBottom", () => {
  it("is true at the bottom and within the threshold", () => {
    expect(isAtBottom(1000, 900, 100)).toBe(true); // gap 0 — exactly at the bottom
    expect(isAtBottom(1000, 850, 100)).toBe(true); // gap 50
    expect(isAtBottom(1000, 821, 100)).toBe(true); // gap 79 — just inside the threshold
  });

  it("is false once scrolled up beyond the threshold (reading history)", () => {
    expect(isAtBottom(1000, 820, 100)).toBe(false); // gap 80 — not < 80
    expect(isAtBottom(1000, 0, 100)).toBe(false); // gap 900 — at the top
  });

  it("uses an 80px threshold", () => {
    expect(STICK_TO_BOTTOM_THRESHOLD_PX).toBe(80);
  });
});
