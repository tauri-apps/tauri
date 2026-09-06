---
"tauri": major:breaking
"tauri-runtime-wry": major:breaking
"tauri-runtime-cef": major:breaking
---

The `devtools`, `macos-private-api` and `unstable` features must now be enabled on the runtime crate (`tauri-runtime-wry` or `tauri-runtime-cef`), which also enables them on `tauri`. Enabling them on `tauri` alone no longer enables them on the runtime.
