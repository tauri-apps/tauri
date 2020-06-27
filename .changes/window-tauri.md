---
"tauri.js": minor
---

Renaming `window.tauri` to `window.__TAURI__`, closing #435.
The `__TAURI__` object now follows the TypeScript API structure (e.g. `window.__TAURI__.readTextFile` is now `window.__TAURI__.fs.readTextFile`).
