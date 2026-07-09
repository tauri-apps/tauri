// Example tauri-ffi app in Deno. Run from the repo root:
//   deno run -A bindings/deno/examples/hello/main.ts
// launch() parks the OS main thread in the Tauri event loop and runs
// ./app.ts in a worker.
import { launch } from '../../mod.ts'

await launch(new URL('./app.ts', import.meta.url), {
  config: {
    productName: 'tauri-ffi-hello-deno',
    version: '0.1.0',
    identifier: 'com.tauri.ffi.hello.deno',
    app: {
      withGlobalTauri: true,
      windows: [
        {
          label: 'main',
          title: 'Tauri FFI — Deno',
          url: 'index.html',
          width: 600,
          height: 440
        }
      ]
    }
  },
  assetsDir: new URL('./assets/', import.meta.url),
  commands: ['greet', 'open-window']
})
