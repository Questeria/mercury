import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    // Proxy real-core decisions to the mercury-ui-bridge dev shim
    // (cargo run -p mercury-ui-bridge, default port 7878), so the client can
    // fetch them same-origin. Selected via the DevControls "decisions" toggle.
    proxy: {
      '/api/bridge': {
        target: 'http://127.0.0.1:7878',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/bridge/, ''),
      },
    },
  },
})
