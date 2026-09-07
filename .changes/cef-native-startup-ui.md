---
"tauri-runtime-cef": patch
---

Add `prepare_macos_application` to initialize the CEF-compatible macOS application before native startup dialogs. Keep browser windows available until CEF acknowledges application shutdown, including popup cleanup.
