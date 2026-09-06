---
"tauri-runtime": major:breaking
"tauri": major:breaking
---

The custom scheme URL format (`tauri://localhost` or `http://tauri.localhost`) is now defined by the runtime instead of the platform: `Runtime::custom_scheme_url` moved to `RuntimeHandle::custom_scheme_url(&self, scheme, https)`, and the `convertFileSrc` JavaScript API takes the format from the runtime.

`tauri::test::MockRuntime` uses `tauri://localhost` on every platform, so tests that sent IPC requests from `http://tauri.localhost` on Windows and Android must use `tauri://localhost` instead.
