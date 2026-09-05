---
'tauri-runtime-cef': 'patch:bug'
---

Fixed CEF failing to start on Linux with `ContentMainRun failed with exit code 28` by passing `--no-first-run` to Chromium.
