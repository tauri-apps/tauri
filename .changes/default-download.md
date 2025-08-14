---
tauri: "patch:enhance"
---

Tauri will now allow downloads (`<a href="..." download="...">`) on local URLs (devPath/frontendDist) by default, essentially matching the behavior of browsers. This behavior can be overwritting by providing a custom [download handler](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html#method.on_download) in Rust.
