# tauri-runtime-qt

A [Tauri](https://tauri.app) v3 runtime for Linux that uses **Qt Widgets + QtWebEngine
(Chromium)** in place of tao + wry + WebKitGTK.

WebRTC, screen sharing, H.264/VP9/AV1 and WebGL behave like Chrome, because they _are_ Chrome.
QtWebEngine comes from the distribution's packages, so apps stay small: the app doesn't bundle
Chromium the way Electron or CEF do. Windows are native Wayland (or X11) windows owned by Qt, so
nothing has to be embedded across toolkits.

```
tauri ──► tauri-runtime (traits) ──► tauri-runtime-qt ──┐
tauri ──► muda (menus, `qt` feature) ────────────────────┴──► qtshim (safe Rust) ──► qtshim-sys (FFI + C++) ──► Qt / QtWebEngine
```

## Using it

This crate lives in the Tauri repository next to `tauri-runtime-wry` and `tauri-runtime-cef`,
and builds against the `tauri` and `tauri-runtime` crates of the same checkout.

It isn't on crates.io yet. An app outside this repository takes the runtime from git and patches
`tauri` to the same branch (so the runtime and the app share one `tauri`, including for plugins),
plus muda, whose Qt backend isn't released either:

```toml
[dependencies]
tauri = "3.0.0-alpha.6"
tauri-runtime-qt = { git = "https://github.com/KRTirtho/tauri", branch = "qt-support" }

[patch.crates-io]
tauri = { git = "https://github.com/KRTirtho/tauri", branch = "qt-support" }
tauri-runtime = { git = "https://github.com/KRTirtho/tauri", branch = "qt-support" }
tauri-utils = { git = "https://github.com/KRTirtho/tauri", branch = "qt-support" }
tauri-macros = { git = "https://github.com/KRTirtho/tauri", branch = "qt-support" }
tauri-codegen = { git = "https://github.com/KRTirtho/tauri", branch = "qt-support" }
tauri-build = { git = "https://github.com/KRTirtho/tauri", branch = "qt-support" }
tauri-plugin = { git = "https://github.com/KRTirtho/tauri", branch = "qt-support" }
muda = { git = "https://github.com/KRTirtho/muda", branch = "qt-backend" }
```

```rust
fn main() {
  tauri::Builder::default()
    .runtime(tauri_runtime_qt::Qt::default())
    .invoke_handler(tauri::generate_handler![/* … */])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
```

`Qt` carries runtime options: `Qt::default().chromium_flags("--ignore-gpu-blocklist")` and
`.remote_debugging_port(9222)`. Enable the crate's `devtools` feature (with `tauri/devtools`) for
DevTools in release builds. See [`examples/qt-app`](../../examples/qt-app) for a complete app, including
`window.open()` popups via `on_new_window` + `window_features`.

### Build requirements

| Distro        | Packages                                                                                                 |
| ------------- | -------------------------------------------------------------------------------------------------------- |
| Fedora        | `qt6-qtwebengine-devel qt6-qtwebchannel-devel` (+ `qt6-qtbase-private-devel` for Wayland window handles) |
| Debian/Ubuntu | `qt6-webengine-dev qt6-webchannel-dev` (+ `qt6-base-private-dev`)                                        |
| Arch          | `qt6-webengine qt6-webchannel`                                                                           |

Runtime dependency for packages: `qt6-qtwebengine` (rpm), `libqt6webenginewidgets6` (deb).

### Running the examples

The Qt crates aren't default members of the Tauri workspace (they need the Qt dev packages, and
in a build that also has `tauri-runtime-wry`, GTK menus take precedence and Qt windows get none),
so select them with `-p`:

- [`examples/qt-app`](../../examples/qt-app): a showcase of (nearly) every Tauri API, plus the
  runtime's end-to-end self-tests (`cargo run -p qt-app`, `QT_APP_AUTO=1 cargo run -p qt-app`).
