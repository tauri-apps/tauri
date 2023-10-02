---
'tauri': 'minor:feat'
---

Add `App::cleanup_before_exit` and `AppHandle::cleanup_before_exit` to manually call the cleanup logic. **You should always exit the tauri app immediately after this function returns and not use any tauri-related APIs.**
