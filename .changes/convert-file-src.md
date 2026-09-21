---
"tauri": minor:feat
---

Add `Webview::convert_file_src` and `WebviewWindow::convert_file_src`, the Rust equivalent of the JavaScript `convertFileSrc` function. The returned URL uses the scheme the webview was configured with (`useHttpsScheme`), so it always matches what the webview can load.
