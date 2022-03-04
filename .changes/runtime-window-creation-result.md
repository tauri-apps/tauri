---
"tauri-runtime": patch
---

The `PendingWindow::new` and `PendingWindow::with_config` functions now return `Result<Self>` validating the window label.
