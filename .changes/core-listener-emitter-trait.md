---
"tauri": "patch:breaking"
---

Added `Emitter` and `Listener` traits that defines what an emitter or a listener can do, this however comes with a few breaking changes:
- Removed `Manager::listen_any`, use `Listener::listen_any` instead.
- Removed `Manager::once_any`, use `Listener::once_any` instead.
- Removed `Manager::unlisten`, use `Listener::unlisten` instead.
- Removed `Manager::emit`, use `Emitter::emit` instead.
- Removed `Manager::emit_to`, use `Emitter::emit_to` instead.
- Removed `Manager::emit_filter`, use `Emitter::emit_filter` instead.
- Removed `App/AppHandle::listen`, `WebviewWindow::listen`, `Window::listen` and `Webview::listen`, use `Listener::listen` instead.
- Removed `App/AppHandle::once`, `WebviewWindow::once`, `Window::once` and `Webview::once`, use `Listener::once` instead.
- Removed `App/AppHandle::unlisten`, `WebviewWindow::unlisten`, `Window::unlisten` and `Webview::unlisten`, use `Listener::unlisten` instead.

