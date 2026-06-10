// Invite links + id parsing — make connecting easy WITHOUT changing the trust model.
//
// An account id is a self-authenticating hash of the identity public key
// (`account_id_from_identity_key`): a contact card can only be published to the slot its own
// signing key hashes to, and the card is cryptographically verified on add. So a link or QR is just
// a friendlier TRANSPORT of the same 32 bytes — it grants no new trust, and a tampered/wrong link
// can at worst resolve to "no such contact" (the card fetch fails closed), never a silent MITM.
//
// The web invite link carries the id in the URL **fragment** (`#`), which browsers never send to
// the server — so sharing/clicking an invite does not leak the id into Cloudflare/relay request
// logs. (It is also why the static /add page reads the id client-side from `location.hash`.)

/** Host page that turns an invite into an "open in Mercury / copy id" hand-off. */
const INVITE_WEB_BASE = "https://mercury-messaging.com/add/#";
/** Custom-scheme deep link that opens the installed desktop app directly (when registered). */
const INVITE_SCHEME_BASE = "mercury://add/";

/** Exactly 64 lowercase hex chars = a 32-byte account id. */
const ACCOUNT_ID_RE = /^[0-9a-f]{64}$/;

/** Whether `s` is already a bare, valid lowercase 64-hex account id. */
export function isAccountId(s: string): boolean {
  return ACCOUNT_ID_RE.test(s);
}

/** A shareable web invite link for `accountId`. The id rides in the URL fragment, so it is never
 *  sent to the server. Assumes `accountId` is a valid id (caller validates). */
export function inviteLink(accountId: string): string {
  return INVITE_WEB_BASE + accountId.trim().toLowerCase();
}

/** The deep-link form that opens the desktop app directly (when the `mercury://` scheme is
 *  registered). Used by the web hand-off page's "Open in Mercury" button. */
export function inviteDeepLink(accountId: string): string {
  return INVITE_SCHEME_BASE + accountId.trim().toLowerCase();
}

/** Extract a 64-hex account id from user input — a bare id, a `mercury://add/<id>` deep link, or an
 *  `https://…/add/#<id>` (or `/add/<id>`) web link. Case-insensitive; tolerant of surrounding
 *  whitespace, query, and fragment markers. Returns the lowercase id, or `null` if the input does
 *  not contain exactly one standalone 64-hex run.
 *
 *  Fail-closed + unambiguous: the id must be bounded by a non-hex char or a string end, so a longer
 *  hex blob can never yield a wrong 64-char slice. This is the trust-relevant boundary, but note the
 *  fetched card is still verified against whatever id this returns — a bad parse degrades to
 *  "contact not found", never a security bypass. */
export function parseInviteOrId(input: string): string | null {
  const s = (input ?? "").trim().toLowerCase();
  if (!s) return null;
  const match = s.match(/(?:^|[^0-9a-f])([0-9a-f]{64})(?:[^0-9a-f]|$)/);
  return match ? match[1] : null;
}

/** Strict parser for an OPENED link/scheme URL (deep link or web invite): it requires an explicit
 *  `add` action token, so only a real `mercury://add/<id>` (or `…/add/…#<id>`) invite triggers a
 *  contact add. A future `mercury://<other-verb>/<id>` is NOT silently treated as add. Returns the
 *  id, or null if the URL isn't an add-invite or has no valid id. Use this for auto-actions
 *  (deep-link handling); use [`parseInviteOrId`] for the manual paste field (which also accepts a
 *  bare id). */
export function inviteActionId(url: string): string | null {
  const s = (url ?? "").trim().toLowerCase();
  // Tokenize on scheme/path/query/fragment separators and require an exact `add` token, e.g.
  // ["mercury","add","<id>"] or ["https","mercury-messaging.com","add","<id>"].
  const tokens = s.split(/[:/#?]+/).filter(Boolean);
  if (!tokens.includes("add")) return null;
  return parseInviteOrId(s);
}
