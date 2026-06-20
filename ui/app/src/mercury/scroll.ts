// Pure helpers for the message-list auto-scroll behavior, split out so the threshold logic is
// unit-testable without a DOM (the live app has no jsdom in tests). The component reads the live
// `scrollHeight`/`scrollTop`/`clientHeight` off the element and passes them in.

/**
 * Distance-from-bottom (px) within which the message list counts as "at the bottom": the user is
 * reading the newest messages, so an arriving or just-sent message should auto-scroll into view.
 * Once the user has scrolled further up than this — i.e. they are reading history — auto-scroll must
 * NOT fire, or it would yank them back down on every new message.
 */
export const STICK_TO_BOTTOM_THRESHOLD_PX = 80;

/** True iff the list is scrolled to within {@link STICK_TO_BOTTOM_THRESHOLD_PX} of the bottom. */
export function isAtBottom(
  scrollHeight: number,
  scrollTop: number,
  clientHeight: number,
): boolean {
  return scrollHeight - scrollTop - clientHeight < STICK_TO_BOTTOM_THRESHOLD_PX;
}
