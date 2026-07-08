---
"tauri": "patch:feat"
"tauri-runtime": "patch:feat"
"tauri-runtime-wry": "patch:feat"
---

Expose the `wry` permission handler API through Tauri.
This includes support for permission types such as `DisplayCapture`, `Midi`, `Sensors`, `MediaKeySystemAccess`, `LocalFonts`, `WindowManagement`, `PointerLock`, `AutomaticDownloads`, `FileSystemAccess`, and `Autoplay`.
Added `PermissionResponse::{Allow, Deny, Default}` for runtime permission decisions.
Added Android support for geolocation, microphone, camera, protected media, and MIDI requests via JNI.
Updated Linux DisplayCapture support for WebKitGTK versions older than 2.42.
