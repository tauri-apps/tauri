# tauri-ffi Deno bindings (experimental)

Deno bindings to Tauri over the `tauri-ffi` C ABI, using the built-in
[`Deno.dlopen`](https://docs.deno.com/runtime/fundamentals/ffi/) — no native glue
code. Part of [/ffi-bindings-plan.md](../../ffi-bindings-plan.md).

## How it works

Tauri must own the **OS main thread** for the app's lifetime (hard requirement on
macOS) — the same thread Deno's event loop runs on. So `launch()` inverts the layout:

- the **main thread** builds the app and parks inside the blocking `tauri_app_run`
  (declared as a synchronous FFI call);
- **your app code** runs in a `Worker` with a live event loop, driving the app through
  thread-safe FFI calls and consuming the serialized event queue (`tauri_events_next`,
  declared `nonblocking` so it polls off-thread and resolves promises).

Deno workers have no `workerData`, so `launch()` hands the app handle to the worker
via URL query params, readable synchronously at module init.

- `symbols.ts` — **generated** from `crates/tauri-ffi/api-manifest.json`
  (`node bindings/bindgen/generate.mjs`); do not edit.
- `ffi.ts` / `mod.ts` / `worker.ts` — hand-written runtime and app API.

## Running the example

```sh
# 1. build the cdylib (repo root)
cargo build -p tauri-ffi

# 2. run (FFI + read/env/write permissions)
deno run -A bindings/deno/examples/hello/main.ts
```

A window opens serving the example's assets; it exercises an invoke round-trip,
host↔frontend events, and runtime window creation (`app.createWindow` — async in
Deno — `app.getWindow`, `WebviewWindow` getters/setters). Close the window to exit.

Use `TAURI_FFI_LIB=/path/to/libtauri_ffi.dylib` to point at a non-default build.
