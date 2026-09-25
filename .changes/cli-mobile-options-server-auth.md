---
"tauri-cli": patch:sec
"@tauri-apps/cli": patch:sec
---

Secure the local server that `tauri android|ios dev|build` uses to pass options to the Android Studio and Xcode build scripts: requests now need a random per-session token, WebSocket connections from web pages (with an `Origin` header) are rejected, and variables whose name contains `TOKEN`, `PASSWORD`, `SECRET` or `CREDENTIAL` are no longer sent. The connection details are now written with owner-only permissions to `gen/<android|apple>/.tauri/cli-options-server.json` (instead of the shared temp directory) and removed when the command exits. The IDE build scripts now report a clear error when the Tauri CLI command is not running.
