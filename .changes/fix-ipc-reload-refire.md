---
'tauri': 'patch:bug'
---

Fixed an issue where an in-flight IPC request aborted by a page reload/navigation was mistaken for a custom-protocol failure and re-sent over the `postMessage` fallback, causing the invoked command to execute twice.
