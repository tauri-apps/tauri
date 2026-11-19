---
"tauri-runtime-cef": patch:bug
---

A second launch of a running application now exits quietly instead of failing with `Could not find the webview runtime, make sure it is installed`, and the running instance calls the new `Cef::on_relaunch` callback.
