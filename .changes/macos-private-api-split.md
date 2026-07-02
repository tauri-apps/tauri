---
"tauri": "minor:feat"
"tauri-runtime-wry": "minor:feat"
"tauri-utils": "minor:feat"
---

Added the `app > macOS > fullscreenApi` and `app > macOS > transparentBackgroundApi` configuration options to opt into the macOS WKWebView fullscreen API and transparent webview backgrounds independently, along with matching `macos-private-api-fullscreen` and `macos-private-api-transparent` Cargo features. The existing `macOSPrivateApi` flag keeps working and enables both capabilities.
