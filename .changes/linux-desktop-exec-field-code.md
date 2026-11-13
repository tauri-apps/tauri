---
"tauri-bundler": "patch:bug"
---

Fixed deep links being dropped on Linux when the app is launched through the bundled desktop entry, and files being opened one process at a time. The desktop entry's `Exec` now ends with `%U` when deep-link schemes are configured and with `%F` when only file associations are. Custom `desktopTemplate`s must add `{{#if exec_arg}} {{exec_arg}}{{/if}}` to their `Exec` line to get the fix.
