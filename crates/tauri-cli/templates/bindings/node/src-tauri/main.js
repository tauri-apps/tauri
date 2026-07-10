// Process entry point. Tauri must own the OS main thread, so `launch()` parks
// this thread in the Tauri event loop and runs ./app.js in a worker_thread
// with a live event loop. Started by `tauri dev` / `tauri build` through the
// `build > runner` command in tauri.conf.json.
import { launch } from '@tauri-apps/node'

launch(new URL('./app.js', import.meta.url), {
  // Command names handled in the worker (see app.js).
  commands: ['greet']
})
