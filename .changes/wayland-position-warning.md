---
"tauri-runtime-wry": patch:enhance
"tauri": patch:enhance
"@tauri-apps/api": patch:enhance
---

Log a warning instead of silently doing nothing when an explicit window position (builder `position`/`x`/`y` or `set_position`) is requested on Wayland, which does not allow clients to position their own windows. The warning points to `set_fullscreen_on_monitor` as the supported way to place a window on a specific monitor. The Rust and JavaScript position APIs now document this platform limitation.
