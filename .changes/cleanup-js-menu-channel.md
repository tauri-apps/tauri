---
"tauri": "patch:bug"
---

Remove the `Channel` used to send event to JavaScript side on dropping the menu. Items sharing the same id each keep their own handler, so dropping one no longer removes another's.
