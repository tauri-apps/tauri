---
"tauri-bundler": "patch:bug"
---

Fixed deep links being dropped on Linux when the app is launched through the bundled desktop entry, and files being opened one process at a time. The desktop entry's `Exec` now ends with `%U` when deep-link schemes are configured and with `%F` when only file associations are. Custom `desktopTemplate`s must append `{{exec_arg}}` to their `Exec` line, as in `Exec={{exec}}{{exec_arg}}`, to get the fix.
