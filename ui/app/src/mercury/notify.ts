// OS notifications for newly-arrived messages — desktop app only. The CALLER decides WHICH arrivals
// warrant a notification (e.g. skip the conversation you're actively looking at, notify for the
// rest); this module only handles the Tauri permission + send. No-ops in the browser. Privacy: the
// notification carries only the (shortened) sender id + a count — never message plaintext.

import { inTauri } from "./messaging";

const ENABLED_KEY = "mercury.notificationsEnabled";

/** The user's MASTER notification toggle (Notifications panel). Default: on. */
export function notificationsEnabled(): boolean {
  try {
    return window.localStorage.getItem(ENABLED_KEY) !== "0";
  } catch {
    return true;
  }
}

/** Persist the master notification toggle. */
export function setNotificationsEnabled(on: boolean): void {
  try {
    window.localStorage.setItem(ENABLED_KEY, on ? "1" : "0");
  } catch {
    /* storage unavailable — the in-session default applies */
  }
}

let permission: Promise<boolean> | null = null;

async function ensurePermission(): Promise<boolean> {
  if (!inTauri()) return false;
  if (!permission) {
    permission = (async () => {
      try {
        const { isPermissionGranted, requestPermission } = await import("@tauri-apps/plugin-notification");
        let granted = await isPermissionGranted();
        if (!granted) granted = (await requestPermission()) === "granted";
        return granted;
      } catch {
        return false;
      }
    })();
  }
  return permission;
}

/** Raise an OS notification for `count` newly-arrived messages from `peerLabel`. Skips entirely in
 *  the browser. Focus / active-conversation scoping is the caller's responsibility. */
export async function notifyIncoming(count: number, peerLabel: string): Promise<void> {
  if (count <= 0 || !inTauri()) return;
  if (!notificationsEnabled()) return; // master toggle (Notifications panel) — user-controlled
  if (!(await ensurePermission())) return;
  try {
    const { sendNotification } = await import("@tauri-apps/plugin-notification");
    sendNotification({
      title: "Mercury",
      body:
        count === 1
          ? `New encrypted message from ${peerLabel}`
          : `${count} new encrypted messages`,
    });
  } catch {
    /* notification plugin unavailable — ignore */
  }
}
