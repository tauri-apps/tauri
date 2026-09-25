---
"tauri-cli": patch:sec
"@tauri-apps/cli": patch:sec
---

The built-in dev server now rejects requests whose `Host` header is not the address it serves on, which blocks DNS rebinding attacks, and rejects cross-site `Origin`s on its reload WebSocket. The `index.html` fallback now goes through the same path scope check as other files, and the reload WebSocket no longer closes when the client sends a message.
