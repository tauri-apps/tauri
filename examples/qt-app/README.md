# Qt example

A full Tauri application running on [`tauri-runtime-qt`](../../crates/tauri-runtime-qt)
(Qt Widgets + QtWebEngine), showcasing (nearly) every portable Tauri API, and the runtime's
end-to-end regression suites. The frontend is in [`ui`](./ui), the commands and hooks in
[`src-tauri/src/main.rs`](./src-tauri/src/main.rs).

The runtime is selected by depending on `tauri-runtime-qt` and passing
`tauri_runtime_qt::Qt` to `tauri::Builder::runtime`. It needs the Qt 6 WebEngine development
packages (see the runtime's README).

## Running the example

From the root of the repository:

```bash
cargo run -p qt-app                       # interactive showcase
QT_APP_AUTO=1 cargo run -p qt-app         # self-test: IPC, events, windows, popups, history, menus, schemes, WebRTC
QT_APP_AUTO=close cargo run -p qt-app     # same, but exits by closing the last window
QT_APP_AUTO=smoke cargo run -p qt-app     # clicks through the showcase controls and checks each result
QT_APP_AUTO=crash cargo run -p qt-app     # kills the renderer once and checks the page recovers
cargo test -p qt-app --test e2e           # the four modes above, offscreen
```

## What it shows

- **About:** engine, Tauri and app info, monitors, cursor position.
- **Commands & events:** sync, async and failing commands, events both ways, `eval` from Rust.
- **Window:** title, minimize, maximize, fullscreen, hide/show, always-on-top, decorations,
  resizable, close prevention, size, position, cursor, theme, progress/badge, drag region.
- **Webview & DevTools:** open/close/toggle, zoom, background, history, navigation (plus a link
  blocked by `on_navigation`), print, clear data, cookies.
- **Windows & webviews:** new windows (with and without a parent), `window.open` popups,
  auto-resizing child webviews.
- **Menus:** the Qt menubar, a right-click context menu from the JS `Menu` API, live renaming,
  and a tray icon.
- **Media:** camera/microphone and screen sharing (through the runtime's permission prompt),
  WebRTC, codecs, WebGL, element fullscreen.
- **Files & protocols:** drag & drop events, downloads, `stream://` with byte ranges, and killing
  the renderer to exercise `on_web_content_process_terminate`.

The self-test (`ui/selftest.js`) and the smoke test (`smokeTest` in `ui/app.js`) report each
check through the `report` command, which prints `[PASS]`/`[FAIL]`.
