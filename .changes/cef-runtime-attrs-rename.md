---
"tauri-runtime-cef": major:breaking
---

`RuntimeInitAttrs` was renamed to `Cef`, the attributes type that selects the runtime, e.g. `tauri::Builder::default().runtime(tauri_runtime_cef::Cef::default())`. The `cef_entry_point` attribute macro is now exported by this crate instead of `tauri`.
