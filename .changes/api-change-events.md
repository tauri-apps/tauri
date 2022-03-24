---
"tauri": patch
---

* **Breaking change:** Renamed `tauri::Event`  to `tauri::RunEvent`
* Exported `tauri::Event` and `tauri::EventHandler` so you can define a function and pass it to `Window::listen`
