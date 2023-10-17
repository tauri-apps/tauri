---
'tauri': 'patch:enhance'
---

Set `main` as the default `id` for the tray icon registered from the configuration file, so if the `id` is not specified, it can be retrieved using `app.tray_by_id("main")`.
