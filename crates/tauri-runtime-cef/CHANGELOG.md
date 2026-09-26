# Changelog

## [3.0.0-alpha.3]

### Bug Fixes

- [`a5e9d3aa9`](https://www.github.com/tauri-apps/tauri/commit/a5e9d3aa936251834d0ac8282aece199e668a5e7) ([#16084](https://www.github.com/tauri-apps/tauri/pull/16084)) Fixed `Window::set_fullscreen` and `set_fullscreen_on_monitor` going wrong on macOS when called while the window was still animating its previous fullscreen transition: AppKit drops the toggle then, leaving the runtime's idea of the fullscreen state out of step with the window (a request to leave fullscreen was lost, and the next request to enter left it instead). Requests made during a transition are now applied once it ended.
- [`a5e9d3aa9`](https://www.github.com/tauri-apps/tauri/commit/a5e9d3aa936251834d0ac8282aece199e668a5e7) ([#16084](https://www.github.com/tauri-apps/tauri/pull/16084)) Custom protocol handlers, the `ipc` handler included, now run on the main thread like they do with the wry runtime, instead of on a thread per request. Commands that wait on the main thread — creating a menu or a menu item, for one — deadlocked with the event loop delivering run events, and commands that dropped main-thread-only objects, such as removing a tray icon on macOS, crashed the app.
- [`c39bba557`](https://www.github.com/tauri-apps/tauri/commit/c39bba55772b497873c397cd8e8061c28e39d242) Fix runtime crates failing to compile when feature unification enables `tauri-runtime/macos-private-api` but not the runtime crate's own `macos-private-api` feature. `WindowBuilder::transparent` is no longer gated on the feature; on macOS, runtimes implement it as a no-op unless their own `macos-private-api` feature is enabled, so the private API is never referenced without it.

### Dependencies

- Upgraded to `tauri@3.0.0-alpha.3`
- Upgraded to `tauri-utils@3.0.0-alpha.2`
- Upgraded to `tauri-runtime@3.0.0-alpha.2`
- Upgraded to `tauri-macros@3.0.0-alpha.2`

## [3.0.0-alpha.2]

### New Features

- [`e2af2c298`](https://www.github.com/tauri-apps/tauri/commit/e2af2c29824e675382ca9601ea5f1ec630504811) Added `Cef::downgrade` and `DowngradePolicy`, for an application whose release is rolled back to an older CEF. The runtime now records the Chromium version in the root cache path (`Last Version`, the breadcrumb Chrome's own downgrade manager keeps and CEF does not write) and, when a newer Chromium milestone last used the profile, either keeps it and logs a warning (`DowngradePolicy::KeepProfile`, the default and what Chrome does) or moves it aside so Chromium starts on an empty profile and deletes the old one in the background (`DowngradePolicy::ResetProfile`). A downgrade within a milestone, and a profile another running instance of the application holds, are left alone.
- [`9c3bb4d28`](https://www.github.com/tauri-apps/tauri/commit/9c3bb4d28b58d4ede705a04fe0c8cf96f6730ea3) `data_store_identifier` is now supported on the CEF runtime: the identifier names a profile directory under the runtime's cache path, giving the same isolation `data_directory` does (`data_directory` wins when both are set). The runtime also logs a warning when a webview sets an attribute it cannot honour — `transparent`, `accept_first_mouse`, `browser_extensions_enabled` / `extensions_path`, a `background_throttling` policy or an overlay `scroll_bar_style` — naming the runtime-wide `Cef` API that does the same thing where one exists. The attributes' documentation on `tauri` and `tauri-utils` now describes each one's CEF behaviour.

### Enhancements

- [`8d0e40b45`](https://www.github.com/tauri-apps/tauri/commit/8d0e40b45213afe90c98f624617974391f75da70) Log a warning when a webview sets `additional_browser_args`, which the CEF runtime does not support: Chromium's command line is per process, so switches go through `Cef::command_line_arg` instead. The attribute's documentation on `tauri`, `tauri-runtime` and `tauri-utils` now says so.
- [`e2af2c298`](https://www.github.com/tauri-apps/tauri/commit/e2af2c29824e675382ca9601ea5f1ec630504811) The build script now passes the CEF binary distribution `cef-dll-sys` resolved on to the build scripts of dependents, as `DEP_TAURI_RUNTIME_CEF_CEF_DIR`.

### Bug Fixes

- [`e2af2c298`](https://www.github.com/tauri-apps/tauri/commit/e2af2c29824e675382ca9601ea5f1ec630504811) `webview_version` now reports Chromium's version as `MAJOR.MINOR.BUILD.PATCH`; the last two components were swapped.

### Dependencies

- Upgraded to `tauri-utils@3.0.0-alpha.1`
- Upgraded to `tauri-runtime@3.0.0-alpha.1`
- Upgraded to `tauri-macros@3.0.0-alpha.1`
- Upgraded to `tauri@3.0.0-alpha.2`
- [`dfa2f0a5c`](https://www.github.com/tauri-apps/tauri/commit/dfa2f0a5c0516852ee6a7631dc2f4ed63bc4814f) Bump the `windows` crate to 0.62 to match `tauri-runtime`, fixing the Windows build (`HWND` type mismatch in the `WindowBuilder::owner`/`parent` implementations).

### Breaking Changes

- [`4e03a1bb2`](https://www.github.com/tauri-apps/tauri/commit/4e03a1bb298d95526b4c97fde0a367cde9b544c2) ([#16043](https://www.github.com/tauri-apps/tauri/pull/16043)) `tauri_runtime::Error::CreateWindow` now carries the underlying error (`CreateWindow(Box<dyn std::error::Error + Send + Sync>)`), like `CreateWebview`.

## [3.0.0-alpha.1]

### Bug Fixes

- [`6f9d7ce56`](https://www.github.com/tauri-apps/tauri/commit/6f9d7ce565e3aa08689fdb3e2713c75db1697607) Fixed a panic on macOS ("tried to handle event while another event is currently being handled") when a `run_on_main_thread` closure or a window event handler spins a nested AppKit event loop, such as a context menu shown with `tauri::menu::ContextMenu::popup` or a modal dialog. Event-loop wake-ups are now delivered from a run-loop observer at the start of a main-loop pass and only once no `ApplicationHandler` callback is running, so winit's proxy is never performed inside such a nested loop.

### Dependencies

- Upgraded to `tauri@3.0.0-alpha.1`
- [`fc5941fb9`](https://www.github.com/tauri-apps/tauri/commit/fc5941fb92dee89b7ed5c58b334d4c08ca955881) Bump CEF to 152.3.0+152.0.6.

## [3.0.0-alpha.0]

### Bug Fixes

- [`e5e42eb75`](https://www.github.com/tauri-apps/tauri/commit/e5e42eb75870cd0639331d7642d047425d57cc87) ([#15980](https://www.github.com/tauri-apps/tauri/pull/15980)) Fixed CEF failing to start on Linux with `ContentMainRun failed with exit code 28` by passing `--no-first-run` to Chromium.

### What's Changed

- [`19929799f`](https://www.github.com/tauri-apps/tauri/commit/19929799f42398a6e85adb00ae02f2e7fe46d214) First v3 alpha release!

### Dependencies

- Upgraded to `tauri@3.0.0-alpha.0`
- Upgraded to `tauri-utils@3.0.0-alpha.0`
- Upgraded to `tauri-macros@3.0.0-alpha.0`
- Upgraded to `tauri-runtime@3.0.0-alpha.0`
