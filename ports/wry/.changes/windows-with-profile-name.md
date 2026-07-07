---
"wry": minor
---

On Windows, add `WebViewBuilderExtWindows::with_profile_name` to opt the webview into a named WebView2 profile. Webviews with different profile names within the same environment have isolated cookies, storage, IndexedDB, and cache while sharing the runtime — matching WebView2's documented multi-profile pattern.
