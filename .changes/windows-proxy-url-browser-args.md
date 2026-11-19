---
"tauri-runtime-wry": patch:bug
---

On Windows, keep the webview's `proxy_url` when `additional_browser_args` is also set. The additional arguments replaced the `--proxy-server` argument wry sets for the proxy, so the proxy was silently ignored.
