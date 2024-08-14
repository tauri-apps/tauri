---
"@tauri-apps/api": patch:breaking
---

Changed `WebviewWindow.getAll`, `WebviewWindow.getByLabel`, `getAllWebviewWindows`, `WebviewWindow.getFocusedWindow`,
`Window.getAll`, `Window.getByLabel`, `getAllWindows`, `Window.getFocusedWindow`,
`Webview.getAll`, `Webview.getByLabel`, `getAllWebviews`
to be async so their return value are synchronized with the state from the Rust side,
meaning new and destroyed windows are reflected.
