---
'tauri': 'patch:bug'
---

Guard the handler lookup in the generated unlisten script. When the unlisten function ran before its listener registration eval reached the webview, the entry was still missing and reading its `handlerId` threw, which aborted `_unlisten` before it sent the backend `plugin:event|unlisten` and left the listener registered.
