---
"tauri-runtime-wry": patch:bug
---

Apply `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` to every WebView2 webview. WebView2 is meant to let that variable override the arguments passed to `CreateCoreWebView2EnvironmentWithOptions` - it is how `msedgedriver` hands an app the `--remote-debugging-port` it attaches to - but some machines ignore it because wry always passes arguments of its own, leaving WebDriver unable to create a session.
