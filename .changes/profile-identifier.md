---
"tauri": minor:feat
"tauri-runtime": minor:feat
"tauri-runtime-wry": minor:feat
"tauri-utils": minor:feat
"@tauri-apps/api": minor:feat
---

Add `WebviewBuilder::profile_identifier`, `WebviewWindowBuilder::profile_identifier`, the `profileIdentifier` window config option and the `profileIdentifier` JS webview option to isolate a webview's browsing data. On macOS and iOS it maps to a `WKWebsiteDataStore` identifier, and on Windows to a WebView2 named profile, so webviews sharing a data directory get isolated cookies and storage while still sharing a single browser process.
