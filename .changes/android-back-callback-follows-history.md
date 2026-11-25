---
'tauri': 'minor:feat'
---

On Android, Tauri's back callback is now enabled only while the webview can go back or a `back-button` listener is registered. At the first page the system handles back itself, so Android 16 plays its predictive back-to-home animation. Plugins get `Plugin#onWebViewHistoryChanged`.
