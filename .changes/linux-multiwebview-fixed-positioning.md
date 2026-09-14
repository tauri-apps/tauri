---
"tauri-runtime-wry": patch
---

On Linux, build child webviews (`WebviewKind::WindowChild`) into the window's `gtk::Fixed` overlay layer (`WindowExtUnix::content_fixed`) instead of the default vertical `gtk::Box`, so `set_bounds` positions them over the window instead of GTK stacking them. Fixes multi-webview positioning on Linux (tauri-apps/tauri#10420). Requires the corresponding `tao` change that adds `content_fixed`.
