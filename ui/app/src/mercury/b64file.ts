// Small file <-> base64 helpers shared by the live attachment + encrypted-backup flows.
// No crypto here: callers hand sealed/plain bytes around as base64; the backend and the Rust
// shell do the real work. Desktop saves go through the `save_file` Tauri command (Downloads
// folder); the browser fallback is a plain Blob download.

import { inTauri } from "./messaging";

/** Read a picked File as raw base64 (no `data:` prefix). Rejects on a read failure. */
export function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(new Error(`could not read "${file.name}"`));
    reader.onload = () => {
      const url = String(reader.result ?? "");
      const comma = url.indexOf(",");
      if (comma === -1) {
        reject(new Error(`could not read "${file.name}"`));
        return;
      }
      resolve(url.slice(comma + 1));
    };
    reader.readAsDataURL(file);
  });
}

/** Desktop only: write `dataB64` as `name` into the user's Downloads folder via the Rust shell.
 *  Returns the saved path. Throws outside Tauri or on a shell error. */
export async function saveToDownloads(name: string, dataB64: string): Promise<string> {
  if (!inTauri()) throw new Error("saving to disk needs the desktop app");
  const { invoke } = await import("@tauri-apps/api/core");
  return await invoke<string>("save_file", { name, dataB64 });
}

/** Browser fallback: trigger a download of base64 bytes via a temporary Blob URL. */
export function downloadViaBlob(name: string, mime: string, dataB64: string): void {
  const bin = atob(dataB64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  const url = URL.createObjectURL(new Blob([bytes], { type: mime || "application/octet-stream" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 5_000);
}
