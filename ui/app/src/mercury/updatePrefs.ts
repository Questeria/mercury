// Device-local UPDATE preference: whether Mercury installs a newer signed build automatically at
// launch. Default ON. When ON, the launch check downloads + installs the signed update in the
// background (it applies on the next restart). When OFF, the app still DETECTS updates and shows the
// prompt banner, but installing stays a manual click. Persisted to localStorage like the other prefs.
//
// This only changes WHEN the install is triggered (auto vs a click) — never WHAT is trusted: the
// Tauri updater still verifies the minisign signature over the final installer bytes before applying,
// auto or not. The binary is never replaced by anything unsigned.

const KEY = "mercury.autoUpdate";

/** Read the saved preference, defaulting to ON. Never throws. */
export function getAutoUpdate(): boolean {
  if (typeof window === "undefined") return true;
  try {
    const raw = window.localStorage.getItem(KEY);
    if (raw === null) return true; // default: auto-update at launch
    return raw === "1";
  } catch {
    return true;
  }
}

/** Persist the preference and return it. Never throws. */
export function setAutoUpdate(value: boolean): boolean {
  try {
    if (typeof window !== "undefined") {
      window.localStorage.setItem(KEY, value ? "1" : "0");
      window.dispatchEvent(new CustomEvent("mercury-autoupdate", { detail: value }));
    }
  } catch {
    /* persistence unavailable — keep the in-memory value */
  }
  return value;
}
