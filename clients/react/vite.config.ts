import { defineConfig } from 'vite'
import tailwindcss from "@tailwindcss/vite";
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
  ],
  server: {
    proxy: {
      // API endpoints
      '/api': {
        target: 'http://172.19.15.150:8000',
        changeOrigin: true,
        // WebSocket support
        ws: true,
      }
    },
    // Let dev server handle file-not-found with index.html
    // This is needed for SPA routing
    strictPort: true
  },
  // Explicitly set SPA mode
  appType: 'spa'
})
