---
"tauri-build": "minor:feat"
---

Link the Windows resource file, including the application manifest, into all targets of the package by default instead of only into binaries, so tests that link Tauri no longer fail with `STATUS_ENTRYPOINT_NOT_FOUND` on Windows. Use `WindowsAttributes::link_resource_into_all_targets(false)` to restore the previous behavior.
