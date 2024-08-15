---
"@tauri-apps/api": patch:breaking
---

Changed `WebviewWindow.getAll`, `WebviewWindow.getByLabel`, `getAllWebviewWindows`,
`Window.getAll`, `Window.getByLabel`, `getAllWindows`,
`Webview.getAll`, `Webview.getByLabel`, `getAllWebviews`
to be async so their return value are synchronized with the state from the Rust side,
meaning new and destroyed windows are reflected.
