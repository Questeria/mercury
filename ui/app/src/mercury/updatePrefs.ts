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

// --- Update diagnostics -----------------------------------------------------------------------
// Every check/install records its outcome here so the Updates panel can show exactly what the
// updater last did (e.g. "found 0.1.42 but install failed: <reason>"). This is what makes a
// silent auto-update FAILURE visible + reportable instead of invisible — critical for a tray app
// where the user never sees the transient launch banner.

const LOG_KEY = "mercury.updateLog";

export interface UpdateLogEntry {
  at: number; // epoch ms
  kind: string; // "launch" | "focus" | "periodic" | "<kind>-install"
  state: string;
  detail: string;
  version?: string;
}

/** Append an update event (keeps the last 8). Never throws. */
export function recordUpdateResult(entry: UpdateLogEntry): void {
  try {
    if (typeof window === "undefined") return;
    const raw = window.localStorage.getItem(LOG_KEY);
    const log: UpdateLogEntry[] = raw ? JSON.parse(raw) : [];
    log.push(entry);
    window.localStorage.setItem(LOG_KEY, JSON.stringify(log.slice(-8)));
  } catch {
    /* ignore */
  }
}

/** Read the recorded update events, newest last. Never throws. */
export function getUpdateLog(): UpdateLogEntry[] {
  try {
    if (typeof window === "undefined") return [];
    const raw = window.localStorage.getItem(LOG_KEY);
    return raw ? (JSON.parse(raw) as UpdateLogEntry[]) : [];
  } catch {
    return [];
  }
}

// --- Auto-relaunch loop guard -----------------------------------------------------------------
// Before quitting+relaunching to apply an update we record the target version + time. If, after the
// relaunch, the SAME version is STILL offered (the install didn't actually apply), we must NOT
// relaunch again — that would be an infinite restart loop. So we relaunch at most once per version
// per RELAUNCH_GUARD_MS; if it didn't take, we fall back to a manual-install prompt.

const RELAUNCH_KEY = "mercury.updateRelaunch";
export const RELAUNCH_GUARD_MS = 5 * 60 * 1000;

export function setRelaunchAttempt(version: string): void {
  try {
    if (typeof window !== "undefined") {
      window.localStorage.setItem(RELAUNCH_KEY, JSON.stringify({ version, at: Date.now() }));
    }
  } catch {
    /* ignore */
  }
}

/** The version we last auto-relaunched for, if it was within the guard window; else null. */
export function getRecentRelaunchVersion(): string | null {
  try {
    if (typeof window === "undefined") return null;
    const raw = window.localStorage.getItem(RELAUNCH_KEY);
    if (!raw) return null;
    const o = JSON.parse(raw) as { version?: string; at?: number };
    if (o.version && typeof o.at === "number" && Date.now() - o.at < RELAUNCH_GUARD_MS) {
      return o.version;
    }
    return null;
  } catch {
    return null;
  }
}

export function clearRelaunchAttempt(): void {
  try {
    if (typeof window !== "undefined") window.localStorage.removeItem(RELAUNCH_KEY);
  } catch {
    /* ignore */
  }
}
