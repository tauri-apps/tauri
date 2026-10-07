---
"tauri-runtime-wry": major:breaking
---

Removed the per-window event listener registry, following its removal from `tauri-runtime`. `tauri-runtime-wry` no longer exports `WindowEventHandler`, `WindowEventListeners`, `WebviewEventHandler` or `WebviewEventListeners`, and `WindowMessage::AddEventListener` / `WebviewMessage::AddEventListener` are gone.
