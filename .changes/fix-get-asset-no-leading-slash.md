---
'tauri': 'patch:bug'
---

Fix `tauri::AssetResolver::get` and `tauri::AssetResolver::get_for_scheme`
skipping the first character of the `path` even if it's not a slash (/).
