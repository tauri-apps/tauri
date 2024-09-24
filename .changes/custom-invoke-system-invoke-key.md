---
"tauri": patch:enhance
---

Inject `__INVOKE_KEY__` into custom invoke systems so their implementations can properly construct `tauri::webview::InvokeRequest`.
