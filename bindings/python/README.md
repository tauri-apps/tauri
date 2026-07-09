# tauri-ffi Python bindings (experimental)

Python bindings to Tauri over the `tauri-ffi` C ABI, using [cffi](https://cffi.readthedocs.io)
in ABI mode — no compiler, no native glue code. Part of [/ffi-bindings-plan.md](../../ffi-bindings-plan.md).

## How it works

`App.run()` blocks the **process main thread** in the Tauri event loop (a hard macOS
requirement) — exactly like every Python GUI toolkit. A daemon thread pumps the
serialized event queue (`tauri_events_next`) and dispatches invokes/events to your
handlers; cffi releases the GIL during C calls, so both threads make progress.

- `tauri_ffi_cdef.py` — **generated** from `crates/tauri-ffi/api-manifest.json`
  (`node bindings/bindgen/generate.mjs`); do not edit.
- `tauri_ffi.py` — hand-written runtime: `App`, `WebviewWindow`, error handling.

## Running the example

```sh
# 1. build the cdylib (repo root)
cargo build -p tauri-ffi

# 2. install cffi
pip install cffi

# 3. run
python3 bindings/python/examples/hello/main.py
```

A window opens serving the example's assets; it exercises an invoke round-trip,
host↔frontend events, and runtime window creation (`App.create_window`,
`App.get_window`, `WebviewWindow` getters/setters). Close the window to exit.

Use `TAURI_FFI_LIB=/path/to/libtauri_ffi.dylib` to point at a non-default build.
