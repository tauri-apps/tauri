// Process entry point. Tauri must own the OS main thread, so `launch()` parks
// this thread in the Tauri event loop and runs ./app.ts in a Worker with a
// live event loop. Started by `tauri dev` / `tauri build` through the
// `build > runner` command in tauri.conf.json.
import { launch } from '@tauri-apps/deno'

await launch(new URL('./app.ts', import.meta.url), {
  // Command names handled in the worker (see app.ts).
  commands: ['greet']
})
