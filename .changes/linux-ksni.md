---
tauri: minor:feat
---

Added the `linux-ksni` feature, which makes tray icons on Linux use the StatusNotifierItem D-Bus protocol (via the `ksni` crate) instead of libappindicator. This enables `TrayIconEvent::Click` for left and middle clicks and tooltips on Linux.
