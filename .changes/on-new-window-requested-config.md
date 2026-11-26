---
"tauri": minor:feat
"tauri-utils": minor:feat
---

Added `WindowConfig::on_new_window` to statically configure `WebviewBuilder::on_new_window`.

The `urls` scope uses glob patterns by default; enable the new `url-pattern` Cargo feature to use the URLPattern spec instead.
