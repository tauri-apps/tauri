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
    // but we want to assign it to the `window` object, this footer takes care of that
    footer: {
      js: 'window.__TAURI__ = __TAURI_IIFE__'
    }
  }
])
