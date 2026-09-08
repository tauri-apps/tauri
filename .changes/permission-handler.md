---
"tauri": "minor:feat"
"tauri-runtime": "minor:feat"
"tauri-runtime-wry": "minor:feat"
---

Expose the `wry` permission handler API through Tauri.
This includes support for permission types such as `DisplayCapture`, `Midi`, `Sensors`, `MediaKeySystemAccess`, `LocalFonts`, `WindowManagement`, `PointerLock`, `AutomaticDownloads`, `FileSystemAccess`, and `Autoplay`.
Added `PermissionResponse::{Allow, Deny, Default}` for runtime permission decisions.
