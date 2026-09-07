---
'tauri-runtime-cef': 'patch:enhance'
---

Added `on_console_message` to `WebviewWindowBuilderCefExt` and the `unstable`-gated `WebviewBuilderCefExt`, so an application can see what its renderer writes to the JavaScript console without opening DevTools. Each `ConsoleMessage` carries the text, the source that wrote it, the line, and a `ConsoleMessageLevel` rather than a raw CEF severity. The callback runs synchronously on CEF's UI thread and must return promptly; observing a message does not suppress it, so CEF still logs it as before. The observer is scoped to the webview's own native browser, so a CEF-owned popup — a separate browser running its own scripts — is never reported through it, and neither is a DevTools window opened on the webview.
