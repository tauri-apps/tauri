---
"tauri": "minor:feat"
"tauri-utils": "minor:feat"
---

Add an optional `uri_schemes` allow-list to a capability. When set, a custom URI scheme protocol (registered by the app or a plugin) and the built-in `asset` protocol are only registered on a webview whose label matches a capability that opts the scheme in; the built-in `ipc` and `tauri` schemes and the isolation scheme remain always available. When unset (the default), all schemes remain available on every webview, preserving the previous behavior.
