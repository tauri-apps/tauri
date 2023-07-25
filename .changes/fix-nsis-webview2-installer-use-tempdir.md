---
'tauri-bundler': 'patch:enhance'
---

On Windows, NSIS installer will write webview2 installer file to the well-known temp dir instead of the install dir, so we don't pollute the install dir.
