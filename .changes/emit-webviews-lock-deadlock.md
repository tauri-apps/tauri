---
"tauri": patch:bug
---

Fix a deadlock when emitting an event from a non-main thread with the `tracing` feature enabled while the main thread looks up a webview, for example in a document title change handler.
