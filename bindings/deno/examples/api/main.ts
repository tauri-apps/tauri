// Deno host for Tauri's `examples/api` validation app.
//
// The frontend is NOT copied here — it is the shared Svelte app in
// /examples/api, referenced through tauri.conf.json (`beforeDevCommand` +
// `devUrl` in dev, `frontendDist` -> ../../../../examples/api/dist in a build).
// This file only provides the host (Rust-equivalent) side: the commands the
// frontend invokes and the events it exchanges.
//
// Run from this directory (starts the shared Vite dev server automatically):
//   ../../../../target/debug/cargo-tauri dev
import { launch } from '../../mod.ts'
import appMenu from './app-menu-plugin.ts'

await launch(new URL('./app.ts', import.meta.url), {
  commands: ['log_operation', 'perform_request', 'echo', 'spam'],
  plugins: [appMenu]
})
