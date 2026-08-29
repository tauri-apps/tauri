---
"tauri": minor:feat
---

Add `Builder::forward_webview_console(bool)` to opt in to forwarding webview `console.*` output, uncaught errors (including capture-phase resource load errors) and unhandled promise rejections to the `log` crate under the `webview:{label}` target. Disabled by default; when disabled the capture script is not injected and forwarded messages are ignored.
