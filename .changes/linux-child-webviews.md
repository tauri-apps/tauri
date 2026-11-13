---
"tauri": patch:bug
"tauri-runtime-wry": patch:bug
---

On Linux with the `unstable` feature, child webviews added with `Window::add_child` are placed at their bounds, instead of being stacked in the window's box. They are clipped at the edge of the window and do not scroll. A webview that resizes with its window, the window's own webview included, takes the size of the room that GTK gives it, instead of the inner size of the window, which counts the menu bar and, on Wayland, the client-side decorations. `Webview::reparent` keeps the position of the webview.
