---
"tauri-runtime": major:breaking
---

`WebviewDispatch::open_devtools`, `close_devtools` and `is_devtools_open` are now required regardless of the `devtools` feature, so the type-erased runtime can forward them. Runtimes should keep their implementation behind the feature and no-op without it.
