---
'tauri': 'patch:bug'
---

Fix parsing `allowlist > http > scope` urls that added a trailing slash which broke matching the incoming requests url.
