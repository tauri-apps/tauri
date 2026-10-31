---
"tauri-codegen": patch:bug
---

Rebuild the app when a capability file passed to `generate_context!(capabilities = [...])` changes, by adding it as a compiler dependency with `include_bytes!`.
