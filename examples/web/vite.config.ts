// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { resolve } from 'path'
import { sveltekit } from '@sveltejs/kit/vite'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import { viteStaticCopy } from 'vite-plugin-static-copy'
import type { UserConfig } from 'vite'

const TARGET = process.env.TARGET

const plugins = [sveltekit()]

if (TARGET === 'web') {
  plugins.push(wasm())
  plugins.push(topLevelAwait())
  plugins.push(
    viteStaticCopy({
      targets: [
        {
          src: 'core/wasm/pkg/wasm_bg.wasm',
          dest: 'wasm'
        }
      ]
    })
  )
}

const config: UserConfig = {
  server: {
    fs: {
      // Allow serving the wasm file from this folder.
      allow: ['.']
    }
  },
  plugins,
  resolve: {
    alias: {
      $api:
        TARGET === 'tauri'
          ? resolve('./src/api/desktop')
          : resolve('./src/api/web')
    }
  }
}

export default config
