# {{ app_name }} — Tauri (C)

A Tauri app whose backend is a C program linked against the `tauri-ffi` C
library. The frontend runs in a native webview; the C side owns the OS main
thread and drives the Tauri event loop.

Unlike the Node.js/Deno/Python bindings, a C app is **compiled and linked** —
it is not run by `tauri dev`. Build it directly and run the produced binary.

## Requirements

You need the `tauri-ffi` distribution for your platform:

- the header `tauri_ffi.h`
- the library `libtauri_ffi.dylib` (macOS), `libtauri_ffi.so` (Linux) or
  `tauri_ffi.dll` (Windows)

Point the Makefile at wherever those live:

```sh
make TAURI_FFI_DIR=/path/to/tauri-ffi
./app
```

`main.c` reads `tauri.conf.json` at startup, so run the binary from this
directory (or adjust the path).

## Serving a local frontend

By default the window loads a URL (`devUrl` if set, otherwise a placeholder).
To serve local HTML/CSS/JS instead, set the window `url` to `index.html` in
`tauri.conf.json` and uncomment the `tauri_app_builder_set_assets_dir` call in
`main.c`.
