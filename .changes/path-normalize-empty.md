---
tauri: patch:bug
---

Fix `path.normalize("")` returning an empty string instead of `"."`, matching Node.js and the existing `path.join("")` behavior.
