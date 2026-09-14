# Changelog

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
