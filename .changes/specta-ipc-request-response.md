---
"tauri": "minor:feat"
---

Added `specta::Type` implementations for `tauri::ipc::Request` and `tauri::ipc::Response` behind the `specta` feature. They export as a reference to a stand-in named type, matching how `tauri::ipc::Channel` is handled, so Specta-based tooling such as tauri-specta can describe commands that take a raw IPC request or return a raw IPC response.
