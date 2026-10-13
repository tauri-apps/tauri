---
'tauri-build': 'patch:enhance'
---

Build-time ACL validation now suggests the closest known identifier when a capability references an unknown permission, for example `Permission fs:allow-read-text-fil not found ... (did you mean "fs:allow-read-text-file"?)`. The hint uses the same `jaro_winkler` similarity and confidence threshold already used for application categories, and is only appended when a candidate is similar enough, so unrelated identifiers keep the previous message.
