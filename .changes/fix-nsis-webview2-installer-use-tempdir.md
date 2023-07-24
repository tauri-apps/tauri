---
'tauri-bundler': 'patch:bug'
---

On Windows, fix NSIS installer writing webview2 installer file to the well-known temp dir instead of the install dir, so we don't pollute the install dir.
