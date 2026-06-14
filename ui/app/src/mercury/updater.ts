// In-app auto-update — PROMPTED, never silent. `checkForUpdates()` only DETECTS a newer signed
// build; it does not install. The app DOES check automatically (on launch + periodically, via
// components/UpdateBanner.tsx) and also from the Updates panel, but INSTALLING happens solely via
// `downloadAndInstallUpdate()` on an explicit user click — the binary is never replaced silently.

import { inTauri } from "./messaging";

export const UPDATE_MANIFEST_URL = "https://mercury-messaging.com/updates/latest.json" as const;
export const DOWNLOAD_URL = "https://mercury-messaging.com/download/Mercury-Setup-Windows-x64.exe" as const;

export type UpdateCheckState =
  | "browser"
  | "checking"
  | "current"
  | "available"
  | "dormant"
  | "error";

export interface UpdateCheckResult {
  state: UpdateCheckState;
  title: string;
  detail: string;
  version?: string;
  manifestUrl: string;
}

export interface UpdateManifestProbe {
  state: "available" | "dormant" | "error";
  title: string;
  detail: string;
  version?: string;
  manifestUrl: string;
}

export async function checkForUpdates(): Promise<UpdateCheckResult> {
  if (!inTauri()) {
    return {
      state: "browser",
      title: "Desktop updater unavailable in browser",
      detail: "The signed updater check only runs inside the Mercury Windows app.",
      manifestUrl: UPDATE_MANIFEST_URL,
    };
  }

  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (update) {
      return {
        state: "available",
        title: `Update available — v${update.version}`,
        detail: "A newer signed build is ready. Use “Install & restart” to apply it.",
        version: update.version,
        manifestUrl: UPDATE_MANIFEST_URL,
      };
    }
    return {
      state: "current",
      title: "Mercury is current",
      detail: "The updater checked the signed manifest and found no newer version.",
      manifestUrl: UPDATE_MANIFEST_URL,
    };
  } catch (e) {
    return {
      state: "dormant",
      title: "Update channel dormant",
      detail: readableUpdateError(e),
      manifestUrl: UPDATE_MANIFEST_URL,
    };
  }
}

/** Explicit, user-initiated install. Re-checks, then downloads + installs the signed update (applies on
 *  next launch). Only ever called from a user click in the Updates panel — never on startup. */
export async function downloadAndInstallUpdate(): Promise<UpdateCheckResult> {
  if (!inTauri()) {
    return {
      state: "browser",
      title: "Desktop updater unavailable in browser",
      detail: "Updates install only inside the Mercury Windows app.",
      manifestUrl: UPDATE_MANIFEST_URL,
    };
  }
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (!update) {
      return {
        state: "current",
        title: "Mercury is current",
        detail: "No newer signed version is available to install.",
        manifestUrl: UPDATE_MANIFEST_URL,
      };
    }
    await update.downloadAndInstall();
    return {
      state: "available",
      title: "Update installed",
      detail: "Restart Mercury to finish applying the signed update.",
      version: update.version,
      manifestUrl: UPDATE_MANIFEST_URL,
    };
  } catch (e) {
    return {
      state: "error",
      title: "Update install failed",
      detail: readableUpdateError(e),
      manifestUrl: UPDATE_MANIFEST_URL,
    };
  }
}

/** Quit and restart the app so a just-installed update takes effect. No-op in the browser. The
 *  caller is responsible for a loop guard (only relaunch once per version) — see UpdateBanner. */
export async function relaunchApp(): Promise<void> {
  if (!inTauri()) return;
  try {
    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
  } catch {
    /* relaunch unavailable — the staged update will apply on the next manual restart instead */
  }
}

export async function probeUpdateManifest(): Promise<UpdateManifestProbe> {
  try {
    const res = await fetch(UPDATE_MANIFEST_URL, { cache: "no-store" });
    if (res.status === 404) {
      return {
        state: "dormant",
        title: "No update manifest hosted",
        detail: "Manual website downloads are the active release path until latest.json is published.",
        manifestUrl: UPDATE_MANIFEST_URL,
      };
    }
    if (!res.ok) {
      return {
        state: "error",
        title: "Manifest check failed",
        detail: `The update endpoint returned HTTP ${res.status}.`,
        manifestUrl: UPDATE_MANIFEST_URL,
      };
    }
    const manifest = (await res.json()) as { version?: unknown };
    return {
      state: "available",
      title: "Update manifest is live",
      detail: "Installed desktop apps can poll this signed update channel.",
      version: typeof manifest.version === "string" ? manifest.version : undefined,
      manifestUrl: UPDATE_MANIFEST_URL,
    };
  } catch (e) {
    return {
      state: "error",
      title: "Manifest check unavailable",
      detail: readableUpdateError(e),
      manifestUrl: UPDATE_MANIFEST_URL,
    };
  }
}

function readableUpdateError(e: unknown): string {
  const text = e instanceof Error ? e.message : String(e);
  if (/404|not found/i.test(text)) {
    return "No latest.json is hosted yet, so the app falls back to manual signed downloads.";
  }
  if (/network|failed to fetch|offline/i.test(text)) {
    return "The update endpoint could not be reached from this session.";
  }
  return text || "The update check did not complete.";
}
