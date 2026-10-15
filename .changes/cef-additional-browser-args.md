---
"tauri-runtime-cef": patch:enhance
---

Log a warning when a webview sets `additional_browser_args`, which the CEF runtime does not support: Chromium's command line is per process, so switches go through `Cef::command_line_arg` instead. The attribute's documentation on `tauri`, `tauri-runtime` and `tauri-utils` now says so.
