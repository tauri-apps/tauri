# tauri-ffi C binding (experimental)

Embed [Tauri](https://tauri.app) from C (or anything with a C FFI) — see
[/ffi-bindings-plan.md](../../ffi-bindings-plan.md) in the repository.

Release archives contain:

- `include/tauri_ffi.h` — the API, with conventions, threading contract and
  per-function docs (generated from `crates/tauri-ffi/api-manifest.json`)
- `lib/` — the prebuilt `tauri_ffi` shared library for one target
  (plus the MSVC import library on Windows)

## Linking

```sh
cc your_app.c -I include -L lib -ltauri_ffi -Wl,-rpath,'$ORIGIN/lib' -o your_app
```

On Linux the library requires the system WebKitGTK at runtime
(`webkit2gtk-4.1`), the same requirement as any Tauri app. On Windows link
against `lib/tauri_ffi.dll.lib` and ship `tauri_ffi.dll` next to your binary.

A minimal example lives at
[`bindings/c/examples/hello.c`](https://github.com/tauri-apps/tauri/blob/dev/bindings/c/examples/hello.c);
in the repository you can run it with `node bindings/run-example.mjs c`.
