---
"tauri": "patch"
---

The event name is now validated. On a IPC message, it returns an error if it fails validation; on the Rust side, it panics.
It must include only alphanumeric characters, `-`, `/`, `:` and `_`.
