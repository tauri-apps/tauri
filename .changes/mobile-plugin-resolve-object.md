---
"tauri": patch:enhance
---

Mobile plugins can now resolve using an arbitrary object instead of using the `JSObject` class via `Invoke.resolve` (on Android the class must implement `InvokeResponse`).
