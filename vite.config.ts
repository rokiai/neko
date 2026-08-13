import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('src', import.meta.url)),
      '@shared': fileURLToPath(new URL('src/shared', import.meta.url))
    }
  },
  build: {
    outDir: fileURLToPath(new URL('dist', import.meta.url)),
    emptyOutDir: true,
    // The webview is known ahead of time, so skip transpiling down to a
    // browser baseline this app never runs on. Matches Tauri's own defaults.
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    sourcemap: false,
    // The Settings chunk is antd-sized by nature and loads once from local
    // disk. The entry chunk that Break windows load is the one to watch, and
    // it sits well under this.
    chunkSizeWarningLimit: 700
  },
  server: {
    port: 1420,
    strictPort: true
  }
})
