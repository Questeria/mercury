// Launch-at-login control (desktop app only). Wraps the Tauri autostart plugin; no-ops in the
// browser. When enabled, the OS launches Mercury with `--minimized` at sign-in, and the Rust shell
// starts it hidden in the tray (see src-tauri/src/main.rs) so it receives without popping a window.

import { inTauri } from "./messaging";

export async function isAutostartEnabled(): Promise<boolean> {
  if (!inTauri()) return false;
  try {
    const { isEnabled } = await import("@tauri-apps/plugin-autostart");
    return await isEnabled();
  } catch {
    return false;
  }
}

/** Enable/disable launch-at-login; returns the resulting state (false if unsupported/failed). */
export async function setAutostartEnabled(on: boolean): Promise<boolean> {
  if (!inTauri()) return false;
  try {
    const mod = await import("@tauri-apps/plugin-autostart");
    if (on) await mod.enable();
    else await mod.disable();
    return await mod.isEnabled();
  } catch {
    return false;
  }
}
