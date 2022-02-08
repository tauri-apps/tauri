import { defineConfig } from 'tsup'

export default defineConfig((options) => {
  // we only minify when building fot `withGlobalTauri`option
  // so we pass `--minify` to tsup cli and check for it here
  // to conditionally provide appropriate config
  const isBrowser = options.minify as boolean

  return isBrowser
    ? {
        entry: { 'bundle': 'src/index.ts' },
        outDir: '../../core/tauri/scripts',
        format: ['iife'],
        globalName: '__TAURI__',
        splitting: false,
        clean: false,
        dts: false,
        sourcemap: false
      }
    : {
        entry: ['src/*.ts'],
        outDir: 'dist',
        format: ['esm', 'cjs'],
        clean: true,
        splitting: false,
        dts: true,
        sourcemap: false, keepNames: true
      }
})
