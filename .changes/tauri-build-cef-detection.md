---
"tauri-build": major:breaking
---

The CEF runtime is now detected through the `tauri-runtime-cef` dependency of the application (the `DEP_TAURI_RUNTIME_CEF_RUNTIME` env var it exports to the app's build script) instead of the removed `cef` feature of `tauri`.
