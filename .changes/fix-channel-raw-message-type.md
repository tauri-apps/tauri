---
tauri: patch:bug
---

Fix a regression that made the raw type messages received from `Channel.onmessage` became `number[]` instead of `ArrayBuffer` when that message is small
