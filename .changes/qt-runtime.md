---
"tauri": minor:feat
"tauri-runtime": minor:feat
"tauri-runtime-qt": minor:feat
"qtshim": minor:feat
"qtshim-sys": minor:feat
---

Add `tauri-runtime-qt`, a Linux runtime built on Qt Widgets and QtWebEngine (Chromium), selected with `tauri::Builder::runtime(tauri_runtime_qt::Qt::default())`. Its windows and context menus are Qt menus: the runtime enables `tauri`'s new `qt` feature, which uses muda's Qt backend and attaches menus through the new `WindowDispatch::qt_window` (when `gtk3`/`gtk4` is also enabled, GTK menus take precedence and the build prints a warning). `qtshim` and `qtshim-sys` are the safe and raw Rust bindings to the small C++ library the runtime and muda's Qt backend share.
