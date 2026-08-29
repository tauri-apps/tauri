---
"tauri": patch:bug
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
"tauri-utils": patch:enhance
---

The `tauri://` asset protocol and the CLI's built-in dev server now return `404` (`text/plain`, naming the requested path) when a request for a static subresource (`.js`, `.css`, images, fonts, ...) does not match any asset, instead of serving `index.html` with `200 text/html`. The SPA `index.html` fallback still applies to extensionless paths and now logs a warning when it serves `index.html` for a missing path. `AssetResolver::get` returns `None` for missing subresource paths accordingly.
