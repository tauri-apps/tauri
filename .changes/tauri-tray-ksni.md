---
"tauri": major:breaking
---

The Linux tray icon now uses the ksni (StatusNotifierItem over D-Bus) backend by default instead of libappindicator, dropping the libayatana-appindicator system dependency. The `tray-icon` feature no longer needs a GTK version to be selected. Enable the new `linux-libappindicator` feature to go back to the libappindicator backend.
