---
"tauri-runtime-wry": patch:bug
---

Fix race condition where `create_window` returned before the HWND was registered on the event loop thread. Adds a response channel (`Sender<Result<()>>`) to `Message::CreateWindow` so the caller blocks until the window is inserted into the window map, preventing downstream `hwnd()` calls from racing against window creation.

Ref: https://github.com/npiesco/wry-arm64-deadlock (minimal reproduction)