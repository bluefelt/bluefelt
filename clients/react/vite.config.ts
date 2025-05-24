import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // API endpoints
      '/api/lobbies': {
        target: 'http://localhost:8000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
      '/api/games': {
        target: 'http://localhost:8000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
      // WebSocket connections - special handling
      '/lobbies': {
        target: 'ws://localhost:8000',
        ws: true,
        changeOrigin: true
      }
    },
    // Let dev server handle file-not-found with index.html
    // This is needed for SPA routing
    strictPort: true
  },
  // Explicitly set SPA mode
  appType: 'spa'
})
