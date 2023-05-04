// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig } from 'tsup'

export default defineConfig(() => [
  {
    entry: ['src/*.ts'],
    outDir: 'dist',
    format: ['esm', 'cjs'],
    clean: true,
    minify: true,
    platform: 'browser',
    dts: {
      resolve: true
    }
  },
  {
    entry: { bundle: 'src/index.ts' },
    outDir: '../../core/tauri/scripts',
    format: ['iife'],
    globalName: '__TAURI_IIFE__',
    clean: false,
    minify: true,
    platform: 'browser',
    dts: false,
    // esbuild `globalName` option generates `var __TAURI_IIFE__ = (() => {})()`
    // and var is not guaranteed to assign to the global `window` object so we make sure to assign it
    footer: {
      js: 'window.__TAURI__ = __TAURI_IIFE__'
    }
  }
])
