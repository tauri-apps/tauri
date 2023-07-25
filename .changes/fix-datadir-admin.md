---
'tauri': 'patch:bug'
---

On Windows, don't create the user data directory manually but leave it to WebView2 instead to prevent permission errors when running Tauri apps with admin privileges.