- Tauri's own examples: `cargo run -p tauri-examples --example helloworld --no-default-features
--features qt`, and `cargo run -p api --no-default-features --features qt,prod`.

### Tests

```sh
cargo test -p qtshim --features webengine --lib                     # qtshim unit tests (no Qt)
cargo test -p qtshim --features webengine,test-harness --test qt    # real Qt, offscreen
cargo test -p tauri-runtime-qt                                      # the runtime through the tauri_runtime traits
cargo test -p qtshim-sys-systest --features webengine               # FFI declarations vs cpp/qtshim.h
cargo test -p qt-app --test e2e                                     # the app self-tests, offscreen
```

## Menus

Window menus and context menus are real Qt menus. Choosing this runtime turns on `tauri`'s `qt`
feature, which sends menus to a new Qt backend in [muda](https://github.com/tauri-apps/muda), so
the normal Tauri APIs work unchanged: `Builder::menu`, `Window::set_menu` / `remove_menu` /
`hide_menu` / `show_menu`, `on_menu_event`, `Menu::popup`, and the JS `Menu` API.

- Accelerators become Qt shortcuts (active while the window is focused). Clipboard items (Copy,
  Paste, …) only _display_ their shortcut, so they don't take keys from the page.
- Predefined items work: Copy, Cut, Paste, Select All, Undo, Redo, Minimize, Maximize,
  Fullscreen, Hide, Close Window (goes through Tauri's close handling), Quit, and About (a Qt
  dialog). macOS-only items render disabled.
- Changes (`set_text`, `set_enabled`, `set_checked`, adding/removing items) rebuild the menubar
  on the next event loop turn.
- `WebviewWindowBuilder::parent` / `transient_for` no longer fail: the new window is made
  transient for the parent's Qt window (`QWindow::setTransientParent`).

Muda's Qt backend isn't released yet: the Tauri workspace patches `muda` to the `qt-backend`
branch of [KRTirtho/muda](https://github.com/KRTirtho/muda) (based on muda 0.20.0). That backend
takes `qtshim` from this repository, and the workspace patches it back to `crates/qtshim`, so the
runtime and muda link a single copy of the `qtshim` native library.

## How it works

- **One main loop.** Qt on Linux uses the GLib event dispatcher, so `QApplication::exec()` also
  drives any GTK/GLib sources an app adds. The runtime itself doesn't use GTK. Tauri v3's tray
  icon defaults to `ksni` (D-Bus), which works without GTK.
- **IPC.** Tauri's `ipc://` custom protocol works as is. The `window.ipc.postMessage`
  fallback goes over `QWebChannel`.
- **Custom schemes.** Tauri v3 lists every scheme in `RuntimeInitArgs::custom_schemes`, and they
  are registered before the web engine starts (QtWebEngine requires that). A browser-side
  `QWebEngineUrlRequestInterceptor` tags each request with its webview, so pages can't spoof it.
  That tag is how scheme requests get routed to the correct Tauri webview label. QtWebEngine
  doesn't pass `Origin` to scheme handlers, so it's rebuilt from the request initiator.
- **Byte ranges.** Qt applies `Range` itself, so a handler's `206` response is fed to Qt as a
  sparse device covering the whole resource. Video seeking through `asset://` works without
  re-reading files.
- **Permissions.** Requests go to Tauri's permission handler. If there's none, or it answers
  `Default`, the user sees an Allow/Deny prompt, which is remembered per origin for the session.
- **Popups.** `window.open()` raises Tauri's new-window handler with a `QtWindowOpener`. A
  window built with `.window_features(features)` and returned as `NewWindowResponse::Create`
  receives the popup through `QWebEngineNewWindowRequest::openIn`, so `window.opener` works.
- **History.** `go_back`/`go_forward` run `history.go(±1)` in an isolated world. Chromium's
  history-manipulation intervention makes browser-initiated Back skip entries an app added with
  `pushState`, which breaks SPA routers; renderer-initiated navigation behaves like WebKitGTK.
- **Renderer crashes.** These are reported to `on_web_content_process_terminate`. Without a
  handler, the page reloads.
- **Lifecycle.** `CloseRequested`/`Destroyed`/`ExitRequested` semantics mirror
  `tauri-runtime-wry`: closing the last window requests an exit, and either can be prevented.

## Limitations

- **Positioning on Wayland.** Wayland doesn't let apps place windows or warp the pointer, so
  `set_position`, `center` and `set_cursor_position` only work on X11.
- **Status codes.** QtWebEngine scheme handlers can't send arbitrary HTTP status codes. Bodies
  are served with 200/206, redirects work, and empty 4xx/5xx responses become network errors.
  Tauri's IPC doesn't depend on status codes.
- **App-wide settings.** User agent, proxy and extra Chromium flags apply per profile or app-wide,
  not per webview.
- **Unsupported (no-ops).** Content protection, visible-on-all-workspaces and shadows.
  `skip_taskbar` maps to `Qt::Tool`.
- **Frameless resizing.** Frameless windows resize from a 5 px border, but the cursor doesn't
  change shape on hover.

## Layout

| Path                          | What                                                                                                                                     |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/qtshim-sys`           | Raw FFI (`links = "qtshim"`) and the only C++: `cpp/qtshim.h` (the C API), `core.cpp`, `menu.cpp`, `webengine.cpp` (feature `webengine`) |
| `crates/qtshim`               | Safe API over it: `app`, `menu`, `webengine` (feature), `test_harness` (feature). Shared by this runtime and muda's Qt backend           |
| `crates/tauri-runtime-qt/src` | `Runtime`, `RuntimeHandle`, `WindowDispatch`, `WebviewDispatch`, `WindowBuilder`                                                         |
| `examples/qt-app`             | Example app and end-to-end suite (see above)                                                                                             |
