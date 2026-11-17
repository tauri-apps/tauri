---
"tauri-runtime-wry": patch:bug
---

On Linux, give a webview created with `incognito: true` every registered custom URI scheme protocol. A protocol already registered on the shared web context, for example by the main window, was skipped for it, while wry runs an incognito webview on a new ephemeral web context of its own; pages on that scheme showed "The URL can't be shown".
