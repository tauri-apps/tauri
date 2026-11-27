---
"tauri-runtime-cef": patch:bug
---

A second launch of a running application now exits quietly instead of failing with `Could not find the webview runtime, make sure it is installed`. The running instance receives its argv and working directory through the new `AppHandleCefExt::on_relaunch`.
