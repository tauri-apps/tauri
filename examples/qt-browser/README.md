# Qt browser example

A Chromium-style browser built with Tauri on [`tauri-runtime-qt`](../../crates/tauri-runtime-qt),
and a showcase of Tauri's features and official plugins. The browser UI is in [`ui`](./ui), the
Rust side in [`src-tauri/src`](./src-tauri/src).

## Running the example

From the root of the repository:

```bash
cargo run -p qt-browser [url]
```

## What it shows

- **Windows and webviews:** frameless windows with a custom title bar, one child webview per tab
  (`browser.rs`), new windows, popups, fullscreen, always on top, theme, taskbar badge, progress
  bar and attention requests, zoom, print, devtools, cookies, file drops.
- **Menus** (`menus.rs`): the app menu as a menu bar, native popup menus (the "⋮" menu, the
  page's right-click menu, the site information menu) and the tray icon with its menu.
- **IPC:** commands, events, a channel (`downloads.rs`), and capabilities that give web pages a
  few commands only (`capabilities/web-pages.json`).
- **Protocols:** the browser's own pages on a custom `qtbrowser://` scheme (`pages.rs`), and the
  asset protocol for download thumbnails.
- **Plugins:** single-instance, store (settings), window-state, notification, dialog,
  clipboard-manager, opener, global-shortcut, os, process, autostart and log.
