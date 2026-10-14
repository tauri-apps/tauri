---
"tauri-bundler": "patch:bug"
---

Fixed deep links being dropped on Linux when the app is launched through the bundled desktop entry. `Exec` now ends with `%u` when deep-link schemes are configured. Custom `desktopTemplate`s must add `{{#if exec_arg}} {{exec_arg}}{{/if}}` to their `Exec` line to get the fix. If the app also has file associations, some launchers may pass opened files as `file://` URLs instead of paths.
