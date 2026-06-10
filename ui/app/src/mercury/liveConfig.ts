// Resolve the live backend (mercury-app-server) URL the UI should drive.
//
// When this returns a URL, the app runs LIVE — real end-to-end-encrypted messaging over that
// `mercury-app-server` dev shim. When it returns null, the app falls back to the decision-view
// simulator demo (`MercuryApp`). This is the switch that "replaces the simulator" without
// deleting it.
//
// Resolution order:
//   1. `?backend=<url>` query param — lets two browser tabs point at two different dev shims
//      (e.g. Alice on :7879, Bob on :7880) for the two-window end-to-end test.
//   2. `VITE_MERCURY_BACKEND` env var (build/dev-time default).
//   3. null -> demo mode.
//
// Args are injectable so this is unit-testable without a real `window`/`import.meta`.

export function resolveBackendUrl(
  search: string = typeof window !== "undefined" ? window.location.search : "",
  env: string | undefined = import.meta.env.VITE_MERCURY_BACKEND as string | undefined,
): string | null {
  const fromQuery = new URLSearchParams(search).get("backend");
  if (fromQuery && fromQuery.trim()) return fromQuery.trim();
  if (env && env.trim()) return env.trim();
  return null;
}
