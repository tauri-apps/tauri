# @tauri-apps/node (experimental)

Node.js bindings to Tauri over the `tauri-ffi` C ABI — pure JS ([koffi](https://koffi.dev)
dynamic FFI), no napi-rs, no node-gyp. M0/M2 spike of [/ffi-bindings-plan.md](../../ffi-bindings-plan.md).

## How it works

Tauri must own the **OS main thread** for the app's lifetime (hard requirement on
macOS) — the same thread Node's event loop runs on. So `launch()` inverts the layout:

- the **main thread** builds the app and parks inside the blocking `tauri_app_run`;
- **your app code** runs in a `worker_threads.Worker` with a live event loop, driving
  the app through thread-safe FFI calls and consuming a serialized event queue
  (`tauri_events_next`, polled via koffi async calls on the libuv thread pool).

No native callbacks into JS anywhere — everything is plain C ABI + JSON.

## Running the fixture

```sh
# 1. build the cdylib (repo root)
cargo build -p tauri-ffi

# 2. install koffi (workspace package — run from the repo root)
pnpm install --filter @tauri-apps/node

# 3. run
node bindings/node/examples/hello/main.js
```

A window opens serving `examples/hello/assets/`; it exercises an invoke round-trip
(`greet` command handled in `app.js`), host→frontend events (`tick`), and
frontend→host events (`frontend-ping`). Close the window to exit.

Use `TAURI_FFI_LIB=/path/to/libtauri_ffi.dylib` to point at a non-default build.

## Plugins

A plugin bundles a name, an init script (injected into every webview) and
commands — mirroring `tauri::plugin::Builder`. It has no dependencies, so it can
ship as its own npm package:

```js
// @acme/tauri-plugin-greet
import { definePlugin } from '@tauri-apps/node/plugin'

export default definePlugin('greet')
  // install a frontend API so callers never write the plugin:greet|hello wire format
  .initScript("window.greet = { hello: (n) => window.__TAURI__.core.invoke('plugin:greet|hello', { name: n }) }")
  .command('hello', ({ name }) => `Hello ${name}!`)
```

Pass it to `launch({ plugins: [greet] })` on the main thread (native side + ACL)
and `app.plugin(greet)` in the worker (handlers). The frontend then calls
`window.greet.hello('Lucas')`. See `examples/hello/demo-plugin.js`.

## Files

- `src/ffi.js` — low-level koffi declarations (generated from the API manifest)
- `src/index.js` — `launch()`: main-thread bootstrap
- `src/worker.js` — worker-side `app` API: `command`, `plugin`, `on`, `listen`, `emit`, `emitTo`, windows, `exit`
- `src/plugin.js` — `definePlugin()`: packageable plugin wrapper
