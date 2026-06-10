import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

// No update check on startup — updates are explicit/prompted only, via the in-app Updates panel
// (checkForUpdates detects, the user clicks to install). Nothing auto-installs silently.

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
