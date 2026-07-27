---
"tauri-runtime-wry": "patch:bug"
---

Fix a main-thread panic in the Linux undecorated-resizing hit-test when the webview has been reparented. The button-press and touch handlers assumed the widget hierarchy `webview → GtkBox → GtkWindow` and unwrapped a downcast of the webview's second ancestor; embedding applications that move the webview into another container (e.g. a `GtkPaned`) crashed with ``called `Result::unwrap()` on an `Err` value: … type: GtkBox`` on the first click. The handlers now resolve the webview's toplevel window instead, which behaves identically for the stock hierarchy and works at any nesting depth.
