---
"tauri": "patch:changes"
---

The `linux-libxdo` Cargo feature is now a no-op. `muda` no longer emulates key presses through `libxdo` to run the `Cut`, `Copy`, `Paste`, `SelectAll`, `Undo` and `Redo` predefined menu items on Linux; it runs them through GTK, and through WebKitGTK's editing commands when a web view is focused. The feature has nothing left to enable, and building with it no longer needs `libxdo-dev`.
