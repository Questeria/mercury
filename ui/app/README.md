# Mercury — frontend (React + TypeScript + Vite)

This is the Mercury UI. It runs in two shells behind the **same** React app:

- **Desktop app (Tauri v2)** — the real client. The UI talks to the in-process
  `mercury_app::AppController` over Tauri IPC; keys and ciphertext never leave the process.
  Build/run: [`../../docs/RUNNING-THE-DESKTOP-APP.md`](../../docs/RUNNING-THE-DESKTOP-APP.md).
- **Browser dev demo** — the same UI over HTTP to a `mercury-app-server` shim (keys live in the
  shim), selected with `?backend=<url>`; with no backend it falls back to a decision-view simulator.
  See [`../../docs/RUNNING-THE-LIVE-DEMO.md`](../../docs/RUNNING-THE-LIVE-DEMO.md).

## Layout

| Path | What |
|------|------|
| `src/mercury/LiveMercuryApp.tsx` | The live chat app — used by the desktop build **and** the HTTP dev demo. |
| `src/mercury/MercuryApp.tsx` | The decision-view simulator demo (browser, no backend). |
| `src/mercury/components/` | `Avatar`, `MercuryLogo` (brand mark), `TitleBar`, `Composer`, … |
| `src/mercury/messaging.ts` | `MercuryMessaging` transport — `createTauriMessaging()` (IPC) / `createHttpMessaging(url)` (dev), plus `inTauri()`. |
| `src/mercury/theme.css` | Theme tokens (light/dark/auto) + the animated background. |
| `src-tauri/` | The Tauri v2 desktop shell (Rust crate `mercury-desktop`). Excluded from the Cargo workspace. |

## Commands

```sh
npm install        # first time only
npm run dev        # browser dev server (Vite)
npm run build      # type-check (tsc -b) + production build → dist/
npx tauri dev      # desktop app: native window, hot-reloads the UI
npx tauri build    # desktop installer (see the desktop-app doc)
```

## Brand assets

The in-app brand mark is `src/mercury/assets/mercury-logo.png` rendered through
`src/mercury/components/MercuryLogo.tsx`. The desktop **app icon** source is
`src-tauri/app-icon.png`; regenerate every bundled icon size with
`npx tauri icon src-tauri/app-icon.png`, then refresh the app/site logo copies from the tighter Mercury logo crop.
