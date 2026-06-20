import { describe, expect, it } from "vitest";

import { previewText } from "./conversationPreview";
import type { ChatAttachment, ChatMessage } from "./messaging";

const ATT: ChatAttachment = { name: "photo.jpg", mime: "image/jpeg", data_b64: "" };

function msg(over: Partial<ChatMessage>): ChatMessage {
  return { direction: "incoming", peer: "p", text: "", seq: 0, ...over };
}

describe("previewText", () => {
  it("uses the text caption when present", () => {
    expect(previewText(msg({ text: "hello there" }))).toBe("hello there");
  });

  it("shows the attachment name for a caption-less attachment (not an empty thread)", () => {
    expect(previewText(msg({ text: "", att: ATT }))).toBe("📎 photo.jpg");
  });

  it("a text caption still wins over an attachment", () => {
    expect(previewText(msg({ text: "see this", att: ATT }))).toBe("see this");
  });

  it("is empty for a genuinely empty message", () => {
    expect(previewText(msg({ text: "" }))).toBe("");
  });
});
