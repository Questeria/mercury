import { MercuryApp } from "./mercury/MercuryApp";
import { LiveMercuryApp } from "./mercury/LiveMercuryApp";
import { resolveBackendUrl } from "./mercury/liveConfig";
import { inTauri } from "./mercury/messaging";

export default function App() {
  // In the Tauri DESKTOP app the backend is the in-process AppController (no socket, keys never
  // leave the process) — run LIVE over its IPC bridge. In the browser, a `?backend=<url>` query
  // param or the VITE_MERCURY_BACKEND env var points at a dev shim and runs LIVE over HTTP;
  // otherwise fall back to the decision-view simulator demo.
  if (inTauri()) return <LiveMercuryApp backendUrl="in-process (Tauri)" />;
  const backend = resolveBackendUrl();
  return backend ? <LiveMercuryApp backendUrl={backend} /> : <MercuryApp />;
}
