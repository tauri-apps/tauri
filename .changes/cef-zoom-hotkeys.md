---
'tauri-runtime-cef': 'patch:breaking'
---

Alt+Left and Alt+Right no longer navigate the webview's session history. The browser is created at an internal placeholder URL and then navigated to the app's own, so the app's first screen already sits on a second history entry and going back from it lands on a blank page with no way forward. The page context menu already removed Back and Forward for exactly that reason; the accelerators now agree with it. `WebviewDispatch::go_back` and `go_forward` are untouched and still drive the browser on the application's own request.
