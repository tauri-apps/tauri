---
"tauri-runtime-cef": patch:bug
---

Fixed Chrome style webviews never admitting a document on Linux and Windows, which left `WebviewSnapshot::document` always `None` and `NativeDialogObservation` silent: CEF keeps listing a frame parked in the back/forward cache, such as the internal initial document the runtime loads before the app's URL, and the final admission check now compares only the frames attached to a renderer.
