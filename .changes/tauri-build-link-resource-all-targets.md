---
"tauri-build": "minor:feat"
---

Add `WindowsAttributes::link_resource_into_all_targets` to link the Windows resource file, including the application manifest, into test binaries as well, so tests that link Tauri no longer fail with `STATUS_ENTRYPOINT_NOT_FOUND` on Windows.
