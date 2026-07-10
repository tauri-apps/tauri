// Example tauri-ffi app in Deno. Run from the repo root:
//   deno run -A bindings/deno/examples/hello/main.ts
// (or `tauri dev` from this directory — the CLI uses `build > runner`).
// launch() parks the OS main thread in the Tauri event loop and runs
// ./app.ts in a worker. Configuration comes from ./tauri.conf.json; assets
// from `build > frontendDist`.
import { launch } from '../../mod.ts'
import demoPlugin from './demo-plugin.ts'

await launch(new URL('./app.ts', import.meta.url), {
  commands: ['greet', 'open-window'],
  plugins: [demoPlugin]
})
