---
tauri-cli: patch:bug
"@tauri-apps/cli": patch:bug
---

Only probe for libappindicator (and add it to the deb dependencies and AppImage files) when the `linux-libappindicator` feature is enabled. The `tray-icon` feature alone now uses the D-Bus StatusNotifierItem backend, so the probe panicked with "Can't detect any appindicator library" on hosts without the GTK 3 library.
