---
'tauri': 'patch:bug'
---

Fix a panic when a `once` event listener receives multiple events before its queued removal is processed.
