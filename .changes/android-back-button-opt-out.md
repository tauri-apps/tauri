---
'tauri': 'minor:feat'
---

On Android, add `TauriActivity#handleBackButton` to opt out of Tauri's back button handling. Tauri's back callback is always enabled, so on Android 16 the system never plays the predictive back-to-home animation. Apps that handle back themselves can override it to `false`; the `onBackButtonPress` JavaScript event is then not emitted.
