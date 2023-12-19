---
"tauri": patch:breaking
---

The initialization script for `Builder::invoke_system` now must initialize the `window.__TAURI_INTERNALS__.postMessage` function instead of `window.__TAURI_POST_MESSAGE__`.
