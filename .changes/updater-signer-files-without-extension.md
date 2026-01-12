---
tauri-cli: patch:bug
---

`ext.to_os_string().unwrap()` returns None on files without an extension in `sign_file()`
