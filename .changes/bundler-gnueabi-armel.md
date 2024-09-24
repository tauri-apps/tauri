---
"tauri-bundler": patch:bug
---

Detect ARM gnueabi as soft-float (armel) instead of hard-float (armhf). Also change the signature of `tauri_bundler::bundle::Settings::binary_arch` to return an enum instead of a `&str`.
