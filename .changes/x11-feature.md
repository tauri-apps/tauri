---
"tauri": minor:feat
"tauri-runtime-wry": minor:feat
---

Added `x11` Cargo feature (enabled by default). Disabling it is useful for apps that only support Wayland, reducing its size.
**NOTE**: When manually disabling tauri default features, you must enable the `x11` feature to support it.
