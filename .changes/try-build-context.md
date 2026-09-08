---
"tauri-build": patch:feat
---

Add `tauri_build::try_build_context` and `ContextAttributes` for packages that expand `tauri::generate_context!` once and share the context with the rest of the workspace. It runs only what the context expansion consumes — config parsing with its rerun instructions, the `OUT_DIR` ACL artifacts and global API script list, the cfg aliases, and `TAURI_ENV_TARGET_TRIPLE` — and skips application artifact staging and executable-specific build configuration, which stay with the package that owns the binary.
