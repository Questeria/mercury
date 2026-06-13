// Device-local CONNECTION-METHOD preferences: which ways of connecting the user wants surfaced.
// Persisted to localStorage like the theme/profile. These are UX preferences ONLY — they do NOT
// change Mercury's trust model (a card is always verified on add; the safety number is always the
// real backstop). Turning a method off just hides its button on the connect screen.

export interface ConnectionPrefs {
  /** Show the "Show QR" button on the connect screen (strongest, but needs proximity). */
  qr: boolean;
  /** Show the "Copy invite link" button (convenient, but a texted id/link can be swapped in transit). */
  link: boolean;
  /** Surface the safety-number verification reminder (the universal MITM backstop). */
  safetyVerify: boolean;
  /** Offer pairing codes (short, relay-brokered, single-use; still verify the safety number). */
  pairing: boolean;
  /** Offer usernames (key-transparency-backed handles; convenient, but trust the directory unless
   *  you verify the safety number). */
  username: boolean;
}

const KEY = "mercury.connectionPrefs";
const DEFAULTS: ConnectionPrefs = {
  qr: true,
  link: true,
  safetyVerify: true,
  pairing: true,
  username: true,
};

/** Read the saved preferences, falling back to all-on. Never throws. */
export function getConnectionPrefs(): ConnectionPrefs {
  if (typeof window === "undefined") return { ...DEFAULTS };
  try {
    const raw = window.localStorage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    const o = JSON.parse(raw) as Partial<ConnectionPrefs>;
    return {
      qr: o.qr ?? DEFAULTS.qr,
      link: o.link ?? DEFAULTS.link,
      safetyVerify: o.safetyVerify ?? DEFAULTS.safetyVerify,
      pairing: o.pairing ?? DEFAULTS.pairing,
      username: o.username ?? DEFAULTS.username,
    };
  } catch {
    return { ...DEFAULTS };
  }
}

/** Persist one preference and return the new full set. Never throws. */
export function setConnectionPref(key: keyof ConnectionPrefs, value: boolean): ConnectionPrefs {
  const next = { ...getConnectionPrefs(), [key]: value };
  try {
    if (typeof window !== "undefined") {
      window.localStorage.setItem(KEY, JSON.stringify(next));
      // Let other surfaces (e.g. the rail add-box) react live, without a reload.
      window.dispatchEvent(new CustomEvent("mercury-connprefs", { detail: next }));
    }
  } catch {
    /* persistence unavailable — keep the in-memory value */
  }
  return next;
}
