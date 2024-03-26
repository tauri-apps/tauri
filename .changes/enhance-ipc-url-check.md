---
"tauri": patch:enhance
---

Enhance the IPC URL check by using the Origin header on the custom protocol IPC and the new request URI field on the postMessage IPC instead of using `Webview::url()` which only returns the URL of the main frame and is not suitable for iframes (iframe URL fetch is still not supported on Android and on Linux when using the postMessage IPC).
