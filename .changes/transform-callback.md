---
"@tauri-apps/api": minor:changes
"tauri": minor:changes
---

`transformCallback` now registers the callbacks inside `window.__TAURI_INTERNALS__.callbacks` instead of directly on `window['_{id}']`
