---
tauri: patch:enhance
---

`Webview::eval` and `WebviewWindow::eval` now takes `impl Into<String>` instead of `&str` to allow passing the scripts more flexible and efficiently
