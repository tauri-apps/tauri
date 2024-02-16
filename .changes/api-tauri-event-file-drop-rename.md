---
'@tauri-apps/api': 'patch:breaking'
---

Renamed the following enum variants of `TauriEvent` enum:

- `TauriEvent.WEBVIEW_FILE_DROP` -> `TauriEvent.FILE_DROP`
- `TauriEvent.WEBVIEW_FILE_DROP_HOVER` -> `TauriEvent.FILE_DROP_HOVER`
- `TauriEvent.WEBVIEW_FILE_DROP_CANCELLED` -> `TauriEvent.FILE_DROP_CANCELLED`
