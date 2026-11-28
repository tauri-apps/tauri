---
"tauri": patch:bug
---

On Android, `JSObject` and `JSArray` fields of plugin command arguments now deserialize with their contents. Previously they came out empty, because Jackson filled them through `JSONObject`'s private fields. Both types now (de)serialize as plain JSON with any `ObjectMapper`, including one a plugin creates itself, for example in a broadcast receiver.
