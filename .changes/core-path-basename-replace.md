---
'tauri': 'patch:bug'
'@tauri-apps/api': patch:bug
---

Fix `basename(path, 'ext')` JS API when removing all occurances of `ext` where it should only remove the last one.
