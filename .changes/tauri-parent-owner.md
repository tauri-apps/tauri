---
'tauri': 'patch:feat'
---

Add `WindowBuilder::parent` which is a convenient wrapper around parent functionality for Windows, Linux and macOS. Also added `WindowBuilder::owner` on Windows only. Also added `WindowBuilder::transient_for` and `WindowBuilder::transient_for_raw` on Linux only.
