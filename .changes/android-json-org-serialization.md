---
tauri: patch:bug
---

On Android, `Invoke.resolveObject` and the other values serialized with the plugin JSON mapper now write `JSObject` and `JSArray` as JSON objects and arrays. They were serialized through their private fields, so a `JSArray` reached the frontend as `{ "values": [...] }` and a `JSObject` as `{ "nameValuePairs": {...} }`.
