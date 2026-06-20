import type { ChatMessage } from "./messaging";

/**
 * The one-line preview shown for a conversation in the rail. A text caption wins; an attachment with
 * no caption shows its file name (so a photo/file-only last message is not mistaken for an empty
 * thread and rendered as "no messages yet"); a genuinely empty message yields "".
 */
export function previewText(message: ChatMessage): string {
  if (message.text) return message.text;
  if (message.att) return `📎 ${message.att.name}`;
  return "";
}
