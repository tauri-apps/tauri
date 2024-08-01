// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig } from 'vite'
import Unocss from 'unocss/vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { internalIpV4Sync } from 'internal-ip'

const host = process.env.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [Unocss(), svelte()],
  build: {
    rollupOptions: {
      output: {
        entryFileNames: `assets/[name].js`,
        chunkFileNames: `assets/[name].js`,
        assetFileNames: `assets/[name].[ext]`
      }
    }
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    host: host ? '0.0.0.0' : false,
    port: 1420,
    strictPort: true,
    hmr: host
      ? {
          protocol: 'ws',
          host: internalIpV4Sync(),
          port: 1430
        }
      : undefined,
    fs: {
      allow: ['.', '../../tooling/api/dist']
    }
  }
})
