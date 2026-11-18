---
"tauri-build": "minor:feat"
---

Link the Windows resource file, including the application manifest, into all targets of the package instead of only into binaries, so tests that link Tauri no longer fail with `STATUS_ENTRYPOINT_NOT_FOUND` on Windows.
