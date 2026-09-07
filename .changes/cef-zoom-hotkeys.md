---
'tauri-runtime-cef': 'patch:breaking'
---

`zoom_hotkeys_enabled` is now honored, and because it defaults to `false`, Ctrl+Plus, Ctrl+Minus and Ctrl+0 no longer zoom the page in a CEF app that did not opt in — this runtime previously ignored the attribute and left Chrome's accelerators live. Call `.zoom_hotkeys_enabled(true)` on the webview builder to get them back. Ctrl+mouse-wheel zoom is unaffected either way: Chromium applies it in the render widget rather than through the command controller, so there is no command to swallow. Note that on Linux and macOS Tauri injects a JavaScript zoom polyfill when the flag is true, which now coexists with Chrome's own accelerator, so a keyboard zoom there steps twice.

Alt+Left and Alt+Right no longer navigate the webview's session history. The browser is created at an internal placeholder URL and then navigated to the app's own, so the app's first screen already sits on a second history entry and going back from it lands on a blank page with no way forward. The page context menu already removed Back and Forward for exactly that reason; the accelerators now agree with it. `WebviewDispatch::set_zoom`, `go_back` and `go_forward` are untouched and still drive the browser on the application's own request.
