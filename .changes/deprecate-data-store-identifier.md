---
"tauri": minor:changes
"tauri-utils": minor:changes
"@tauri-apps/api": minor:changes
---

Deprecate the macOS / iOS only `WebviewBuilder::data_store_identifier`, `WebviewWindowBuilder::data_store_identifier`, the `dataStoreIdentifier` window config option and the `dataStoreIdentifier` JS webview option in favor of `profile_identifier` / `profileIdentifier`. They are not applied on Windows, so apps that already set them keep using the default WebView2 profile.
