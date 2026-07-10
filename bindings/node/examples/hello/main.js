// Fixture entry point — parks the OS main thread in the Tauri event loop and
// runs ./app.js in a worker. Run with: node bindings/node/examples/hello/main.js
// (or `tauri dev` from this directory — the CLI uses `build > runner`).
// Configuration comes from ./tauri.conf.json; assets from `build > frontendDist`.
import { launch } from '../../src/index.js'
import demoPlugin from './demo-plugin.js'

launch(new URL('./app.js', import.meta.url), {
  commands: ['greet', 'open-window'],
  plugins: [demoPlugin]
})
