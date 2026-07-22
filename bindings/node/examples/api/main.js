// Node.js host for Tauri's `examples/api` validation app.
//
// The frontend is NOT copied here — it is the shared Svelte app in
// /examples/api, referenced through tauri.conf.json (`beforeDevCommand` +
// `devUrl` in dev, `frontendDist` -> ../../../../examples/api/dist in a build).
// This file only provides the host (Rust-equivalent) side: the commands the
// frontend invokes and the events it exchanges.
//
// Run it through the example launcher (from the repo root), which stages the
// native library and starts the shared Vite dev server automatically:
//   pnpm node:example:api dev            (TAURI_RUNTIME=cef … to run against cef)
import { launch } from '../../src/index.js'
import appMenu from './app-menu-plugin.js'

launch(new URL('./app.js', import.meta.url), {
  commands: ['log_operation', 'perform_request', 'echo', 'spam'],
  plugins: [appMenu]
})
