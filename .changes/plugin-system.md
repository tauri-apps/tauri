---
"tauri": minor
---

Plugin system added. You can hook into the webview lifecycle (`created`, `ready`) and extend the API adding logic to the `invoke_handler` by implementing the `tauri::plugin::PLugin` trait.
