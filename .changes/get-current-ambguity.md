---
"@tauri-apps/api": "patch:breaking"
"tauri": "patch:breaking"
---

Renamed the JS `getCurrent` functions to a clearer name to avoid ambiguity:
- `getCurrent` in `window` module has been renamed to `getCurrentWindow`
- `getCurrent` in `webview` module has been renamed to `getCurrentWebview`
- `getCurrent` in `webviewWindow` module has been renamed to `getCurrentWebviewWindow`

