---
'tauri': 'minor:feat'
'@tauri-apps/api': 'minor:feat'
---

Add the `openSettings`, `setKeepScreenOn` and `setPrivacyScreen` functions to the `app` module, to open the app's page in the system settings, keep the screen on and hide the app's content from the app switcher (and from screenshots on Android) on Android and iOS. They are unsupported on desktop and reject there. `core:app:default` now includes `allow-open-settings`; the other two need the `core:app:allow-set-keep-screen-on` and `core:app:allow-set-privacy-screen` permissions. On iOS, the `app` plugin now has a native counterpart, so its `registerListener` and `removeListener` commands no longer fail with "Plugin app not initialized".
