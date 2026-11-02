---
"tauri": "patch:bug"
"tauri-bundler": "patch:bug"
---

Fix the MSI installer writing its per-machine registry values under `HKEY_CURRENT_USER`, which left them behind when another user uninstalled the app.
