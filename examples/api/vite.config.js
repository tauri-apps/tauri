// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig } from 'vite'
import Unocss from 'unocss/vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { internalIpV4 } from 'internal-ip'

// https://vitejs.dev/config/
export default defineConfig(async ({ command, mode }) => {
  const mobile =
    process.env.TAURI_PLATFORM === 'android' ||
    process.env.TAURI_PLATFORM === 'ios'

  return {
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
    server: {
      host: mobile ? '0.0.0.0' : 'localhost',
      port: 5173,
      strictPort: true,
      hmr: mobile
        ? {
            protocol: 'ws',
            host: await internalIpV4(),
            port: 5183
          }
        : undefined,
      fs: {
        allow: ['.', '../../tooling/api/dist']
      }
    }
  }
})
