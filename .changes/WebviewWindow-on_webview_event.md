---
tauri: minor:enhance
---

- Implemented `Webview::on_webview_event` for `WebviewWindow` as well
- Implemented `AsRef<Window<R>>` for `WebviewWindow<R>`

    This can be considered a *BREAKING CHANGE* in very specific cases:
    Typically, this means you are relying on implicit type inference, such as `let webview: _ = WebviewWindow.as_ref()`.
    To resolve this, you should explicitly specify the type, for example `let webview: &Window<R> = WebviewWindow.as_ref()`
    or `let webview: _ = AsRef::<Webview<R>>::as_ref(&WebviewWindow)`.
