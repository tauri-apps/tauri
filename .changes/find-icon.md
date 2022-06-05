---
"tauri-build": patch
---

Search `tauri.conf.json > tauri > bundle > icons` for a `.ico` file for the window icon instead of simple default `icons/icon.ico` when `WindowsAttributes::window_icon_path` is not set.
