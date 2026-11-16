---
"tauri-runtime-wry": patch:bug
---

Fix `window.open('')` being denied on macOS before reaching the `on_new_window` handler. An empty URL is now passed to the handler as `about:blank`.
