---
'tauri-build': 'patch:enhance'
---

Build-time ACL validation now suggests the closest known identifier when a capability references an unknown permission that looks like a typo of an existing one, for example `Permission fs:allow-read-text-fil not found (did you mean "fs:allow-read-text-file"?), expected one of ...`. Typos in the plugin namespace and a missing `allow-` prefix are also covered; unrelated identifiers keep the previous message.
