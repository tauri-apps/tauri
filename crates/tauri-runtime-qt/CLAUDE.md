# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

A Tauri runtime for Linux that replaces tao + wry + WebKitGTK with Qt Widgets + QtWebEngine (Chromium). See README.md for user-facing usage and the list of known limitations.

Paths below are relative to the Tauri repository root. The Qt crates are `crates/tauri-runtime-qt`, `crates/qtshim`, `crates/qtshim-sys` (+ `systest`), and the example `examples/qt-app` (tauri's example layout: `src-tauri/` for the Rust crate, `ui/` for the frontend). Two more examples stay in KRTirtho/tauri-runtime-qt (`examples/qt-modes`: IPC under each Tauri mode; `examples/qt-browser`): they were left out of the Tauri PR to keep it reviewable. They are workspace members but not `default-members` (they need Qt dev packages, and when `tauri`'s `qt` feature meets the `gtk3` feature `tauri-runtime-wry` enables, GTK menus win in both muda and `tauri`'s `build.rs`, with a build warning, and Qt windows get no menus), so always pass `-p`. They were moved here from the now-temporary KRTirtho/tauri-runtime-qt repository.

muda's Qt backend lives in KRTirtho/muda (branch `qt-backend`, muda 0.20.0 + a `qt` backend; local clone at `../muda`), patched in through `[patch.crates-io]` in the workspace `Cargo.toml`. muda takes `qtshim` from this repository on GitHub (KRTirtho/tauri, branch `qt-support`); the workspace's `[patch."https://github.com/KRTirtho/tauri"]` maps them back to the local crates so there is one `qtshim` native library. To try unpushed muda changes: `cargo build -p qt-app --config 'patch.crates-io.muda.path="../muda"'`.

Repository conventions apply: every `.rs`/`.js`/`.cpp`/`.h` file starts with Tauri's license header (`node .scripts/ci/check-license-header.js <files>`), user-facing changes get a `.changes/*.md` covector file (packages `tauri-runtime-qt`, `qtshim`, `qtshim-sys` are in `.changes/config.json`), and the crates use `[lints] workspace = true` (`cargo clippy -p <crate> --all-targets -- -D warnings`).

## Commands

Qt/Chromium tests are heavy: run one suite at a time, capped (6 GB RAM, 3 CPUs), e.g.
`systemd-run --user --scope -p MemoryMax=6G -p MemorySwapMax=0 -p CPUQuota=300% -- nice -n 19 cargo test ...`.

```sh
cargo build -p tauri-runtime-qt -p qt-app      # builds C++ shim (via build.rs), runtime, example
cargo test -p qtshim --features webengine --lib          # qtshim unit tests (no Qt)
cargo test -p qtshim --features webengine,test-harness --test qt -- 'view:'   # real-Qt cases (filter optional)
cargo test -p tauri-runtime-qt --lib                     # runtime unit tests
cargo test -p tauri-runtime-qt --test qt                 # runtime driven through tauri_runtime traits
cargo test -p qtshim-sys-systest --features webengine    # FFI decls vs qtshim.h (ctest + signature check)
cargo test -p qt-app --test e2e                          # the four QT_APP_AUTO modes, offscreen
cargo llvm-cov -p qtshim --features webengine --summary-only   # coverage (child processes included)
# Tauri's examples on this runtime:
cargo run -p tauri-examples --example helloworld --no-default-features --features qt
cargo run -p api --no-default-features --features qt,prod
# muda's Qt backend (in ../muda):
RUSTFLAGS="--cfg muda_qt_tests" cargo test --no-default-features --features qt --lib --test qt

QT_APP_AUTO=1 cargo run -p qt-app            # end-to-end self-test in a real Tauri app; exits 0 when all checks pass
QT_APP_AUTO=close cargo run -p qt-app        # same, but exits by closing the last window (tests ExitRequested{code: None})
QT_APP_AUTO=smoke cargo run -p qt-app        # clicks through the showcase UI (ui/app.js `smokeTest`) and checks the event log
QT_APP_AUTO=crash cargo run -p qt-app        # kills the renderer (chrome://kill) once; on_web_content_process_terminate must reload
cargo run -p qt-app                          # interactive (camera / screen-share buttons)
RUST_LOG=tauri_runtime_qt=trace ...          # logs every scheme request/response and view event
```

Qt allows one `QApplication` per process (and qtshim refuses to create it twice), so real-Qt tests are `harness = false` binaries on `qtshim::test_harness` (feature `test-harness`; Qt-agnostic, the caller passes teardown): each `Case` runs in its own child process on the main thread, offscreen (`QT_QPA_PLATFORM=offscreen`; WebGL and real window-manager behaviour aren't available), at most 2 at a time with one Chromium renderer each. Helpers for qtshim cases are in `crates/qtshim/tests/qt/common.rs` (`run_until`, `wait_for`, `record_events`). A bug found by a test is kept as an `.ignored()` case with a `// BUG:` comment until fixed.

`examples/qt-app` is an interactive showcase (`ui/index.html`, `ui/app.js`, commands and hooks in `src-tauri/src/main.rs`) plus the regression suites: the self-test (`ui/selftest.js`, `runSelfTest`) and the showcase smoke test (`smokeTest` in `ui/app.js`). Each check reports through the `report` command and prints `[PASS]`/`[FAIL]` to stdout. Add a check when adding runtime behaviour.

Requires Qt 6 WebEngine dev packages (Fedora: `qt6-qtwebengine-devel qt6-qtwebchannel-devel`, optionally `qt6-qtbase-private-devel`). `crates/qtshim-sys/build.rs` finds Qt via `pkg-config`, runs `moc` (override with `QT_MOC`) when the `webengine` feature is on, and defines `QS_HAVE_QPA` only if the QPA private header exists (needed for Wayland `wl_surface` handles).

Fast C++-only syntax check (run under bash; zsh won't word-split `$I`):
```sh
bash -c 'I="$(pkg-config --cflags Qt6WebEngineWidgets Qt6WebChannel Qt6Widgets Qt6PrintSupport Qt6DBus)"; for f in core menu webengine; do g++ -std=c++17 -fsyntax-only -Wall -Wextra crates/qtshim-sys/cpp/$f.cpp -Icrates/qtshim-sys/cpp $I; done'
```

## Architecture

Layers (a `-sys` crate, a safe crate, then the runtime); changes usually touch all of them:

1. `crates/qtshim-sys/cpp/` — the only C++, a flat C API with the `qs_` prefix. `qtshim.h` is the contract (event kinds, flag bits, field meanings; sections marked `[webengine]` are feature-gated). `internal.h` holds shared C++ helpers (`qs::Ev` event builder, `runOnMain`, shutdown hooks). `core.cpp` (app, event channel, screens), `menu.cpp` (QMenuBar/QMenu/QAction, debug helpers), `webengine.cpp` (windows, views, profiles, schemes, IPC; only with the `webengine` feature). `ipc_bridge.h` is the only moc'd class.
2. `crates/qtshim-sys/src/lib.rs` — raw `extern "C"` declarations mirroring `qtshim.h` one to one (`#![no_std]`, webengine items behind `#[cfg(feature = "webengine")]`). `links = "qtshim"` guarantees one copy per app.
3. `crates/qtshim/src/` — the safe API: `app` (init/exec/dispatch/post, `Event`/`Reply`, screens, theme), `menu` (`Widget`, a `QPointer`-tracked handle; `MenuBar`/`Menu`/`PopupMenu` via the sealed `Container` trait, inert once Qt destroys them; `StandardAction`, `debug`), `webengine` (`Window`, `View`, `Profile`, schemes, `Responder`). The public API has no `unsafe fn`: `lifecycle.rs` makes every entry point panic off the main thread or outside `init`..`shutdown` (`dispatch`/`Responder` use the `ALIVE` lock instead), every Qt→Rust trampoline holds `enter_callback()` so `shutdown` can refuse to run inside one, and foreign pointers enter only through `Widget::from_ptr` (checked against `QApplication::allWidgets()`). `#![warn(missing_docs)]` is on. Adding a C function means editing `qtshim.h`, the right `.cpp`, `qtshim-sys/src/lib.rs`, and the `qtshim` module.
4. `crates/tauri-runtime-qt/src` — implements `tauri_runtime` traits. `lib.rs`: the `Qt` init-attrs struct (selects the runtime via `tauri::Builder::runtime`, `From<Qt> for DynRuntimeInitAttrs`), `QtRuntime<T>`, shared `Context`, main-thread `MainState`, Qt event routing, window/webview creation, run loop. `dispatch.rs`: `WindowDispatch`/`WebviewDispatch`. `window_builder.rs`: attribute bag. Behaviour is modelled on `tauri-runtime-wry` (lifecycle, close/exit semantics).

In v3 the runtime delivers window/webview events only through the `RunEvent` callback (there are no per-dispatcher listeners), apps use the type-erased `tauri::DynRuntime` by default, and the runtime defines its own `Webview` (`QtWebview`), `WindowOpener` (`QtWindowOpener`), `RuntimeWebviewAttributes` and `RuntimeInitAttrs` types.

### Event flow and threading
- One main loop: `QApplication::exec()`. Qt uses the GLib dispatcher, so GTK sources (tray, dialogs) run in the same loop; the runtime also calls `gtk::init()`.
- All Qt → Rust callbacks go through a single event channel (`qs_set_event_handler` → `qtshim::app::Event` → `handle_event` in the runtime). Events needing an answer (navigate, permission, new window, download, drag) return a `Reply`.
- Dispatchers are `Send`; they reach the main thread via `Context::run_on_main` (blocking; runs inline if already on main) or `Context::post` (always queued). Main-thread state lives in a type-erased `thread_local` (`state::<T>()`).
- Re-entrancy rules — Qt fires events synchronously from inside our own Qt calls:
  - Never hold a `windows` `RefCell` borrow while calling user/Tauri code. Window events that arrive while the map is borrowed are re-posted (`try_borrow_mut` check in `handle_event`).
  - The app callback is wrapped by `MainState::emit`/`flush`: re-entrant events are queued. Close/exit handling, which needs a synchronous answer from the callback, is deferred via `post` when the callback is busy.

### QtWebEngine constraints that shape the design
- Custom schemes must be registered before the first `QWebEngineProfile` is created (engine start). `QtRuntime::init` registers `RuntimeInitArgs::custom_schemes` plus Tauri's defaults. Chromium flags and proxy have the same timing constraint (`Qt::chromium_flags`, or the first webview's `additional_browser_args`/`proxy_url`).
- Scheme handlers are per profile, not per page. Each page has a `ViewTagger` interceptor adding an `X-Qs-View` header; the shim strips it and passes the view id so the runtime can route to the right Tauri webview label (Tauri IPC depends on this).
- Scheme handlers get no `Origin` header; the shim rebuilds it from `job->initiator()` (Tauri's IPC rejects requests without it).
- Status codes can't be set; Qt applies `Range` itself. 206 responses are served through `RangeDevice` (sparse, full-size) — parsed from `Content-Range` in `Responder::respond`.
- Popups: Tauri's new-window handler synchronously builds the target window (`about:blank` + `opener`) and returns `Create`; `create_webview` skips loading when `opener` is set, and the shim calls `QWebEngineNewWindowRequest::openIn(target page)` before the signal returns.
- `go_back`/`go_forward` navigate to an explicit history index (`QWebEngineHistory::goToItem`), and `can_go_*` compare indices: Chromium's history-manipulation intervention makes relative Back/Forward (`triggerAction(Back)`, and in Chromium 140 also `history.go(-1)`) skip gesture-less `pushState` entries, but not `GoToIndex`.
- Request bodies: Qt's body device (`requestBody()`) only works when one `read()` drains it (partial reads re-copy or skip; bodies over 16 KiB are streamed; `size()` is 0), so `readRequestBody` reads once into a lazily committed 256 MiB mapping. Larger bodies fail the request.
- Failing a request: `job->fail()` leaves `fetch()`'s body hanging, so subresources (tagged `X-Qs-Subresource` by the interceptor) fail via a redirect to an unloadable URL; navigations still use `fail()` (error page). Both `X-Qs-*` tags are only added to custom-scheme requests and stripped before the handler.
- `setAdditionalResponseHeaders` (Qt 6.11) lowercases header *values*. `Content-Security-Policy` of HTML responses is therefore moved into `<meta http-equiv>` tags (`injectCsp`), or hash sources stop matching (this broke Tauri's isolation pattern: every IPC call hung). Other headers can't be fixed through the public API.
- Qt serves `Range: bytes=-N` from the first N bytes of the reply device, so suffix-range replies put the resource's last N bytes there (`JobHandle::suffix`).
- Window close is always blocked in Qt (`closeEvent` → `QS_EV_WINDOW_CLOSE_REQ`); Rust decides and destroys.

### Ownership / teardown
- `WindowEntry.webviews` is declared before `window` so views drop first. If a window is destroyed first, the shim marks its views `dead` (Qt side torn down, struct kept) and every `qs_view_*` call becomes a no-op until Rust drops the `View`.
- Qt objects are deleted with `qs::deleteSoon`: immediately only outside the event loop *and* outside any call into Rust (`CallbackScope`, held by `Ev::fire`, menu actions, scheme handler, eval callbacks and posted closures), because Rust may destroy the very object whose signal is running; otherwise `deleteLater` in the loop, or a queued call that `process_events` runs. Event handlers that use a view after `ev.fire()` look it up again by id (`g_views`), since the handler may have destroyed it.
- `run_return` → `teardown` drops windows, then `qtshim::app::shutdown()` runs the registered shutdown hooks (views, windows, profiles) and destroys the `QApplication`.

### GTK
The runtime has no GTK dependency and declares no `tauri_runtime::gtk` version; `gtk_window()`/`default_vbox()` return an error.

### Menus
- `tauri-runtime-qt` depends on `tauri` with `features = ["qt"]` (`crates/tauri/build.rs` sets the `qt` cfg alias). That feature selects muda's Qt backend (`muda/qt`) and adds `#[cfg(qt)]` branches next to `tauri`'s `#[cfg(gtk)]` menu code, calling `menu.init_for_qt_window(window.qt_window())`, `show_context_menu_for_qt_window`, etc. The window/app menu is attached in `WindowBuilder::build` right after the runtime returns the window (Qt has no `after_window_creation` path).
- `QtWindowDispatcher::qt_window` returns the `QsWindow` (`QWidget*`). Each `QsWindow` has a `QVBoxLayout` with a `content` widget holding the webviews; muda puts its `QMenuBar` at the top via `QLayout::setMenuBar`. So view geometry and auto-resize are relative to `content` (`Window::content_size`), not the window. Like a native client area, the window's *inner* size (geometry, `resize`, min/max, `WindowResized`) is that content area below a visible menubar; `WindowResized` also fires when the menubar is shown or hidden.
- muda's Qt backend (`../muda/src/platform_impl/qt/`) has no C++ of its own: it depends on `qtshim` (git, KRTirtho/tauri `qt-support`; patched back to `crates/qtshim` in this workspace) and uses `qtshim::menu`. It rebuilds every attached menubar from muda's item state on any change (coalesced, posted to the Qt loop), tracks windows with `QPointer`, and reuses `MenuItemAction` for clicks.
- `tauri_runtime_qt::debug::{menubar_tree, trigger_menu_item, take_popup_menu}` inspect/drive the Qt menus; the example's self-test uses them. On Wayland a popup opened without user input can be dismissed by the compositor immediately, which is why `take_popup_menu` reports the last popup *shown*.
