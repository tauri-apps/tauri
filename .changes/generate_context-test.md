---
"tauri-macros": "patch"
"tauri-codegen": "patch"
---

Add support for `test = true` in `generate_context!` macro to skip some code generation that could affect some tests, for now it only skips empedding a plist on macOS.
