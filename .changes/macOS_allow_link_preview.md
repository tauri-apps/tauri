---
"tauri": patch:feat
"tauri-runtime": patch:feat
"tauri-runtime-wry": patch:feat
---

macOS/iOS: add option to disable or enable link previews when building a webview (the webkit api has it enabled by default)
  -  `WebViewBuilder.allow_link_preview(allow_link_preview: bool)`
  -  `WebviewWindowBuilder.allow_link_preview(allow_link_preview: bool)`
