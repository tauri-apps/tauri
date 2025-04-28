---
"tauri": "minor:bug"
"@tauri-apps/api": "minor:bug"
---

Fixed path joining behavior where `path.join('', 'a')` incorrectly returns "/a" instead of "a".
