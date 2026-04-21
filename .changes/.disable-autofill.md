---
'tauri-runtime': 'minor:feat'
'tauri-runtime-wry': 'minor:feat'
'tauri-utils': 'minor:feat'
'tauri': 'minor:feat'
'@tauri-apps/api': 'minor:feat'
---

Add a WebView option to control browser-level general autofill behavior. This option does not disable password or credit card autofill. On Windows (WebView2), setting it to true disables the general autofill "Suggestions" UI, which may appear even when `autocomplete="off"` is specified on input elements. On Linux, macOS, iOS, and Android, this option is currently unsupported and performs no operation.