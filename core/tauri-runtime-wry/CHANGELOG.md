# Changelog

## \[0.12.1]

- Fix `allowlist > app > show/hide` always disabled when `allowlist > app > all: false`.
  - Bumped due to a bump in tauri-utils.
  - [bb251087](https://www.github.com/tauri-apps/tauri/commit/bb2510876d0bdff736d36bf3a465cdbe4ad2b90c) fix(core): extend allowlist with `app`'s allowlist, closes [#5650](https://www.github.com/tauri-apps/tauri/pull/5650) ([#5652](https://www.github.com/tauri-apps/tauri/pull/5652)) on 2022-11-18

## \[0.12.0]

- Add `accept_first_mouse` option for macOS windows.
  - [95f467ad](https://www.github.com/tauri-apps/tauri/commit/95f467add51448319983c54e2f382c7c09fb72d6) feat(core): add window `accept_first_mouse` option, closes [#5347](https://www.github.com/tauri-apps/tauri/pull/5347) ([#5374](https://www.github.com/tauri-apps/tauri/pull/5374)) on 2022-10-17
- Disable automatic window tabbing on macOS when the `tabbing_identifier` option is not defined, the window is transparent or does not have decorations.
  - [4137ab44](https://www.github.com/tauri-apps/tauri/commit/4137ab44a81d739556cbc7583485887e78952bf1) feat(macos): add `tabbing_identifier` option, closes [#2804](https://www.github.com/tauri-apps/tauri/pull/2804), [#3912](https://www.github.com/tauri-apps/tauri/pull/3912) ([#5399](https://www.github.com/tauri-apps/tauri/pull/5399)) on 2022-10-19
- Drop the WebContext when the WebView is dropped.
  - [9d8b3774](https://www.github.com/tauri-apps/tauri/commit/9d8b377481abf975dc37f9050d2ac7b63ce353e9) feat(tauri-runtime-wry): drop the WebContext on WebView drop ([#5240](https://www.github.com/tauri-apps/tauri/pull/5240)) on 2022-10-19
- Readd the option to create an unfocused window via the `focused` method. The `focus` function has been deprecated.
  - [4036e15f](https://www.github.com/tauri-apps/tauri/commit/4036e15f5af933bdc0d0913508b5103958afc143) feat(core): reimplement window initial focus flag, closes [#5120](https://www.github.com/tauri-apps/tauri/pull/5120) ([#5338](https://www.github.com/tauri-apps/tauri/pull/5338)) on 2022-10-08
- Add `hidden_title` option for macOS windows.
  - [321f3fed](https://www.github.com/tauri-apps/tauri/commit/321f3fed19df40c1223099bce953332b7f00f7a9) feat(macos): `title_bar_style` and `hidden_title` window options, closes [#2663](https://www.github.com/tauri-apps/tauri/pull/2663) ([#3965](https://www.github.com/tauri-apps/tauri/pull/3965)) on 2022-09-30
- Custom protocol headers are now implemented on Linux when running on webkit2gtk 2.36 or above.
  - [357480f4](https://www.github.com/tauri-apps/tauri/commit/357480f4ae43aa8da99f7ba61ae2ee51b4552c60) feat(core): custom protocol headers on Linux, closes [#4496](https://www.github.com/tauri-apps/tauri/pull/4496) ([#5421](https://www.github.com/tauri-apps/tauri/pull/5421)) on 2022-10-17
- Added `Runtime::show()`, `RuntimeHandle::show()`, `Runtime::hide()`, `RuntimeHandle::hide()` for hiding/showing the entire application on macOS.
  - [39bf895b](https://www.github.com/tauri-apps/tauri/commit/39bf895b73ec6b53f5758815396ba85dda6b9c67) feat(macOS): Add application `show` and `hide` methods ([#3689](https://www.github.com/tauri-apps/tauri/pull/3689)) on 2022-10-03
- Fix regression in `SystemTray::with_menu_on_left_click`
  - [f8a3becb](https://www.github.com/tauri-apps/tauri/commit/f8a3becb287942db7f7b551b5db6aeb5a2e939ee) feat(core): add option to disable tray menu on left click, closes [#4584](https://www.github.com/tauri-apps/tauri/pull/4584) ([#4587](https://www.github.com/tauri-apps/tauri/pull/4587)) on 2022-07-05
  - [7bbf167c](https://www.github.com/tauri-apps/tauri/commit/7bbf167c1c84493ea6e2353f720edafd7daa47e4) Apply Version Updates From Current Changes ([#4560](https://www.github.com/tauri-apps/tauri/pull/4560)) on 2022-07-06
  - [63011ca8](https://www.github.com/tauri-apps/tauri/commit/63011ca84e7a22c8c0d8bd1c1be6592140f93ff2) fix(macos): fix regression in `with_menu_on_left_click`, closes [#5220](https://www.github.com/tauri-apps/tauri/pull/5220) ([#5235](https://www.github.com/tauri-apps/tauri/pull/5235)) on 2022-09-30
- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28
- Added `tabbing_identifier` to the window builder on macOS.
  - [4137ab44](https://www.github.com/tauri-apps/tauri/commit/4137ab44a81d739556cbc7583485887e78952bf1) feat(macos): add `tabbing_identifier` option, closes [#2804](https://www.github.com/tauri-apps/tauri/pull/2804), [#3912](https://www.github.com/tauri-apps/tauri/pull/3912) ([#5399](https://www.github.com/tauri-apps/tauri/pull/5399)) on 2022-10-19
- Add `title_bar_style` option for macOS windows.
  - [321f3fed](https://www.github.com/tauri-apps/tauri/commit/321f3fed19df40c1223099bce953332b7f00f7a9) feat(macos): `title_bar_style` and `hidden_title` window options, closes [#2663](https://www.github.com/tauri-apps/tauri/pull/2663) ([#3965](https://www.github.com/tauri-apps/tauri/pull/3965)) on 2022-09-30
- Fix regression introduce in tauri@1.1 which prevented removing tray icon when the app exits on Windows.
  - [f756cd5e](https://www.github.com/tauri-apps/tauri/commit/f756cd5e7ecc86f178f8d602eded1e1b6ecb51f3) fix(core): wait for tray cleanup before exiting app, closes [#5244](https://www.github.com/tauri-apps/tauri/pull/5244) ([#5245](https://www.github.com/tauri-apps/tauri/pull/5245)) on 2022-10-04
- Added methods to set the system tray title on macOS.
  - [8f1ace77](https://www.github.com/tauri-apps/tauri/commit/8f1ace77956ac3477826ceb059a191e55b3fff93) feat: expose `set_title` for MacOS tray ([#5182](https://www.github.com/tauri-apps/tauri/pull/5182)) on 2022-09-30
- Added the `user_agent` option when creating a window.
  - [a6c94119](https://www.github.com/tauri-apps/tauri/commit/a6c94119d8545d509723b147c273ca5edfe3729f) feat(core): expose user_agent to window config ([#5317](https://www.github.com/tauri-apps/tauri/pull/5317)) on 2022-10-02

## \[0.11.1]

- Add missing allowlist config for `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position` APIs.
  - Bumped due to a bump in tauri-utils.
  - [c764408d](https://www.github.com/tauri-apps/tauri/commit/c764408da7fae123edd41115bda42fa75a4731d2) fix: Add missing allowlist config for cursor apis, closes [#5207](https://www.github.com/tauri-apps/tauri/pull/5207) ([#5211](https://www.github.com/tauri-apps/tauri/pull/5211)) on 2022-09-16

## \[0.11.0]

- Ignore window events with unknown IDs.
  - [0668dd42](https://www.github.com/tauri-apps/tauri/commit/0668dd42204b163f11aaf31f45106c8551f15942) fix(tauri-runtime-wry): ignore events on unknown windows on 2022-08-29
- Implement theme APIs for Linux.
  - [f21cbecd](https://www.github.com/tauri-apps/tauri/commit/f21cbecdeb3571ac4ad971b9a865ff62a131a176) feat(core): implement theme APIs for Linux ([#4808](https://www.github.com/tauri-apps/tauri/pull/4808)) on 2022-08-02
- Changed `windows` map to be stored in a `RefCell` instead of a `Mutex`.
  - [64546cb9](https://www.github.com/tauri-apps/tauri/commit/64546cb9cca2fe56cf81cfc4aaf85c4e1d58877c) refactor: use RefCell instead of Mutex for windows map, closes [#4870](https://www.github.com/tauri-apps/tauri/pull/4870) ([#4909](https://www.github.com/tauri-apps/tauri/pull/4909)) on 2022-08-10
- Added APIs to create a system tray at runtime.
  - [4d063ae9](https://www.github.com/tauri-apps/tauri/commit/4d063ae9ee9538cd6fa5e01b80070c6edf8eaeb9) feat(core): create system tray at runtime, closes [#2278](https://www.github.com/tauri-apps/tauri/pull/2278) ([#4862](https://www.github.com/tauri-apps/tauri/pull/4862)) on 2022-08-09
- Update windows to 0.39.0 and webview2-com to 0.19.1.
  - [e6d9b670](https://www.github.com/tauri-apps/tauri/commit/e6d9b670b0b314ed667b0e164f2c8d27048e678f) refactor: remove unneeded focus code ([#5065](https://www.github.com/tauri-apps/tauri/pull/5065)) on 2022-09-03

## \[0.10.2]

- Disable drag-n-drop of tao based on `fileDropEnabled` value.
  - [a1d569bb](https://www.github.com/tauri-apps/tauri/commit/a1d569bbc9cfdd58258916df594911e1c512a75e) fix(core): disable tao's drag-n-drop based on `fileDropEnabled`, closes [#4580](https://www.github.com/tauri-apps/tauri/pull/4580) ([#4592](https://www.github.com/tauri-apps/tauri/pull/4592)) on 2022-07-05
- Added option to disable tray menu on left click on macOS.
  - [f8a3becb](https://www.github.com/tauri-apps/tauri/commit/f8a3becb287942db7f7b551b5db6aeb5a2e939ee) feat(core): add option to disable tray menu on left click, closes [#4584](https://www.github.com/tauri-apps/tauri/pull/4584) ([#4587](https://www.github.com/tauri-apps/tauri/pull/4587)) on 2022-07-05

## \[0.10.1]

- Fixes a deadlock on the file drop handler.
  - [23a48007](https://www.github.com/tauri-apps/tauri/commit/23a48007c0df7346fa45c76dfaf9235a157f59ec) fix(tauri-runtime-wry): deadlock on file drop, closes [#4527](https://www.github.com/tauri-apps/tauri/pull/4527) ([#4535](https://www.github.com/tauri-apps/tauri/pull/4535)) on 2022-06-30
- Send theme value only once on the getter function implementation on macOS.
  - [63841c10](https://www.github.com/tauri-apps/tauri/commit/63841c10609c3d7337ba6cd68ae126b18987014d) fix(tauri-runtime-wry): do not send theme twice on macOS, closes [#4532](https://www.github.com/tauri-apps/tauri/pull/4532) ([#4540](https://www.github.com/tauri-apps/tauri/pull/4540)) on 2022-06-30
- Fixes a deadlock when the window focus change on Windows.
  - [185b0e31](https://www.github.com/tauri-apps/tauri/commit/185b0e314ece9563cd7c83a16466b2b8b9167eb3) fix(tauri-runtime-wry): deadlock when window focus change, closes [#4533](https://www.github.com/tauri-apps/tauri/pull/4533) ([#4539](https://www.github.com/tauri-apps/tauri/pull/4539)) on 2022-06-30

## \[0.10.0]

- Implement `raw_window_handle::HasRawWindowHandle` on Linux.
  - [3efbc67f](https://www.github.com/tauri-apps/tauri/commit/3efbc67f7469ce65a2d9ea4ff2b60b51d2a36aa5) feat: implement `raw_window_handle` on Linux ([#4469](https://www.github.com/tauri-apps/tauri/pull/4469)) on 2022-06-26
- Moved the window and menu event listeners to the window struct.
  - [46196fe9](https://www.github.com/tauri-apps/tauri/commit/46196fe922f4f1b38057155c6113236cfa4b3597) refactor(tauri-runtime-wry): move window and menu listeners to window ([#4485](https://www.github.com/tauri-apps/tauri/pull/4485)) on 2022-06-27
- Refactored the `tauri-runtime-wry` plugin interface.
  - [e39e2999](https://www.github.com/tauri-apps/tauri/commit/e39e2999e0ab1843a8195ba83aea3d6de705c3d8) refactor(tauri-runtime-wry): enhance plugin interface ([#4476](https://www.github.com/tauri-apps/tauri/pull/4476)) on 2022-06-27
- Removed the `hwnd` and `ns_window` functions from `Dispatch` in favor of `raw_window_handle`.
  - [3efbc67f](https://www.github.com/tauri-apps/tauri/commit/3efbc67f7469ce65a2d9ea4ff2b60b51d2a36aa5) feat: implement `raw_window_handle` on Linux ([#4469](https://www.github.com/tauri-apps/tauri/pull/4469)) on 2022-06-26
- The theme API is now implemented on macOS 10.14+.
  - [6d94ce42](https://www.github.com/tauri-apps/tauri/commit/6d94ce42353204a02fe9c82ed397d349439f75ef) feat(core): theme is now implemented on macOS ([#4380](https://www.github.com/tauri-apps/tauri/pull/4380)) on 2022-06-17
- Suppress unused variable warning in release builds.
  - [45981851](https://www.github.com/tauri-apps/tauri/commit/45981851e35119266c1a079e1ff27a39f1fdfaed) chore(lint): unused variable warnings for release builds ([#4411](https://www.github.com/tauri-apps/tauri/pull/4411)) on 2022-06-22
- Update tao to 0.12 and wry to 0.19.
  - [f6edc6df](https://www.github.com/tauri-apps/tauri/commit/f6edc6df29b1c45b483fa87c481a3b95730b131b) chore(deps): update tao to 0.12, wry to 0.19, closes [#3220](https://www.github.com/tauri-apps/tauri/pull/3220) ([#4502](https://www.github.com/tauri-apps/tauri/pull/4502)) on 2022-06-28
- Fixes deadlocks when using window setters in the main thread.
  - [123f6e69](https://www.github.com/tauri-apps/tauri/commit/123f6e69f60ca6d4b2fd738ca3ff5cf016d8e814) fix(tauri-runtime-wry): release windows lock immediately, closes [#4390](https://www.github.com/tauri-apps/tauri/pull/4390) ([#4392](https://www.github.com/tauri-apps/tauri/pull/4392)) on 2022-06-19

## \[0.9.0]

- Upgrade to `stable`!
  - Bumped due to a bump in tauri-utils.
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[0.8.1]

- Add `Menu::os_default` which will create a menu filled with default menu items and submenus.
  - Bumped due to a bump in tauri-runtime.
  - [4c4acc30](https://www.github.com/tauri-apps/tauri/commit/4c4acc3094218dd9cee0f1ad61810c979e0b41fa) feat: implement `Default` for `Menu`, closes [#2398](https://www.github.com/tauri-apps/tauri/pull/2398) ([#4291](https://www.github.com/tauri-apps/tauri/pull/4291)) on 2022-06-15

## \[0.8.0]

- Removed `TrayIcon` and renamed `WindowIcon` to `Icon`, a shared type for both icons.
  - [4ce8e228](https://www.github.com/tauri-apps/tauri/commit/4ce8e228134cd3f22973b74ef26ca0d165fbbbd9) refactor(core): use `Icon` for tray icons ([#4342](https://www.github.com/tauri-apps/tauri/pull/4342)) on 2022-06-14

## \[0.7.0]

- **Breaking change**: Removed the `gtk-tray` and `ayatana-tray` Cargo features.
  - [6216eb49](https://www.github.com/tauri-apps/tauri/commit/6216eb49e72863bfb6d4c9edb8827b21406ac393) refactor(core): drop `ayatana-tray` and `gtk-tray` Cargo features ([#4247](https://www.github.com/tauri-apps/tauri/pull/4247)) on 2022-06-02

## \[0.6.0]

- Account the monitor position when centering a window.
  - [a7a9fde1](https://www.github.com/tauri-apps/tauri/commit/a7a9fde16fb7c35d48d4f97e83ff95b8baf9e090) fix(core): account for monitor position when centering window ([#4166](https://www.github.com/tauri-apps/tauri/pull/4166)) on 2022-05-21
- Update `windows-rs` to `0.37.0`, which requires Rust 1.61.0+.
  - [2326be39](https://www.github.com/tauri-apps/tauri/commit/2326be39821890cdd4de76e7029a531424dcb26f) feat(core): update windows-rs to 0.37.0 ([#4199](https://www.github.com/tauri-apps/tauri/pull/4199)) on 2022-05-24

## \[0.5.2]

- Use the event loop proxy to create a window so it doesn't deadlock on Windows.
  - [61e37652](https://www.github.com/tauri-apps/tauri/commit/61e37652b931520424d6a93a134e67893703d992) fix(core): deadlock when creating window from IPC handler, closes [#4121](https://www.github.com/tauri-apps/tauri/pull/4121) ([#4123](https://www.github.com/tauri-apps/tauri/pull/4123)) on 2022-05-13

## \[0.5.1]

- Added the `plugin` method to the `Wry` runtime, allowing extensions to the event loop.
  - [c8e0e5b9](https://www.github.com/tauri-apps/tauri/commit/c8e0e5b97d542e549b37be08b545515c862af0e5) feat(tauri-runtime-wry): add plugin API ([#4094](https://www.github.com/tauri-apps/tauri/pull/4094)) on 2022-05-10
- Update wry to 0.16.2 and webkit2gtk to 0.18.0.
  - [71a553b7](https://www.github.com/tauri-apps/tauri/commit/71a553b715312e2bcceb963c83e42cffca7a63bc) chore(deps): update wry to 0.16.2, webkit2gtk to 0.18.0 ([#4099](https://www.github.com/tauri-apps/tauri/pull/4099)) on 2022-05-10

## \[0.5.0]

- The file drop event payloads are now percent-decoded.
  - [a0ecd81a](https://www.github.com/tauri-apps/tauri/commit/a0ecd81a934e1aa8935151a74cad686786054204) fix(core): percent decode file drop payloads, closes [#4034](https://www.github.com/tauri-apps/tauri/pull/4034) ([#4035](https://www.github.com/tauri-apps/tauri/pull/4035)) on 2022-05-03
- Fixes a crash when using the menu with the inspector window focused on macOS. In this case the `window_id` will be the id of the first app window.
  - [891eb748](https://www.github.com/tauri-apps/tauri/commit/891eb748cf590895dc3f1666f8dbd6082b21e04e) fix(tauri-runtime-wry): menu even panic on macOS inspector, closes [#3875](https://www.github.com/tauri-apps/tauri/pull/3875) ([#4027](https://www.github.com/tauri-apps/tauri/pull/4027)) on 2022-05-02
- Fixes a freeze when calling `set_size` in the main thread on Windows.
  - [8f259f4e](https://www.github.com/tauri-apps/tauri/commit/8f259f4ef89be3da11b57222c8b66af9487ab736) fix(core): use EventLoopProxy to prevent set_size freeze closes [#3990](https://www.github.com/tauri-apps/tauri/pull/3990) ([#4014](https://www.github.com/tauri-apps/tauri/pull/4014)) on 2022-04-30
- Expose methods to access the underlying native handles of the webview.
  - [c82b4761](https://www.github.com/tauri-apps/tauri/commit/c82b4761e1660592472dc55308ad69d9efc5855b) feat(core): expose `with_webview` API to access the platform webview ([#4058](https://www.github.com/tauri-apps/tauri/pull/4058)) on 2022-05-04

## \[0.4.0]

- \**Breaking change::* Added the `clipboard` Cargo feature.
  - [24e4ff20](https://www.github.com/tauri-apps/tauri/commit/24e4ff208ee0fe1a4cc5b10667ea0922ac63dfb5) refactor(core): add clipboard Cargo feature, enhancing binary size ([#3957](https://www.github.com/tauri-apps/tauri/pull/3957)) on 2022-04-24
- Expose Window cursor APIs `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position`.
  - [c54ddfe9](https://www.github.com/tauri-apps/tauri/commit/c54ddfe9338e7eb90b4d5b02dfde687d432d5bc1) feat: expose window cursor APIs, closes [#3888](https://www.github.com/tauri-apps/tauri/pull/3888) [#3890](https://www.github.com/tauri-apps/tauri/pull/3890) ([#3935](https://www.github.com/tauri-apps/tauri/pull/3935)) on 2022-04-21
- Fixes a panic when using the `create_tao_window` API.
  - [320329a9](https://www.github.com/tauri-apps/tauri/commit/320329a9a7d8a249c0fc9dee6db5669057ca8b39) fix(core): insert to webview_id_map on tao window creation, closes [#3883](https://www.github.com/tauri-apps/tauri/pull/3883) ([#3932](https://www.github.com/tauri-apps/tauri/pull/3932)) on 2022-04-21
- Fixes a panic when a menu event is triggered when all windows are minimized on macOS.
  - [70ff55c1](https://www.github.com/tauri-apps/tauri/commit/70ff55c1aa69ed59cd2a78d865e1cb398ef2a4ba) fix(core): panic on menu event with minimized windows, closes [#3902](https://www.github.com/tauri-apps/tauri/pull/3902) ([#3918](https://www.github.com/tauri-apps/tauri/pull/3918)) on 2022-04-20
- Fixes a rendering issue when resizing the window with the devtools open.
  - [80b714af](https://www.github.com/tauri-apps/tauri/commit/80b714af6b31365b9026bc92f8631b1721950447) fix: rendering issue when resizing with devtools open closes [#3914](https://www.github.com/tauri-apps/tauri/pull/3914) [#3814](https://www.github.com/tauri-apps/tauri/pull/3814) ([#3915](https://www.github.com/tauri-apps/tauri/pull/3915)) on 2022-04-19
- \**Breaking change::* Added the `global-shortcut` Cargo feature.
  - [e11878bc](https://www.github.com/tauri-apps/tauri/commit/e11878bcf7174b261a1fa146fc7d564d12e6312a) refactor(core): add global-shortcut Cargo feature, enhancing binary size ([#3956](https://www.github.com/tauri-apps/tauri/pull/3956)) on 2022-04-24
- Added `WindowEvent::ThemeChanged(theme)`.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21
- Added `theme` getter on `Window`.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21
- Added `theme` setter to the WindowBuilder.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21
- Create webview immediately when executed in the main thread.
  - [fa2baba7](https://www.github.com/tauri-apps/tauri/commit/fa2baba76c8f59c81f2a2f7139033a09d14d89da) feat(core): create webview immediately when running in main thread ([#3891](https://www.github.com/tauri-apps/tauri/pull/3891)) on 2022-04-12

## \[0.3.5]

- Fixes `WindowEvent::Destroyed` not firing.
  - [169b5035](https://www.github.com/tauri-apps/tauri/commit/169b5035a93e3f33a420d4b2b0f8943e6404e07f) fix(core): actually fire `WindowEvent::Destroyed` ([#3797](https://www.github.com/tauri-apps/tauri/pull/3797)) on 2022-03-28

## \[0.3.4]

- Added `close_devtools` and `is_devtools_open` APIs to the `Dispatch` trait.
  - [e05d718a](https://www.github.com/tauri-apps/tauri/commit/e05d718a7b46476d1fe4817c169008080e84f959) feat(core): add hotkey to toggle devtools, closes [#3776](https://www.github.com/tauri-apps/tauri/pull/3776) ([#3791](https://www.github.com/tauri-apps/tauri/pull/3791)) on 2022-03-28
- Emit `RunEvent::Exit` on `tao::event::Event::LoopDestroyed` instead of after `RunEvent::ExitRequested`.
  - [3c4ee7c9](https://www.github.com/tauri-apps/tauri/commit/3c4ee7c997fa3ff696bcfd5b8c82fecaca16bf49) refactor(wry): emit `RunEvent::Exit` on `Event::LoopDestroyed` ([#3785](https://www.github.com/tauri-apps/tauri/pull/3785)) on 2022-03-27
- **Breaking change:** The `MenuItem::About` variant is now associated with a tuple value `(String, AboutMetadata)`.
  - [5fb74332](https://www.github.com/tauri-apps/tauri/commit/5fb74332ab9210ac062d96b0e9afd1c942ee2911) chore(deps): update wry to 0.14, tao to 0.7 ([#3790](https://www.github.com/tauri-apps/tauri/pull/3790)) on 2022-03-28
- Support window parenting on macOS
  - [4e807a53](https://www.github.com/tauri-apps/tauri/commit/4e807a53e2d6d3f3cd5293d90013d5cdded5454e) Support window parenting on macOS, closes [#3751](https://www.github.com/tauri-apps/tauri/pull/3751) ([#3754](https://www.github.com/tauri-apps/tauri/pull/3754)) on 2022-03-23
- The file drop event is now part of the `WindowEvent` enum instead of a having a dedicated handler.
  - [07d1584c](https://www.github.com/tauri-apps/tauri/commit/07d1584cf06ea326aa45d8044bee1b77ecba5006) feat(core): add `WindowEvent::FileDrop`, closes [#3664](https://www.github.com/tauri-apps/tauri/pull/3664) ([#3686](https://www.github.com/tauri-apps/tauri/pull/3686)) on 2022-03-13
- **Breaking change:** Use the dedicated `WindowEvent` enum on `RunEvent`.
  - [edad9f4f](https://www.github.com/tauri-apps/tauri/commit/edad9f4f55dcc69a06cd9d6d5a5068c94ecb77dd) refactor(core): add `RunEvent::WindowEvent` ([#3793](https://www.github.com/tauri-apps/tauri/pull/3793)) on 2022-03-28
- Added `create_proxy` to the `Runtime` and `RuntimeHandle` traits.
  - [5d538ec2](https://www.github.com/tauri-apps/tauri/commit/5d538ec27c246274df4ff5b8057ff78b6364a43f) refactor(core): use the event loop proxy to send updater events ([#3687](https://www.github.com/tauri-apps/tauri/pull/3687)) on 2022-03-15
- Allow specifying a user event type for the event loop message.
  - [5d538ec2](https://www.github.com/tauri-apps/tauri/commit/5d538ec27c246274df4ff5b8057ff78b6364a43f) refactor(core): use the event loop proxy to send updater events ([#3687](https://www.github.com/tauri-apps/tauri/pull/3687)) on 2022-03-15
- Use a random window id instead of `tao::window::WindowId` to not block the thread waiting for the event loop to process the window creation.
  - [7cd39c70](https://www.github.com/tauri-apps/tauri/commit/7cd39c70c9ecd62cc9b60d0ab93f10ce0a6dd8b4) refactor(core): use random window id to simplify window creation, closes [#3645](https://www.github.com/tauri-apps/tauri/pull/3645) [#3597](https://www.github.com/tauri-apps/tauri/pull/3597) ([#3684](https://www.github.com/tauri-apps/tauri/pull/3684)) on 2022-03-15
- Update `wry` to `0.14` and `tao` to `0.7`.
  - [f2d24ef2](https://www.github.com/tauri-apps/tauri/commit/f2d24ef2fbd95ec7d3433ba651964f4aa3b7f48c) chore(deps): update wry ([#1482](https://www.github.com/tauri-apps/tauri/pull/1482)) on 2021-04-14
  - [e267ebf1](https://www.github.com/tauri-apps/tauri/commit/e267ebf1f1009b99829e0a7d71519925f5792f9f) Apply Version Updates From Current Changes ([#1486](https://www.github.com/tauri-apps/tauri/pull/1486)) on 2021-04-14
  - [5fb74332](https://www.github.com/tauri-apps/tauri/commit/5fb74332ab9210ac062d96b0e9afd1c942ee2911) chore(deps): update wry to 0.14, tao to 0.7 ([#3790](https://www.github.com/tauri-apps/tauri/pull/3790)) on 2022-03-28
- Added the `WindowEvent::FileDrop` variant.
  - [07d1584c](https://www.github.com/tauri-apps/tauri/commit/07d1584cf06ea326aa45d8044bee1b77ecba5006) feat(core): add `WindowEvent::FileDrop`, closes [#3664](https://www.github.com/tauri-apps/tauri/pull/3664) ([#3686](https://www.github.com/tauri-apps/tauri/pull/3686)) on 2022-03-13

## \[0.3.3]

- Fixes a deadlock on the `Focused` event when the window is not visible.
  - [c08cc6d5](https://www.github.com/tauri-apps/tauri/commit/c08cc6d50041ec887d3070c41bb2c793dbac5155) fix(core): deadlock on focus events with invisible window,[#3534](https://www.github.com/tauri-apps/tauri/pull/3534) ([#3622](https://www.github.com/tauri-apps/tauri/pull/3622)) on 2022-03-06
- **Breaking change:** Move `ico` and `png` parsing behind `icon-ico` and `icon-png` Cargo features.
  - [8c935872](https://www.github.com/tauri-apps/tauri/commit/8c9358725a17dcc2acaf4d10c3f654afdff586b0) refactor(core): move `png` and `ico` behind Cargo features ([#3588](https://www.github.com/tauri-apps/tauri/pull/3588)) on 2022-03-05
- Print a warning to stderr if the window transparency has been set to true but `macos-private-api` is not enabled.
  - [080755b5](https://www.github.com/tauri-apps/tauri/commit/080755b5377a3c0a17adf1d03e63555350422f0a) feat(core): warn if private APIs are not enabled, closes [#3481](https://www.github.com/tauri-apps/tauri/pull/3481) ([#3511](https://www.github.com/tauri-apps/tauri/pull/3511)) on 2022-02-19

## \[0.3.2]

- Fix requirements for `RuntimeHandle`, `ClipboardManager`, `GlobalShortcutHandle` and `TrayHandle`.
  - Bumped due to a bump in tauri-runtime.
  - [84895a9c](https://www.github.com/tauri-apps/tauri/commit/84895a9cd270fc743e236d0f4d4cd6210b24a30f) fix(runtime): trait requirements ([#3489](https://www.github.com/tauri-apps/tauri/pull/3489)) on 2022-02-17

## \[0.3.1]

- Change default value for the `freezePrototype` configuration to `false`.
  - Bumped due to a bump in tauri-utils.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

## \[0.3.0]

- Fix `window.center` panic when window size is bigger than screen size.
  - [76ce9f61](https://www.github.com/tauri-apps/tauri/commit/76ce9f61dd3c5bdd589c7557543894e1f770dd16) fix(core): fix `window.center` panic when window size > screen, closes [#2978](https://www.github.com/tauri-apps/tauri/pull/2978) ([#3002](https://www.github.com/tauri-apps/tauri/pull/3002)) on 2021-12-09
- Enable non-session cookie persistence on Linux.
  - [d7c02a30](https://www.github.com/tauri-apps/tauri/commit/d7c02a30a56de79100804969138b379e703f0e07) feat(core): persist non-session cookies on Linux ([#3052](https://www.github.com/tauri-apps/tauri/pull/3052)) on 2021-12-09
- Fixes a deadlock when creating a window from a menu event handler.
  - [9c82006b](https://www.github.com/tauri-apps/tauri/commit/9c82006b2fe166d20510183e36cee099bf96e8d9) fix(core): deadlock when creating window from menu handler, closes [#3110](https://www.github.com/tauri-apps/tauri/pull/3110) ([#3126](https://www.github.com/tauri-apps/tauri/pull/3126)) on 2021-12-28
- Fixes `WindowEvent::Focus` and `WindowEvent::Blur` events not firing.
  - [3b33d67a](https://www.github.com/tauri-apps/tauri/commit/3b33d67aa4f48dcf4e32b3b8a5f45e83808efc2d) fix: re-adding focus/blur events for linux and macos (fix [#2485](https://www.github.com/tauri-apps/tauri/pull/2485)) ([#2489](https://www.github.com/tauri-apps/tauri/pull/2489)) on 2021-08-24
- Use webview's inner_size instead of window's value to get the correct size on macOS.
  - [4c0c780e](https://www.github.com/tauri-apps/tauri/commit/4c0c780e00d8851be38cb1c22f636d9e4ed34a23) fix(core): window's inner_size usage, closes [#2187](https://www.github.com/tauri-apps/tauri/pull/2187) ([#2690](https://www.github.com/tauri-apps/tauri/pull/2690)) on 2021-09-29
- Reimplement `remove_system_tray` on Windows to drop the `SystemTray` to run its cleanup code.
  - [a03b8554](https://www.github.com/tauri-apps/tauri/commit/a03b85545a4b0b61a598a43eabe96e03565dcaf0) fix(core): tray not closing on Windows ([#3351](https://www.github.com/tauri-apps/tauri/pull/3351)) on 2022-02-07
- Replace `WindowBuilder`'s `has_menu` with `get_menu`.
  - [ac37b56e](https://www.github.com/tauri-apps/tauri/commit/ac37b56ef43c9e97039967a5fd99f0d2dccb5b5a) fix(core): menu id map not reflecting the current window menu ([#2726](https://www.github.com/tauri-apps/tauri/pull/2726)) on 2021-10-08
- Fix empty header from CORS on Linux.
  - [b48487e6](https://www.github.com/tauri-apps/tauri/commit/b48487e6a7b33f5a352e542fae21a2efd53ce295) Fix empty header from CORS on Linux, closes [#2327](https://www.github.com/tauri-apps/tauri/pull/2327) ([#2762](https://www.github.com/tauri-apps/tauri/pull/2762)) on 2021-10-18
- The `run_return` API is now available on Linux.
  - [8483fde9](https://www.github.com/tauri-apps/tauri/commit/8483fde975aac8833d2ce426e42fb40aeaeecba9) feat(core): expose `run_return` on Linux ([#3352](https://www.github.com/tauri-apps/tauri/pull/3352)) on 2022-02-07
- Allow window, global shortcut and clipboard APIs to be called on the main thread.
  - [2812c446](https://www.github.com/tauri-apps/tauri/commit/2812c4464b93a365ab955935d05b5cea8cb03aab) feat(core): window, shortcut and clipboard API calls on main thread ([#2659](https://www.github.com/tauri-apps/tauri/pull/2659)) on 2021-09-26
  - [d24fd8d1](https://www.github.com/tauri-apps/tauri/commit/d24fd8d10242da3da143a971d976b42ec4de6079) feat(tauri-runtime-wry): allow window creation and closing on the main thread ([#2668](https://www.github.com/tauri-apps/tauri/pull/2668)) on 2021-09-27
- Change event loop callbacks definition to allow callers to move in mutable values.
  - [bdbf905e](https://www.github.com/tauri-apps/tauri/commit/bdbf905e5d802b58693d2bd27582ce4269faf79c) Transformed event-loop callback to FnMut to allow mutable values ([#2667](https://www.github.com/tauri-apps/tauri/pull/2667)) on 2021-09-27
- **Breaking change:** Add `macos-private-api` feature flag, enabled via `tauri.conf.json > tauri > macOSPrivateApi`.
  - [6ac21b3c](https://www.github.com/tauri-apps/tauri/commit/6ac21b3cef7f14358df38cc69ea3d277011accaf) feat: add private api feature flag ([#7](https://www.github.com/tauri-apps/tauri/pull/7)) on 2022-01-09
- Refactor `create_tao_window` API to return `Weak<Window>` instead of `Arc<Window>`.
  - [c1494b35](https://www.github.com/tauri-apps/tauri/commit/c1494b353233c6a9552d7ace962fdf8d5b1f199a) refactor: return Weak<Window> on create_tao_window on 2021-08-31
- Added `any_thread` constructor on the `Runtime` trait (only possible on Linux and Windows).
  - [af44bf81](https://www.github.com/tauri-apps/tauri/commit/af44bf8168310cf77fbe102a53e7c433f11641a3) feat(core): allow app run on any thread on Linux & Windows, closes [#3172](https://www.github.com/tauri-apps/tauri/pull/3172) ([#3353](https://www.github.com/tauri-apps/tauri/pull/3353)) on 2022-02-07
- Added `run_on_main_thread` API on `RuntimeHandle`.
  - [53fdfe52](https://www.github.com/tauri-apps/tauri/commit/53fdfe52bb30d52653c72ca9f42506c3863dcf4a) feat(core): expose `run_on_main_thread` API ([#2711](https://www.github.com/tauri-apps/tauri/pull/2711)) on 2021-10-04
- **Breaking change:** Renamed the `RPC` interface to `IPC`.
  - [3420aa50](https://www.github.com/tauri-apps/tauri/commit/3420aa5031b3274a95c6c5fa0f8683ca13213396) refactor: IPC handler \[TRI-019] ([#9](https://www.github.com/tauri-apps/tauri/pull/9)) on 2022-01-09
- Added `open_devtools` to the `Dispatcher` trait.
  - [55aa22de](https://www.github.com/tauri-apps/tauri/commit/55aa22de80c3de873e29bcffcb5b2fe236a637a6) feat(core): add `Window#open_devtools` API, closes [#1213](https://www.github.com/tauri-apps/tauri/pull/1213) ([#3350](https://www.github.com/tauri-apps/tauri/pull/3350)) on 2022-02-07
- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22
- Replace all of the `winapi` crate references with the `windows` crate, and replace `webview2` and `webview2-sys` with `webview2-com` and `webview2-com-sys` built with the `windows` crate. This goes along with updates to the TAO and WRY `next` branches.
  - [bb00d5bd](https://www.github.com/tauri-apps/tauri/commit/bb00d5bd6c9dfcb6bdd0d308dadb70e6c6aafe5c) Replace winapi with windows crate and use webview2-com instead of webview2 ([#2615](https://www.github.com/tauri-apps/tauri/pull/2615)) on 2021-09-24
- Update the `windows` crate to 0.25.0, which comes with pre-built libraries. WRY and Tao can both reference the same types directly from the `windows` crate instead of sharing bindings in `webview2-com-sys`.
  - [34be6cf3](https://www.github.com/tauri-apps/tauri/commit/34be6cf37a98ee7cbd66623ebddae08e5a6520fd) Update webview2-com and windows crates ([#2875](https://www.github.com/tauri-apps/tauri/pull/2875)) on 2021-11-11
- This is a temporary fix of null pointer crash on `get_content` of web resource request.
  We will switch it back once upstream is updated.
  - [84f6e3e8](https://www.github.com/tauri-apps/tauri/commit/84f6e3e84a34b01b7fa04f5c4719acb921ef4263) Switch to next branch of wry ([#2574](https://www.github.com/tauri-apps/tauri/pull/2574)) on 2021-09-10
- Update wry to 0.13.
  - [343ea3e2](https://www.github.com/tauri-apps/tauri/commit/343ea3e2e8d51bac63ab651289295c26fcc841d8) Update wry to 0.13 ([#3336](https://www.github.com/tauri-apps/tauri/pull/3336)) on 2022-02-06

## \[0.2.1]

- Migrate to latest custom protocol allowing `Partial content` streaming and Header parsing.
  - [539e4489](https://www.github.com/tauri-apps/tauri/commit/539e4489e0bac7029d86917e9982ea49e02fe489) refactor: custom protocol ([#2503](https://www.github.com/tauri-apps/tauri/pull/2503)) on 2021-08-23

## \[0.2.0]

- Fix blur/focus events being incorrect on Windows.
  - [d832d575](https://www.github.com/tauri-apps/tauri/commit/d832d575d9b03a0ff78accabe4631cc638c08c3b) fix(windows): use webview events on windows ([#2277](https://www.github.com/tauri-apps/tauri/pull/2277)) on 2021-07-23

- Add `ExitRequested` event that allows preventing the app from exiting when all windows are closed, and an `AppHandle.exit()` function to exit the app manually.
  - [892c63a0](https://www.github.com/tauri-apps/tauri/commit/892c63a0538f8d62680dce5848657128ad6b7af3) feat([#2287](https://www.github.com/tauri-apps/tauri/pull/2287)): Add `ExitRequested` event to let users prevent app from exiting ([#2293](https://www.github.com/tauri-apps/tauri/pull/2293)) on 2021-08-09

- Update gtk and its related libraries to v0.14. This also remove requirements of `clang` as build dependency.
  - [63ad3039](https://www.github.com/tauri-apps/tauri/commit/63ad303903bbee7c9a7382413b342e2a05d3ea75) chore(linux): bump gtk to v0.14 ([#2361](https://www.github.com/tauri-apps/tauri/pull/2361)) on 2021-08-07

- Implement `Debug` on public API structs and enums.
  - [fa9341ba](https://www.github.com/tauri-apps/tauri/commit/fa9341ba18ba227735341530900714dba0f27291) feat(core): implement `Debug` on public API structs/enums, closes [#2292](https://www.github.com/tauri-apps/tauri/pull/2292) ([#2387](https://www.github.com/tauri-apps/tauri/pull/2387)) on 2021-08-11

- Fix the error "cannot find type MenuHash in this scope"
  - [226414d1](https://www.github.com/tauri-apps/tauri/commit/226414d1a588c8bc2b540a71fcd84c318319d6af) "cannot find type `MenuHash` in this scope" ([#2240](https://www.github.com/tauri-apps/tauri/pull/2240)) on 2021-07-20

- Panic when a dispatcher getter method (`Window`, `GlobalShortcutHandle`, `ClipboardManager` and `MenuHandle` APIs) is called on the main thread.
  - [50ffdc06](https://www.github.com/tauri-apps/tauri/commit/50ffdc06fbde56aba32b4291fd130104935d1408) feat(core): panic when a dispatcher getter is used on the main thread ([#2455](https://www.github.com/tauri-apps/tauri/pull/2455)) on 2021-08-16

- Remove menu feature flag since there's no package dependency need to be installed on any platform anymore.
  - [f81ebddf](https://www.github.com/tauri-apps/tauri/commit/f81ebddfcc1aea0d4989706aef43538e8ea98bea) feat: remove menu feature flag ([#2415](https://www.github.com/tauri-apps/tauri/pull/2415)) on 2021-08-13

- Adds `Resumed` and `MainEventsCleared` variants to the `RunEvent` enum.
  - [6be3f433](https://www.github.com/tauri-apps/tauri/commit/6be3f4339168651fe4e003b09f7d181fd12cd5a8) feat(core): add `Resumed` and `MainEventsCleared` events, closes [#2127](https://www.github.com/tauri-apps/tauri/pull/2127) ([#2439](https://www.github.com/tauri-apps/tauri/pull/2439)) on 2021-08-15

- Adds `set_activation_policy` API to the `Runtime` trait (macOS only).
  - [4a031add](https://www.github.com/tauri-apps/tauri/commit/4a031add69014a1f3823f4ea19b172a2557f6794) feat(core): expose `set_activation_policy`, closes [#2258](https://www.github.com/tauri-apps/tauri/pull/2258) ([#2420](https://www.github.com/tauri-apps/tauri/pull/2420)) on 2021-08-13

- Allow creation of empty Window with `create_tao_window()` and management with `send_tao_window_event()` on the AppHandler.
  - [88080855](https://www.github.com/tauri-apps/tauri/commit/8808085541a629b8e22b612a06cef01cf9b3722e) feat(window): Allow creation of Window without `wry` ([#2321](https://www.github.com/tauri-apps/tauri/pull/2321)) on 2021-07-29
  - [15566cfd](https://www.github.com/tauri-apps/tauri/commit/15566cfd64f5072fa4980a6ce5b33259958e9021) feat(core): add API to send wry window message to the event loop ([#2339](https://www.github.com/tauri-apps/tauri/pull/2339)) on 2021-08-02

- - Support [macOS tray icon template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) to adjust automatically based on taskbar color.

- Images you mark as template images should consist of only black and clear colors. You can use the alpha channel in the image to adjust the opacity of black content, however.

- [426a6b49](https://www.github.com/tauri-apps/tauri/commit/426a6b49962de8faf061db2e820ac10fcbb300d6) feat(macOS): Implement tray icon template ([#2322](https://www.github.com/tauri-apps/tauri/pull/2322)) on 2021-07-29

- Add `Event::Ready` on the `run()` callback. Triggered once when the event loop is ready.
  - [28c6b7ad](https://www.github.com/tauri-apps/tauri/commit/28c6b7adfe98e701b158e936eafb7541ddc700e0) feat: add `Event::Ready` ([#2433](https://www.github.com/tauri-apps/tauri/pull/2433)) on 2021-08-15

- Add webdriver support to Tauri.
  - [be76fb1d](https://www.github.com/tauri-apps/tauri/commit/be76fb1dfe73a1605cc2ad246418579f4c2e1999) WebDriver support ([#1972](https://www.github.com/tauri-apps/tauri/pull/1972)) on 2021-06-23
  - [b4426eda](https://www.github.com/tauri-apps/tauri/commit/b4426eda9e64fcdd25a2d72e548b8b0fbfa09619) Revert "WebDriver support ([#1972](https://www.github.com/tauri-apps/tauri/pull/1972))" on 2021-06-23
  - [4b2aa356](https://www.github.com/tauri-apps/tauri/commit/4b2aa35684632ed2afd7dec4ad848df5704868e4) Add back WebDriver support ([#2324](https://www.github.com/tauri-apps/tauri/pull/2324)) on 2021-08-01

## \[0.1.4]

- Allow preventing window close when the user requests it.
  - [8157a68a](https://www.github.com/tauri-apps/tauri/commit/8157a68af1d94de1b90a14aa44139bb123b3436b) feat(core): allow listening to event loop events & prevent window close ([#2131](https://www.github.com/tauri-apps/tauri/pull/2131)) on 2021-07-06
- Fixes SVG loading on custom protocol.
  - [e663bdd5](https://www.github.com/tauri-apps/tauri/commit/e663bdd5938830ab4eba961e69c3985191b499dd) fix(core): svg mime type ([#2129](https://www.github.com/tauri-apps/tauri/pull/2129)) on 2021-06-30
- Fixes `center` and `focus` not being allowed in `tauri.conf.json > tauri > windows` and ignored in `WindowBuilderWrapper`.
  - [bc2c331d](https://www.github.com/tauri-apps/tauri/commit/bc2c331dec3dec44c79e659b082b5fb6b65cc5ea) fix: center and focus not being allowed in config ([#2199](https://www.github.com/tauri-apps/tauri/pull/2199)) on 2021-07-12
- Expose `gtk_window` getter.
  - [e0a8e09c](https://www.github.com/tauri-apps/tauri/commit/e0a8e09cab6799eeb9ec524b5f7780d1e5a84299) feat(core): expose `gtk_window`, closes [#2083](https://www.github.com/tauri-apps/tauri/pull/2083) ([#2141](https://www.github.com/tauri-apps/tauri/pull/2141)) on 2021-07-02
- Remove a few locks requirement in tauri-runtime-wry
  - [6569c2bf](https://www.github.com/tauri-apps/tauri/commit/6569c2bf5caf24b009cad1e2cffba25418d6bb68) refactor(wry): remove a few locks requirements ([#2137](https://www.github.com/tauri-apps/tauri/pull/2137)) on 2021-07-02
- Fix macOS high CPU usage.
  - [a280ee90](https://www.github.com/tauri-apps/tauri/commit/a280ee90af0749ce18d6d0b00939b06473717bc9) Fix high cpu usage on mac, fix [#2074](https://www.github.com/tauri-apps/tauri/pull/2074) ([#2125](https://www.github.com/tauri-apps/tauri/pull/2125)) on 2021-06-30
- Bump `wry` 0.11 and fix focus integration to make it compatible with tao 0.4.
  - [f0a8db62](https://www.github.com/tauri-apps/tauri/commit/f0a8db62e445dbbc5770e7addf0390ce3844c1ea) core(deps): bump `wry` to `0.11` ([#2210](https://www.github.com/tauri-apps/tauri/pull/2210)) on 2021-07-15
- `Params` has been removed, along with all the associated types on it. Functions that previously accepted those
  associated types now accept strings instead. Type that used a generic parameter `Params` now use `Runtime` instead. If
  you use the `wry` feature, then types with a `Runtime` generic parameter should default to `Wry`, letting you omit the
  explicit type and let the compiler infer it instead.

`tauri`:

- See `Params` note
- If you were using `Params` inside a function parameter or definition, all references to it have been replaced with a
  simple runtime that defaults to `Wry`. If you are not using a custom runtime, just remove `Params` from the definition
  of functions/items that previously took it. If you are using a custom runtime, you *may* need to pass the runtime type
  to these functions.
- If you were using custom types for `Params` (uncommon and if you don't understand you probably were not using it), all
  methods that were previously taking the custom type now takes an `Into<String>` or a `&str`. The types were already
  required to be string-able, so just make sure to convert it into a string before passing it in if this breaking change
  affects you.

`tauri-macros`:

- (internal) Added private `default_runtime` proc macro to allow us to give item definitions a custom runtime only when
  the specified feature is enabled.

`tauri-runtime`:

- See `Params` note
- Removed `Params`, `MenuId`, `Tag`, `TagRef`.
- Added `menu::{MenuHash, MenuId, MenuIdRef}` as type aliases for the internal type that menu types now use.
  - All previous menu items that had a `MenuId` generic now use the underlying `MenuId` type without a generic.
- `Runtime`, `RuntimeHandle`, and `Dispatch` have no more generic parameter on `create_window(...)` and instead use the
  `Runtime` type directly
- `Runtime::system_tray` has no more `MenuId` generic and uses the string based `SystemTray` type directly.
- (internal) `CustomMenuItem::id_value()` is now hashed on creation and exposed as the `id` field with type `MenuHash`.

`tauri-runtime-wry`:

- See `Params` note
- update menu and runtime related types to the ones changed in `tauri-runtime`.

`tauri-utils`:

- `Assets::get` signature has changed to take a `&AssetKey` instead of `impl Into<AssetKey>` to become trait object
  safe.
- [fd8fab50](https://www.github.com/tauri-apps/tauri/commit/fd8fab507c8fa1b113b841af14c6693eb3955f6b) refactor(core): remove `Params` and replace with strings ([#2191](https://www.github.com/tauri-apps/tauri/pull/2191)) on 2021-07-15

## \[0.1.3]

- `Window` is now `Send + Sync` on Windows.
  - [fe32afcc](https://www.github.com/tauri-apps/tauri/commit/fe32afcc933920d6282ae1d63b041b182278a031) fix(core): `Window` must be `Send + Sync` on Windows, closes [#2078](https://www.github.com/tauri-apps/tauri/pull/2078) ([#2093](https://www.github.com/tauri-apps/tauri/pull/2093)) on 2021-06-27

## \[0.1.2]

- Adds `clipboard` APIs (write and read text).
  - [285bf64b](https://www.github.com/tauri-apps/tauri/commit/285bf64bf9569efb2df904c69c6df405ff0d62e2) feat(core): add clipboard writeText and readText APIs ([#2035](https://www.github.com/tauri-apps/tauri/pull/2035)) on 2021-06-21
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Fixes window event being emitted to all windows listeners.
  - [fca97640](https://www.github.com/tauri-apps/tauri/commit/fca976404e6bec373a81332572458c4c44f7bb3a) fix(wry): window event listeners being emitted to all windows ([#2056](https://www.github.com/tauri-apps/tauri/pull/2056)) on 2021-06-23
- Panic on window getters usage on the main thread when the event loop is not running and document it.
  - [ab3eb44b](https://www.github.com/tauri-apps/tauri/commit/ab3eb44bac7a3bf73a4985df38ccc2b87a913be7) fix(core): deadlock on window getters, fixes [#1893](https://www.github.com/tauri-apps/tauri/pull/1893) ([#1998](https://www.github.com/tauri-apps/tauri/pull/1998)) on 2021-06-16
- Adds `focus` API to the WindowBuilder.
  - [5f351622](https://www.github.com/tauri-apps/tauri/commit/5f351622c7812ad1bb56ddb37364ccaa4124c24b) feat(core): add focus API to the WindowBuilder and WindowOptions, [#1737](https://www.github.com/tauri-apps/tauri/pull/1737) on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds support to PNG icons.
  - [40b717ed](https://www.github.com/tauri-apps/tauri/commit/40b717edc57288a1393fad0529390e101ab903c1) feat(core): set window icon on Linux, closes [#1922](https://www.github.com/tauri-apps/tauri/pull/1922) ([#1937](https://www.github.com/tauri-apps/tauri/pull/1937)) on 2021-06-01
- Adds `is_decorated` getter on Window.
  - [f58a2114](https://www.github.com/tauri-apps/tauri/commit/f58a2114fbfd5307c349f05c88f2e08fd8baa8aa) feat(core): add `is_decorated` Window getter on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds `is_resizable` getter on Window.
  - [1e8af280](https://www.github.com/tauri-apps/tauri/commit/1e8af280c27f381828d6209722b10e889082fa00) feat(core): add `is_resizable` Window getter on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds `is_visible` getter on Window.
  - [36506c96](https://www.github.com/tauri-apps/tauri/commit/36506c967de82bc7ff453d11e6104ecf66d7a588) feat(core): add `is_visible` API on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Removes `image` dependency. For now only `.ico` icons on Windows are supported, and we'll implement other types on demand to optimize bundle size.
  - [1be37a3f](https://www.github.com/tauri-apps/tauri/commit/1be37a3f30ff789d9396ec9009f9c0dd0bb928a7) refactor(core): remove `image` dependency ([#1859](https://www.github.com/tauri-apps/tauri/pull/1859)) on 2021-05-18
- The `run_on_main_thread` API now uses WRY's UserEvent, so it wakes the event loop.
  - [9bf82f0d](https://www.github.com/tauri-apps/tauri/commit/9bf82f0d9261808f58bdb5b5dbd6a255e5dcd333) fix(core): `run_on_main_thread` now wakes the event loop ([#1949](https://www.github.com/tauri-apps/tauri/pull/1949)) on 2021-06-04
- Adds global shortcut interfaces.
  - [3280c4aa](https://www.github.com/tauri-apps/tauri/commit/3280c4aa91e50a8ccdd561a8b48a12a4a13ea8d5) refactor(core): global shortcut is now provided by `tao` ([#2031](https://www.github.com/tauri-apps/tauri/pull/2031)) on 2021-06-21
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds `request_user_attention` API to the `Dispatcher` trait.
  - [7dcca6e9](https://www.github.com/tauri-apps/tauri/commit/7dcca6e9281182b11ad3d4a79871f09b30b9b419) feat(core): add `request_user_attention` API, closes [#2023](https://www.github.com/tauri-apps/tauri/pull/2023) ([#2026](https://www.github.com/tauri-apps/tauri/pull/2026)) on 2021-06-20
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds `fn run_iteration` (macOS and Windows only) to the Runtime trait.
  - [8c0d0739](https://www.github.com/tauri-apps/tauri/commit/8c0d0739eebf7286b64a5380e922746411eb52c6) feat(core): add `run_iteration`, `parent_window` and `owner_window` APIs, closes [#1872](https://www.github.com/tauri-apps/tauri/pull/1872) ([#1874](https://www.github.com/tauri-apps/tauri/pull/1874)) on 2021-05-21
- Adds `show_menu`, `hide_menu` and `is_menu_visible` APIs to the `Dispatcher` trait.
  - [954460c5](https://www.github.com/tauri-apps/tauri/commit/954460c5205d57444ef4b1412051fbedf3e38676) feat(core): MenuHandle `show`, `hide`, `is_visible` and `toggle` APIs ([#1958](https://www.github.com/tauri-apps/tauri/pull/1958)) on 2021-06-15
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds `set_focus` API on Window.
  - [bb6992f8](https://www.github.com/tauri-apps/tauri/commit/bb6992f888196ca7c87bb2fe74ad2bd8bf393e05) feat(core): add `set_focus` window API, fixes [#1737](https://www.github.com/tauri-apps/tauri/pull/1737) on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds `set_skip_taskbar` API on Window.
  - [e06aa277](https://www.github.com/tauri-apps/tauri/commit/e06aa277384450cfef617c0e57b0d5d403bb1e7f) feat(core): add `set_skip_taskbar` API on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Update `wry` to v0.10.0 and replace the removed `dispatch_script` and `evaluate_script` methods with the new `evaluate_script` method in `handle_event_loop`.
  - [cca8115d](https://www.github.com/tauri-apps/tauri/commit/cca8115d9c813d13efb30a38445d5bda009a7f97) refactor: update wry, simplify script eval ([#1965](https://www.github.com/tauri-apps/tauri/pull/1965)) on 2021-06-16
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds `skip_taskbar` API to the WindowBuilder.
  - [5525b03a](https://www.github.com/tauri-apps/tauri/commit/5525b03a78a2232c650043fbd9894ce1553cad41) feat(core): add `skip_taskbar` API to the WindowBuilder/WindowOptions on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- Adds `Window#center` and `WindowBuilder#center` APIs.
  - [5cba6eb4](https://www.github.com/tauri-apps/tauri/commit/5cba6eb4d28d53f06855d60d4d0eae6b95233ccf) feat(core): add window `center` API, closes [#1822](https://www.github.com/tauri-apps/tauri/pull/1822) ([#1954](https://www.github.com/tauri-apps/tauri/pull/1954)) on 2021-06-05
- Adds `parent_window` and `owner_window` setters to the `WindowBuilder` (Windows only).
  - [8c0d0739](https://www.github.com/tauri-apps/tauri/commit/8c0d0739eebf7286b64a5380e922746411eb52c6) feat(core): add `run_iteration`, `parent_window` and `owner_window` APIs, closes [#1872](https://www.github.com/tauri-apps/tauri/pull/1872) ([#1874](https://www.github.com/tauri-apps/tauri/pull/1874)) on 2021-05-21
- Adds window native handle getter (HWND on Windows).
  - [abf78c58](https://www.github.com/tauri-apps/tauri/commit/abf78c5860cdc52fbfd2bc5dbca29a864e2da8f9) fix(core): set parent window handle on dialogs, closes [#1876](https://www.github.com/tauri-apps/tauri/pull/1876) ([#1889](https://www.github.com/tauri-apps/tauri/pull/1889)) on 2021-05-21

## \[0.1.1]

- Fixes `system-tray` feature usage.
  - [1ab8dd9](https://www.github.com/tauri-apps/tauri/commit/1ab8dd93e670d2a2d070c7a6ec48308a0ab32f1a) fix(core): `system-tray` cargo feature usage, fixes [#1798](https://www.github.com/tauri-apps/tauri/pull/1798) ([#1801](https://www.github.com/tauri-apps/tauri/pull/1801)) on 2021-05-12
- Fixes webview transparency.
  - [f5a480f](https://www.github.com/tauri-apps/tauri/commit/f5a480fea34ab3a75751f1ca760a38b3e53da2cc) fix(core): window transparency ([#1800](https://www.github.com/tauri-apps/tauri/pull/1800)) on 2021-05-12

## \[0.1.0]

- **Breaking:** `Context` fields are now private, and is expected to be created through `Context::new(...)`.
  All fields previously available through `Context` are now public methods.
  - [5542359](https://www.github.com/tauri-apps/tauri/commit/55423590ddbf560684dab6a0214acf95aadfa8d2) refactor(core): Context fields now private, Icon used on all platforms ([#1774](https://www.github.com/tauri-apps/tauri/pull/1774)) on 2021-05-11
- `tauri-runtime-wry` initial release.
  - [45a7a11](https://www.github.com/tauri-apps/tauri/commit/45a7a111e0cf9d9956d713cc9a99fa7a5313eec7) feat(core): add `tauri-wry` crate ([#1756](https://www.github.com/tauri-apps/tauri/pull/1756)) on 2021-05-09
