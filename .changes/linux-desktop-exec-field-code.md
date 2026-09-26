---
"tauri-bundler": "patch:bug"
---

Fixed deep links being dropped on Linux when the app is launched through the bundled desktop entry. `Exec` now ends with `%u` when deep-link schemes are configured. If the app also has file associations, launchers may now pass opened files as `file://` URLs instead of paths.
