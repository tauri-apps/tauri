---
"tauri": "patch:bug"
---

Fix `WebviewWindow::once` can be called multiple times if they trigger `emit`(s) inside the handler
