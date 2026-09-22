---
"@tauri-apps/api": patch:bug
---

Fix `Window.setBackgroundColor`, `Webview.setBackgroundColor` and `WebviewWindow.setBackgroundColor` sending a `color` argument while the `plugin:window|set_background_color` and `plugin:webview|set_webview_background_color` commands expect `label` and `value`. The color was always deserialized as `None`, so instead of applying the given color the call cleared the background of the calling window/webview.
