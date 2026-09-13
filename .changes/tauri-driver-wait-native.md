---
"tauri-driver": patch:bug
---

Wait until the native WebDriver server accepts connections before serving clients, instead of accepting connections that cannot be proxied yet.
Fails fast when the native driver exits during startup or does not come up within 30 seconds.
