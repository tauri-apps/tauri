import { defineConfig } from 'tsup'

export default defineConfig(() => [
  {
    entry: ['src/*.ts'],
    outDir: 'dist',
    format: ['esm', 'cjs'],
    clean: true,
    splitting: true,
    dts: true,
    sourcemap: false,
    keepNames: true,
  },
  {
    entry: { bundle: 'src/index.ts' },
    outDir: '../../core/tauri/scripts',
    format: ['iife'],
    globalName: '__TAURI__',
    splitting: false,
    clean: false,
    dts: false,
    sourcemap: false,
  }
])
