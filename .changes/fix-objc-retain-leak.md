---
"tauri": patch:bug
"tauri-runtime-wry": patch:bug
---

Avoid leaking Objective-C objects in `WebviewMessage::WithWebview` on Apple targets by replacing `Retained::into_raw` with scoped retained bindings and `Retained::as_ptr` pointer handoff.
