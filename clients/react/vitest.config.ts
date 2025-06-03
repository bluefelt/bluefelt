/// <reference types="vitest" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: [
      'src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'
    ],
    coverage: {
      reporter: ['text', 'json', 'html'],
      include: [
        'src/**/*.{ts,tsx}',
        '!src/**/*.{test,spec}.{ts,tsx}',
        '!src/test/**',
        '!src/vite-env.d.ts'
      ],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts'
      ]
    }
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, './src')
    }
  }
})