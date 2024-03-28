# Changelog

## \[2.0.0-beta.14]

### Bug Fixes

- [`c33f6e6cf`](https://www.github.com/tauri-apps/tauri/commit/c33f6e6cf35a0d34b5598875a2e5b642a01c8b38)([#9211](https://www.github.com/tauri-apps/tauri/pull/9211)) Fixed an issue preventing webview/window creation events to not be emitted. This also fixed the `getByLabel` and `getAll` JavaScript functions.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.11`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.11`
- Upgraded to `tauri-runtime@2.0.0-beta.11`
- Upgraded to `tauri-macros@2.0.0-beta.11`
- Upgraded to `tauri-build@2.0.0-beta.11`

## \[2.0.0-beta.13]

### Enhancements

- [`75f5cb401`](https://www.github.com/tauri-apps/tauri/commit/75f5cb4015f72745161110ad0076cf4945411a6d)([#9214](https://www.github.com/tauri-apps/tauri/pull/9214)) `tauri::Window` and `tauri::WebviewWindow` now implement `raw_window_handle::HasDisplayHandle`.

### Bug Fixes

- [`81b853bc8`](https://www.github.com/tauri-apps/tauri/commit/81b853bc875ce2da4e300614ca234f10d54966a6)([#9213](https://www.github.com/tauri-apps/tauri/pull/9213)) Fixed an issue where errors where returned as strings instead of objects from commands.
- [`43230cb6b`](https://www.github.com/tauri-apps/tauri/commit/43230cb6b7a4b14a23ea8f05636ae06f03c718e9)([#9219](https://www.github.com/tauri-apps/tauri/pull/9219)) Fixes the menu plugin `remove` command signature.

## \[2.0.0-beta.12]

### New Features

- [`e227fe02f`](https://www.github.com/tauri-apps/tauri/commit/e227fe02f986e145c0731a64693e1c830a9eb5b0)([#9156](https://www.github.com/tauri-apps/tauri/pull/9156)) Allow plugins to define (at compile time) JavaScript that are initialized when `withGlobalTauri` is true.

### Enhancements

- [`79b8a3514`](https://www.github.com/tauri-apps/tauri/commit/79b8a3514baedcd9c35e777d2b6d89a7a086ddec)([#9151](https://www.github.com/tauri-apps/tauri/pull/9151)) Improve and optimize event emit calls.

### Bug Fixes

- [`379cc2b35`](https://www.github.com/tauri-apps/tauri/commit/379cc2b3547395474d4b66b4222679cf4538428d)([#9165](https://www.github.com/tauri-apps/tauri/pull/9165)) Fix `basename(path, 'ext')` JS API when removing all occurances of `ext` where it should only remove the last one.

### Dependencies

- Upgraded to `tauri-build@2.0.0-beta.10`
- Upgraded to `tauri-utils@2.0.0-beta.10`
- Upgraded to `tauri-runtime@2.0.0-beta.10`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.10`
- Upgraded to `tauri-macros@2.0.0-beta.10`

### Breaking Changes

- [`acdd76833`](https://www.github.com/tauri-apps/tauri/commit/acdd76833db6d81f4012418133d0042220de100b)([#9155](https://www.github.com/tauri-apps/tauri/pull/9155)) Removed `App/AppHandle::tray` and `App/AppHandle::remove_tray`, use `App/AppHandle::tray_by_id` and `App/AppHandle::remove_tray_by_id` instead. If these APIs were used to access tray icon configured in `tauri.conf.json`, you can use `App/AppHandle::tray_by_id` with ID `main` or the configured value.
- [`ea0242db4`](https://www.github.com/tauri-apps/tauri/commit/ea0242db4aa6c127d2bb4a2e275000ba47c9e68c)([#9179](https://www.github.com/tauri-apps/tauri/pull/9179)) Removed `width` and `height` methods on the JS `Image` class, use `size` instead.

## \[2.0.0-beta.11]

### New Features

- [`490a6b424`](https://www.github.com/tauri-apps/tauri/commit/490a6b424e81714524150aef96fbf6cf7004b940)([#9147](https://www.github.com/tauri-apps/tauri/pull/9147)) The `Assets` trait now include a `setup` method that lets you run initialization code for your custom asset provider.

### Bug Fixes

- [`85de230f3`](https://www.github.com/tauri-apps/tauri/commit/85de230f313da81cbbd061e66e8de64e5b33104c)([#9144](https://www.github.com/tauri-apps/tauri/pull/9144)) Fix old JS listeners being dropped on page load after it was possible to create new listeners.
- [`e673854c8`](https://www.github.com/tauri-apps/tauri/commit/e673854c8333cb8a8d298471737293f17ec5a3ee)([#9133](https://www.github.com/tauri-apps/tauri/pull/9133)) Fixes capability remote domain not allowing subpaths, query parameters and hash when those values are empty.

### Dependencies

- Upgraded to `tauri-macros@2.0.0-beta.9`
- Upgraded to `tauri-utils@2.0.0-beta.9`
- Upgraded to `tauri-build@2.0.0-beta.9`
- Upgraded to `tauri-runtime@2.0.0-beta.9`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.9`

### Breaking Changes

- [`490a6b424`](https://www.github.com/tauri-apps/tauri/commit/490a6b424e81714524150aef96fbf6cf7004b940)([#9147](https://www.github.com/tauri-apps/tauri/pull/9147)) The `Context` struct and the `Assets` trait now takes a `R: Runtime` generic.
- [`ba0206d8a`](https://www.github.com/tauri-apps/tauri/commit/ba0206d8a30a9b43ec5090dcaabd1a23baa1420c)([#9141](https://www.github.com/tauri-apps/tauri/pull/9141)) `Context::assets` now returns `&dyn Assets` instead of `&A` generic.
- [`ba0206d8a`](https://www.github.com/tauri-apps/tauri/commit/ba0206d8a30a9b43ec5090dcaabd1a23baa1420c)([#9141](https://www.github.com/tauri-apps/tauri/pull/9141)) The `Context` type no longer uses the `<A: Assets>` generic so the assets implementation can be swapped with `Context::assets_mut`.
- [`490a6b424`](https://www.github.com/tauri-apps/tauri/commit/490a6b424e81714524150aef96fbf6cf7004b940)([#9147](https://www.github.com/tauri-apps/tauri/pull/9147)) Removed `Context::assets_mut` and added `Context::set_assets`.
- [`db0a24a97`](https://www.github.com/tauri-apps/tauri/commit/db0a24a973191752aeecfbd556faa254b0f17e79)([#9132](https://www.github.com/tauri-apps/tauri/pull/9132)) Use the image crate for `tauri::image::Image` and remove the `from_png_bytes` and `from_ico_bytes` APIs.

## \[2.0.0-beta.10]

### New Features

- [`3e472d0af`](https://www.github.com/tauri-apps/tauri/commit/3e472d0afcd67545dd6d9f18d304580a3b2759a8)([#9115](https://www.github.com/tauri-apps/tauri/pull/9115)) Added `CapabilityBuilder::platform` to link the runtime capability with a specific platform.

### Enhancements

- [`3e472d0af`](https://www.github.com/tauri-apps/tauri/commit/3e472d0afcd67545dd6d9f18d304580a3b2759a8)([#9115](https://www.github.com/tauri-apps/tauri/pull/9115)) Changed the permission and capability platforms to be optional.
- [`9dc9ca6e3`](https://www.github.com/tauri-apps/tauri/commit/9dc9ca6e38be62ef2746c7a4c2b77b2d67c0d998)([#9113](https://www.github.com/tauri-apps/tauri/pull/9113)) Added `tauri::dev()` to determine whether we are running in development mode or not.

### Bug Fixes

- [`5541aafef`](https://www.github.com/tauri-apps/tauri/commit/5541aafef33113bc292558ba125e685135aabab4)([#9107](https://www.github.com/tauri-apps/tauri/pull/9107)) Fix `emit` and `emit_to` (when used with `EventTarget::Any`) always skipping the webview listeners.
- [`80c12ead4`](https://www.github.com/tauri-apps/tauri/commit/80c12ead4655af91f08046f19c2d478a4cbf94cd)([#9121](https://www.github.com/tauri-apps/tauri/pull/9121)) Fix regression on IPC response when using a channel to return objects.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.8`
- Upgraded to `tauri-runtime@2.0.0-beta.8`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.8`
- Upgraded to `tauri-macros@2.0.0-beta.8`
- Upgraded to `tauri-build@2.0.0-beta.8`

### Breaking Changes

- [`4ef17d083`](https://www.github.com/tauri-apps/tauri/commit/4ef17d08336a2e0df4a7ef9adea746d7419710b6)([#9116](https://www.github.com/tauri-apps/tauri/pull/9116)) The ACL configuration for remote URLs now uses the URLPattern standard instead of glob patterns.
- [`ed48e2b3c`](https://www.github.com/tauri-apps/tauri/commit/ed48e2b3c7fa914e4c9af432c02b8154f872c68a)([#9122](https://www.github.com/tauri-apps/tauri/pull/9122)) Expose `tauri::image` module to export the `JsImage` type and removed the `Image` root re-export.

## \[2.0.0-beta.9]

### New Features

- [`46de49aaa`](https://www.github.com/tauri-apps/tauri/commit/46de49aaad4a148fafc31d591be0e2ed12256507)([#9059](https://www.github.com/tauri-apps/tauri/pull/9059)) Added `set_auto_resize` method for the webview.
- [`d1e77acd8`](https://www.github.com/tauri-apps/tauri/commit/d1e77acd8dfdf554b90b542513a58a2de1ef2360)([#9011](https://www.github.com/tauri-apps/tauri/pull/9011)) Add a new `Image` type in Rust and JS.

### Enhancements

- [`a77be9747`](https://www.github.com/tauri-apps/tauri/commit/a77be9747443ffc29c34160b55893483bb5f0d74)([#9038](https://www.github.com/tauri-apps/tauri/pull/9038)) Fallback to the postMessage IPC interface if we cannot reach the IPC custom protocol.
- [`e62ca4ee9`](https://www.github.com/tauri-apps/tauri/commit/e62ca4ee95f4308a6ad128d0f100c85634e28223)([#9070](https://www.github.com/tauri-apps/tauri/pull/9070)) Added a mechanism to preserve channel message order.
- [`03098b531`](https://www.github.com/tauri-apps/tauri/commit/03098b531562e4d58ab12ad9da2acb1eb3480497)([#9036](https://www.github.com/tauri-apps/tauri/pull/9036)) `Manager::add_capability` now allows adding a dynamically defined capability instead of only relying on static strings.
- [`b5c743276`](https://www.github.com/tauri-apps/tauri/commit/b5c7432769b84ffe22db721dcfc6af218026f5d4)([#9086](https://www.github.com/tauri-apps/tauri/pull/9086)) Use a strict content security policy on the isolation pattern iframe.
- [`46de49aaa`](https://www.github.com/tauri-apps/tauri/commit/46de49aaad4a148fafc31d591be0e2ed12256507)([#9059](https://www.github.com/tauri-apps/tauri/pull/9059)) When using the `unstable` feature flag, `WebviewWindow` will internally use the child webview interface for flexibility.

### Bug Fixes

- [`86fa339de`](https://www.github.com/tauri-apps/tauri/commit/86fa339de7b176efafa9b3e89f94dcad5fcd03da)([#9071](https://www.github.com/tauri-apps/tauri/pull/9071)) Fix compile time error in context generation when using `app.windows.windowEffects.color`
- [`947a50b8e`](https://www.github.com/tauri-apps/tauri/commit/947a50b8e28379c452c32eddc3e0101870e50055)([#9049](https://www.github.com/tauri-apps/tauri/pull/9049)) Fix `tauri migrate` for http plugin ACL.
- [`fe18012d3`](https://www.github.com/tauri-apps/tauri/commit/fe18012d30d1d8b3ffa10c8e321710eba644ef94)([#9072](https://www.github.com/tauri-apps/tauri/pull/9072)) Resolve symlinks on the filesystem scope check.
- [`6c0683224`](https://www.github.com/tauri-apps/tauri/commit/6c068322460300e9d56a4fac5b018d4c437daa9e)([#9068](https://www.github.com/tauri-apps/tauri/pull/9068)) Fixes scope resolution grouping scopes for all windows.

### Dependencies

- Upgraded to `tauri-build@2.0.0-beta.7`
- Upgraded to `tauri-utils@2.0.0-beta.7`
- Upgraded to `tauri-runtime@2.0.0-beta.7`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.7`
- Upgraded to `tauri-macros@2.0.0-beta.7`

### Breaking Changes

- [`d1e77acd8`](https://www.github.com/tauri-apps/tauri/commit/d1e77acd8dfdf554b90b542513a58a2de1ef2360)([#9011](https://www.github.com/tauri-apps/tauri/pull/9011)) Renamed `icon-ico` and `icon-png` feature flags to `image-ico` and `image-png` respectively
- [`720357fd5`](https://www.github.com/tauri-apps/tauri/commit/720357fd5cd1fefef8485077dfb116ee39ef4ab4)([#9104](https://www.github.com/tauri-apps/tauri/pull/9104)) Removed `tauri::path::Result` and `tauri::path::Error` which were merely an unintentional re-export of `tauri::Result` and `tauri::Error` so use those instead.
- [`6c0683224`](https://www.github.com/tauri-apps/tauri/commit/6c068322460300e9d56a4fac5b018d4c437daa9e)([#9068](https://www.github.com/tauri-apps/tauri/pull/9068)) The `allows` and `denies` methods from `ipc::ScopeValue`, `ipc::CommandScope` and `ipc::GlobalScope` now returns `&Vec<Arc<T>>` instead of `&Vec<T>`.
- [`d1e77acd8`](https://www.github.com/tauri-apps/tauri/commit/d1e77acd8dfdf554b90b542513a58a2de1ef2360)([#9011](https://www.github.com/tauri-apps/tauri/pull/9011)) Removed `Context::default_window_icon_mut` and `Context::tray_icon_mut`, use `Context::set_default_window_icon` and `Context::set_tray_icon` instead. Also changed `Context::set_tray_icon` to accept `Option<T>`.
- [`d1e77acd8`](https://www.github.com/tauri-apps/tauri/commit/d1e77acd8dfdf554b90b542513a58a2de1ef2360)([#9011](https://www.github.com/tauri-apps/tauri/pull/9011)) Removed `Icon` enum, use the new `Image` type instead. All APIs that previously accepted `Icon` have changed to accept `Image` instead.

## \[2.0.0-beta.8]

### New Features

- [`d7f56fef`](https://www.github.com/tauri-apps/tauri/commit/d7f56fef85cac3af4e2dbac1eac40e5567b1f160)([#9014](https://www.github.com/tauri-apps/tauri/pull/9014)) Allow defining a permission that only applies to a set of target platforms via the `platforms` configuration option.

### Bug Fixes

- [`e1d5b790`](https://www.github.com/tauri-apps/tauri/commit/e1d5b7906369a40df19e8ee86c56f90a27d6357c)([#8995](https://www.github.com/tauri-apps/tauri/pull/8995)) Fixes capability webview label check.
- [`222a96b7`](https://www.github.com/tauri-apps/tauri/commit/222a96b74b145fb48d3f0c109897962d56fae57a)([#8999](https://www.github.com/tauri-apps/tauri/pull/8999)) Fixes `Window::add_child` deadlock.
- [`e4463f08`](https://www.github.com/tauri-apps/tauri/commit/e4463f08145c044bd37dc1c6f5f39e6a572ace3e)([#8930](https://www.github.com/tauri-apps/tauri/pull/8930)) Clear JS event listeneres on page load, which fixes zombie listeners when the page reloads.
- [`222a96b7`](https://www.github.com/tauri-apps/tauri/commit/222a96b74b145fb48d3f0c109897962d56fae57a)([#8999](https://www.github.com/tauri-apps/tauri/pull/8999)) Fixes `Webview::reparent` not updating the webview parent window reference.

### Dependencies

- Upgraded to `tauri-build@2.0.0-beta.6`
- Upgraded to `tauri-utils@2.0.0-beta.6`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.6`
- Upgraded to `tauri-runtime@2.0.0-beta.6`
- Upgraded to `tauri-macros@2.0.0-beta.6`

### Breaking Changes

- [`3657ad82`](https://www.github.com/tauri-apps/tauri/commit/3657ad82f88ce528551d032d521c52eed3f396b4)([#9008](https://www.github.com/tauri-apps/tauri/pull/9008)) Allow defining permissions for the application commands via `tauri_build::Attributes::app_manifest`.

### Breaking Changes

- [`b9e6a018`](https://www.github.com/tauri-apps/tauri/commit/b9e6a01879d9233040f3d3fab11c59e70563da7e)([#8937](https://www.github.com/tauri-apps/tauri/pull/8937)) The `custom-protocol` Cargo feature is no longer required on your application and is now ignored. To check if running on production, use `#[cfg(not(dev))]` instead of `#[cfg(feature = "custom-protocol")]`.

## \[2.0.0-beta.7]

### Enhancements

- [`bc5b5e67`](https://www.github.com/tauri-apps/tauri/commit/bc5b5e671a546512f823f1c157421b4c3311dfc0)([#8984](https://www.github.com/tauri-apps/tauri/pull/8984)) Do not include a CSP tag in the application HTML and rely on the custom protocol response header instead.

### Bug Fixes

- [`6cb601d4`](https://www.github.com/tauri-apps/tauri/commit/6cb601d42e2af75aa818371b8b8f7d5b2e77dc90)([#8983](https://www.github.com/tauri-apps/tauri/pull/8983)) Convert the command name to camelCase when executing a mobile plugin command.
- [`60bf11ab`](https://www.github.com/tauri-apps/tauri/commit/60bf11abcbec8d0362aa256e2293985bfd62620f)([#8986](https://www.github.com/tauri-apps/tauri/pull/8986)) Export `ProgressBarStatus`, regression introduced in `2.0.0-beta.4`

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.5`
- Upgraded to `tauri-runtime@2.0.0-beta.5`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.5`
- Upgraded to `tauri-macros@2.0.0-beta.5`
- Upgraded to `tauri-build@2.0.0-beta.5`

## \[2.0.0-beta.6]

### Bug Fixes

- [`6edc563c`](https://www.github.com/tauri-apps/tauri/commit/6edc563cf9ca26b4622c3135d92e493a5d5bd6e8)([#8953](https://www.github.com/tauri-apps/tauri/pull/8953)) Fixes a deadlock when the window is destroyed.

## \[2.0.0-beta.5]

### New Features

- [`fdcaf935`](https://www.github.com/tauri-apps/tauri/commit/fdcaf935fa75ecfa2806939c4faad4fe9e880386)([#8939](https://www.github.com/tauri-apps/tauri/pull/8939)) Added the `reparent` function to the webview API.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.4`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.4`
- Upgraded to `tauri-build@2.0.0-beta.4`
- Upgraded to `tauri-runtime@2.0.0-beta.4`
- Upgraded to `tauri-macros@2.0.0-beta.4`

## \[2.0.0-beta.4]

### Enhancements

- [`3fb414b6`](https://www.github.com/tauri-apps/tauri/commit/3fb414b61ad7cfce67751230826fddfb39effec5)([#8914](https://www.github.com/tauri-apps/tauri/pull/8914)) Return an id when using from `Manager::once_any`, `App::once`, `Window::once`, `Webview::once`, `WebviewWindow::once` and `fs::Scope::once`.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.3`
- Upgraded to `tauri-runtime@2.0.0-beta.3`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.3`
- Upgraded to `tauri-macros@2.0.0-beta.3`
- Upgraded to `tauri-build@2.0.0-beta.3`

### Breaking Changes

- [`361ec37f`](https://www.github.com/tauri-apps/tauri/commit/361ec37fd4a5caa5b6630b9563ef079f53c6c336)([#8932](https://www.github.com/tauri-apps/tauri/pull/8932)) Moved `ProgressBarState` from `tauri-utils` to the `tauri::window` module and removed the `unity_uri` field.

## \[2.0.0-beta.3]

### New Features

- [`16e550ec`](https://www.github.com/tauri-apps/tauri/commit/16e550ec1503765158cdc3bb2a20e70ec710e981)([#8844](https://www.github.com/tauri-apps/tauri/pull/8844)) Add webview-specific events for multi-webview windows:

  - Add `WebviewEvent` enum
  - Add `RunEvent::WebviewEvent` variant.
  - Add `Builder::on_webview_event` and `Webview::on_webview_event` methods.

### Enhancements

- [`11a5816b`](https://www.github.com/tauri-apps/tauri/commit/11a5816bdffcbaa20df936dee43751de2cf67530)([#8864](https://www.github.com/tauri-apps/tauri/pull/8864)) A file-drop now allows sub-directories recursively when the path is a directory.
- [`0cb0a15c`](https://www.github.com/tauri-apps/tauri/commit/0cb0a15ce22af3d649cf219ac04188c14c5f4905)([#8789](https://www.github.com/tauri-apps/tauri/pull/8789)) Add `webviews` array on the capability for usage on multiwebview contexts.
- [`258494bd`](https://www.github.com/tauri-apps/tauri/commit/258494bd247b6d36485bf16bf7184b93fd299da9)([#8806](https://www.github.com/tauri-apps/tauri/pull/8806)) Added `Manager::add_capability` to add a capability file at runtime.
- [`5618f6d2`](https://www.github.com/tauri-apps/tauri/commit/5618f6d2ffc9ebf40710145538b06bebfa55f878)([#8856](https://www.github.com/tauri-apps/tauri/pull/8856)) Relax requirements on plugin's identifiers to be alphanumeric and `-` instead of only lower alpha and `-`.

### Bug Fixes

- [`16e550ec`](https://www.github.com/tauri-apps/tauri/commit/16e550ec1503765158cdc3bb2a20e70ec710e981)([#8844](https://www.github.com/tauri-apps/tauri/pull/8844)) Fix JS event listeners registered using JS `listen` api with `EventTarget::Any` never fired.
- [`8751c329`](https://www.github.com/tauri-apps/tauri/commit/8751c3299f2b7229c6108aa37dedf1dc5edb3e5c)([#8793](https://www.github.com/tauri-apps/tauri/pull/8793)) Fix invoking toggle devtools by hotkey.
- [`bd73ab0a`](https://www.github.com/tauri-apps/tauri/commit/bd73ab0a1adcf648e38d579b92515dababf34993)([#8766](https://www.github.com/tauri-apps/tauri/pull/8766)) When using the multiwebview mode, properly remove the webview from memory on `Webview::close`.
- [`46b6598a`](https://www.github.com/tauri-apps/tauri/commit/46b6598a94cd0c6fa4a1654ac67432d94ea20ebf)([#8826](https://www.github.com/tauri-apps/tauri/pull/8826)) Fix JS `onCloseRequested` catching close event from other windows.
- [`2e6db908`](https://www.github.com/tauri-apps/tauri/commit/2e6db908d7b3a2c928c46b0ad9ccf9ec55a29480)([#8777](https://www.github.com/tauri-apps/tauri/pull/8777)) Fix regression in `tauri::Error` not being `Sync`.

### What's Changed

- [`76ce9f61`](https://www.github.com/tauri-apps/tauri/commit/76ce9f61dd3c5bdd589c7557543894e1f770dd16)([#3002](https://www.github.com/tauri-apps/tauri/pull/3002)) Enhance centering a newly created window, it will no longer jump to center after being visible.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.2`
- Upgraded to `tauri-build@2.0.0-beta.2`
- Upgraded to `tauri-macros@2.0.0-beta.2`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.2`
- Upgraded to `tauri-runtime@2.0.0-beta.2`

### Breaking Changes

- [`258494bd`](https://www.github.com/tauri-apps/tauri/commit/258494bd247b6d36485bf16bf7184b93fd299da9)([#8806](https://www.github.com/tauri-apps/tauri/pull/8806)) Removed the lifetime parameter from `ipc::GlobalScope` and `ipc::CommandScope`.
- [`f284f9c5`](https://www.github.com/tauri-apps/tauri/commit/f284f9c545deeb77d15b6e8b1d0d05f49c40634c)([#8898](https://www.github.com/tauri-apps/tauri/pull/8898)) Changed the capability `remote` configuration to take a list of `urls` instead of `domains` for more flexibility.
- [`2f55bfec`](https://www.github.com/tauri-apps/tauri/commit/2f55bfecbf0244f3b5aa1ad7622182fca3fcdcbb)([#8795](https://www.github.com/tauri-apps/tauri/pull/8795)) Update raw-window-handle to 0.6.
- [`2e6db908`](https://www.github.com/tauri-apps/tauri/commit/2e6db908d7b3a2c928c46b0ad9ccf9ec55a29480)([#8777](https://www.github.com/tauri-apps/tauri/pull/8777)) Require `ScopeObject::Error` to be `Sync` as well.

## \[2.0.0-beta.2]

### Bug Fixes

- [`fe67ab7f`](https://www.github.com/tauri-apps/tauri/commit/fe67ab7f2532c668b35f15415d876e576b3fb74e)([#8761](https://www.github.com/tauri-apps/tauri/pull/8761)) Workaround for zbus not enabling the proper Cargo features for its nix dependency.

### Dependencies

- Upgraded to `tauri-build@2.0.0-beta.1`
- Upgraded to `tauri-utils@2.0.0-beta.1`
- Upgraded to `tauri-runtime@2.0.0-beta.1`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.1`
- Upgraded to `tauri-macros@2.0.0-beta.1`

## \[2.0.0-beta.1]

### Bug Fixes

- [`863bc9e5`](https://www.github.com/tauri-apps/tauri/commit/863bc9e55f9099207403e34cb5c218231265b2c1)([#8749](https://www.github.com/tauri-apps/tauri/pull/8749)) Fix regression on the JavaScript code that processes the IPC message.

## \[2.0.0-beta.0]

### New Features

- [`74a2a603`](https://www.github.com/tauri-apps/tauri/commit/74a2a6036a5e57462f161d728cbd8a6f121028ca)([#8661](https://www.github.com/tauri-apps/tauri/pull/8661)) Implement access control list for IPC usage.
- [`a2fc3a63`](https://www.github.com/tauri-apps/tauri/commit/a2fc3a63579ca739646d696870cbecbb3a169d33)([#8657](https://www.github.com/tauri-apps/tauri/pull/8657)) Add `visibleOnAllWorkspaces` option when creating the window in JS and `Window.setVisibleOnAllWorkspaces` method.
- [`af610232`](https://www.github.com/tauri-apps/tauri/commit/af6102327376884364b2075b468bdf08ee0d02aa)([#8710](https://www.github.com/tauri-apps/tauri/pull/8710)) Added `Window::destroy` to force close a window.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Add multiwebview support behind the `unstable` feature flag. See `WindowBuilder` and `WebviewBuilder` for more information.
- [`9eaeb5a8`](https://www.github.com/tauri-apps/tauri/commit/9eaeb5a8cd95ae24b5e66205bdc2763cb7f965ce)([#8622](https://www.github.com/tauri-apps/tauri/pull/8622)) Add `WindowBuilder::parent` which is a convenient wrapper around parent functionality for Windows, Linux and macOS. Also added `WindowBuilder::owner` on Windows only. Also added `WindowBuilder::transient_for` and `WindowBuilder::transient_for_raw` on Linux only.

### Enhancements

- [`e8d3793c`](https://www.github.com/tauri-apps/tauri/commit/e8d3793c3c34715569312a91633fde4d58d7621c)([#8732](https://www.github.com/tauri-apps/tauri/pull/8732)) Add `common-controls-v6` cargo feature flag (enabled by default).
- [`58fe2e81`](https://www.github.com/tauri-apps/tauri/commit/58fe2e812a85b9f4eba105286a63f271ea637836)([#8670](https://www.github.com/tauri-apps/tauri/pull/8670)) Allow IPC calls when window origin is a defined custom protocol.

### Bug Fixes

- [`95da1a27`](https://www.github.com/tauri-apps/tauri/commit/95da1a27476e01e06f6ce0335df8535b662dd9c4)([#8713](https://www.github.com/tauri-apps/tauri/pull/8713)) Fix calling `set_activation_policy` when the event loop is running.
- [`e1eb911f`](https://www.github.com/tauri-apps/tauri/commit/e1eb911f5ebe84285aae710e0ebdd945ad389431)([#8582](https://www.github.com/tauri-apps/tauri/pull/8582)) Ensure initalize logic and dropping of menu item is done on the main thread, this fixes the crash when a menu item is dropped on another thread.
- [`a093682d`](https://www.github.com/tauri-apps/tauri/commit/a093682d2df7169b024bb4f736c7f1fd2ea8b327)([#8621](https://www.github.com/tauri-apps/tauri/pull/8621)) Fix can not prevent closing a window from another webview.
- [`7f033f6d`](https://www.github.com/tauri-apps/tauri/commit/7f033f6dcd54c69a4193765a5c1584755ba92c61)([#8537](https://www.github.com/tauri-apps/tauri/pull/8537)) Fix undecorated window resizing on Windows and Linux.

### What's Changed

- [`9f8037c2`](https://www.github.com/tauri-apps/tauri/commit/9f8037c2882abac19582025001675370f0d7b669)([#8633](https://www.github.com/tauri-apps/tauri/pull/8633)) On Windows, fix decorated window not transparent initially until resized.
- [`7f033f6d`](https://www.github.com/tauri-apps/tauri/commit/7f033f6dcd54c69a4193765a5c1584755ba92c61)([#8537](https://www.github.com/tauri-apps/tauri/pull/8537)) Add `Window::start_resize_dragging` and `ResizeDirection` enum.
- [`6639a579`](https://www.github.com/tauri-apps/tauri/commit/6639a579c76d45210f33a72d37e21d4c5a9d334b)([#8441](https://www.github.com/tauri-apps/tauri/pull/8441)) Added the `WindowConfig::proxy_url` `WebviewBuilder::proxy_url() / WebviewWindowBuilder::proxy_url()` options when creating a webview.

### Dependencies

- Upgraded to `tauri-build@22.0.0-beta.0`
- Upgraded to `tauri-utils@2.0.0-beta.0`
- Upgraded to `tauri-macros@2.0.0-beta.0`
- Upgraded to `tauri-runtime@2.0.0-beta.0`
- Upgraded to `tauri-runtime-wry@2.0.0-beta.0`

### Breaking Changes

- [`8de308d1`](https://www.github.com/tauri-apps/tauri/commit/8de308d1bf6a855d7a26af58bd0e744938ba47d8)([#8723](https://www.github.com/tauri-apps/tauri/pull/8723)) Restructured Tauri config per [RFC#5](https://github.com/tauri-apps/rfcs/blob/f3e82a6b0c5390401e855850d47dc7b7d9afd684/texts/0005-tauri-config-restructure.md):

  - Moved `package.productName`, `package.version` and `tauri.bundle.identifier` fields to the top-level.
  - Removed `package` object.
  - Renamed `tauri` object to `app`.
  - Moved `tauri.bundle` object to the top-level.
  - Renamed `build.distDir` field to `frontendDist`.
  - Renamed `build.devPath` field to `devUrl` and will no longer accepts paths, it will only accept URLs.
  - Moved `tauri.pattern` to `app.security.pattern`.
  - Removed `tauri.bundle.updater` object, and its fields have been moved to the updater plugin under `plugins.updater` object.
  - Moved `build.withGlobalTauri` to `app.withGlobalTauri`.
  - Moved `tauri.bundle.dmg` object to `bundle.macOS.dmg`.
  - Moved `tauri.bundle.deb` object to `bundle.linux.deb`.
  - Moved `tauri.bundle.appimage` object to `bundle.linux.appimage`.
  - Removed all license fields from each bundle configuration object and instead added `bundle.license` and `bundle.licenseFile`.
  - Renamed `AppUrl` to `FrontendDist` and refactored its variants to be more explicit.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) The `invoke_system`, `on_page_load` hooks now gives you a `Webview` argument instead of a `Window`.
- [`e1eb911f`](https://www.github.com/tauri-apps/tauri/commit/e1eb911f5ebe84285aae710e0ebdd945ad389431)([#8582](https://www.github.com/tauri-apps/tauri/pull/8582)) All menu item constructors `accelerator` argument have been changed to `Option<impl AsRef<str>>` so when providing `None` you need to specify the generic argument like `None::<&str>`.
- [`e1eb911f`](https://www.github.com/tauri-apps/tauri/commit/e1eb911f5ebe84285aae710e0ebdd945ad389431)([#8582](https://www.github.com/tauri-apps/tauri/pull/8582)) All menu item constructors have been changed to return a `Result<Self>`
- [`aa758a85`](https://www.github.com/tauri-apps/tauri/commit/aa758a850f7a3c8e57520ee4ea63a17689469cb0)([#8716](https://www.github.com/tauri-apps/tauri/pull/8716)) Moved the `command` module items to the `ipc` module so its import name does not clash with the `command` macro.
- [`00e15675`](https://www.github.com/tauri-apps/tauri/commit/00e1567584721644797b587205187f9cbe4e5cd1)([#8708](https://www.github.com/tauri-apps/tauri/pull/8708)) `AppHandle::exit` and `AppHandle::restart` now go triggers `RunEvent::ExitRequested` and `RunEvent::Exit` and cannot be executed on the event loop handler.
- [`ec9818ac`](https://www.github.com/tauri-apps/tauri/commit/ec9818accba567816cb5647c0fac2019f11c027a)([#8696](https://www.github.com/tauri-apps/tauri/pull/8696)) Added a callback to the `App::run_iteration` and removed its return value.
- [`a093682d`](https://www.github.com/tauri-apps/tauri/commit/a093682d2df7169b024bb4f736c7f1fd2ea8b327)([#8621](https://www.github.com/tauri-apps/tauri/pull/8621)) Refactored the event system to better accommodate the new window types:

  - Added `EventTarget` enum.
  - Added `App/AppHandle::listen`, `App/AppHandle::once` and `App/AppHandle::unlisten` to listen to events targeting `App/AppHandle`
  - `App/AppHandle/Window/Webview/WebviewWindow::emit` will now emit to all event listeners.
  - `App/AppHandle/Window/Webview/WebviewWindow::emit_to` will emit to event targets that match the given label, see `EventTarget` enum.
  - `App/AppHandle/Window/Webview/WebviewWindow::emit_filter` will emit to event targets based on a filter callback which now takes `&EventTarget` instead of `&Window`.
  - Renamed `Manager::listen_global` and `Manager::once_global` to `listen_any` and `once_any` respectively to be consistent with `EventTarget::Any`, it will now also listen to any event to any target (aka event sniffer).
- [`9eaeb5a8`](https://www.github.com/tauri-apps/tauri/commit/9eaeb5a8cd95ae24b5e66205bdc2763cb7f965ce)([#8622](https://www.github.com/tauri-apps/tauri/pull/8622)) Renamed `WindowBuilder::owner_window` to `WindowBuilder::owner_raw` and `WindowBuilder::parent_window` to `WindowBuilder::parent_raw`.
- [`9eaeb5a8`](https://www.github.com/tauri-apps/tauri/commit/9eaeb5a8cd95ae24b5e66205bdc2763cb7f965ce)([#8622](https://www.github.com/tauri-apps/tauri/pull/8622)) Changed `WindowBuilder::from_config` to return a `Result<Self>`.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Renamed `Window` to `WebviewWindow`, `WindowBuilder` to `WebviewWindowBuilder`, `Manager::windows` to `Manager::webview_windows` and `Manager::get_window` to `Manager::get_webview_window`.
- [`af610232`](https://www.github.com/tauri-apps/tauri/commit/af6102327376884364b2075b468bdf08ee0d02aa)([#8710](https://www.github.com/tauri-apps/tauri/pull/8710)) `Window::close` now triggers a close requested event instead of forcing the window to be closed.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Renamed the `window-data-url` feature flag to `webview-data-url`.

## \[2.0.0-alpha.21]

### New Features

- [`29ced5ce`](https://www.github.com/tauri-apps/tauri/commit/29ced5ceec40b2934094ade2db9a8855f294e1d1)([#8159](https://www.github.com/tauri-apps/tauri/pull/8159)) Added `WindowBuilder::on_download` to handle download request events.

### Enhancements

- [`d621d343`](https://www.github.com/tauri-apps/tauri/commit/d621d3437ce3947175eecf345b2c6d1c4c7ce020)([#8607](https://www.github.com/tauri-apps/tauri/pull/8607)) Added tracing for window startup, plugins, `Window::eval`, events, IPC, updater and custom protocol request handlers behind the `tracing` feature flag.

### What's Changed

- [`cb640c8e`](https://www.github.com/tauri-apps/tauri/commit/cb640c8e949a3d78d78162e2e61b51bf8afae983)([#8393](https://www.github.com/tauri-apps/tauri/pull/8393)) Fix `RunEvent::WindowEvent(event: WindowEvent::FileDrop(FileDropEvent))` never triggered and always prevent default OS behavior when `disable_file_drop_handler` is not used.

### Dependencies

- Upgraded to `tauri-runtime-wry@1.0.0-alpha.9`
- Upgraded to `tauri-utils@2.0.0-alpha.13`
- Upgraded to `tauri-runtime@1.0.0-alpha.8`
- Upgraded to `tauri-macros@2.0.0-alpha.13`
- Upgraded to `tauri-build@2.0.0-alpha.14`

### Breaking Changes

- [`2032228c`](https://www.github.com/tauri-apps/tauri/commit/2032228cad0de6500616ca765af5c9ff1f231f0f)([#8430](https://www.github.com/tauri-apps/tauri/pull/8430)) Removed `GlobalWindowEvent` struct, and unpacked its field to be passed directly to `tauri::Builder::on_window_event`.

## \[2.0.0-alpha.20]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.12`
- Upgraded to `tauri-runtime@1.0.0-alpha.7`
- Upgraded to `tauri-runtime-wry@1.0.0-alpha.8`
- Upgraded to `tauri-macros@2.0.0-alpha.12`
- Upgraded to `tauri-build@2.0.0-alpha.13`

## \[2.0.0-alpha.19]

### New Features

- [`b59f2f54`](https://www.github.com/tauri-apps/tauri/commit/b59f2f54e7c1e01e33baf710d50c046401edb9d8)([#8432](https://www.github.com/tauri-apps/tauri/pull/8432)) Expose `scope::fs::Scope::new`.
- [`bf095df5`](https://www.github.com/tauri-apps/tauri/commit/bf095df55aa27fb22c9240ddf8d673cfe0a4a2db)([#8276](https://www.github.com/tauri-apps/tauri/pull/8276)) Exposed `Manager::resources_table` to access the resources table used by tauri, which could be used by plugins or app authors to store their resources and retrieve it later using an id and can be used to create Rust-backed resources in JS.

### Enhancements

- [`5848b4e8`](https://www.github.com/tauri-apps/tauri/commit/5848b4e8e9fde1e0d895238cafdb57ed20be4c07)([#8386](https://www.github.com/tauri-apps/tauri/pull/8386)) Fixed the deserialisation of a `Channel` in iOS.
- [`11a1529d`](https://www.github.com/tauri-apps/tauri/commit/11a1529d6ace47e731f5d69bf421e7f59f0d7567)([#8419](https://www.github.com/tauri-apps/tauri/pull/8419)) Include CORS header on custom protocol response errors to ensure frontend can read the error message.
- [`db127777`](https://www.github.com/tauri-apps/tauri/commit/db127777423e467758781d58c1121cbe94844161)([#8380](https://www.github.com/tauri-apps/tauri/pull/8380)) Added `test::get_ipc_response`.

### Bug Fixes

- [`effe5871`](https://www.github.com/tauri-apps/tauri/commit/effe5871aff1267a73ecbba1693304941a691932)([#8420](https://www.github.com/tauri-apps/tauri/pull/8420)) Fixes file scope checks on Android.
- [`f98ce5aa`](https://www.github.com/tauri-apps/tauri/commit/f98ce5aa475d1b3f8606e773579e77f41309feb0)([#8328](https://www.github.com/tauri-apps/tauri/pull/8328)) Fix incorrect menu item for `PredefinedMenuItem::close_window`

## \[2.0.0-alpha.18]

### Bug Fixes

- [`b5f40ae5`](https://www.github.com/tauri-apps/tauri/commit/b5f40ae58dded34b9ddef8e301d0e6e777d135d6)([#8147](https://www.github.com/tauri-apps/tauri/pull/8147)) Fixes global events not reaching to window listeners.

### Dependencies

- Upgraded to `tauri-macros@2.0.0-alpha.11`
- Upgraded to `tauri-build@2.0.0-alpha.12`
- Upgraded to `tauri-utils@2.0.0-alpha.11`
- Upgraded to `tauri-runtime@1.0.0-alpha.5`
- Upgraded to `tauri-runtime-wry@1.0.0-alpha.6`

## \[2.0.0-alpha.17]

### Enhancements

- [`b89de9fa`](https://www.github.com/tauri-apps/tauri/commit/b89de9fa43b793c74a42230c7a82c11c3734278e)([#8092](https://www.github.com/tauri-apps/tauri/pull/8092)) Add support for onResume and onPause events in android plugins.
- [`c6c59cf2`](https://www.github.com/tauri-apps/tauri/commit/c6c59cf2373258b626b00a26f4de4331765dd487) Pull changes from Tauri 1.5 release.
- [`198abe3c`](https://www.github.com/tauri-apps/tauri/commit/198abe3c2cae06dacab860b3a93f715dcf529a95)([#8076](https://www.github.com/tauri-apps/tauri/pull/8076)) Mobile plugins can now resolve using an arbitrary object instead of using the `JSObject` class via `Invoke.resolve` on iOS and `Invoke.resolveObject` on Android.

### Bug Fixes

- [`22f26882`](https://www.github.com/tauri-apps/tauri/commit/22f26882cfe0adbfe4c51586a1c9fdcf8e9cfb68)([#8049](https://www.github.com/tauri-apps/tauri/pull/8049)) Prevent crash on iOS when the Swift plugin data is not a valid JSON string.

### Dependencies

- Upgraded to `tauri-build@2.0.0-alpha.11`
- Upgraded to `tauri-macros@2.0.0-alpha.10`
- Upgraded to `tauri-utils@2.0.0-alpha.10`
- Upgraded to `tauri-runtime@1.0.0-alpha.4`
- Upgraded to `tauri-runtime-wry@1.0.0-alpha.5`
- [`9580df1d`](https://www.github.com/tauri-apps/tauri/commit/9580df1d7b027befb9e5f025ea2cbaf2dcc82c8e)([#8084](https://www.github.com/tauri-apps/tauri/pull/8084)) Upgrade `gtk` to 0.18.
- [`c7c2507d`](https://www.github.com/tauri-apps/tauri/commit/c7c2507da16a9beb71bf06745fe7ac1325ab7c2a)([#8035](https://www.github.com/tauri-apps/tauri/pull/8035)) Update `windows` to version `0.51` and `webview2-com` to version `0.27`
- [`9580df1d`](https://www.github.com/tauri-apps/tauri/commit/9580df1d7b027befb9e5f025ea2cbaf2dcc82c8e)([#8084](https://www.github.com/tauri-apps/tauri/pull/8084)) Updated to wry@0.34, removing the `dox` feature flag.

### Breaking Changes

- [`198abe3c`](https://www.github.com/tauri-apps/tauri/commit/198abe3c2cae06dacab860b3a93f715dcf529a95)([#8076](https://www.github.com/tauri-apps/tauri/pull/8076)) The Android `PluginManager.loadConfig` now takes a third parameter to define the class type of the config object.
- [`198abe3c`](https://www.github.com/tauri-apps/tauri/commit/198abe3c2cae06dacab860b3a93f715dcf529a95)([#8076](https://www.github.com/tauri-apps/tauri/pull/8076)) Mobile plugins now have access to a parser for the invoke arguments instead of relying on the `Invoke#get${TYPE}` methods.
- [`74d2464d`](https://www.github.com/tauri-apps/tauri/commit/74d2464d0e490fae341ad73bdf2964cf215fe6c5)([#8116](https://www.github.com/tauri-apps/tauri/pull/8116)) Added `WindowBuilder::on_page_load` and refactored the `Builder::on_page_load` handler to take references.
  The page load hook is now triggered for load started and finished events, to determine what triggered it see `PageLoadPayload::event`.
- [`93c8a77b`](https://www.github.com/tauri-apps/tauri/commit/93c8a77b347b9934ec0732784d4b78b3260abc08)([#7996](https://www.github.com/tauri-apps/tauri/pull/7996)) The event system APIS on Rust is recieving a few changes for consistency and quality of life improvements:

  - Renamed `Manager::emit_all` to just `Manager::emit` and will now both trigger the events on JS side as well as Rust.
  - Removed `Manager::trigger_global`, use `Manager::emit`
  - Added `Manager::emit_filter`.
  - Removed `Window::emit`, and moved the implementation to `Manager::emit`.
  - Removed `Window::emit_and_trigger` and `Window::trigger`, use `Window::emit` instead.
  - Changed `Window::emit_to` to only trigger the target window listeners so it won't be catched by `Manager::listen_global`

## \[2.0.0-alpha.16]

### New Features

- [`c085adda`](https://www.github.com/tauri-apps/tauri/commit/c085addab58ba851398373c6fd13f9cb026d71e8)([#8009](https://www.github.com/tauri-apps/tauri/pull/8009)) Added `set_progress_bar` to `Window`.
- [`c1ec0f15`](https://www.github.com/tauri-apps/tauri/commit/c1ec0f155118527361dd5645d920becbc8afd569)([#7933](https://www.github.com/tauri-apps/tauri/pull/7933)) Added `Window::set_always_on_bottom` and the `always_on_bottom` option when creating a window.
- [`880266a7`](https://www.github.com/tauri-apps/tauri/commit/880266a7f697e1fe58d685de3bb6836ce5251e92)([#8031](https://www.github.com/tauri-apps/tauri/pull/8031)) Bump the MSRV to 1.70.
- [`ed32257d`](https://www.github.com/tauri-apps/tauri/commit/ed32257d044f90b5eb15053efd1667125def2d2b)([#7794](https://www.github.com/tauri-apps/tauri/pull/7794)) On Windows, add `Effect::Tabbed`,`Effect::TabbedDark` and `Effect::TabbedLight` effects.

### Enhancements

- [`46dcb941`](https://www.github.com/tauri-apps/tauri/commit/46dcb94110ac16d0d4328fa149bb86975b658f59)([#8006](https://www.github.com/tauri-apps/tauri/pull/8006)) Include mobile on docs.rs targets.

### What's Changed

- [`fb10b879`](https://www.github.com/tauri-apps/tauri/commit/fb10b87970a43320ef4d14564f45e7579b774eaf)([#8039](https://www.github.com/tauri-apps/tauri/pull/8039)) Added the `app` plugin back into core.
- [`c9a9246c`](https://www.github.com/tauri-apps/tauri/commit/c9a9246c37bdf190661355c8ee406dac6c427344)([#8007](https://www.github.com/tauri-apps/tauri/pull/8007)) Added the `window` plugin back into core.

### Dependencies

- Upgraded to `tauri-runtime@1.0.0-alpha.3`
- Upgraded to `tauri-runtime-wry@1.0.0-alpha.4`
- Upgraded to `tauri-utils@2.0.0-alpha.9`
- Upgraded to `tauri-build@2.0.0-alpha.10`
- Upgraded to `tauri-macros@2.0.0-alpha.9`

### Breaking Changes

- [`a63e71f9`](https://www.github.com/tauri-apps/tauri/commit/a63e71f9799e9bbc82521d2f17b5238fbf690e89)([#7942](https://www.github.com/tauri-apps/tauri/pull/7942)) The initialization script for `Builder::invoke_system` now must initialize the `window.__TAURI_INTERNALS__.postMessage` function instead of `window.__TAURI_POST_MESSAGE__`.
- [`12b8d18b`](https://www.github.com/tauri-apps/tauri/commit/12b8d18bf7ff833047cb87f356fd007d47d082c9)([#7875](https://www.github.com/tauri-apps/tauri/pull/7875)) -   Removed `tauri::path::Error` and added its variants to `tauri::Error`
  - Removed `tauri::path::Result` and `tauri::plugin::Result` aliases, you should use `tauri::Result` or your own `Result` type.
- [`8b166e9b`](https://www.github.com/tauri-apps/tauri/commit/8b166e9bf82e69ddb3200a3a825614980bd8d433)([#7949](https://www.github.com/tauri-apps/tauri/pull/7949)) Changed `TAURI_AUTOMATION` to `TAURI_WEBVIEW_AUTOMATION`
- [`2558fab8`](https://www.github.com/tauri-apps/tauri/commit/2558fab861006936296e8511e43ccd69a38f61b0)([#7939](https://www.github.com/tauri-apps/tauri/pull/7939)) This release contains a number of breaking changes to improve the consistency of tauri internals and the public facing APIs
  and simplifying the types where applicable:

  - Removed `EventHandler` type.
  - Added `EventId` type
  - Changed `Manager::listen_global` and `Window::listen` to return the new `EventId` type instead of `EventHandler`.
  - Removed the return type of `Manager::once_global` and `Window::once`
  - Changed `Manager::unlisten` and `Window::unlisten` to take he new `EventId` type.
  - Added `tauri::scope::ScopeEventId`
  - Changed `FsScope::listen` to return the new `ScopeEventId` instead of `Uuid`.
  - Added `FsScope::unlisten`

## \[2.0.0-alpha.15]

### Enhancements

- [`b597aa5f`](https://www.github.com/tauri-apps/tauri/commit/b597aa5f3974f5ca5ca5159d441abc9ed3e80721)([#7871](https://www.github.com/tauri-apps/tauri/pull/7871)) Set `main` as the default `id` for the tray icon registered from the configuration file, so if the `id` is not specified, it can be retrieved using `app.tray_by_id("main")`.

### Bug Fixes

- [`a2021c30`](https://www.github.com/tauri-apps/tauri/commit/a2021c30ba1139fcfe4db2522b96125f3fa4d9d6)([#7866](https://www.github.com/tauri-apps/tauri/pull/7866)) Changed `IconMenuItem::set_native_icon` signature to take `&self` instead of `&mut self` to fix compilation error on macos.
- [`a68ccaf5`](https://www.github.com/tauri-apps/tauri/commit/a68ccaf59a6731dc030bdb4642a35e3bc64d5769)([#7822](https://www.github.com/tauri-apps/tauri/pull/7822)) Fix `asset` protocol failing to fetch files.
- [`6fbd6dba`](https://www.github.com/tauri-apps/tauri/commit/6fbd6dba5290dc017ab0ba5a44cf4358b022836f)([#17](https://www.github.com/tauri-apps/tauri/pull/17)) Fix the validation of `std::env::current_exe` warn the user if AppImage is not mounted instead of panicking

### Dependencies

- Upgraded to `tauri-macros@2.0.0-alpha.8`
- Upgraded to `tauri-utils@2.0.0-alpha.8`
- Upgraded to `tauri-build@2.0.0-alpha.9`
- Upgraded to `tauri-runtime@1.0.0-alpha.2`
- Upgraded to `tauri-runtime-wry@1.0.0-alpha.3`

### Breaking Changes

- [`092a561c`](https://www.github.com/tauri-apps/tauri/commit/092a561ca69a631d2a03777e29debeba37b197a7)([#7874](https://www.github.com/tauri-apps/tauri/pull/7874)) Removed `tauri::api` module as most apis have been moved to either a plugin or we recommend using other crates.
- [`deea9436`](https://www.github.com/tauri-apps/tauri/commit/deea9436261f651188e0bc86104779bf30029c32)([#7876](https://www.github.com/tauri-apps/tauri/pull/7876)) Changed `Env.args` to `Env.args_os` and now uses `OsString` instead of `String`
- [`b7fd88e1`](https://www.github.com/tauri-apps/tauri/commit/b7fd88e18d24e4450129a5a5007f2e740c69afe5)([#7944](https://www.github.com/tauri-apps/tauri/pull/7944)) `tauri::scope` module is recieving a couple of consistency changes:

  - Added `tauri::scope::fs` module.
  - Removed `scope::IpcScope` re-export, use `scope::ipc::Scope`.
  - Removed `FsScope`, `GlobPattern` and `FsScopeEvent`, use `scope::fs::Scope`, `scope::fs::Pattern` and `scope::fs::Event` respectively.
- [`c0d03af4`](https://www.github.com/tauri-apps/tauri/commit/c0d03af4704c828698e06e9662dab1087c99c42e)([#7943](https://www.github.com/tauri-apps/tauri/pull/7943)) Changed `TrayIconBuilder/TrayIcon::on_tray_event` to `TrayIconBuilder/TrayIcon::on_tray_icon_event` for consistency of naming.

## \[2.0.0-alpha.14]

### Bug Fixes

- [`d5074af5`](https://www.github.com/tauri-apps/tauri/commit/d5074af562b2b5cb6c5711442097c4058af32db6)([#7801](https://www.github.com/tauri-apps/tauri/pull/7801)) Fixes custom protocol not working on Windows.

### Dependencies

- Upgraded to `tauri-runtime-wry@1.0.0-alpha.2`

### Breaking Changes

- [`d5074af5`](https://www.github.com/tauri-apps/tauri/commit/d5074af562b2b5cb6c5711442097c4058af32db6)([#7801](https://www.github.com/tauri-apps/tauri/pull/7801)) The custom protocol on Android now uses the `http` scheme instead of `https`.

## \[2.0.0-alpha.13]

### Breaking Changes

- [`4cb51a2d`](https://www.github.com/tauri-apps/tauri/commit/4cb51a2d56cfcae0749062c79ede5236bd8c02c2)([#7779](https://www.github.com/tauri-apps/tauri/pull/7779)) The custom protocol on Windows now uses the `http` scheme instead of `https`.

## \[2.0.0-alpha.12]

### Enhancements

- [`8a676617`](https://www.github.com/tauri-apps/tauri/commit/8a6766173b3da4446a87642e7282c73a5b631afe)([#7618](https://www.github.com/tauri-apps/tauri/pull/7618)) Ensure Builder is Send by requiring the menu closure to be Send.
- [`0d63732b`](https://www.github.com/tauri-apps/tauri/commit/0d63732b962e71b98430f8d7b34ea5b59a2e8bb4)([#7754](https://www.github.com/tauri-apps/tauri/pull/7754)) Added `Builder::register_asynchronous_uri_scheme_protocol` to allow resolving a custom URI scheme protocol request asynchronously to prevent blocking the main thread.

### Bug Fixes

- [`0d63732b`](https://www.github.com/tauri-apps/tauri/commit/0d63732b962e71b98430f8d7b34ea5b59a2e8bb4)([#7754](https://www.github.com/tauri-apps/tauri/pull/7754)) Fixes invalid header value type when requesting IPC body through a channel.
- [`e98393e4`](https://www.github.com/tauri-apps/tauri/commit/e98393e499c03504851e97dd9f740b817c4534df)([#7673](https://www.github.com/tauri-apps/tauri/pull/7673)) No longer unpacking and flattening the `payload` over the IPC so that commands with arguments called `cmd`, `callback`, `error`, `options` or `payload` aren't breaking the IPC.
- [`29818de6`](https://www.github.com/tauri-apps/tauri/commit/29818de682146a75cd9a886f7b215154cad1d42d)([#7662](https://www.github.com/tauri-apps/tauri/pull/7662)) Fixes IPC failing to communicate for remote URLs on macOS and iOS.

### What's Changed

- [`6177150b`](https://www.github.com/tauri-apps/tauri/commit/6177150b6f83b52ca359d6e20f7e540f7554e4eb)([#7601](https://www.github.com/tauri-apps/tauri/pull/7601)) Changed `FileDropEvent` to include drop and hover position.

### Dependencies

- Upgraded to `tauri-runtime@1.0.0-alpha.1`
- Upgraded to `tauri-runtime-wry@1.0.0-alpha.1`

### Breaking Changes

- [`0d63732b`](https://www.github.com/tauri-apps/tauri/commit/0d63732b962e71b98430f8d7b34ea5b59a2e8bb4)([#7754](https://www.github.com/tauri-apps/tauri/pull/7754)) Changed `Builder::register_uri_scheme_protocol` to return a `http::Response` instead of `Result<http::Response>`. To return an error response, manually create a response with status code >= 400.
- [`0d63732b`](https://www.github.com/tauri-apps/tauri/commit/0d63732b962e71b98430f8d7b34ea5b59a2e8bb4)([#7754](https://www.github.com/tauri-apps/tauri/pull/7754)) `tauri-runtime` no longer implements its own HTTP types and relies on the `http` crate instead.
- [`0d63732b`](https://www.github.com/tauri-apps/tauri/commit/0d63732b962e71b98430f8d7b34ea5b59a2e8bb4)([#7754](https://www.github.com/tauri-apps/tauri/pull/7754)) Changed `Builder::invoke_system` to take references instead of owned values.
- [`5c95152c`](https://www.github.com/tauri-apps/tauri/commit/5c95152c76391607746f6da942ec57d23c89e89e)([#7621](https://www.github.com/tauri-apps/tauri/pull/7621)) Changed `MenuBuilder\SubmenuBuilder::text`, `MenuBuilder\SubmenuBuilder::check`, `MenuBuilder\SubmenuBuilder::icon` and `MenuBuilder\SubmenuBuilder::native_icon` to take an `id` as the first argument.
- [`0d63732b`](https://www.github.com/tauri-apps/tauri/commit/0d63732b962e71b98430f8d7b34ea5b59a2e8bb4)([#7754](https://www.github.com/tauri-apps/tauri/pull/7754)) Changed `Window::on_message` signature to take a responder closure instead of returning the response object in order to asynchronously process the request.

## \[2.0.0-alpha.11]

### New Features

- [`4db363a0`](https://www.github.com/tauri-apps/tauri/commit/4db363a03c182349f8491f46ced258d84723b11f)([#6589](https://www.github.com/tauri-apps/tauri/pull/6589)) Added `visible_on_all_workspaces` configuration option to `WindowBuilder`, `Window`, and `WindowConfig`.
- [`84c41597`](https://www.github.com/tauri-apps/tauri/commit/84c4159754b2e59244211ed9e1fc702d851a0562)([#6394](https://www.github.com/tauri-apps/tauri/pull/6394)) Add `App::primary_monitor`, `App::available_monitors`, `AppHandle::primary_monitor`, and `AppHandle::available_monitors`
- [`2a000e15`](https://www.github.com/tauri-apps/tauri/commit/2a000e150d02dff28c8b20ad097b29e209160045)([#7235](https://www.github.com/tauri-apps/tauri/pull/7235)) Added `Window::navigate`.
- [`3b98141a`](https://www.github.com/tauri-apps/tauri/commit/3b98141aa26f74c641a4090874247b97079bd58a)([#3736](https://www.github.com/tauri-apps/tauri/pull/3736)) Added support to file associations.
- [`3a2c3e74`](https://www.github.com/tauri-apps/tauri/commit/3a2c3e74710bef9a14932dce74c351cca6215429)([#7306](https://www.github.com/tauri-apps/tauri/pull/7306)) Added `PluginBuilder::on_navigation`.
  Added `Plugin::on_navigation`.
- [`753900dd`](https://www.github.com/tauri-apps/tauri/commit/753900dd6e549aaf56f419144382669e3b246404)([#7440](https://www.github.com/tauri-apps/tauri/pull/7440)) Expose `RunEvent::Opened` on macOS and iOS for deep link support.
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) Add `App::cleanup_before_exit` and `AppHandle::cleanup_before_exit` to manually call the cleanup logic. **You should always exit the tauri app immediately after this function returns and not use any tauri-related APIs.**
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) On Linux, add `Window::default_vbox` to get a reference to the `gtk::Box` that contains the menu bar and the webview.
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) Add `linux-libxdo` feature flag (disabled by default) to enable linking to `libxdo` which is used to make `Cut`, `Copy`, `Paste` and `SelectAll` native menu items work on Linux.
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) On macOS, add `Window::ns_view` to get a pointer to the NSWindow content view.
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) Expose `run_on_main_thread` method on `App` that is similar to `AppHandle::run_on_main_thread`.

### Enhancements

- [`a5752db9`](https://www.github.com/tauri-apps/tauri/commit/a5752db9852bb852e61f19dfb48a9435c1fdc79c)([#7436](https://www.github.com/tauri-apps/tauri/pull/7436)) Listen to `onNewIntent` and forward it to registered plugins.
- [`fbeb5b91`](https://www.github.com/tauri-apps/tauri/commit/fbeb5b9185baeda19e865228179e3e44c165f1d9)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Added `Channel::new` allowing communication from a mobile plugin with Rust.
- [`fbeb5b91`](https://www.github.com/tauri-apps/tauri/commit/fbeb5b9185baeda19e865228179e3e44c165f1d9)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Use custom protocols on the IPC implementation to enhance performance.

### Dependencies

- Upgraded to `tauri-runtime@1.0.0-alpha.0`
- Upgraded to `tauri-utils@2.0.0-alpha.7`
- Upgraded to `tauri-macros@2.0.0-alpha.7`
- Upgraded to `tauri-runtime-wry@1.0.0-alpha.0`
- Upgraded to `tauri-build@2.0.0-alpha.7`
- [`d1a6e2f3`](https://www.github.com/tauri-apps/tauri/commit/d1a6e2f33326161a78a9a72bd9320dcb1b1f9710)([#7252](https://www.github.com/tauri-apps/tauri/pull/7252)) Update `state` to v0.6.

### Breaking Changes

- [`fd5dc788`](https://www.github.com/tauri-apps/tauri/commit/fd5dc788d10b2a048e0804b5415b84ae8f9152ea)([#7352](https://www.github.com/tauri-apps/tauri/pull/7352)) -   Removed `tauri::api::file` and `tauri::api::dir` modules, use `std::fs` instead.
  - Removed `tauri::api::version` module, use `semver` crate instead.
- [`fbeb5b91`](https://www.github.com/tauri-apps/tauri/commit/fbeb5b9185baeda19e865228179e3e44c165f1d9)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Moved `tauri::api::ipc` to `tauri::ipc` and refactored all types.
- [`fbeb5b91`](https://www.github.com/tauri-apps/tauri/commit/fbeb5b9185baeda19e865228179e3e44c165f1d9)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Removed the `linux-protocol-headers` feature (now always enabled) and added `linux-ipc-protocol`.
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) Changed `App::handle` and `Manager::app_handle` to return a reference to an `AppHandle` instead of an owned value.
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) The tray icon and menu have received a huge refactor with a lot of breaking changes in order to add new functionalities and improve the DX around using them and here is an overview of the changes:

  - All menu and tray types are now exported from `tauri::menu` and `tauri::tray` modules with new names so make sure to check the new types.
  - Removed `tauri::Builder::system_tray`, instead you should use `tauri::tray::TrayIconBuilder` inside `tauri::Builder::setup` hook to create your tray icons.
  - Changed `tauri::Builder::menu` to be a function to accommodate for new menu changes, you can passe `tauri::menu::Menu::default` to it to create a default menu.
  - Renamed `tauri::Context` methods `system_tray_icon`, `tauri::Context::system_tray_icon_mut` and `tauri::Context::set_system_tray_icon` to `tauri::Context::tray_icon`, `tauri::Context::tray_icon_mut` and `tauri::Context::set_tray_icon` to be consistent with new type names.
  - Added `RunEvent::MenuEvent` and `RunEvent::TrayIconEvent`.
  - Added `App/AppHandle::set_menu`, `App/AppHandle::remove_menu`, `App/AppHandle::show_menu`, `App/AppHandle::hide_menu` and `App/AppHandle::menu` to access, remove, hide or show the app-wide menu that is used as the global menu on macOS and on all windows that don't have a specific menu set for it on Windows and Linux.
  - Added `Window::set_menu`, `Window::remove_menu`, `Window::show_menu`, `Window::hide_menu`, `Window::is_menu_visible` and `Window::menu` to access, remove, hide or show the menu on this window.
  - Added `Window::popup_menu` and `Window::popup_menu_at` to show a context menu on the window at the cursor position or at a specific position. You can also popup a context menu using `popup` and `popup_at` methods from `ContextMenu` trait which is implemented for `Menu` and `Submenu` types.
  - Added `App/AppHandle::tray`, `App/AppHandle::tray_by_id`, `App/AppHandle::remove_tray` and `App/AppHandle::remove_tray_by_id` to access or remove a registered tray.
  - Added `WindowBuilder/App/AppHandle::on_menu_event` to register a new menu event handler.
  - Added `App/AppHandle::on_tray_icon_event` to register a new tray event handler.
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) Renamed `system-tray` feature flag to `tray-icon`.
- [`3a2c3e74`](https://www.github.com/tauri-apps/tauri/commit/3a2c3e74710bef9a14932dce74c351cca6215429)([#7306](https://www.github.com/tauri-apps/tauri/pull/7306)) The `Window#on_navigation` closure now receives a `&Url` argument instead of `Url`.

## \[2.0.0-alpha.10]

### New Features

- [`f2d68cf7`](https://www.github.com/tauri-apps/tauri/commit/f2d68cf7d4e53443b2d53d2ae841e56c16a92514)([#6767](https://www.github.com/tauri-apps/tauri/pull/6767)) Add `incognito` option to the window configuration object.
- [`f2d68cf7`](https://www.github.com/tauri-apps/tauri/commit/f2d68cf7d4e53443b2d53d2ae841e56c16a92514)([#6767](https://www.github.com/tauri-apps/tauri/pull/6767)) Add `WindowBuilder::incognito`
- [`e0f0dce2`](https://www.github.com/tauri-apps/tauri/commit/e0f0dce220730e2822fc202463aedf0166145de7)([#6442](https://www.github.com/tauri-apps/tauri/pull/6442)) Added the `window_effects` option when creating a window and `Window::set_effects` to change it at runtime.

### Enhancements

- [`2d2fd6ab`](https://www.github.com/tauri-apps/tauri/commit/2d2fd6abe291ddf645fa2fdecc08111d2c0e258e)([#7191](https://www.github.com/tauri-apps/tauri/pull/7191)) Use correct HTTP method when making requests to the proxied server on mobile.
- [`b66e7d60`](https://www.github.com/tauri-apps/tauri/commit/b66e7d60f27d9a7973eae48d54cb72e30a710cca)([#7174](https://www.github.com/tauri-apps/tauri/pull/7174)) Implement `Clone` for `Channel`
- [`8124145d`](https://www.github.com/tauri-apps/tauri/commit/8124145d6c6a629809c138d2c34082e1feb4fdbf)([#7171](https://www.github.com/tauri-apps/tauri/pull/7171)) Fixes path commands not being added.
- [`4652c446`](https://www.github.com/tauri-apps/tauri/commit/4652c446b361a801252bcf45e9da39813bf85482)([#7144](https://www.github.com/tauri-apps/tauri/pull/7144)) Add `temp_dir` method to `PathResolver`

### Bug Fixes

- [`8e855765`](https://www.github.com/tauri-apps/tauri/commit/8e85576506f5dea066d7e9317dbcab3681baff73)([#6809](https://www.github.com/tauri-apps/tauri/pull/6809)) Fix default log path for linux and windows

## \[2.0.0-alpha.9]

- [`256c30c7`](https://www.github.com/tauri-apps/tauri/commit/256c30c72b737e49ced0d6a6483910dc779fc185)([#6863](https://www.github.com/tauri-apps/tauri/pull/6863)) Enhance parsing of annotated Android plugin methods to support private functions.
- [`73c803a5`](https://www.github.com/tauri-apps/tauri/commit/73c803a561181137f20366f5d52511392a619f2b)([#6837](https://www.github.com/tauri-apps/tauri/pull/6837)) Added static function `loadConfig` on the Android `PluginManager` class.
- [`edb16d13`](https://www.github.com/tauri-apps/tauri/commit/edb16d13a503da4b264ce459319fec25374c5c4f)([#6831](https://www.github.com/tauri-apps/tauri/pull/6831)) Adjust Android plugin exception error.
- [`0ab5f40d`](https://www.github.com/tauri-apps/tauri/commit/0ab5f40d3a4207f20e4440587b41c4e78f91d233)([#6813](https://www.github.com/tauri-apps/tauri/pull/6813)) Add channel API for sending data across the IPC.
- [`31444ac1`](https://www.github.com/tauri-apps/tauri/commit/31444ac196add770f2ad18012d7c18bce7538f22)([#6725](https://www.github.com/tauri-apps/tauri/pull/6725)) On Android, update proguard rules.
- [`8ce32e74`](https://www.github.com/tauri-apps/tauri/commit/8ce32e74b5573931c3bc81e8e893a6d3b9686b0e)([#6986](https://www.github.com/tauri-apps/tauri/pull/6986)) Add `default_window_icon` getter on `App` and `AppHandle`.
- [`2a5175a8`](https://www.github.com/tauri-apps/tauri/commit/2a5175a8f8f318aac9a6434271f2cc065e5989ae)([#6779](https://www.github.com/tauri-apps/tauri/pull/6779)) Enhance Android's `JSObject` return types.
- [`bb2a8ccf`](https://www.github.com/tauri-apps/tauri/commit/bb2a8ccf1356e59b98947d827d61e4e99533f2bc)([#6830](https://www.github.com/tauri-apps/tauri/pull/6830)) Use actual iOS plugin instance to run command with `throws`.
- [`94224906`](https://www.github.com/tauri-apps/tauri/commit/942249060ed12a5d21a2b21c30e0638c1d2b9df0)([#6783](https://www.github.com/tauri-apps/tauri/pull/6783)) Generate `TauriActivity` Kotlin class on the build script.
- [`7a4b1fb9`](https://www.github.com/tauri-apps/tauri/commit/7a4b1fb96da475053c61960f362bbecf18cd00d4)([#6839](https://www.github.com/tauri-apps/tauri/pull/6839)) Added support to attibutes for each command path in the `generate_handler` macro.
- [`9a79dc08`](https://www.github.com/tauri-apps/tauri/commit/9a79dc085870e0c1a5df13481ff271b8c6cc3b78)([#6947](https://www.github.com/tauri-apps/tauri/pull/6947)) Remove `enable_tauri_api` from the IPC scope.
- [`dfa407ff`](https://www.github.com/tauri-apps/tauri/commit/dfa407ffcbc8a853d61139b68b55747ae49fb231)([#6763](https://www.github.com/tauri-apps/tauri/pull/6763)) Expose plugin configuration on the Android and iOS plugin classes.
- [`3245d14b`](https://www.github.com/tauri-apps/tauri/commit/3245d14b9eb256a5c5675c7030bac7082855df47)([#6895](https://www.github.com/tauri-apps/tauri/pull/6895)) Moved the `app` feature to its own plugin in the plugins-workspace repository.
- [`09376af5`](https://www.github.com/tauri-apps/tauri/commit/09376af59424cc27803fa2820d2ac0d4cdc90a6d)([#6704](https://www.github.com/tauri-apps/tauri/pull/6704)) Moved the `cli` feature to its own plugin in the plugins-workspace repository.
- [`2d5378bf`](https://www.github.com/tauri-apps/tauri/commit/2d5378bfc1ba817ee2f331b41738a90e5997e5e8)([#6717](https://www.github.com/tauri-apps/tauri/pull/6717)) Moved the dialog APIs to its own plugin in the plugins-workspace repository.
- [`39f1b04f`](https://www.github.com/tauri-apps/tauri/commit/39f1b04f7be4966488484829cd54c8ce72a04200)([#6943](https://www.github.com/tauri-apps/tauri/pull/6943)) Moved the `event` JS APIs to a plugin.
- [`fc4d687e`](https://www.github.com/tauri-apps/tauri/commit/fc4d687ef0ef2ea069ed73c40916da733b5dcb8f)([#6716](https://www.github.com/tauri-apps/tauri/pull/6716)) Moved the file system APIs to its own plugin in the plugins-workspace repository.
- [`f78a3783`](https://www.github.com/tauri-apps/tauri/commit/f78a378344bbec48533641661d865920a8f46f8f)([#6742](https://www.github.com/tauri-apps/tauri/pull/6742)) Moved the `http` feature to its own plugin in the plugins-workspace repository.
- [`29ce9ce2`](https://www.github.com/tauri-apps/tauri/commit/29ce9ce2ce7dfb260d556d5cffd075e8fe06660c)([#6902](https://www.github.com/tauri-apps/tauri/pull/6902)) Moved the `os` feature to its own plugin in the plugins-workspace repository.
- [`60cf9ed2`](https://www.github.com/tauri-apps/tauri/commit/60cf9ed2fcd7be4df41e86cf18735efe9b6cb254)([#6905](https://www.github.com/tauri-apps/tauri/pull/6905)) Moved the `process` feature to its own plugin in the plugins-workspace repository.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Moved the `protocol` scope configuration to the `asset_protocol` field in `SecurityConfig`.
- [`96639ca2`](https://www.github.com/tauri-apps/tauri/commit/96639ca239c9e4f75142fc07868ac46822111cff)([#6749](https://www.github.com/tauri-apps/tauri/pull/6749)) Moved the `shell` functionality to its own plugin in the plugins-workspace repository.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Moved the updater configuration to the `BundleConfig`.
- [`b072daa3`](https://www.github.com/tauri-apps/tauri/commit/b072daa3bd3e38b808466666619ddb885052c5b2)([#6919](https://www.github.com/tauri-apps/tauri/pull/6919)) Moved the `updater` feature to its own plugin in the plugins-workspace repository.
- [`3188f376`](https://www.github.com/tauri-apps/tauri/commit/3188f3764978c6d1452ee31d5a91469691e95094)([#6883](https://www.github.com/tauri-apps/tauri/pull/6883)) Bump the MSRV to 1.65.
- [`d693e526`](https://www.github.com/tauri-apps/tauri/commit/d693e526e8607129d7f7b62a10db715f3b87d2b9)([#6780](https://www.github.com/tauri-apps/tauri/pull/6780)) Added the `onNewIntent` Plugin hook on Android.
- [`34b8f339`](https://www.github.com/tauri-apps/tauri/commit/34b8f339a4276ebff20b9d52caa103e8e3a7af66)([#6705](https://www.github.com/tauri-apps/tauri/pull/6705)) Add `app` method for the `PluginApi` struct.
- [`96639ca2`](https://www.github.com/tauri-apps/tauri/commit/96639ca239c9e4f75142fc07868ac46822111cff)([#6749](https://www.github.com/tauri-apps/tauri/pull/6749)) Moved the `tauri::api::process` module to `tauri::process`.
- [`cdad6e08`](https://www.github.com/tauri-apps/tauri/commit/cdad6e083728ea61bd6fc734ef93f6306056ea2e)([#6774](https://www.github.com/tauri-apps/tauri/pull/6774)) Changed how the `tauri-android` dependency is injected. This requires the `gen/android` project to be recreated.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Removed the allowlist configuration.
- [`cebd7526`](https://www.github.com/tauri-apps/tauri/commit/cebd75261ac71b98976314a450cb292eeeec1515)([#6728](https://www.github.com/tauri-apps/tauri/pull/6728)) Moved the `clipboard` feature to its own plugin in the plugins-workspace repository.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Removed extract and move APIs from `tauri::api::file`.
- [`3f17ee82`](https://www.github.com/tauri-apps/tauri/commit/3f17ee82f6ff21108806edb7b00500b8512b8dc7)([#6737](https://www.github.com/tauri-apps/tauri/pull/6737)) Moved the `global-shortcut` feature to its own plugin in the plugins-workspace repository.
- [`ae102980`](https://www.github.com/tauri-apps/tauri/commit/ae102980fcdde3f55effdc0623ea425b48d07dd1)([#6719](https://www.github.com/tauri-apps/tauri/pull/6719)) Refactor the `Context` conditional fields and only parse the tray icon on desktop.
- [`2d5378bf`](https://www.github.com/tauri-apps/tauri/commit/2d5378bfc1ba817ee2f331b41738a90e5997e5e8)([#6717](https://www.github.com/tauri-apps/tauri/pull/6717)) Remove the updater's dialog option.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Removed `UpdaterEvent`. See `tauri-plugin-updater` for new usage.
- [`9a79dc08`](https://www.github.com/tauri-apps/tauri/commit/9a79dc085870e0c1a5df13481ff271b8c6cc3b78)([#6947](https://www.github.com/tauri-apps/tauri/pull/6947)) Moved the `window` JS APIs to its own plugin in the plugins-workspace repository.
- [`22a76338`](https://www.github.com/tauri-apps/tauri/commit/22a763381622407d58ae72aa24c0afff00b40e04)([#6713](https://www.github.com/tauri-apps/tauri/pull/6713)) Expose `SafePathBuf` type in `tauri::path`.
- [`c4171152`](https://www.github.com/tauri-apps/tauri/commit/c4171152c1846f425a937e82f8af1759bcc8c9ac)([#6909](https://www.github.com/tauri-apps/tauri/pull/6909)) Enable shadows by default.
- [`dfa407ff`](https://www.github.com/tauri-apps/tauri/commit/dfa407ffcbc8a853d61139b68b55747ae49fb231)([#6763](https://www.github.com/tauri-apps/tauri/pull/6763)) Change iOS plugin init function signature to `func init_plugin() -> Plugin`.

## \[2.0.0-alpha.8]

- Fixes boolean plugin parameters freezing the application.
  - [9de89791](https://www.github.com/tauri-apps/tauri/commit/9de897919aa7236913ba6ca7c34a68099f4ff600) fix(core): iOS plugin freezing when receiving a bool parameter ([#6700](https://www.github.com/tauri-apps/tauri/pull/6700)) on 2023-04-13

## \[2.0.0-alpha.7]

- Change minimum Android SDK version to 21 for the plugin library.
  - [db4c9dc6](https://www.github.com/tauri-apps/tauri/commit/db4c9dc655e07ee2184fe04571f500f7910890cd) feat(core): add option to configure Android's minimum SDK version ([#6651](https://www.github.com/tauri-apps/tauri/pull/6651)) on 2023-04-07
- Improve the `run_mobile_plugin` function error handling.
  - [f0570d9f](https://www.github.com/tauri-apps/tauri/commit/f0570d9feee05792cc720d26ef32da5eaed7f797) feat(core): improve `run_mobile_plugin` error handling ([#6655](https://www.github.com/tauri-apps/tauri/pull/6655)) on 2023-04-07
- Implement `Clone` for `plugin::PluginHandle`.
  - [052c5822](https://www.github.com/tauri-apps/tauri/commit/052c5822b53d55e118674d13914f58113a0d1121) feat(core): implement Clone for PluginHandle ([#6644](https://www.github.com/tauri-apps/tauri/pull/6644)) on 2023-04-05

## \[2.0.0-alpha.6]

- Fix compilation issues without the shell API features.
  - [a8137927](https://www.github.com/tauri-apps/tauri/commit/a813792786b55c51173e557834f515d4b2f7ce00) fix(core): compilation issues without execute or sidecar features ([#6621](https://www.github.com/tauri-apps/tauri/pull/6621)) on 2023-04-03

## \[2.0.0-alpha.5]

- Fixes ProGuard rules.
  - [adf4627b](https://www.github.com/tauri-apps/tauri/commit/adf4627b73bd7098772b7f3020b4aca7228bf239) fix(core): adjust ProGuard rules ([#6588](https://www.github.com/tauri-apps/tauri/pull/6588)) on 2023-03-31
- Added `raw` encoding option to read stdout and stderr raw bytes.
  - [f992e7f5](https://www.github.com/tauri-apps/tauri/commit/f992e7f58bf975c654a3daf36780b31a32bac064) chore(changes): readd change file on 2023-04-03
- Renamed the `default-tls` feature to `native-tls` and added `rustls-tls` feature.
  - [cfdee00f](https://www.github.com/tauri-apps/tauri/commit/cfdee00f2b1455a9719bc44823fdaeabbe4c1cb2) refactor(core): fix tls features, use rustls on mobile ([#6591](https://www.github.com/tauri-apps/tauri/pull/6591)) on 2023-03-30

## \[2.0.0-alpha.4]

- Allow a wry plugin to be registered at runtime.
  - [ae296f3d](https://www.github.com/tauri-apps/tauri/commit/ae296f3de16fb6a8badbad5555075a5861681fe5) refactor(tauri-runtime-wry): register runtime plugin after run() ([#6478](https://www.github.com/tauri-apps/tauri/pull/6478)) on 2023-03-17
- Inject `proguard-tauri.pro` file in the Android project.
  - [bef4ef51](https://www.github.com/tauri-apps/tauri/commit/bef4ef51bc2c633b88db121c2087a38dddb7d6bf) feat(android): enable minify on release, add proguard rules ([#6257](https://www.github.com/tauri-apps/tauri/pull/6257)) on 2023-02-13
- Return `bool` in the invoke handler.
  - [05dad087](https://www.github.com/tauri-apps/tauri/commit/05dad0876842e2a7334431247d49365cee835d3e) feat: initial work for iOS plugins ([#6205](https://www.github.com/tauri-apps/tauri/pull/6205)) on 2023-02-11
- Use correct lib name in xcode project.
  - [d1752fb1](https://www.github.com/tauri-apps/tauri/commit/d1752fb1f6223fa47d224cb6c62df9b74944a507) fix(cli): use correct lib name in xcode project ([#6387](https://www.github.com/tauri-apps/tauri/pull/6387)) on 2023-03-08
- Run Android and iOS native plugins on the invoke handler if a Rust plugin command is not found.
  - [05dad087](https://www.github.com/tauri-apps/tauri/commit/05dad0876842e2a7334431247d49365cee835d3e) feat: initial work for iOS plugins ([#6205](https://www.github.com/tauri-apps/tauri/pull/6205)) on 2023-02-11
- Added `initialize_android_plugin` and `initialize_ios_plugin` APIs on `AppHandle`.
  - [05dad087](https://www.github.com/tauri-apps/tauri/commit/05dad0876842e2a7334431247d49365cee835d3e) feat: initial work for iOS plugins ([#6205](https://www.github.com/tauri-apps/tauri/pull/6205)) on 2023-02-11
- Changed the plugin setup hook to take a second argument of type `PluginApi`.
  - [6aaba834](https://www.github.com/tauri-apps/tauri/commit/6aaba83476339fa413fe34d28877a932cb485117) refactor(plugin): add PluginApi and PluginHandle, expose on setup hook ([#6291](https://www.github.com/tauri-apps/tauri/pull/6291)) on 2023-02-16
- Refactored the implementation of the `mobile_entry_point` macro.
  - [9feab904](https://www.github.com/tauri-apps/tauri/commit/9feab904bf08b5c168d4779c21d0419409a68d30) feat(core): add API to call Android plugin ([#6239](https://www.github.com/tauri-apps/tauri/pull/6239)) on 2023-02-10
- Removed the attohttpc client. The `reqwest-*` Cargo features were also removed.
  - [dddaa943](https://www.github.com/tauri-apps/tauri/commit/dddaa943e7e0bf13935d567ef2f3f73e1c913300) refactor(core): remove attohttpc client, closes [#6415](https://www.github.com/tauri-apps/tauri/pull/6415) ([#6468](https://www.github.com/tauri-apps/tauri/pull/6468)) on 2023-03-17
- Added `App::run_mobile_plugin` and `AppHandle::run_mobile_plugin`.
  - [bfb2ab24](https://www.github.com/tauri-apps/tauri/commit/bfb2ab24e0b1d0860ea6e37688b5209541f0eda1) feat: add API to call iOS plugin ([#6242](https://www.github.com/tauri-apps/tauri/pull/6242)) on 2023-02-11
- Added the `shadow` option when creating a window and `Window::set_shadow`.
  - [a81750d7](https://www.github.com/tauri-apps/tauri/commit/a81750d779bc72f0fdb7de90b7fbddfd8049b328) feat(core): add shadow APIs ([#6206](https://www.github.com/tauri-apps/tauri/pull/6206)) on 2023-02-08
- Implemented `with_webview` on Android and iOS.
  - [05dad087](https://www.github.com/tauri-apps/tauri/commit/05dad0876842e2a7334431247d49365cee835d3e) feat: initial work for iOS plugins ([#6205](https://www.github.com/tauri-apps/tauri/pull/6205)) on 2023-02-11

## \[2.0.0-alpha.3]

- Update gtk to 0.16.
  - [7eb9aa75](https://www.github.com/tauri-apps/tauri/commit/7eb9aa75cfd6a3176d3f566fdda02d88aa529b0f) Update gtk to 0.16 ([#6155](https://www.github.com/tauri-apps/tauri/pull/6155)) on 2023-01-30
- Show all application logs on iOS.
  - [dee9460f](https://www.github.com/tauri-apps/tauri/commit/dee9460f9c9bc92e9c638e7691e616849ac2085b) feat: keep CLI alive when iOS app exits, show logs, closes [#5855](https://www.github.com/tauri-apps/tauri/pull/5855) ([#5902](https://www.github.com/tauri-apps/tauri/pull/5902)) on 2022-12-27
- Bump the MSRV to 1.64.
  - [7eb9aa75](https://www.github.com/tauri-apps/tauri/commit/7eb9aa75cfd6a3176d3f566fdda02d88aa529b0f) Update gtk to 0.16 ([#6155](https://www.github.com/tauri-apps/tauri/pull/6155)) on 2023-01-30
- Only proxy the dev server on mobile to simplify desktop usage.
  - [78eaadae](https://www.github.com/tauri-apps/tauri/commit/78eaadae2e75ab165d1970e592bb1455bb8636e3) refactor(core): only proxy on mobile ([#6126](https://www.github.com/tauri-apps/tauri/pull/6126)) on 2023-01-23
- Removed mobile logging initialization, which will be handled by `tauri-plugin-log`.
  - [](https://www.github.com/tauri-apps/tauri/commit/undefined)  on undefined
- Update rfd to 0.11.
  - [f0a1d9cd](https://www.github.com/tauri-apps/tauri/commit/f0a1d9cdbcfb645ce1c5f1cdd597f764991772cd) chore: update rfd and wry versions ([#6174](https://www.github.com/tauri-apps/tauri/pull/6174)) on 2023-02-03

## \[2.0.0-alpha.2]

- Fix the filesystem scope allowing sub-directories of the directory picked by the dialog when `recursive` option was `false`.
  - [9ad0a9a0](https://www.github.com/tauri-apps/tauri/commit/9ad0a9a0aa88a67c3d81ef84df4aad23556affde) Merge pull request from GHSA-6mv3-wm7j-h4w5 on 2022-12-22

## \[2.0.0-alpha.1]

- Implement response cache on the dev server proxy, used when the server responds with status 304.
  - [3ad5e72f](https://www.github.com/tauri-apps/tauri/commit/3ad5e72ff147b76267c010c778a3b94bba209bb0) feat(core): cache dev server proxy responses for 304 status code ([#5818](https://www.github.com/tauri-apps/tauri/pull/5818)) on 2022-12-12
- Properly proxy dev server requests with query strings and fragments.
  - [a9b4cf20](https://www.github.com/tauri-apps/tauri/commit/a9b4cf20a3e9a5cc984727a56111591504e084c0) fix(core): use entire request URL on dev server proxy ([#5819](https://www.github.com/tauri-apps/tauri/pull/5819)) on 2022-12-12

## \[2.0.0-alpha.0]

- Added the `default-tls` and `reqwest-default-tls` Cargo features for enabling TLS suppport to connect over HTTPS.
  - [f6f9192a](https://www.github.com/tauri-apps/tauri/commit/f6f9192aa51bd842df8aa1d1aa538b12aa6c2d29) fix(core): Android compilation on Windows ([#5658](https://www.github.com/tauri-apps/tauri/pull/5658)) on 2022-11-20
- **Breaking change:** Use the custom protocol as a proxy to the development server on all platforms except Linux.
  - [6f061504](https://www.github.com/tauri-apps/tauri/commit/6f0615044d09ec58393a7ebca5e45bb175e20db3) feat(cli): add `android dev` and `ios dev` commands ([#4982](https://www.github.com/tauri-apps/tauri/pull/4982)) on 2022-08-20
- Support `with_webview` for Android platform alowing execution of JNI code in context.
  - [8ea87e9c](https://www.github.com/tauri-apps/tauri/commit/8ea87e9c9ca8ba4c7017c8281f78aacd08f45785) feat(android): with_webview access for jni execution ([#5148](https://www.github.com/tauri-apps/tauri/pull/5148)) on 2022-09-08
- First mobile alpha release!
  - [fa3a1098](https://www.github.com/tauri-apps/tauri/commit/fa3a10988a03aed1b66fb17d893b1a9adb90f7cd) feat(ci): prepare 2.0.0-alpha.0 ([#5786](https://www.github.com/tauri-apps/tauri/pull/5786)) on 2022-12-08
- **Breaking change:** The window creation and setup hook are now called when the event loop is ready.
  - [b4622ea4](https://www.github.com/tauri-apps/tauri/commit/b4622ea4d32720bc3bb2a8c740bb70cfe32fed93) refactor(app): run setup and window creation when event loop is ready ([#4914](https://www.github.com/tauri-apps/tauri/pull/4914)) on 2022-08-11
- Export types required by the `mobile_entry_point` macro.
  - [98904863](https://www.github.com/tauri-apps/tauri/commit/9890486321c9c79ccfb7c547fafee85b5c3ffa71) feat(core): add `mobile_entry_point` macro ([#4983](https://www.github.com/tauri-apps/tauri/pull/4983)) on 2022-08-21

## \[1.6.0]

### New Features

- [`6e488378`](https://www.github.com/tauri-apps/tauri/commit/6e48837860203582d2ef8e59d4524f98511a14c0)([#8474](https://www.github.com/tauri-apps/tauri/pull/8474)) Re-export `Url` type.

### Enhancements

- [`8ce51cec`](https://www.github.com/tauri-apps/tauri/commit/8ce51cec3baf4ed88d80c59bf3bbe96fd369c7a0)([#7718](https://www.github.com/tauri-apps/tauri/pull/7718)) On Windows, retain command line args when relaunching the app after an update. Supports NSIS and WiX (without elevated update task).

### Bug Fixes

- [`cc3d8e77`](https://www.github.com/tauri-apps/tauri/commit/cc3d8e77313672f25520e278bbe8fae1b275a735)([#8539](https://www.github.com/tauri-apps/tauri/pull/8539)) Fixes a deadlock when reading a stdout or stderr line returns an error.
- [`b546b42d`](https://www.github.com/tauri-apps/tauri/commit/b546b42db7e75a59232367dd6212fe3b75bb4c6d)([#8577](https://www.github.com/tauri-apps/tauri/pull/8577)) Preserve the order of JS object/map keys in IPC calls. This also fixes issues with the JS `http` module when calling to servers that required a specific order of `FormBody` contents.
- [`8f8729d9`](https://www.github.com/tauri-apps/tauri/commit/8f8729d91843acd2bd2a24731db865d690dd9ab1)([#8312](https://www.github.com/tauri-apps/tauri/pull/8312)) On macOS, allow cancelling maximization when doubleclick happens on `data-tauri-drag-region` by simply keeping the left moust button pressed and then moving the mouse away of the starting position of the click, which is consistent with the native behavior of macOS.

### Dependencies

- Upgraded to `tauri-runtime-wry@0.14.4`

## \[1.5.4]

### Enhancements

- [`3c371aa8`](https://www.github.com/tauri-apps/tauri/commit/3c371aa8ee4032998f859b570702e81e26e77c6c)([#8228](https://www.github.com/tauri-apps/tauri/pull/8228)) Added `test::get_ipc_response`.

### Bug Fixes

- [`50a3d170`](https://www.github.com/tauri-apps/tauri/commit/50a3d170f242178d41fe7e8a3adf964541f6fe9c)([#8408](https://www.github.com/tauri-apps/tauri/pull/8408)) On Windows, fix `open` dialog `defaultPath`, when invoked from JS, not working if the path uses forward slash (`/`)
- [`645e1dcc`](https://www.github.com/tauri-apps/tauri/commit/645e1dcc6e113564e2ddaacf9cb8338aed1a0bd0)([#8404](https://www.github.com/tauri-apps/tauri/pull/8404)) Fix NSIS updater failing to launch when using `basicUi` mode.

### Dependencies

- Upgraded to `tauri-runtime-wry@0.14.3`
- Upgraded to `tauri-utils@1.5.2`
- Upgraded to `tauri-runtime@0.14.2`
- Upgraded to `tauri-macros@1.4.3`

## \[1.5.3]

### Enhancements

- [`b3e53e72`](https://www.github.com/tauri-apps/tauri/commit/b3e53e7243311a2659b7569dddc20c56ac9f9d8e)([#8288](https://www.github.com/tauri-apps/tauri/pull/8288)) Added `AssetResolver::iter` to iterate on all embedded assets.
- [`5e05236b`](https://www.github.com/tauri-apps/tauri/commit/5e05236b4987346697c7caae0567d3c50714c198)([#8289](https://www.github.com/tauri-apps/tauri/pull/8289)) Added tracing for window startup, plugins, `Window::eval`, events, IPC, updater and custom protocol request handlers behind the `tracing` feature flag.

### Bug Fixes

- [`2ba88563`](https://www.github.com/tauri-apps/tauri/commit/2ba8856343e284ed022f28cff6d16db15ad4645f)([#8095](https://www.github.com/tauri-apps/tauri/pull/8095)) Fix docs.rs build for `x86_64-apple-darwin`.
- [`4b6a602a`](https://www.github.com/tauri-apps/tauri/commit/4b6a602a89b36f24d34d6ccd8e3c9b7ce202c9eb)([#8234](https://www.github.com/tauri-apps/tauri/pull/8234)) Escape path of the updater msi to avoid crashing on installers with spaces.

### Dependencies

- Upgraded to `tauri-runtime-wry@0.14.2`
- Upgraded to `tauri-macros@1.4.2`

## \[1.5.2]

### Bug Fixes

- [`21cdbb41`](https://www.github.com/tauri-apps/tauri/commit/21cdbb41a38f465148bbeb82feb3e7886c320182)([#7982](https://www.github.com/tauri-apps/tauri/pull/7982)) Set the correct `truncate` option on `OpenOptions` so that `write_file` can completely overwrite existing files.

## \[1.5.1]

### Bug Fixes

- [`3671edbc`](https://www.github.com/tauri-apps/tauri/commit/3671edbcff37447c95382ab4c9fd1c36a460a037)([#7937](https://www.github.com/tauri-apps/tauri/pull/7937)) Fix devtools not toggling on `ctrl+shift+i` or `cmd+alt+i` shortcuts.

## \[1.5.0]

### New Features

- [`eeb6be54`](https://www.github.com/tauri-apps/tauri/commit/eeb6be54228f3e5463a28c68956abb06a694c010)([#7512](https://www.github.com/tauri-apps/tauri/pull/7512)) Add `tauri::Manager::emit_filter` and only serialize once when emitting to multiple windows.
- [`6c408b73`](https://www.github.com/tauri-apps/tauri/commit/6c408b736c7aa2a0a91f0a40d45a2b7a7dedfe78)([#7269](https://www.github.com/tauri-apps/tauri/pull/7269)) Add option to specify notification sound.
- [`fdaee9a5`](https://www.github.com/tauri-apps/tauri/commit/fdaee9a5ce988c448dd035c2050c339d275e8d15)([#7350](https://www.github.com/tauri-apps/tauri/pull/7350)) Add `tauri::plugin::Builder::register_uri_scheme_protocol`
- [`10e362d0`](https://www.github.com/tauri-apps/tauri/commit/10e362d098c9bed48f832bad471fb2fab83ab0bb)([#7432](https://www.github.com/tauri-apps/tauri/pull/7432)) Added `UpdateBuilder::endpoints` to add request endpoints at runtime.
- [`10e362d0`](https://www.github.com/tauri-apps/tauri/commit/10e362d098c9bed48f832bad471fb2fab83ab0bb)([#7432](https://www.github.com/tauri-apps/tauri/pull/7432)) Added `UpdateResponse::header` and `UpdateResponse::remove_header` to modify the update download request headers.

### Enhancements

- [`757e959e`](https://www.github.com/tauri-apps/tauri/commit/757e959eb276ed535cfddb0dea8897c56441c644)([#7344](https://www.github.com/tauri-apps/tauri/pull/7344)) Open links externally when `<base target="_blank" />` exists
- [`c9827338`](https://www.github.com/tauri-apps/tauri/commit/c98273387c0ffbb8d0de78ce17006411a1f503ee)([#7416](https://www.github.com/tauri-apps/tauri/pull/7416)) Enhance `readDir` API error with path information.
- [`58d6b899`](https://www.github.com/tauri-apps/tauri/commit/58d6b899e21d37bb42810890d289deb57f2273bd)([#7636](https://www.github.com/tauri-apps/tauri/pull/7636)) Add `append` option to `FsOptions` in the `fs` JS module, used in `writeTextFile` and `writeBinaryFile`, to be able to append to existing files instead of overwriting it.
- [`9aa34ada`](https://www.github.com/tauri-apps/tauri/commit/9aa34ada5769dbefa7dfe5f7a6288b3d20b294e4)([#7645](https://www.github.com/tauri-apps/tauri/pull/7645)) Add setting to switch to `http://<scheme>.localhost/` for custom protocols on Windows.

### Bug Fixes

- [`4bf1e85e`](https://www.github.com/tauri-apps/tauri/commit/4bf1e85e6bf85a7ec92d50c8465bc0588a6399d8)([#7722](https://www.github.com/tauri-apps/tauri/pull/7722)) Properly respect the `focused` option when creating the webview.
- [`0797a002`](https://www.github.com/tauri-apps/tauri/commit/0797a002caad29cd8bedccf01f64bf3b45a5e528)([#7746](https://www.github.com/tauri-apps/tauri/pull/7746)) On macOS, fixed tapping on custom title bar doesn't maximize the window.
- [`1a3dcdb8`](https://www.github.com/tauri-apps/tauri/commit/1a3dcdb8302fad511f2c1cd418fbc4cff0bd62ac)([#7185](https://www.github.com/tauri-apps/tauri/pull/7185)) On Windows, fix NSIS installers requiring administrator rights failing to be launched by updater.
- [`fa7f9b77`](https://www.github.com/tauri-apps/tauri/commit/fa7f9b77ab8f0c890e9d7b120901610e0d3e4c46)([#7341](https://www.github.com/tauri-apps/tauri/pull/7341)) Fix updater not following endpoint redirects.
- [`6fbd6dba`](https://www.github.com/tauri-apps/tauri/commit/6fbd6dba5290dc017ab0ba5a44cf4358b022836f)([#17](https://www.github.com/tauri-apps/tauri/pull/17)) Fix the validation of `std::env::current_exe` warn the user if AppImage is not mounted instead of panicking

### Dependencies

- Upgraded to `tauri-utils@1.5.0`
- Upgraded to `tauri-runtime-wry@0.14.1`
- Upgraded to `tauri-runtime@0.14.1`
- Upgraded to `tauri-macros@1.4.1`

## \[1.4.1]

### Bug Fixes

- [`6afd3472`](https://www.github.com/tauri-apps/tauri/commit/6afd34727f153b32dbc568d169dbb17fb8dc3539)([#6680](https://www.github.com/tauri-apps/tauri/pull/6680)) Revert [#6680](https://github.com/tauri-apps/tauri/pull/6680) which added a default sound for notifications on Windows. This introduced inconsistency with other platforms that has silent notifications by default. In the upcoming releases, we will add support for modifying the notification sound across all platforms.

### Security fixes

- [`066c09a6`](https://www.github.com/tauri-apps/tauri/commit/066c09a6ea06f42f550d090715e06beb65cd5564)([#7227](https://www.github.com/tauri-apps/tauri/pull/7227)) Fix regression in `1.4` where the default behavior of the file system scope was changed to allow reading hidden files and directories by default.

## \[1.4.0]

### New Features

- [`7c237209`](https://www.github.com/tauri-apps/tauri/commit/7c237209207cd2938df660b6fd87d3b7d728bd03)([#6546](https://www.github.com/tauri-apps/tauri/pull/6546)) Added `tauri::VERSION` const to get Tauri's version from Rust.
- [`4c39e46a`](https://www.github.com/tauri-apps/tauri/commit/4c39e46a3b438d007f139166ab2a0ba34291a10a)([#7026](https://www.github.com/tauri-apps/tauri/pull/7026)) Added `tauri::webview_version` , to get webview version.
- [`359058ce`](https://www.github.com/tauri-apps/tauri/commit/359058cecca44a9c30b65140c44a8bb3a6dd3be8)([#5939](https://www.github.com/tauri-apps/tauri/pull/5939)) Add `tauri::api::os::locale` function to get the system locale.
- [`c4d6fb4b`](https://www.github.com/tauri-apps/tauri/commit/c4d6fb4b1ea8acf02707a9fe5dcab47c1c5bae7b)([#2353](https://www.github.com/tauri-apps/tauri/pull/2353)) Added the `maximizable`, `minimizable` and `closable` options to the window builder.
- [`c4d6fb4b`](https://www.github.com/tauri-apps/tauri/commit/c4d6fb4b1ea8acf02707a9fe5dcab47c1c5bae7b)([#2353](https://www.github.com/tauri-apps/tauri/pull/2353)) Added the `set_maximizable`, `set_minimizable`, `set_closable`, `is_maximizable`, `is_minimizable` and `is_closable` methods on `Window`.
- [`1d99f8a3`](https://www.github.com/tauri-apps/tauri/commit/1d99f8a3c2f989d1a5ba4d805e3a40b07a3ca8a5)([#4752](https://www.github.com/tauri-apps/tauri/pull/4752)) Expose the `test` module behind the `test` Cargo feature.
- [`000104bc`](https://www.github.com/tauri-apps/tauri/commit/000104bc3bc0c9ff3d20558ab9cf2080f126e9e0)([#6472](https://www.github.com/tauri-apps/tauri/pull/6472)) Add `Window::is_focused` and `Manager::get_focused_window` getters.
- [`441f9646`](https://www.github.com/tauri-apps/tauri/commit/441f96465488a4f8a5731cc51b8ac97b685898c7)([#5491](https://www.github.com/tauri-apps/tauri/pull/5491)) Add `MenuHandle::try_get_item` and `SystemTrayHandle::try_get_item` which returns a `Option` instead of panicking.

### Enhancements

- [`45330e38`](https://www.github.com/tauri-apps/tauri/commit/45330e38193d0b2a01aa926aec433acc6b8f6597)([#6375](https://www.github.com/tauri-apps/tauri/pull/6375)) Enhance the `asset` protocol to support streaming of large files.
- [`df89ccc1`](https://www.github.com/tauri-apps/tauri/commit/df89ccc1912db6b81d43d56c9e6d66980ece2e8d)([#6955](https://www.github.com/tauri-apps/tauri/pull/6955)) Support `passive` mode for NSIS updater.
- [`cd3846c8`](https://www.github.com/tauri-apps/tauri/commit/cd3846c8ce61ab2879b3911e831525e6242aaab2)([#6955](https://www.github.com/tauri-apps/tauri/pull/6955)) Restart the app after the NSIS updater is finished.
- [`db7c5fbf`](https://www.github.com/tauri-apps/tauri/commit/db7c5fbf2e86f3694720f65834eb2c258b7c1291)([#7143](https://www.github.com/tauri-apps/tauri/pull/7143)) Remove `attohttpc` in favor of `reqwest`.
- [`d2710e9d`](https://www.github.com/tauri-apps/tauri/commit/d2710e9d2e8fd93975ef6494512370faa8cb3b7e)([#6944](https://www.github.com/tauri-apps/tauri/pull/6944)) Unpin `time`, `ignore`, and `winnow` crate versions. Developers now have to pin crates if needed themselves. A list of crates that need pinning to adhere to Tauri's MSRV will be visible in Tauri's GitHub workflow: https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/test-core.yml#L85.
- [`5d85d099`](https://www.github.com/tauri-apps/tauri/commit/5d85d0990cd13a1446953a58633edc24eda55afe)([#7128](https://www.github.com/tauri-apps/tauri/pull/7128)) Send updater status events even if default dialog is enabled.

### Bug Fixes

- [`82169e69`](https://www.github.com/tauri-apps/tauri/commit/82169e69fc904d2c7980534c4479bb6f38259fb4)([#5208](https://www.github.com/tauri-apps/tauri/pull/5208)) Fix parsing `allowlist > http > scope` urls that added a trailing slash which broke matching the incoming requests url.
- [`b41b57eb`](https://www.github.com/tauri-apps/tauri/commit/b41b57ebb27befd366db5befaafb6043c18fdfef)([#7105](https://www.github.com/tauri-apps/tauri/pull/7105)) Fix panics when registering an invalid global shortcuts or checking it is registered and return proper errors instead.
- [`aecf1469`](https://www.github.com/tauri-apps/tauri/commit/aecf14690947d109745b4ad823a3e8f4338de47a)([#6889](https://www.github.com/tauri-apps/tauri/pull/6889)) Fix IPC failing after a failed navigation to an external URL.
- [`076e1a81`](https://www.github.com/tauri-apps/tauri/commit/076e1a81a50468e3dfb34ae9ca7e77c5e1758daa)([#7119](https://www.github.com/tauri-apps/tauri/pull/7119)) Fix unlistening to window events failing sometimes.
- [`3f35b452`](https://www.github.com/tauri-apps/tauri/commit/3f35b452637ef1c794a423f1eda62a15d2ddaf42)([#4080](https://www.github.com/tauri-apps/tauri/pull/4080)) Fix `WindowBuilder::on_navigation` handler not registered properly.
- [`0503eb69`](https://www.github.com/tauri-apps/tauri/commit/0503eb69ce7df6b4ed8f5249fdb519b86cd57d8d)([#7078](https://www.github.com/tauri-apps/tauri/pull/7078)) On macOS and Linux, fix app crashing when creating a window with `data:` uri.
- [`3700793a`](https://www.github.com/tauri-apps/tauri/commit/3700793a2f1ea3686b1889c345d73007bb622a29)([#6934](https://www.github.com/tauri-apps/tauri/pull/6934)) Emit `UPTODATE` update status to javascript when the updater server returns status code `204`
- [`ff5e4dbb`](https://www.github.com/tauri-apps/tauri/commit/ff5e4dbbb01bf3fc9c5143df732c75eef6fd98cb)([#6794](https://www.github.com/tauri-apps/tauri/pull/6794)) Fix some configurations not applied when creating the window through Javascript.
- [`65fd674f`](https://www.github.com/tauri-apps/tauri/commit/65fd674f50ba0395d5cea23daf9d2f49f95647fe)([#6652](https://www.github.com/tauri-apps/tauri/pull/6652)) Play a sound when showing a notification on Windows.
- [`696d77c3`](https://www.github.com/tauri-apps/tauri/commit/696d77c3ce480f4a3b4c1c57dae64ff2bb7842ce)([#4493](https://www.github.com/tauri-apps/tauri/pull/4493)) Fixes global events not being received on window-specific event listeners.

## \[1.3.0]

- Added the `additional_browser_args` option when creating a window.
  - [3dc38b15](https://www.github.com/tauri-apps/tauri/commit/3dc38b150ea8c59c8ba67fd586f921016928f47c) feat(core): expose additional_browser_args to window config (fix: [#5757](https://www.github.com/tauri-apps/tauri/pull/5757)) ([#5799](https://www.github.com/tauri-apps/tauri/pull/5799)) on 2022-12-14
- Fix passing `--profile` to cargo in `tauri build` causing conflict with `--release` passed by the CLI.
  - [bfa69691](https://www.github.com/tauri-apps/tauri/commit/bfa69691a5171af97cc6a6d880cb3090338ed9e9) fix(cli): detect `--profile`. closes [#6255](https://www.github.com/tauri-apps/tauri/pull/6255) ([#6268](https://www.github.com/tauri-apps/tauri/pull/6268)) on 2023-02-18
- Added the `content_protected` option when creating a window and `Window::set_content_protected` to change it at runtime.
  - [4ab5545b](https://www.github.com/tauri-apps/tauri/commit/4ab5545b7a831c549f3c65e74de487ede3ab7ce5) feat: add content protection api, closes [#5132](https://www.github.com/tauri-apps/tauri/pull/5132) ([#5513](https://www.github.com/tauri-apps/tauri/pull/5513)) on 2022-12-13
- Fix serialization of js `Map` when used in `invoke`.
  - [d4d6a98d](https://www.github.com/tauri-apps/tauri/commit/d4d6a98d98fd09ba4e9ee4857ef3604d9e454337) fix(core): convert js `Map` to object before serialization, closes [#6078](https://www.github.com/tauri-apps/tauri/pull/6078) ([#6099](https://www.github.com/tauri-apps/tauri/pull/6099)) on 2023-01-19
- Added `Window::on_navigation`.
  - [3f35b452](https://www.github.com/tauri-apps/tauri/commit/3f35b452637ef1c794a423f1eda62a15d2ddaf42) Expose wry navigation_handler via WindowBuilder closes [#4080](https://www.github.com/tauri-apps/tauri/pull/4080) ([#5686](https://www.github.com/tauri-apps/tauri/pull/5686)) on 2022-12-27
- Sync `__TAURI_METADATA__.__windows` across all windows.
  - [146a794c](https://www.github.com/tauri-apps/tauri/commit/146a794cb696816854648d33e2124e82154c7b2f) fix(core): sync windows metadata across all windows, closes [#5571](https://www.github.com/tauri-apps/tauri/pull/5571) ([#5615](https://www.github.com/tauri-apps/tauri/pull/5615)) on 2022-12-27
- Fix `UpdaterBuilder::check` returning a parsing error when `204` is sent from server where it should instead return a `UpToDate` error.
  - [eb1ec041](https://www.github.com/tauri-apps/tauri/commit/eb1ec0416c52c75830dee77e4d714d882d1145b3) fix(core/updater): read and parse response after checking status code, closes [#6192](https://www.github.com/tauri-apps/tauri/pull/6192) ([#6575](https://www.github.com/tauri-apps/tauri/pull/6575)) on 2023-03-31
- Added `OkWithLabel` and `OkCancelWithLabels` variants to the `api::dialog::MessageDialogButtons` enum to set the text of the dialog buttons.
  - [00e1efaa](https://www.github.com/tauri-apps/tauri/commit/00e1efaa9b33876d41dd360624b69971e70d3856) feat: customize button texts of message dialog ([#4383](https://www.github.com/tauri-apps/tauri/pull/4383)) on 2022-12-28
- Added `Builder::device_event_filter` and `App::set_device_event_filter` methods.
  - [73fd60ee](https://www.github.com/tauri-apps/tauri/commit/73fd60eef2b60f5dc84525ef9c315f4d80c4414f) expose set_device_event_filter in tauri ([#5562](https://www.github.com/tauri-apps/tauri/pull/5562)) on 2022-12-13
- Fix resize glitch when double clicking a custom titlebar in the top resize area.
  - [4892637f](https://www.github.com/tauri-apps/tauri/commit/4892637f83b0f64822a5d304e8786a4c92a8957d) fix: Resizing glitch on custom titlebar click (closes [#2549](https://www.github.com/tauri-apps/tauri/pull/2549)) ([#5966](https://www.github.com/tauri-apps/tauri/pull/5966)) on 2023-01-04
- Fixes tray events not being delivered.
  - [138cb8d7](https://www.github.com/tauri-apps/tauri/commit/138cb8d739b15bccdb388e555c20f17ffe16318c) fix(tauri-runtime-wry): tray event listener not registered ([#6270](https://www.github.com/tauri-apps/tauri/pull/6270)) on 2023-02-14
- Fix the filesystem scope allowing sub-directories of the directory picked by the dialog when `recursive` option was `false`.
  - [72389b00](https://www.github.com/tauri-apps/tauri/commit/72389b00d7b495ffd7750eb1e75a3b8537d07cf3) Merge pull request from GHSA-6mv3-wm7j-h4w5 on 2022-12-22
- Add `is_minimized()` window method.
  - [62144ef3](https://www.github.com/tauri-apps/tauri/commit/62144ef3be63b237869e511826edfb938e2c7174) feat: add is_minimized (fix [#3878](https://www.github.com/tauri-apps/tauri/pull/3878)) ([#5618](https://www.github.com/tauri-apps/tauri/pull/5618)) on 2022-12-13
- Bump minimum supported Rust version to 1.60.
  - [5fdc616d](https://www.github.com/tauri-apps/tauri/commit/5fdc616df9bea633810dcb814ac615911d77222c) feat: Use the zbus-backed of notify-rust ([#6332](https://www.github.com/tauri-apps/tauri/pull/6332)) on 2023-03-31
- Update the `open` crate to v3.2 to fix an URL encoding bug on Windows.
  - [708efbd9](https://www.github.com/tauri-apps/tauri/commit/708efbd9b72508a5a5aa55092a48d3218e008ce3) fix(core/tauri): upgrade `open` to 3.2 to fix a bug on Windows ([#6441](https://www.github.com/tauri-apps/tauri/pull/6441)) on 2023-04-06
- Added support to `mailto:` and `tel:` links on the shell API.
  - [d0d873e3](https://www.github.com/tauri-apps/tauri/commit/d0d873e39a3cd5e51e9cf0145a024ffdb0c2a941) feat(core): add support to mailto: and tel: links, closes [#5521](https://www.github.com/tauri-apps/tauri/pull/5521) ([#5544](https://www.github.com/tauri-apps/tauri/pull/5544)) on 2022-12-12
- Pin `os_info` to `=3.5`.
  - [a8d640b3](https://www.github.com/tauri-apps/tauri/commit/a8d640b3c659c7cfea23fe60cc5d9ef377841c5e) fix(core): pin unarray and os_info ([#6212](https://www.github.com/tauri-apps/tauri/pull/6212)) on 2023-02-07
- Pin raw-window-handle to 0.5.0 to keep MSRV.
  - [c46c09f3](https://www.github.com/tauri-apps/tauri/commit/c46c09f31d9f5169ca8a7e62406a9ea170e3a5c5) fix(deps): pin raw-window-handle to 0.5.0 ([#6480](https://www.github.com/tauri-apps/tauri/pull/6480)) on 2023-03-17
- Pin `time` to `0.3.15`.
  - [3d16461b](https://www.github.com/tauri-apps/tauri/commit/3d16461b68583ba7db037fbc217786e79b46ddf2) fix(core): pin time to 0.3.15 ([#6312](https://www.github.com/tauri-apps/tauri/pull/6312)) on 2023-02-19
- Added configuration to specify remote URLs allowed to access the IPC.
  - [ee71c31f](https://www.github.com/tauri-apps/tauri/commit/ee71c31fd09cc5427da6d29d37c003a914547696) feat(core): allow configuring remote domains with IPC access, closes [#5088](https://www.github.com/tauri-apps/tauri/pull/5088) ([#5918](https://www.github.com/tauri-apps/tauri/pull/5918)) on 2023-04-11
- Add `title` getter on window.
  - [233e43b0](https://www.github.com/tauri-apps/tauri/commit/233e43b0c34fada1ca025378533a0b76931a6540) feat: add `title` getter on window, closes [#5023](https://www.github.com/tauri-apps/tauri/pull/5023) ([#5515](https://www.github.com/tauri-apps/tauri/pull/5515)) on 2022-12-13
- Implement `SystemTray::with_tooltip` and `SystemTrayHandle::set_tooltip` for Windows and macOS.
  - [2265e097](https://www.github.com/tauri-apps/tauri/commit/2265e09718f6ebfeb1d200f11e1e1e069075af6e) feat(windows): implement `with_tooltip` ([#5938](https://www.github.com/tauri-apps/tauri/pull/5938)) on 2023-01-01
- Added window's `url()` getter.
  - [d17027e1](https://www.github.com/tauri-apps/tauri/commit/d17027e1a0db3e8c5ae81fc4f472c5918fbce611) feat: expose url method ([#5914](https://www.github.com/tauri-apps/tauri/pull/5914)) on 2022-12-26
- On Windows, change webview theme based on Window theme for more accurate `prefers-color-scheme` support.
  - [7a8d570d](https://www.github.com/tauri-apps/tauri/commit/7a8d570db72667367eb24b75ddc5dd07a968f7c0) fix: sync webview theme with window theme on Windows, closes [#5802](https://www.github.com/tauri-apps/tauri/pull/5802) ([#5874](https://www.github.com/tauri-apps/tauri/pull/5874)) on 2022-12-27
- Add a method to the `WindowBuilder` struct to recreate windows from tauri.conf.json configurations.
  - [49dff27e](https://www.github.com/tauri-apps/tauri/commit/49dff27ef1b36c48dbfd49f44b9b3ac07b372bdf) feat(core): create WindowBuilder from WindowConfig ([#6073](https://www.github.com/tauri-apps/tauri/pull/6073)) on 2023-01-17
- On Windows, Fix missing `WindowEvent::Focused` in `App::run` callback.
  - [ff4ea1ea](https://www.github.com/tauri-apps/tauri/commit/ff4ea1eabbf2874b113c6b4698002929bbac737a) fix: dispatch focus event to app.run on Windows, closes [#6460](https://www.github.com/tauri-apps/tauri/pull/6460) ([#6504](https://www.github.com/tauri-apps/tauri/pull/6504)) on 2023-03-31
- Pin `winnow` crate to 0.4.1 to keep the 1.60 MSRV.

## \[1.2.5]

- Block remote URLs from accessing the IPC.
  - [9c0593c33](https://www.github.com/tauri-apps/tauri/commit/9c0593c33af52cd9e00ec784d15f63efebdf039c) feat(core): block remote URLs from accessing the IPC on 2023-04-12

## \[1.2.4]

- Pin `ignore` to `=0.4.18`.
  - [adcb082b](https://www.github.com/tauri-apps/tauri/commit/adcb082b1651ecb2a6208b093e12f4185aa3fc98) chore(deps): pin `ignore` to =0.4.18 on 2023-01-17

## \[1.2.3]

- Fix the filesystem scope allowing sub-directories of the directory picked by the dialog when `recursive` option was `false`.
  - [f1b0ad6e](https://www.github.com/tauri-apps/tauri/commit/f1b0ad6e8b721cf1420a9a4b9be5b05c39941d16) Merge pull request from GHSA-6mv3-wm7j-h4w5 on 2022-12-22

## \[1.2.2]

- Invoke event listener in windows safely to avoid causing uncaught errors in windows that have loaded external urls
  - [c14b1df3](https://www.github.com/tauri-apps/tauri/commit/c14b1df37284020b3edb32400eb4b9e32c945472) fix(core): Invoke event listener in windows safely to avoid causing uncaught errors in windows that have loaded external urls ([#5563](https://www.github.com/tauri-apps/tauri/pull/5563)) on 2022-12-08
- Cleanup sidecar and tray icons when calling `app.exit()` from JS.
  - [0f269608](https://www.github.com/tauri-apps/tauri/commit/0f26960891228c5909e06d9f850c44ffaebf536c) fix(core/api): cleanup before exit ([#5765](https://www.github.com/tauri-apps/tauri/pull/5765)) on 2022-12-07
- Fix compatibility with older Linux distributions.
  - [b490308c](https://www.github.com/tauri-apps/tauri/commit/b490308c8897b893292951754607c2253abbc6e1) fix(core): compilation error on older Linux versions, fixes [#5684](https://www.github.com/tauri-apps/tauri/pull/5684) ([#5697](https://www.github.com/tauri-apps/tauri/pull/5697)) on 2022-11-28
- Add `tauri::Builder::enable_macos_default_menu` to enable or disable the default menu creation on macOS.
  - [8866ecac](https://www.github.com/tauri-apps/tauri/commit/8866ecac3cd1af8bf02e29569d605be5a1afe22c) feat(core): add `tauri::Builder::enable_macos_default_menu` ([#5756](https://www.github.com/tauri-apps/tauri/pull/5756)) on 2022-12-07
  - [b293da35](https://www.github.com/tauri-apps/tauri/commit/b293da35dd5ae8c1569a3f3c994b4c1a4c227f4a) fix(changes): change `enable_macos_default_menu` bump to patch on 2022-12-08

## \[1.2.1]

- Fixes a double serialization on the IPC.
  - [677838cc](https://www.github.com/tauri-apps/tauri/commit/677838ccfadfdf37039be53bfad666bbe1dab8c3) fix double serialize on invoke ([#5639](https://www.github.com/tauri-apps/tauri/pull/5639)) on 2022-11-20
- Moved the custom protocol headers support on Linux behind the `linux-protocol-headers` Cargo feature to enhance compatibility with older Linux distributions.
  - [d7109460](https://www.github.com/tauri-apps/tauri/commit/d710946064c47fa488eca01a62403e70b2b5ff87) refactor: move Linux custom protocol headers support behind feature flag ([#5683](https://www.github.com/tauri-apps/tauri/pull/5683)) on 2022-11-24
- Fixes definition of `impl HasRawDisplayHandle` for `AppHandle` and `App`.
  - [ed43ff32](https://www.github.com/tauri-apps/tauri/commit/ed43ff324330d1bd9c042a53a6636dfc7d97b410) fix(tauri): add missing generics on AppHandle and App ([#5642](https://www.github.com/tauri-apps/tauri/pull/5642)) on 2022-11-17

## \[1.2.0]

- Add `accept_first_mouse` option for macOS windows.
  - [95f467ad](https://www.github.com/tauri-apps/tauri/commit/95f467add51448319983c54e2f382c7c09fb72d6) feat(core): add window `accept_first_mouse` option, closes [#5347](https://www.github.com/tauri-apps/tauri/pull/5347) ([#5374](https://www.github.com/tauri-apps/tauri/pull/5374)) on 2022-10-17
- Add new app-specific `BaseDirectory` enum variants `AppConfig`, `AppData`, `AppLocalData`, `AppCache` and `AppLog` along with equivalent functions in `path` module and deprecated ambiguous variants `Log` and `App` along with their equivalent functions in `path` module.
  - [5d89905e](https://www.github.com/tauri-apps/tauri/commit/5d89905e39ce0e6eaaec50a693679335449edb32) feat(api): add app-specific directory APIs, closes [#5263](https://www.github.com/tauri-apps/tauri/pull/5263) ([#5272](https://www.github.com/tauri-apps/tauri/pull/5272)) on 2022-09-28
- Set the correct mimetype when streaming files through `asset:` protocol
  - [39443b43](https://www.github.com/tauri-apps/tauri/commit/39443b4350bd208c4d6eec5e1095f215199f8aa3) fix(core): set correct mimetype for asset protocol streams, closes [#5203](https://www.github.com/tauri-apps/tauri/pull/5203) ([#5210](https://www.github.com/tauri-apps/tauri/pull/5210)) on 2022-09-30
  - [2d9c2b47](https://www.github.com/tauri-apps/tauri/commit/2d9c2b472416339829f9113f976f193bf8e0665f) Revert "fix(core): set correct mimetype for asset protocol streams, closes [#5203](https://www.github.com/tauri-apps/tauri/pull/5203) ([#5210](https://www.github.com/tauri-apps/tauri/pull/5210))" on 2022-09-30
  - [9b1a6a1c](https://www.github.com/tauri-apps/tauri/commit/9b1a6a1c02b8d62dd47d9ce42aa05723d7c1b892) fix(core): set correct mimetype for asset protocol streams,  [#5203](https://www.github.com/tauri-apps/tauri/pull/5203) ([#5536](https://www.github.com/tauri-apps/tauri/pull/5536)) on 2022-11-04
- Disable automatic window tabbing on macOS when the `tabbing_identifier` option is not defined, the window is transparent or does not have decorations.
  - [4137ab44](https://www.github.com/tauri-apps/tauri/commit/4137ab44a81d739556cbc7583485887e78952bf1) feat(macos): add `tabbing_identifier` option, closes [#2804](https://www.github.com/tauri-apps/tauri/pull/2804), [#3912](https://www.github.com/tauri-apps/tauri/pull/3912) ([#5399](https://www.github.com/tauri-apps/tauri/pull/5399)) on 2022-10-19
- The custom protocol now validates the request URI. This has implications when using the `asset` protocol without the `convertFileSrc` helper, the URL must now use the `asset://localhost/$filePath` format.
  - [357480f4](https://www.github.com/tauri-apps/tauri/commit/357480f4ae43aa8da99f7ba61ae2ee51b4552c60) feat(core): custom protocol headers on Linux, closes [#4496](https://www.github.com/tauri-apps/tauri/pull/4496) ([#5421](https://www.github.com/tauri-apps/tauri/pull/5421)) on 2022-10-17
- Escape glob special characters in files/directories when dropping files or using the open/save dialogs.
  - [4cbdf0fb](https://www.github.com/tauri-apps/tauri/commit/4cbdf0fb1c0de5004eab51c36d5843a9816f18af) fix(core): escape glob characters in drop/dialogs , closes [#5234](https://www.github.com/tauri-apps/tauri/pull/5234) ([#5237](https://www.github.com/tauri-apps/tauri/pull/5237)) on 2022-10-05
- Properly emit events with object payload.
  - [79dd6e16](https://www.github.com/tauri-apps/tauri/commit/79dd6e16a7306351e2acf21166506b2876b58a7e) fix(core): properly emit events with object payload, closes [#5482](https://www.github.com/tauri-apps/tauri/pull/5482) ([#5492](https://www.github.com/tauri-apps/tauri/pull/5492)) on 2022-10-27
- Fixes access to the `WebviewWindow.getByLabel` function in a `tauri://window-created` event listener.
  - [e00b1e5f](https://www.github.com/tauri-apps/tauri/commit/e00b1e5f94b3f841bf107cc17ee74be9203ea080) fix(core): update metadata before window-created listeners, closes [#5191](https://www.github.com/tauri-apps/tauri/pull/5191) ([#5458](https://www.github.com/tauri-apps/tauri/pull/5458)) on 2022-10-22
- Fixes resource reading being always rejected by the scope.
  - [a06dc699](https://www.github.com/tauri-apps/tauri/commit/a06dc6993148f10ff7623c9dcc81f313dd960ad0) fix(core): canonicalize resource dir to fix scope check, closes [#5196](https://www.github.com/tauri-apps/tauri/pull/5196) ([#5218](https://www.github.com/tauri-apps/tauri/pull/5218)) on 2022-09-29
- Fixes `__TAURI_PATTERN__` object freeze.
  - [49f06ca4](https://www.github.com/tauri-apps/tauri/commit/49f06ca4b9f1d02933e46bbc50330b84ac81be87) fix: deepfreeze check by prop ([#5407](https://www.github.com/tauri-apps/tauri/pull/5407)) on 2022-10-17
- Readd the option to create an unfocused window via the `focused` method. The `focus` function has been deprecated.
  - [4036e15f](https://www.github.com/tauri-apps/tauri/commit/4036e15f5af933bdc0d0913508b5103958afc143) feat(core): reimplement window initial focus flag, closes [#5120](https://www.github.com/tauri-apps/tauri/pull/5120) ([#5338](https://www.github.com/tauri-apps/tauri/pull/5338)) on 2022-10-08
- Add `hidden_title` option for macOS windows.
  - [321f3fed](https://www.github.com/tauri-apps/tauri/commit/321f3fed19df40c1223099bce953332b7f00f7a9) feat(macos): `title_bar_style` and `hidden_title` window options, closes [#2663](https://www.github.com/tauri-apps/tauri/pull/2663) ([#3965](https://www.github.com/tauri-apps/tauri/pull/3965)) on 2022-09-30
- Custom protocol headers are now implemented on Linux when running on webkit2gtk 2.36 or above.
  - [357480f4](https://www.github.com/tauri-apps/tauri/commit/357480f4ae43aa8da99f7ba61ae2ee51b4552c60) feat(core): custom protocol headers on Linux, closes [#4496](https://www.github.com/tauri-apps/tauri/pull/4496) ([#5421](https://www.github.com/tauri-apps/tauri/pull/5421)) on 2022-10-17
- Add `App::show()`, `AppHandle::show()`, `App::hide()` and `AppHandle::hide()` for hiding/showing the entire application on macOS.
  - [39bf895b](https://www.github.com/tauri-apps/tauri/commit/39bf895b73ec6b53f5758815396ba85dda6b9c67) feat(macOS): Add application `show` and `hide` methods ([#3689](https://www.github.com/tauri-apps/tauri/pull/3689)) on 2022-10-03
- Fix a deadlock when modifying the menu in the `on_menu_event` closure.
  - [ae65951b](https://www.github.com/tauri-apps/tauri/commit/ae65951bc477126b71816d77424f8167814bbe8d) fix(core): fix deadlock in `on_menu_event`, closes [#5254](https://www.github.com/tauri-apps/tauri/pull/5254) ([#5257](https://www.github.com/tauri-apps/tauri/pull/5257)) on 2022-09-28
- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28
- Resolve base system directory in shell scope.
  - [99fe1c56](https://www.github.com/tauri-apps/tauri/commit/99fe1c562ffcea4089f785c73f4e6706d4ebc16b) fix(core): resolve base dir in shell scope, closes [#5480](https://www.github.com/tauri-apps/tauri/pull/5480) ([#5508](https://www.github.com/tauri-apps/tauri/pull/5508)) on 2022-11-04
- Added `tabbing_identifier` to the window builder on macOS.
  - [4137ab44](https://www.github.com/tauri-apps/tauri/commit/4137ab44a81d739556cbc7583485887e78952bf1) feat(macos): add `tabbing_identifier` option, closes [#2804](https://www.github.com/tauri-apps/tauri/pull/2804), [#3912](https://www.github.com/tauri-apps/tauri/pull/3912) ([#5399](https://www.github.com/tauri-apps/tauri/pull/5399)) on 2022-10-19
- Add `title_bar_style` option for macOS windows.
  - [321f3fed](https://www.github.com/tauri-apps/tauri/commit/321f3fed19df40c1223099bce953332b7f00f7a9) feat(macos): `title_bar_style` and `hidden_title` window options, closes [#2663](https://www.github.com/tauri-apps/tauri/pull/2663) ([#3965](https://www.github.com/tauri-apps/tauri/pull/3965)) on 2022-09-30
- Added methods to set the system tray title on macOS.
  - [8f1ace77](https://www.github.com/tauri-apps/tauri/commit/8f1ace77956ac3477826ceb059a191e55b3fff93) feat: expose `set_title` for MacOS tray ([#5182](https://www.github.com/tauri-apps/tauri/pull/5182)) on 2022-09-30
- Added the `user_agent` option when creating a window.
  - [a6c94119](https://www.github.com/tauri-apps/tauri/commit/a6c94119d8545d509723b147c273ca5edfe3729f) feat(core): expose user_agent to window config ([#5317](https://www.github.com/tauri-apps/tauri/pull/5317)) on 2022-10-02

## \[1.1.4]

- Block remote URLs from accessing the IPC.
  - [58ea0b452](https://www.github.com/tauri-apps/tauri/commit/58ea0b45268dbd46cbac0ebb0887353d057ca767) feat(core): block remote URLs from accessing the IPC on 2023-04-12

## \[1.1.3]

- Fix the filesystem scope allowing sub-directories of the directory picked by the dialog when `recursive` option was `false`.
  - [2654c0f4](https://www.github.com/tauri-apps/tauri/commit/2654c0f49da23434d36447d0908fa24e61ff5e4e) Merge pull request from GHSA-6mv3-wm7j-h4w5 on 2022-12-22

## \[1.1.2]

- Escape glob special characters in files/directories when dropping files or using the open/save dialogs.
  - [e4dc5bed](https://www.github.com/tauri-apps/tauri/commit/e4dc5bedbb54fbe6e06ab833d7fb7e0cacebad10) fix(core): escape glob characters in drop/dialogs , closes [#5234](https://www.github.com/tauri-apps/tauri/pull/5234) ([#5237](https://www.github.com/tauri-apps/tauri/pull/5237)) on 2022-10-05

## \[1.1.1]

- Add missing allowlist config for `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position` APIs.
  - Bumped due to a bump in tauri-utils.
  - [c764408d](https://www.github.com/tauri-apps/tauri/commit/c764408da7fae123edd41115bda42fa75a4731d2) fix: Add missing allowlist config for cursor apis, closes [#5207](https://www.github.com/tauri-apps/tauri/pull/5207) ([#5211](https://www.github.com/tauri-apps/tauri/pull/5211)) on 2022-09-16

## \[1.1.0]

- Implement `api::http::ClientBuilder::max_redirections` for the default attohttpc client.
  - [ba5560b2](https://www.github.com/tauri-apps/tauri/commit/ba5560b2a1a61666d8e0bda31424333815714b2f) feat(core): implement max_redirections for attohttpc, ref [#4795](https://www.github.com/tauri-apps/tauri/pull/4795) ([#4811](https://www.github.com/tauri-apps/tauri/pull/4811)) on 2022-07-31
- Implement `From<api::process::Command> for std::process::Command`.
  - [9f1d34c2](https://www.github.com/tauri-apps/tauri/commit/9f1d34c288cbe64f0453cf210bc9488bb42ed19a) feat: implement From<Command> for std::process::Command, closes [#4673](https://www.github.com/tauri-apps/tauri/pull/4673) ([#4836](https://www.github.com/tauri-apps/tauri/pull/4836)) on 2022-08-02
- Added support to configuration files in TOML format (Tauri.toml file).
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02
- Enhance `SystemTray::with_icon` to accept `tauri::Icon`.
  - [964926ff](https://www.github.com/tauri-apps/tauri/commit/964926ff850b82e104d29fec4c8a1d9a16798c06) feat(core): enhance `SystemTray::with_icon` ([#4849](https://www.github.com/tauri-apps/tauri/pull/4849)) on 2022-08-03
- Fixes CLI parser ignoring inner subcommands.
  - [dcd50667](https://www.github.com/tauri-apps/tauri/commit/dcd506676c0a15ac4af7705b62574cc2eea3bb43) fix(core): parse inner CLI subcommands, closes [#4688](https://www.github.com/tauri-apps/tauri/pull/4688) ([#4841](https://www.github.com/tauri-apps/tauri/pull/4841)) on 2022-08-02
- Fix `fs.readDir` recursive option reading symlinked directories that are not allowed by the scope.
  - [f4121c12](https://www.github.com/tauri-apps/tauri/commit/f4121c128e69b06c3eb5eea14dd2af4720afed49) fix(endpoints/fs/readDir): don't read symlinks that are not allowed b… ([#5123](https://www.github.com/tauri-apps/tauri/pull/5123)) on 2022-09-08
- Fix typo in invalid state access panic message.
  - [c7fec3e1](https://www.github.com/tauri-apps/tauri/commit/c7fec3e1ff73f7d857548cab78777a3aaf084c68) fix typo in state.rs ([#4699](https://www.github.com/tauri-apps/tauri/pull/4699)) on 2022-07-25
- Fixes updater breaking the app icon in Finder.
  - [58fc1f21](https://www.github.com/tauri-apps/tauri/commit/58fc1f2150b6ddd1b322deb03ca9083222fc3522) fix(updater): blank icon after update on macOS, closes [#4613](https://www.github.com/tauri-apps/tauri/pull/4613) ([#4861](https://www.github.com/tauri-apps/tauri/pull/4861)) on 2022-08-04
- Implement theme APIs for Linux.
  - [f21cbecd](https://www.github.com/tauri-apps/tauri/commit/f21cbecdeb3571ac4ad971b9a865ff62a131a176) feat(core): implement theme APIs for Linux ([#4808](https://www.github.com/tauri-apps/tauri/pull/4808)) on 2022-08-02
- Implement `raw_window_handle::HasRawDisplayHandle` for `App` and `AppHandle`
  - [0ad9531d](https://www.github.com/tauri-apps/tauri/commit/0ad9531d799d81e1f807000a9d74dfd7998206fe) chore(deps): update tao to 0.13, wry to 0.20, rfd to 0.10, raw-window-handle to 0.5 ([#4804](https://www.github.com/tauri-apps/tauri/pull/4804)) on 2022-07-31
- Retain command line arguments in `api::process::restart`.
  - [6218c31e](https://www.github.com/tauri-apps/tauri/commit/6218c31e175d43b59b87bd5b36ee773467566f67) fix(core): retain command line arguments on restart, closes [#4760](https://www.github.com/tauri-apps/tauri/pull/4760) ([#4763](https://www.github.com/tauri-apps/tauri/pull/4763)) on 2022-07-25
- Added APIs to create a system tray at runtime.
  - [4d063ae9](https://www.github.com/tauri-apps/tauri/commit/4d063ae9ee9538cd6fa5e01b80070c6edf8eaeb9) feat(core): create system tray at runtime, closes [#2278](https://www.github.com/tauri-apps/tauri/pull/2278) ([#4862](https://www.github.com/tauri-apps/tauri/pull/4862)) on 2022-08-09
- Add `api::Command::encoding` method to set the stdout/stderr encoding.
  - [d8cf9f9f](https://www.github.com/tauri-apps/tauri/commit/d8cf9f9fcd617ac24fa418952fd4a32c08804f5c) Command support for specified character encoding, closes [#4644](https://www.github.com/tauri-apps/tauri/pull/4644) ([#4772](https://www.github.com/tauri-apps/tauri/pull/4772)) on 2022-07-28
- Do not follow redirects when `api::http::ClientBuilder::max_redirections` is `0`.
  - [d576e8ae](https://www.github.com/tauri-apps/tauri/commit/d576e8ae72b025ca41f96ddf7a885b84f950a4b1) feat(core): do not follow redirects if `max_redirects` is 0 closes [#4795](https://www.github.com/tauri-apps/tauri/pull/4795) ([#4812](https://www.github.com/tauri-apps/tauri/pull/4812)) on 2022-07-31
- Added the `SystemTrayHandle::destroy` method.
  - [4d063ae9](https://www.github.com/tauri-apps/tauri/commit/4d063ae9ee9538cd6fa5e01b80070c6edf8eaeb9) feat(core): create system tray at runtime, closes [#2278](https://www.github.com/tauri-apps/tauri/pull/2278) ([#4862](https://www.github.com/tauri-apps/tauri/pull/4862)) on 2022-08-09
- Added `native-tls-vendored` and `reqwest-native-tls-vendored` Cargo features to compile and statically link to a vendored copy of OpenSSL on Linux.
  - [331f3460](https://www.github.com/tauri-apps/tauri/commit/331f3460027614738ddbbbbcd04bfc59a349f3de) feat(core): add option to use vendored openssl, closes [#4470](https://www.github.com/tauri-apps/tauri/pull/4470) ([#4809](https://www.github.com/tauri-apps/tauri/pull/4809)) on 2022-08-02
- Update windows to 0.39.0 and webview2-com to 0.19.1.
  - [e6d9b670](https://www.github.com/tauri-apps/tauri/commit/e6d9b670b0b314ed667b0e164f2c8d27048e678f) refactor: remove unneeded focus code ([#5065](https://www.github.com/tauri-apps/tauri/pull/5065)) on 2022-09-03
- Add `exists` function to the fs module.
  - [3c62dbc9](https://www.github.com/tauri-apps/tauri/commit/3c62dbc902c904d35a7472ce72a969084c95fbbe) feat(api): Add `exists` function to the fs module. ([#5060](https://www.github.com/tauri-apps/tauri/pull/5060)) on 2022-09-15

## \[1.0.9]

- Block remote URLs from accessing the IPC.
  - [fa90214b0](https://www.github.com/tauri-apps/tauri/commit/fa90214b052b1a5d38d54fbf1ca422b4c37cfd1f) feat(core): block remote URLs from accessing the IPC on 2023-04-12

## \[1.0.8]

- Fix the filesystem scope allowing sub-directories of the directory picked by the dialog when `recursive` option was `false`.
  - [f0602e7c](https://www.github.com/tauri-apps/tauri/commit/f0602e7c294245ab6ef6fbf2a976ef398340ef58) Merge pull request from GHSA-6mv3-wm7j-h4w5 on 2022-12-22

## \[1.0.7]

- Escape glob special characters in files/directories when dropping files or using the open/save dialogs.
  - [bcd9dc7f](https://www.github.com/tauri-apps/tauri/commit/bcd9dc7f859fa7e65fea5de835fa938ca1368eaf) fix(core): escape glob characters in drop/dialogs , closes [#5234](https://www.github.com/tauri-apps/tauri/pull/5234) ([#5237](https://www.github.com/tauri-apps/tauri/pull/5237)) on 2022-11-08

## \[1.0.6]

- Fix `fs.readDir` recursive option reading symlinked directories that are not allowed by the scope.
  - [bb178829](https://www.github.com/tauri-apps/tauri/commit/bb178829086e80916f9be190f02d83bc25802799) fix(endpoints/fs/readDir): don't read symlinks that are not allowed b… ([#5123](https://www.github.com/tauri-apps/tauri/pull/5123)) on 2022-09-08

## \[1.0.5]

- Escape the MSI file path when running msiexec via powershell.
  - [9af43134](https://www.github.com/tauri-apps/tauri/commit/9af43134e1e58369907281024d31bdb4d16ee6f6) fix(updater): escape MSI path ([#4737](https://www.github.com/tauri-apps/tauri/pull/4737)) on 2022-07-22

## \[1.0.4]

- Reduce the amount of allocations when converting cases.
  - [bc370e32](https://www.github.com/tauri-apps/tauri/commit/bc370e326810446e15b1f50fb962b980114ba16b) feat: reduce the amount of `heck`-related allocations ([#4634](https://www.github.com/tauri-apps/tauri/pull/4634)) on 2022-07-11

## \[1.0.3]

- `tauri::Builder` will now include a default menu for macOS without explicitly using `Menu::os_default`, you can still override it through `tauri::Builder::menu` or remove it using `tauri::Builder::enable_macos_default_menu(false)`.
  - [91055883](https://www.github.com/tauri-apps/tauri/commit/9105588373cc8401bd9ad79bdef26f509b2d76b7) feat: add implicit default menu for macOS only, closes [#4551](https://www.github.com/tauri-apps/tauri/pull/4551) ([#4570](https://www.github.com/tauri-apps/tauri/pull/4570)) on 2022-07-04
- Use `toString()` on message/confirm/ask dialogs title and message values.
  - [b8cd2a79](https://www.github.com/tauri-apps/tauri/commit/b8cd2a7993cd2aa5b71b30c545b3307245d254bf) feat(api): call `toString()` on dialog title and message, closes [#4583](https://www.github.com/tauri-apps/tauri/pull/4583) ([#4588](https://www.github.com/tauri-apps/tauri/pull/4588)) on 2022-07-04
- Fix stack overflow on Windows on commands by changing the implementation of the `async_runtime::spawn` method.
  - [7e3ac847](https://www.github.com/tauri-apps/tauri/commit/7e3ac8475cfa146f80e13cd4e3cdf82502018d9a) fix(core): command stack overflow on Windows, closes [#4548](https://www.github.com/tauri-apps/tauri/pull/4548) ([#4562](https://www.github.com/tauri-apps/tauri/pull/4562)) on 2022-07-03
- Emits RunEvent::Exit prior to killing child processes managed by Tauri, allowing graceful shutdown of sidecar binaries.
  - [34879f73](https://www.github.com/tauri-apps/tauri/commit/34879f73446e218fd08d0d079c086fa556a08b4e) fix: allow graceful shutdown of sidecar commands on exit ([#4561](https://www.github.com/tauri-apps/tauri/pull/4561)) on 2022-07-03
- Added option to disable tray menu on left click on macOS.
  - [f8a3becb](https://www.github.com/tauri-apps/tauri/commit/f8a3becb287942db7f7b551b5db6aeb5a2e939ee) feat(core): add option to disable tray menu on left click, closes [#4584](https://www.github.com/tauri-apps/tauri/pull/4584) ([#4587](https://www.github.com/tauri-apps/tauri/pull/4587)) on 2022-07-05
- Only run the updater default dialog mode in supported platforms or development mode.
  - [e29fff25](https://www.github.com/tauri-apps/tauri/commit/e29fff2566bf130e9f9cae432edba2f842607516) fix(updater): do not run in dialog mode on .deb, closes [#4573](https://www.github.com/tauri-apps/tauri/pull/4573) ([#4577](https://www.github.com/tauri-apps/tauri/pull/4577)) on 2022-07-05
- Configure the updater to relaunch after installing the update on Windows.
  - [0fa74534](https://www.github.com/tauri-apps/tauri/commit/0fa745344e0667434ebf3011a8b9fd776dfc5f59) feat(updater): relaunch on Windows, closes [#4220](https://www.github.com/tauri-apps/tauri/pull/4220) ([#4568](https://www.github.com/tauri-apps/tauri/pull/4568)) on 2022-07-03

## \[1.0.2]

- Fixes check for local URL when an external URL is provided to the window and it is based on the configured devPath.
  - [2e74d20e](https://www.github.com/tauri-apps/tauri/commit/2e74d20e834c0cb8c19fed25f8e7968f758ca8b1) fix(core): check whether external url is local, ref [#4449](https://www.github.com/tauri-apps/tauri/pull/4449) ([#4536](https://www.github.com/tauri-apps/tauri/pull/4536)) on 2022-06-30

## \[1.0.1]

- Added `fn new` constructors for `PhysicalSize`, `LogicalSize`, `PhysicalPosition` and `LogicalPosition` and missing conversion methods.
  - [c7d13a1c](https://www.github.com/tauri-apps/tauri/commit/c7d13a1c60cdbe0c42834ea059321d7a3a7f01a0) feat(core): add missing methods to the dpi module ([#4393](https://www.github.com/tauri-apps/tauri/pull/4393)) on 2022-06-19
- Set the bundle name and app metadata in the Info.plist file in development mode.
  - [38f5db6e](https://www.github.com/tauri-apps/tauri/commit/38f5db6e6a8b496b50d486db6fd8204266de3a69) feat(codegen): fill app metadata in development Info.plist on 2022-06-21
- Set the application icon in development mode on macOS.
  - [307c2ebf](https://www.github.com/tauri-apps/tauri/commit/307c2ebfb68238dacab6088f9c6ba310c727c68c) feat(core): set macOS app icon in development ([#4385](https://www.github.com/tauri-apps/tauri/pull/4385)) on 2022-06-19
- Fixes the error message when using the `window.unminimize` API without enabling it in the allowlist.
  - [cbceb7d6](https://www.github.com/tauri-apps/tauri/commit/cbceb7d6cf7c9ee8c093e81d8569285ef3ca5fe3) fix: some typos ([#4403](https://www.github.com/tauri-apps/tauri/pull/4403)) on 2022-06-19
- Initialize Tauri script when `devPath` is an external URL with path.
  - [079b1cc0](https://www.github.com/tauri-apps/tauri/commit/079b1cc06e1e5437686bac9049d5ac569c3f42df) fix(core): properly get external URL origin, closes [#4414](https://www.github.com/tauri-apps/tauri/pull/4414) ([#4417](https://www.github.com/tauri-apps/tauri/pull/4417)) on 2022-06-21
- Fixes deadlock when a plugin window ready event needs to block the thread waiting on the event loop.
  - [9d33d093](https://www.github.com/tauri-apps/tauri/commit/9d33d09341fd995740149f92387b88120e33dcad) fix(core): deadlock on plugin webview ready hook ([#4462](https://www.github.com/tauri-apps/tauri/pull/4462)) on 2022-06-24
- Adjust the updater to fallback to `$HOME/.cache` or the current working directory as temp directory if the system default is in a different mount point.
  - [fd125f76](https://www.github.com/tauri-apps/tauri/commit/fd125f76d768099dc3d4b2d4114349ffc31ffac9) fix(updater): fallback if tmp is on different mount point, closes [#4500](https://www.github.com/tauri-apps/tauri/pull/4500) ([#4504](https://www.github.com/tauri-apps/tauri/pull/4504)) on 2022-06-28
- Properly fill the origin window when using `emit_to` and `emit_all` from `Window`.
  - [643ae846](https://www.github.com/tauri-apps/tauri/commit/643ae846d86108b00cec748cd02a307b5badba18) fix: fire window-specific event on Window emit_to/emit_all, closes [#4493](https://www.github.com/tauri-apps/tauri/pull/4493) ([#4498](https://www.github.com/tauri-apps/tauri/pull/4498)) on 2022-06-28
- Implement `raw_window_handle::HasRawWindowHandle` on Linux.
  - [3efbc67f](https://www.github.com/tauri-apps/tauri/commit/3efbc67f7469ce65a2d9ea4ff2b60b51d2a36aa5) feat: implement `raw_window_handle` on Linux ([#4469](https://www.github.com/tauri-apps/tauri/pull/4469)) on 2022-06-26
- Added `on_drop` hook to the `plugin::Builder`.
  - [be4bb391](https://www.github.com/tauri-apps/tauri/commit/be4bb391a9bcd76dd949c001b1ace11684a8c6dc) feat: add `AppHandle::remove_plugin` and plugin `on_drop`, closes [#4361](https://www.github.com/tauri-apps/tauri/pull/4361) ([#4443](https://www.github.com/tauri-apps/tauri/pull/4443)) on 2022-06-23
- Refactored the `tauri-runtime-wry` plugin interface.
  - [e39e2999](https://www.github.com/tauri-apps/tauri/commit/e39e2999e0ab1843a8195ba83aea3d6de705c3d8) refactor(tauri-runtime-wry): enhance plugin interface ([#4476](https://www.github.com/tauri-apps/tauri/pull/4476)) on 2022-06-27
- Added `AppHandle::remove_plugin`.
  - [be4bb391](https://www.github.com/tauri-apps/tauri/commit/be4bb391a9bcd76dd949c001b1ace11684a8c6dc) feat: add `AppHandle::remove_plugin` and plugin `on_drop`, closes [#4361](https://www.github.com/tauri-apps/tauri/pull/4361) ([#4443](https://www.github.com/tauri-apps/tauri/pull/4443)) on 2022-06-23
- The theme API is now implemented on macOS 10.14+.
  - [6d94ce42](https://www.github.com/tauri-apps/tauri/commit/6d94ce42353204a02fe9c82ed397d349439f75ef) feat(core): theme is now implemented on macOS ([#4380](https://www.github.com/tauri-apps/tauri/pull/4380)) on 2022-06-17
- Suppress unused variable warning in release builds.
  - [45981851](https://www.github.com/tauri-apps/tauri/commit/45981851e35119266c1a079e1ff27a39f1fdfaed) chore(lint): unused variable warnings for release builds ([#4411](https://www.github.com/tauri-apps/tauri/pull/4411)) on 2022-06-22
- Update tao to 0.12 and wry to 0.19.
  - [f6edc6df](https://www.github.com/tauri-apps/tauri/commit/f6edc6df29b1c45b483fa87c481a3b95730b131b) chore(deps): update tao to 0.12, wry to 0.19, closes [#3220](https://www.github.com/tauri-apps/tauri/pull/3220) ([#4502](https://www.github.com/tauri-apps/tauri/pull/4502)) on 2022-06-28
- Added `Notification::notify` API behind the `windows7-compat` Cargo feature, which includes Windows 7 support.
  - [57039fb2](https://www.github.com/tauri-apps/tauri/commit/57039fb2166571de85271b014a8711a29f06be1a) fix(core): add windows 7 notification support ([#4491](https://www.github.com/tauri-apps/tauri/pull/4491)) on 2022-06-28

## \[1.0.0]

- Allow choosing multiple folders in `dialog.open`.
  - [4e51dce6](https://www.github.com/tauri-apps/tauri/commit/4e51dce6ca21c7664de779bc78a04be1051371f7) fix: dialog open supports multiple dirs, fixes [#4091](https://www.github.com/tauri-apps/tauri/pull/4091) ([#4354](https://www.github.com/tauri-apps/tauri/pull/4354)) on 2022-06-15
- Upgrade to `stable`!
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[1.0.0-rc.17]

- Add `Menu::os_default` which will create a menu filled with default menu items and submenus.
  - [4c4acc30](https://www.github.com/tauri-apps/tauri/commit/4c4acc3094218dd9cee0f1ad61810c979e0b41fa) feat: implement `Default` for `Menu`, closes [#2398](https://www.github.com/tauri-apps/tauri/pull/2398) ([#4291](https://www.github.com/tauri-apps/tauri/pull/4291)) on 2022-06-15

## \[1.0.0-rc.16]

- **Breaking change:** The `TrayIcon` enum has been removed and now `Icon` is used instead.
  This allows you to use more image formats and use embedded icons on Linux.
  - [4ce8e228](https://www.github.com/tauri-apps/tauri/commit/4ce8e228134cd3f22973b74ef26ca0d165fbbbd9) refactor(core): use `Icon` for tray icons ([#4342](https://www.github.com/tauri-apps/tauri/pull/4342)) on 2022-06-14

## \[1.0.0-rc.15]

- Fixes filesystem scope check when using the HTTP API to upload files.
  - [8ce5b762](https://www.github.com/tauri-apps/tauri/commit/8ce5b7626478db5c4767e894d005fafb79aa5a7f) fix: scope check when using the HTTP API to upload files closes [#4312](https://www.github.com/tauri-apps/tauri/pull/4312) on 2022-06-10
- Fixes a memory leak in the command system.
  - [f72cace3](https://www.github.com/tauri-apps/tauri/commit/f72cace36821dc675a6d25268ae85a21bdbd6296) fix: never remove ipc callback & mem never be released ([#4274](https://www.github.com/tauri-apps/tauri/pull/4274)) on 2022-06-05
- Fixes the `Content-Type` header value when sending multipart requests using the `reqwest-client` feature.
  - [f6205afc](https://www.github.com/tauri-apps/tauri/commit/f6205afc0d0e419ecb56b9b6b04bd0a0cc9f62ca) fix(core): wrong Content-Type when using reqwest's multipart, ref [#4312](https://www.github.com/tauri-apps/tauri/pull/4312) on 2022-06-10
- Kill sidecar processes on app exit even when only the `shell-sidecar` feature is enabled.
  - [6ba91272](https://www.github.com/tauri-apps/tauri/commit/6ba9127298632531b64d2831bccec6d22ef6c874) Fix: sidecar cleanup when only `shell-sidecar` is enabled ([#4254](https://www.github.com/tauri-apps/tauri/pull/4254)) on 2022-06-04
- Fixes a crash when a request is made to `https://tauri.$URL` on Windows where `$URL` is not `localhost/**` e.g. `https://tauri.app`.
  - [74457222](https://www.github.com/tauri-apps/tauri/commit/74457222b47221f08388f528a7d52133b6734af6) fix(core): handle requests to `https://tauri.*` on Windows ([#4270](https://www.github.com/tauri-apps/tauri/pull/4270)) on 2022-06-05
- Set notification icon to app icon on Linux.
  - [235e448d](https://www.github.com/tauri-apps/tauri/commit/235e448defd8271739804d9b005ffee9c149dd8e) fix: add a default icon to notifications on linux ([#4283](https://www.github.com/tauri-apps/tauri/pull/4283)) on 2022-06-09
- **Breaking change:** Revert the window creation to be blocking in the main thread. This ensures the window is created before using other methods, but has an issue on Windows where the program deadlocks when creating a window in a Tauri command if it is not `async`. The documentation now states that commands must be `async` in other to prevent it until the issue is fixed in Webview2.
  - [69ae6f14](https://www.github.com/tauri-apps/tauri/commit/69ae6f14943f94285f4279c3a92f9003f74aacff) refactor(window): block main thread when creating a new window ([#4298](https://www.github.com/tauri-apps/tauri/pull/4298)) on 2022-06-08
- No longer ask for permission to send notifications and always allow it.
  - [f482b094](https://www.github.com/tauri-apps/tauri/commit/f482b0942276e9402ab3725957535039bacb4fef) fix: remove notification permission prompt ([#4302](https://www.github.com/tauri-apps/tauri/pull/4302)) on 2022-06-09
- **Breaking change:** Removed the `settings` module.
  - [f482b094](https://www.github.com/tauri-apps/tauri/commit/f482b0942276e9402ab3725957535039bacb4fef) fix: remove notification permission prompt ([#4302](https://www.github.com/tauri-apps/tauri/pull/4302)) on 2022-06-09
- **Breaking change**: Removed the `gtk-tray` and `ayatana-tray` Cargo features.
  - [6216eb49](https://www.github.com/tauri-apps/tauri/commit/6216eb49e72863bfb6d4c9edb8827b21406ac393) refactor(core): drop `ayatana-tray` and `gtk-tray` Cargo features ([#4247](https://www.github.com/tauri-apps/tauri/pull/4247)) on 2022-06-02
- Call `preventDefault()` in the mousedown event handler for `[data-tauri-drag-region]` elements.
  - [a0e20621](https://www.github.com/tauri-apps/tauri/commit/a0e20621f5369682700daa234ec1f4df0c3c4d28) fix: preventDefault mousedown on data-tauri-drag-region, closes [#4059](https://www.github.com/tauri-apps/tauri/pull/4059) on 2022-06-13
- Set permission to `0o700` for the tmp folder used to move the current AppImage on the updater process.
  - [b77877fd](https://www.github.com/tauri-apps/tauri/commit/b77877fd2c643c810659fd059cb1bca5b2c68238) fix(updater): set tmp folder permissions ([#4311](https://www.github.com/tauri-apps/tauri/pull/4311)) on 2022-06-12

## \[1.0.0-rc.14]

- **Breaking change:** `PackageInfo::version` is now a `semver::Version` instead of a `String`.
  - [2badbd2d](https://www.github.com/tauri-apps/tauri/commit/2badbd2d7ed51bf33c1b547b4c837b600574bd4a) refactor: force semver versions, change updater `should_install` sig ([#4215](https://www.github.com/tauri-apps/tauri/pull/4215)) on 2022-05-25
  - [a7388e23](https://www.github.com/tauri-apps/tauri/commit/a7388e23c3b9019d48b078cae00a75c74d74d11b) fix(ci): adjust change file to include tauri-utils and tauri-codegen on 2022-05-27
- **Breaking change**: `UpdateBuilder::should_update` now takes the current version as a `semver::Version` and a `RemoteRelease` struct, allowing you to check other release fields.
  - [2badbd2d](https://www.github.com/tauri-apps/tauri/commit/2badbd2d7ed51bf33c1b547b4c837b600574bd4a) refactor: force semver versions, change updater `should_install` sig ([#4215](https://www.github.com/tauri-apps/tauri/pull/4215)) on 2022-05-25
- **Breaking change:** The `tauri::UpdaterEvent::UpdateEvent` date field is now an `Option<time::OffsetDateTime>`.
  - [ac7656ab](https://www.github.com/tauri-apps/tauri/commit/ac7656ab19ebeda1955698ff7a16199d7055d640) refactor(updater): strong type for the `pub_date` field, ref [#4162](https://www.github.com/tauri-apps/tauri/pull/4162) ([#4218](https://www.github.com/tauri-apps/tauri/pull/4218)) on 2022-05-25
- **Breaking change:** The updater response `pub_date` now must be a valid RFC 3339 string.
  - [ac7656ab](https://www.github.com/tauri-apps/tauri/commit/ac7656ab19ebeda1955698ff7a16199d7055d640) refactor(updater): strong type for the `pub_date` field, ref [#4162](https://www.github.com/tauri-apps/tauri/pull/4162) ([#4218](https://www.github.com/tauri-apps/tauri/pull/4218)) on 2022-05-25

## \[1.0.0-rc.13]

- Fix `` cannot find macro `message_dialog`  `` error.
  - [80458a03](https://www.github.com/tauri-apps/tauri/commit/80458a031b15c83019f11a6c222a2993e16d5dc3) Fix `message_dialog` flag ([#4204](https://www.github.com/tauri-apps/tauri/pull/4204)) on 2022-05-25

## \[1.0.0-rc.12]

- Expose option to set the dialog type.
  - [f46175d5](https://www.github.com/tauri-apps/tauri/commit/f46175d5d46fa3eae66ad2415a0eb1efb7d31da2) feat(core): expose option to set dialog type, closes [#4183](https://www.github.com/tauri-apps/tauri/pull/4183) ([#4187](https://www.github.com/tauri-apps/tauri/pull/4187)) on 2022-05-21
- Expose `title` option in the message dialog API.
  - [ae99f991](https://www.github.com/tauri-apps/tauri/commit/ae99f991674d77c322a2240d10ed4b78ed2f4d4b) feat(core): expose message dialog's title option, ref [#4183](https://www.github.com/tauri-apps/tauri/pull/4183) ([#4186](https://www.github.com/tauri-apps/tauri/pull/4186)) on 2022-05-21
- Immediately create windows when using `tauri::App` as manager.
  - [52d17754](https://www.github.com/tauri-apps/tauri/commit/52d177543ab5d0b316628ef30c2a32f303bc451d) fix(core): immediately create window when using tauri::App, closes [#4170](https://www.github.com/tauri-apps/tauri/pull/4170) ([#4172](https://www.github.com/tauri-apps/tauri/pull/4172)) on 2022-05-21
- Account the monitor position when centering a window.
  - [a7a9fde1](https://www.github.com/tauri-apps/tauri/commit/a7a9fde16fb7c35d48d4f97e83ff95b8baf9e090) fix(core): account for monitor position when centering window ([#4166](https://www.github.com/tauri-apps/tauri/pull/4166)) on 2022-05-21
- Allow the canonical, absolute form of a path for the filesystem scope on Windows if `std::fs::canonicalize` returns a path, fallback to `\\?\$PATH`.
  - [78f2565e](https://www.github.com/tauri-apps/tauri/commit/78f2565e14a5a8292045200967a36d6a40899721) fix: allow return value of fs::canonicalize on fs scope, closes [#4130](https://www.github.com/tauri-apps/tauri/pull/4130) ([#4188](https://www.github.com/tauri-apps/tauri/pull/4188)) on 2022-05-21
- Fixes updater documentation not showing on docs.rs.
  - [55892c35](https://www.github.com/tauri-apps/tauri/commit/55892c35f5c60836a8b7244e1660a83a01a4bc37) fix(core): updater documentation not showing on docs.rs ([#4190](https://www.github.com/tauri-apps/tauri/pull/4190)) on 2022-05-22
- Fixes HTTP timeout not working on Windows when using the `attohttpc` client.
  - [d99c5d58](https://www.github.com/tauri-apps/tauri/commit/d99c5d583bdb1429966283cc5cd4a977cf0770b2) fix(core): HTTP timeout not working on Windows, closes [#4050](https://www.github.com/tauri-apps/tauri/pull/4050) ([#4185](https://www.github.com/tauri-apps/tauri/pull/4185)) on 2022-05-21
- Update `windows-rs` to `0.37.0`, which requires Rust 1.61.0+.
  - [2326be39](https://www.github.com/tauri-apps/tauri/commit/2326be39821890cdd4de76e7029a531424dcb26f) feat(core): update windows-rs to 0.37.0 ([#4199](https://www.github.com/tauri-apps/tauri/pull/4199)) on 2022-05-24
- **Breaking change:** The `WindowBuilder` struct now has a lifetime annotation `WindowBuilder<R: Runtime, 'a>`.
  - [52d17754](https://www.github.com/tauri-apps/tauri/commit/52d177543ab5d0b316628ef30c2a32f303bc451d) fix(core): immediately create window when using tauri::App, closes [#4170](https://www.github.com/tauri-apps/tauri/pull/4170) ([#4172](https://www.github.com/tauri-apps/tauri/pull/4172)) on 2022-05-21

## \[1.0.0-rc.11]

- Added the `App::get_cli_matches` helper function.
  - [617f1144](https://www.github.com/tauri-apps/tauri/commit/617f1144f3f5c4a88e229ff410a511aa72795eef) feat(core): add `App::get_cli_matches` helper ref [#4145](https://www.github.com/tauri-apps/tauri/pull/4145) on 2022-05-17
- Fixes `fileDropEnabled` option not working.
  - [706fcbd3](https://www.github.com/tauri-apps/tauri/commit/706fcbd39c9c08f58da12f874199bd0c06016fb2) fix(core): fileDropEnabled option is not working when creating a new WebviewWindow ([#4146](https://www.github.com/tauri-apps/tauri/pull/4146)) on 2022-05-18
- Prepare window icon and menu even when loading remote URLs. Previously it was behind a `is local URL?` condition.
  - [25aa4347](https://www.github.com/tauri-apps/tauri/commit/25aa4347b3a85c66d445e09f864b7fe6c140a0f9) fix(core): prepare window icon and menu on remote URLs, closes [#4131](https://www.github.com/tauri-apps/tauri/pull/4131) ([#4140](https://www.github.com/tauri-apps/tauri/pull/4140)) on 2022-05-16
- Fix `.mjs` not being recognised as a file extension for JavaScript files (`text/javascript`).
  - [45c45253](https://www.github.com/tauri-apps/tauri/commit/45c45253866ce0de317a6a547af3ea0434d4bcac) fix: add mjs mime type (fix: [#4098](https://www.github.com/tauri-apps/tauri/pull/4098)) ([#4108](https://www.github.com/tauri-apps/tauri/pull/4108)) on 2022-05-13
- Added `PathResolver::resolve_resource` API.
  - [e35aaebc](https://www.github.com/tauri-apps/tauri/commit/e35aaebc309f5796cf6f380d1ac7d5a8094b9033) feat(core): add `PathResolver::resolve_resource` API ([#4116](https://www.github.com/tauri-apps/tauri/pull/4116)) on 2022-05-13
- Allow configuring the display options for the MSI execution allowing quieter updates.
  - [9f2c3413](https://www.github.com/tauri-apps/tauri/commit/9f2c34131952ea83c3f8e383bc3cec7e1450429f) feat(core): configure msiexec display options, closes [#3951](https://www.github.com/tauri-apps/tauri/pull/3951) ([#4061](https://www.github.com/tauri-apps/tauri/pull/4061)) on 2022-05-15

## \[1.0.0-rc.10]

- Update wry to 0.16.2 and webkit2gtk to 0.18.0.
  - [71a553b7](https://www.github.com/tauri-apps/tauri/commit/71a553b715312e2bcceb963c83e42cffca7a63bc) chore(deps): update wry to 0.16.2, webkit2gtk to 0.18.0 ([#4099](https://www.github.com/tauri-apps/tauri/pull/4099)) on 2022-05-10
- Adds the `App#wry_plugin` API to inject a plugin for the wry integration.
  - [c8e0e5b9](https://www.github.com/tauri-apps/tauri/commit/c8e0e5b97d542e549b37be08b545515c862af0e5) feat(tauri-runtime-wry): add plugin API ([#4094](https://www.github.com/tauri-apps/tauri/pull/4094)) on 2022-05-10

## \[1.0.0-rc.9]

- The `dangerous_allow_asset_csp_modification` configuration value has been changed to allow a list of CSP directives to disable.
  - [164078c0](https://www.github.com/tauri-apps/tauri/commit/164078c0b719ccbc12e956fecf8a7d4a3c5044e1) feat: allow limiting dangerousDisableAssetCspModification, closes [#3831](https://www.github.com/tauri-apps/tauri/pull/3831) ([#4021](https://www.github.com/tauri-apps/tauri/pull/4021)) on 2022-05-02
- The file drop event payloads are now percent-decoded.
  - [a0ecd81a](https://www.github.com/tauri-apps/tauri/commit/a0ecd81a934e1aa8935151a74cad686786054204) fix(core): percent decode file drop payloads, closes [#4034](https://www.github.com/tauri-apps/tauri/pull/4034) ([#4035](https://www.github.com/tauri-apps/tauri/pull/4035)) on 2022-05-03
- Fix dialog crash on macOS when the `default_path` value is just the file name.
  - [d31167c5](https://www.github.com/tauri-apps/tauri/commit/d31167c520e4e5ea5c75518c180574f5fffc1a40) fix(core): dialog crashing on macOS when the parent is empty ([#4028](https://www.github.com/tauri-apps/tauri/pull/4028)) on 2022-05-02
- Fixes the `title` option being ignored in the dialog API endpoints.
  - [220e7460](https://www.github.com/tauri-apps/tauri/commit/220e7460148df476171579878c3cfffcdb1423d8) fix(core): set dialog title via API, closes [#4029](https://www.github.com/tauri-apps/tauri/pull/4029) ([#4030](https://www.github.com/tauri-apps/tauri/pull/4030)) on 2022-05-02
- Fixes nested isolation iframe injection.
  - [022eed46](https://www.github.com/tauri-apps/tauri/commit/022eed46675976e8dfe5f352a875754b4bd78131) fix(core): nested isolation iframes, closes [#4015](https://www.github.com/tauri-apps/tauri/pull/4015) ([#4020](https://www.github.com/tauri-apps/tauri/pull/4020)) on 2022-05-01
- Deserialize numeric values (seconds) in the http API `ClientBuilder.connect_timeout` and `HttpRequestBuilder.timeout` fields.
  - [f3c5ca89](https://www.github.com/tauri-apps/tauri/commit/f3c5ca89e79d429183c4e15a9e7cebada2b493a0) fix(core): http api `connect_timeout` deserialization, closes [#4004](https://www.github.com/tauri-apps/tauri/pull/4004) ([#4006](https://www.github.com/tauri-apps/tauri/pull/4006)) on 2022-04-29
- Fix updater dialog removing single and double quotes from the release notes
  - [0180dcc8](https://www.github.com/tauri-apps/tauri/commit/0180dcc812bacb78822bc0f97a3202633821dbce) fix(updater): remove single\&double quotes escaping in updater dialog … ([#4047](https://www.github.com/tauri-apps/tauri/pull/4047)) on 2022-05-04
- Expose methods to access the underlying native handles of the webview.
  - [c82b4761](https://www.github.com/tauri-apps/tauri/commit/c82b4761e1660592472dc55308ad69d9efc5855b) feat(core): expose `with_webview` API to access the platform webview ([#4058](https://www.github.com/tauri-apps/tauri/pull/4058)) on 2022-05-04

## \[1.0.0-rc.8]

- **Breaking change:** Removed the `ayatana-tray` from the default features. You must select one of `ayatana-tray` and `gtk-tray` to use system tray on Linux.
  - [62cdb2b3](https://www.github.com/tauri-apps/tauri/commit/62cdb2b307534cf1cb69fdb8688f7c0f3176f41b) refactor(tauri): remove ayatana-tray from the default features ([#3976](https://www.github.com/tauri-apps/tauri/pull/3976)) on 2022-04-26
- Re-export the `GlobalShortcutManager` when the `global-shortcut` feature is enabled.
  - [62cdb2b3](https://www.github.com/tauri-apps/tauri/commit/62cdb2b307534cf1cb69fdb8688f7c0f3176f41b) refactor(tauri): remove ayatana-tray from the default features ([#3976](https://www.github.com/tauri-apps/tauri/pull/3976)) on 2022-04-26
- Fixes `docs.rs` documentation build.
  - [dd94917b](https://www.github.com/tauri-apps/tauri/commit/dd94917b619f22849a7487805315c5306e4847b8) fix(tauri): docs.rs build error ([#3974](https://www.github.com/tauri-apps/tauri/pull/3974)) on 2022-04-26

## \[1.0.0-rc.7]

- **Breaking change:** Removed `tauri::api::file::ArchiveFormat::Plain`.
  - [f7d3d93b](https://www.github.com/tauri-apps/tauri/commit/f7d3d93b625ffc8bccf8068793f44af2f9a4f997) refactor(core): improve performance of the `extract` API ([#3963](https://www.github.com/tauri-apps/tauri/pull/3963)) on 2022-04-25
- Fallback to `{path}.html` when `{path}` is not found in the Tauri custom protocol handler.
  - [7864d41d](https://www.github.com/tauri-apps/tauri/commit/7864d41de8650f20fa73cefbddd78381b9d8aa11) feat(core): fallback to `{path}.html` in Tauri protocol loader ref [#3887](https://www.github.com/tauri-apps/tauri/pull/3887) ([#3939](https://www.github.com/tauri-apps/tauri/pull/3939)) on 2022-04-21
- **Breaking change:** Use ayatana-appindicator for Linux system tray by default. Use the `gtk-tray` Cargo feature to use `libappindicator` instead.
  - [f2a30d8b](https://www.github.com/tauri-apps/tauri/commit/f2a30d8bc54fc3ba49e16f69a413eca5f61a9b1f) refactor(core): use ayatana appindicator by default, keep option to use gtk ([#3916](https://www.github.com/tauri-apps/tauri/pull/3916)) on 2022-04-19
- Reduce the amount of generated code for the API endpoints.
  - [c23f139b](https://www.github.com/tauri-apps/tauri/commit/c23f139ba86628fe0217a966bc8676afe7202a05) perf(core): improve binary size with api enum serde refactor ([#3952](https://www.github.com/tauri-apps/tauri/pull/3952)) on 2022-04-24
- \**Breaking change::* Added the `clipboard` Cargo feature.
  - [24e4ff20](https://www.github.com/tauri-apps/tauri/commit/24e4ff208ee0fe1a4cc5b10667ea0922ac63dfb5) refactor(core): add clipboard Cargo feature, enhancing binary size ([#3957](https://www.github.com/tauri-apps/tauri/pull/3957)) on 2022-04-24
- **Breaking change:** The process Command API stdio lines now includes the trailing `\r`.
  - [b5622882](https://www.github.com/tauri-apps/tauri/commit/b5622882cf3748e1e5a90915f415c0cd922aaaf8) fix(cli): exit on non-compilation Cargo errors, closes [#3930](https://www.github.com/tauri-apps/tauri/pull/3930) ([#3942](https://www.github.com/tauri-apps/tauri/pull/3942)) on 2022-04-22
- Expose Window cursor APIs `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position`.
  - [c54ddfe9](https://www.github.com/tauri-apps/tauri/commit/c54ddfe9338e7eb90b4d5b02dfde687d432d5bc1) feat: expose window cursor APIs, closes [#3888](https://www.github.com/tauri-apps/tauri/pull/3888) [#3890](https://www.github.com/tauri-apps/tauri/pull/3890) ([#3935](https://www.github.com/tauri-apps/tauri/pull/3935)) on 2022-04-21
- **Breaking change:** The `tauri::api::file::Extract#extract_file` function has been moved to `tauri::api::file::Entry#extract`.
  - [f7d3d93b](https://www.github.com/tauri-apps/tauri/commit/f7d3d93b625ffc8bccf8068793f44af2f9a4f997) refactor(core): improve performance of the `extract` API ([#3963](https://www.github.com/tauri-apps/tauri/pull/3963)) on 2022-04-25
- **Breaking change:** The `tauri::api::file::Extract#files` function has been renamed to `with_files` for performance reasons.
  - [f7d3d93b](https://www.github.com/tauri-apps/tauri/commit/f7d3d93b625ffc8bccf8068793f44af2f9a4f997) refactor(core): improve performance of the `extract` API ([#3963](https://www.github.com/tauri-apps/tauri/pull/3963)) on 2022-04-25
- Improved the performance of the `tauri::api::fs::Extract` API.
  - [f7d3d93b](https://www.github.com/tauri-apps/tauri/commit/f7d3d93b625ffc8bccf8068793f44af2f9a4f997) refactor(core): improve performance of the `extract` API ([#3963](https://www.github.com/tauri-apps/tauri/pull/3963)) on 2022-04-25
- Fixes a panic when using the `create_tao_window` API.
  - [320329a9](https://www.github.com/tauri-apps/tauri/commit/320329a9a7d8a249c0fc9dee6db5669057ca8b39) fix(core): insert to webview_id_map on tao window creation, closes [#3883](https://www.github.com/tauri-apps/tauri/pull/3883) ([#3932](https://www.github.com/tauri-apps/tauri/pull/3932)) on 2022-04-21
- Fixes the HTTP API form text fields.
  - [cc356084](https://www.github.com/tauri-apps/tauri/commit/cc35608430b47101255b93ae2f58dfd15e1297d3) fix(core): do not serialize strings in http api form, closes [#3910](https://www.github.com/tauri-apps/tauri/pull/3910) ([#3928](https://www.github.com/tauri-apps/tauri/pull/3928)) on 2022-04-20
- Set the application bundle identifier for the notifications on macOS.
  - [f67ae6bd](https://www.github.com/tauri-apps/tauri/commit/f67ae6bd6052d5bc30305ecaa3b83d071da3cfca) fix(core): set bundle id for notifications on macOS ([#3839](https://www.github.com/tauri-apps/tauri/pull/3839)) on 2022-04-01
- Fixes a panic when a menu event is triggered when all windows are minimized on macOS.
  - [70ff55c1](https://www.github.com/tauri-apps/tauri/commit/70ff55c1aa69ed59cd2a78d865e1cb398ef2a4ba) fix(core): panic on menu event with minimized windows, closes [#3902](https://www.github.com/tauri-apps/tauri/pull/3902) ([#3918](https://www.github.com/tauri-apps/tauri/pull/3918)) on 2022-04-20
- Fixes a rendering issue when resizing the window with the devtools open.
  - [80b714af](https://www.github.com/tauri-apps/tauri/commit/80b714af6b31365b9026bc92f8631b1721950447) fix: rendering issue when resizing with devtools open closes [#3914](https://www.github.com/tauri-apps/tauri/pull/3914) [#3814](https://www.github.com/tauri-apps/tauri/pull/3814) ([#3915](https://www.github.com/tauri-apps/tauri/pull/3915)) on 2022-04-19
- Fixes the `WindowBuilder` export.
  - [985d2508](https://www.github.com/tauri-apps/tauri/commit/985d250898f07ebc975b579ea25555337947ab76) fix(tauri): export `WindowBuilder` struct instead of trait, closes [#3827](https://www.github.com/tauri-apps/tauri/pull/3827) ([#3833](https://www.github.com/tauri-apps/tauri/pull/3833)) on 2022-03-31
- The HTTP API now supports `multipart/form-data` requests. You need to set the `Content-Type` header and enable the `http-multipart` Cargo feature.
  - [1397d912](https://www.github.com/tauri-apps/tauri/commit/1397d9121aae8188bceceacae5565fbbeb67ddd9) feat(core): add support to multipart/form-data requests, closes [#2118](https://www.github.com/tauri-apps/tauri/pull/2118) ([#3929](https://www.github.com/tauri-apps/tauri/pull/3929)) on 2022-04-22
- \**Breaking change::* Added the `global-shortcut` Cargo feature.
  - [e11878bc](https://www.github.com/tauri-apps/tauri/commit/e11878bcf7174b261a1fa146fc7d564d12e6312a) refactor(core): add global-shortcut Cargo feature, enhancing binary size ([#3956](https://www.github.com/tauri-apps/tauri/pull/3956)) on 2022-04-24
- Added `tauri::api::http::HttpRequestBuilder#header` method.
  - [81705bb3](https://www.github.com/tauri-apps/tauri/commit/81705bb332623a53433c28d941ff00da1a6d50fa) feat(updater): add method to set request headers closes [#3896](https://www.github.com/tauri-apps/tauri/pull/3896) ([#3931](https://www.github.com/tauri-apps/tauri/pull/3931)) on 2022-04-22
- **Breaking change:** The `tauri::api::http::HttpRequestBuilder#headers` method now takes `header::HeaderMap` instead of a `HashMap`.
  - [81705bb3](https://www.github.com/tauri-apps/tauri/commit/81705bb332623a53433c28d941ff00da1a6d50fa) feat(updater): add method to set request headers closes [#3896](https://www.github.com/tauri-apps/tauri/pull/3896) ([#3931](https://www.github.com/tauri-apps/tauri/pull/3931)) on 2022-04-22
- **Breaking change:** The `tauri::api::http::Response#headers` method now returns `&header::HeaderMap` instead of `&HashMap`.
  - [81705bb3](https://www.github.com/tauri-apps/tauri/commit/81705bb332623a53433c28d941ff00da1a6d50fa) feat(updater): add method to set request headers closes [#3896](https://www.github.com/tauri-apps/tauri/pull/3896) ([#3931](https://www.github.com/tauri-apps/tauri/pull/3931)) on 2022-04-22
- **Breaking change:** The `api::http` timeouts are now represented as `std::time::Duration` instead of a `u64`.
  - [0ecfad59](https://www.github.com/tauri-apps/tauri/commit/0ecfad5924d7e19719812fd2e460e5242d8f352f) refactor(updater): unset request timeout, add builder setter ([#3847](https://www.github.com/tauri-apps/tauri/pull/3847)) on 2022-04-02
- **Breaking change:** The `tauri::api::http::FormPart::Bytes` enum variant has been renamed to `File` with a value object `{ file, mime, file_name }`.
  - [1397d912](https://www.github.com/tauri-apps/tauri/commit/1397d9121aae8188bceceacae5565fbbeb67ddd9) feat(core): add support to multipart/form-data requests, closes [#2118](https://www.github.com/tauri-apps/tauri/pull/2118) ([#3929](https://www.github.com/tauri-apps/tauri/pull/3929)) on 2022-04-22
- **Breaking change:** Removed `App::create_window`, `AppHandle::create_window`, `Builder::create_window` and `Window::create_window`.
  - [7c7d854a](https://www.github.com/tauri-apps/tauri/commit/7c7d854ab9183d3f78bafd9027f0304d84abe159) refactor(core): remove deprecated APIs ([#3834](https://www.github.com/tauri-apps/tauri/pull/3834)) on 2022-04-01
- **Breaking change:** Removed `tauri::api::http::FormPart::File`.
  - [cc356084](https://www.github.com/tauri-apps/tauri/commit/cc35608430b47101255b93ae2f58dfd15e1297d3) fix(core): do not serialize strings in http api form, closes [#3910](https://www.github.com/tauri-apps/tauri/pull/3910) ([#3928](https://www.github.com/tauri-apps/tauri/pull/3928)) on 2022-04-20
- Added `WindowEvent::ThemeChanged(theme)`.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21
- Added `theme` getter on `Window`.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21
- Added `UpdateResponse::body` and `UpdateResponse::date`.
  - [c7696f34](https://www.github.com/tauri-apps/tauri/commit/c7696f34ecb7baa0da583736e727f74a01f5996a) feat(updater): add `body` and `date` getters ([#3802](https://www.github.com/tauri-apps/tauri/pull/3802)) on 2022-03-29
- **Breaking change**: Removed the `tauri::updater::Error::UnsupportedPlatform` variant and added `UnsupportedLinuxPackage`, `UnsupportedOs` and `UnsupportedArch` for better error information.
  - [ed716793](https://www.github.com/tauri-apps/tauri/commit/ed71679368845f603680465fdffcc90fe842bb8c) refactor(updater): improve unsupported error variants, closes [#3817](https://www.github.com/tauri-apps/tauri/pull/3817) ([#3849](https://www.github.com/tauri-apps/tauri/pull/3849)) on 2022-04-05
- Add updater `Downloaded` status event.
  - [9712ed1a](https://www.github.com/tauri-apps/tauri/commit/9712ed1a6893438270e99cfecbc7968b68716cba) feat(updater): add `Downloaded` status event ([#3804](https://www.github.com/tauri-apps/tauri/pull/3804)) on 2022-04-01
- Allow setting app updater request headers via `AppHandle::updater().header()`.
  - [81705bb3](https://www.github.com/tauri-apps/tauri/commit/81705bb332623a53433c28d941ff00da1a6d50fa) feat(updater): add method to set request headers closes [#3896](https://www.github.com/tauri-apps/tauri/pull/3896) ([#3931](https://www.github.com/tauri-apps/tauri/pull/3931)) on 2022-04-22
- The updater default timeout is now unset, and the `UpdateBuilder` has a `timeout` setter.
  - [0ecfad59](https://www.github.com/tauri-apps/tauri/commit/0ecfad5924d7e19719812fd2e460e5242d8f352f) refactor(updater): unset request timeout, add builder setter ([#3847](https://www.github.com/tauri-apps/tauri/pull/3847)) on 2022-04-02
- Added `theme` setter to the WindowBuilder.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21

## \[1.0.0-rc.6]

- Fixes the `WindowEvent` type used on `Window::on_window_event`.
  - [06aa87b6](https://www.github.com/tauri-apps/tauri/commit/06aa87b65f4608ded0dc312d526e83065618a1c8) fix(core): `WindowEvent` type used on `Window::on_window_event` ([#3796](https://www.github.com/tauri-apps/tauri/pull/3796)) on 2022-03-28
- Fixes `WindowEvent::Destroyed` not firing.
  - [169b5035](https://www.github.com/tauri-apps/tauri/commit/169b5035a93e3f33a420d4b2b0f8943e6404e07f) fix(core): actually fire `WindowEvent::Destroyed` ([#3797](https://www.github.com/tauri-apps/tauri/pull/3797)) on 2022-03-28

## \[1.0.0-rc.5]

- Added `updater_target` method to the `Builder` struct.
  - [579312f8](https://www.github.com/tauri-apps/tauri/commit/579312f834f08dca15e51e4f43c1d0bb65b54a51) feat(updater): separate intel and apple silicon targets, closes [#3359](https://www.github.com/tauri-apps/tauri/pull/3359) ([#3739](https://www.github.com/tauri-apps/tauri/pull/3739)) on 2022-03-23

- Added an option to disable the CSP injection of distributable assets nonces and hashes.
  - [f6e32ee1](https://www.github.com/tauri-apps/tauri/commit/f6e32ee1880eb364ed76beb937c9d12e14d54910) feat(core): add dangerous option to disable compile time CSP injection ([#3775](https://www.github.com/tauri-apps/tauri/pull/3775)) on 2022-03-28

- Toggle devtools when `Ctrl + Shift + I` or `Command + Option + I` is pressed.
  - [e05d718a](https://www.github.com/tauri-apps/tauri/commit/e05d718a7b46476d1fe4817c169008080e84f959) feat(core): add hotkey to toggle devtools, closes [#3776](https://www.github.com/tauri-apps/tauri/pull/3776) ([#3791](https://www.github.com/tauri-apps/tauri/pull/3791)) on 2022-03-28

- Use asynchronous file dialog on macOS and Windows to properly set the parent window.
  - [bf89a05f](https://www.github.com/tauri-apps/tauri/commit/bf89a05fcfef976886a833b24346e010fd1bd06c) fix(core): dialog parent window on macOS, closes [#3312](https://www.github.com/tauri-apps/tauri/pull/3312) ([#3753](https://www.github.com/tauri-apps/tauri/pull/3753)) on 2022-03-23

- The `Error` enum is now `Send + Sync`.
  - [da1e8793](https://www.github.com/tauri-apps/tauri/commit/da1e879358895f7b190b1c1b20d23da23666a74b) feat(core): improve and cleanup the `Error` enum ([#3748](https://www.github.com/tauri-apps/tauri/pull/3748)) on 2022-03-22

- Do not allow path traversal on the asset protocol.
  - [34a402f9](https://www.github.com/tauri-apps/tauri/commit/34a402f9b559af377b276d73b800e5e8b7dacbb1) fix(core): do not allow path traversal on the asset protocol ([#3774](https://www.github.com/tauri-apps/tauri/pull/3774)) on 2022-03-27

- Properly apply the CSP when loading a route that fallbacks to index.html.
  - [bcd43168](https://www.github.com/tauri-apps/tauri/commit/bcd43168a528dc4c54e28788430a93654c8fb452) fix(core): properly add CSP header to fallback routes ([#3641](https://www.github.com/tauri-apps/tauri/pull/3641)) on 2022-03-08

- Fix CSP usage on Linux when changing it via the `on_web_resource_request` handler.
  - [f5efc248](https://www.github.com/tauri-apps/tauri/commit/f5efc248da511e0924c9673b947d5de7ef69ac45) fix(core): runtime CSP changes on Linux on 2022-03-07

- Improved the updater response validation and error messages.
  - [dbc2873e](https://www.github.com/tauri-apps/tauri/commit/dbc2873e82dd56e13025f73281769fff323d32aa) feat(updater): improve validation and error messages, closes [#3761](https://www.github.com/tauri-apps/tauri/pull/3761) ([#3780](https://www.github.com/tauri-apps/tauri/pull/3780)) on 2022-03-27

- **Breaking change:** The `MenuItem::About` variant is now associated with a tuple value `(String, AboutMetadata)`.
  - [5fb74332](https://www.github.com/tauri-apps/tauri/commit/5fb74332ab9210ac062d96b0e9afd1c942ee2911) chore(deps): update wry to 0.14, tao to 0.7 ([#3790](https://www.github.com/tauri-apps/tauri/pull/3790)) on 2022-03-28

- Replace multiple dependencies who's C code compiled concurrently and caused
  the other ones to bloat compile time significantly.

- `zstd` -> `brotli`

- `blake3` -> a vendored version of the blake3 reference

- `ring` -> `getrandom`

See https://github.com/tauri-apps/tauri/pull/3773 for more information about
these specific choices.

- [8661e3e2](https://www.github.com/tauri-apps/tauri/commit/8661e3e24d96c399bfbcdee5d8e9d6beba2265a7) replace dependencies with long build times when used together (closes [#3571](https://www.github.com/tauri-apps/tauri/pull/3571)) ([#3773](https://www.github.com/tauri-apps/tauri/pull/3773)) on 2022-03-27
- **Breaking change:** The `Window::hwnd` method now returns *HWND* from `windows-rs` crate instead of *c_void* on Windows.
  - [4e807a53](https://www.github.com/tauri-apps/tauri/commit/4e807a53e2d6d3f3cd5293d90013d5cdded5454e) Support window parenting on macOS, closes [#3751](https://www.github.com/tauri-apps/tauri/pull/3751) ([#3754](https://www.github.com/tauri-apps/tauri/pull/3754)) on 2022-03-23
- Support window parenting on macOS
  - [4e807a53](https://www.github.com/tauri-apps/tauri/commit/4e807a53e2d6d3f3cd5293d90013d5cdded5454e) Support window parenting on macOS, closes [#3751](https://www.github.com/tauri-apps/tauri/pull/3751) ([#3754](https://www.github.com/tauri-apps/tauri/pull/3754)) on 2022-03-23
- **Breaking change:** The updater default targets have been renamed to include better support for different architectures.
  - [579312f8](https://www.github.com/tauri-apps/tauri/commit/579312f834f08dca15e51e4f43c1d0bb65b54a51) feat(updater): separate intel and apple silicon targets, closes [#3359](https://www.github.com/tauri-apps/tauri/pull/3359) ([#3739](https://www.github.com/tauri-apps/tauri/pull/3739)) on 2022-03-23
- **Breaking change:** Removed `RunEvent::CloseRequested` and `RunEvent::WindowClosed` and added `RunEvent::WindowEvent`.
  - [edad9f4f](https://www.github.com/tauri-apps/tauri/commit/edad9f4f55dcc69a06cd9d6d5a5068c94ecb77dd) refactor(core): add `RunEvent::WindowEvent` ([#3793](https://www.github.com/tauri-apps/tauri/pull/3793)) on 2022-03-28
- **Breaking change:** Removed `window_label` from `RunEvent::ExitRequested`.
  - [9ddf8d84](https://www.github.com/tauri-apps/tauri/commit/9ddf8d84a22cd6ccdce04bcc98b2b0f5fc54381a) fix(core): properly fire `WindowEvent::Destroyed`, closes [#3688](https://www.github.com/tauri-apps/tauri/pull/3688) ([#3778](https://www.github.com/tauri-apps/tauri/pull/3778)) on 2022-03-28
- **Breaking change:** The `tauri://` events are no longer emitted to listeners using `Window::listen`. Use the `App::run` closure, `Window::on_window_event` and `Window::on_menu_event` instead.
  - [5d538ec2](https://www.github.com/tauri-apps/tauri/commit/5d538ec27c246274df4ff5b8057ff78b6364a43f) refactor(core): use the event loop proxy to send updater events ([#3687](https://www.github.com/tauri-apps/tauri/pull/3687)) on 2022-03-15
- The `App::setup` closure can now return a boxed error directly.
  - [da1e8793](https://www.github.com/tauri-apps/tauri/commit/da1e879358895f7b190b1c1b20d23da23666a74b) feat(core): improve and cleanup the `Error` enum ([#3748](https://www.github.com/tauri-apps/tauri/pull/3748)) on 2022-03-22
- Implement `Debug` for `tauri::State`.
  - [0b49dd56](https://www.github.com/tauri-apps/tauri/commit/0b49dd566dae21c4dcb1cf110115aab982a7dab6) impl Debug for State closes [#3676](https://www.github.com/tauri-apps/tauri/pull/3676) ([#3677](https://www.github.com/tauri-apps/tauri/pull/3677)) on 2022-03-12
- **Breaking change:** The `Manager::manage` function now returns a bool indicating whether the type is already managed or not.
  - [263b45e1](https://www.github.com/tauri-apps/tauri/commit/263b45e1b4e72d6c99fc27e41d0c5e1d134f363b) refactor(core): return boolean on `Manager::manage` ([#3682](https://www.github.com/tauri-apps/tauri/pull/3682)) on 2022-03-13
- Set the `Access-Control-Allow-Origin` header on the `tauri` protocol response with the initial webview URL as value.
  - [1730b1a5](https://www.github.com/tauri-apps/tauri/commit/1730b1a51d1220e6c6a2eec405a3830cc3878224) feat(core): enable CORS on the tauri protocol ([#3750](https://www.github.com/tauri-apps/tauri/pull/3750)) on 2022-03-22
- **Breaking change:** The `tauri_runtime` crate is no longer exported since its API is not stable.
  - [1099a969](https://www.github.com/tauri-apps/tauri/commit/1099a9696e6639b46d736f8f3b446d2dfc4ef2f0) refactor(core): do not export `tauri_runtime` on `tauri` ([#3749](https://www.github.com/tauri-apps/tauri/pull/3749)) on 2022-03-22
- Added `Temp` to the `BaseDirectory` enum.
  - [266156a0](https://www.github.com/tauri-apps/tauri/commit/266156a0b08150b21140dd552c8bc252fe413cdd) feat(core): add `BaseDirectory::Temp` and `$TEMP` variable ([#3763](https://www.github.com/tauri-apps/tauri/pull/3763)) on 2022-03-24
- Added `$TEMP` to the allowed variables to the filesystem and asset protocol scopes.
  - [266156a0](https://www.github.com/tauri-apps/tauri/commit/266156a0b08150b21140dd552c8bc252fe413cdd) feat(core): add `BaseDirectory::Temp` and `$TEMP` variable ([#3763](https://www.github.com/tauri-apps/tauri/pull/3763)) on 2022-03-24
- Update `wry` to `0.14` and `tao` to `0.7`.
  - [f2d24ef2](https://www.github.com/tauri-apps/tauri/commit/f2d24ef2fbd95ec7d3433ba651964f4aa3b7f48c) chore(deps): update wry ([#1482](https://www.github.com/tauri-apps/tauri/pull/1482)) on 2021-04-14
  - [e267ebf1](https://www.github.com/tauri-apps/tauri/commit/e267ebf1f1009b99829e0a7d71519925f5792f9f) Apply Version Updates From Current Changes ([#1486](https://www.github.com/tauri-apps/tauri/pull/1486)) on 2021-04-14
  - [5fb74332](https://www.github.com/tauri-apps/tauri/commit/5fb74332ab9210ac062d96b0e9afd1c942ee2911) chore(deps): update wry to 0.14, tao to 0.7 ([#3790](https://www.github.com/tauri-apps/tauri/pull/3790)) on 2022-03-28
- Added `updater` method to `App` and `AppHandle`, a builder to check for app updates.
  - [4094494a](https://www.github.com/tauri-apps/tauri/commit/4094494a1b3125bf01676dabaa69e56cc8741d59) feat(core): add API to manually trigger updater check ([#3712](https://www.github.com/tauri-apps/tauri/pull/3712)) on 2022-03-17
  - [c64268f9](https://www.github.com/tauri-apps/tauri/commit/c64268f9274bdb7352da1a53184e487b03437dc2) feat(updater): expose builder, allow setting a custom version checker ([#3792](https://www.github.com/tauri-apps/tauri/pull/3792)) on 2022-03-28
- Allow using a custom updater version checker via `App::updater().should_install()`.
  - [c64268f9](https://www.github.com/tauri-apps/tauri/commit/c64268f9274bdb7352da1a53184e487b03437dc2) feat(updater): expose builder, allow setting a custom version checker ([#3792](https://www.github.com/tauri-apps/tauri/pull/3792)) on 2022-03-28
- Added download progress events to the updater.
  - [f0db3f9b](https://www.github.com/tauri-apps/tauri/commit/f0db3f9b8357dc304a8254426034c4d1733fbd45) feat(updater): add download progress events ([#3734](https://www.github.com/tauri-apps/tauri/pull/3734)) on 2022-03-18
- Send updater events to the `App::run` closure.
  - [5d538ec2](https://www.github.com/tauri-apps/tauri/commit/5d538ec27c246274df4ff5b8057ff78b6364a43f) refactor(core): use the event loop proxy to send updater events ([#3687](https://www.github.com/tauri-apps/tauri/pull/3687)) on 2022-03-15
- Run the updater on startup even if no window was created.
  - [c4ca80f9](https://www.github.com/tauri-apps/tauri/commit/c4ca80f919551cbcae53d931f860115c2d591d14) feat(core): use AppHandle instead of Window on the updater logic ([#3702](https://www.github.com/tauri-apps/tauri/pull/3702)) on 2022-03-15
- Properly fire the window destroyed event.
  - [9ddf8d84](https://www.github.com/tauri-apps/tauri/commit/9ddf8d84a22cd6ccdce04bcc98b2b0f5fc54381a) fix(core): properly fire `WindowEvent::Destroyed`, closes [#3688](https://www.github.com/tauri-apps/tauri/pull/3688) ([#3778](https://www.github.com/tauri-apps/tauri/pull/3778)) on 2022-03-28
- Added `close_devtools` and `is_devtools_open` APIs to the `Window` struct.
  - [e05d718a](https://www.github.com/tauri-apps/tauri/commit/e05d718a7b46476d1fe4817c169008080e84f959) feat(core): add hotkey to toggle devtools, closes [#3776](https://www.github.com/tauri-apps/tauri/pull/3776) ([#3791](https://www.github.com/tauri-apps/tauri/pull/3791)) on 2022-03-28
- Added the `WindowEvent::FileDrop` variant.
  - [07d1584c](https://www.github.com/tauri-apps/tauri/commit/07d1584cf06ea326aa45d8044bee1b77ecba5006) feat(core): add `WindowEvent::FileDrop`, closes [#3664](https://www.github.com/tauri-apps/tauri/pull/3664) ([#3686](https://www.github.com/tauri-apps/tauri/pull/3686)) on 2022-03-13
- Added a configuration flag for disallowing install downgrades on Windows.
  **Breaking change:** The default behavior on Windows is now to allow downgrades.
  - [8b807e09](https://www.github.com/tauri-apps/tauri/commit/8b807e09d6868f6bff8357f16d27b15bd1fccadd) refactor(bundler): allow downgrades, add option to disallow on Windows ([#3777](https://www.github.com/tauri-apps/tauri/pull/3777)) on 2022-03-27

## \[1.0.0-rc.4]

- Run `AppHandle` cleanup code before restarting the application on the `process > relaunch` API.
  - [9c65abce](https://www.github.com/tauri-apps/tauri/commit/9c65abce1bd3895b7f8d9d4d8a449ac4802fce24) feat(core): run cleanup code on the relaunch API ([#3629](https://www.github.com/tauri-apps/tauri/pull/3629)) on 2022-03-07
- **Breaking change:** The `Builder#create_window` API now returns a Result validating the window label.
  - [64e00542](https://www.github.com/tauri-apps/tauri/commit/64e0054299c95f10ef5a1a9d3f914bbaeff3d73f) refactor(core): do not panic on invalid window labels,[#3544](https://www.github.com/tauri-apps/tauri/pull/3544) ([#3596](https://www.github.com/tauri-apps/tauri/pull/3596)) on 2022-03-03
- Added `tsp` config option under `tauri > bundle > windows`, which enables Time-Stamp Protocol (RFC 3161) for the timestamping
  server under code signing on Windows if set to `true`.
  - [bdd5f7c2](https://www.github.com/tauri-apps/tauri/commit/bdd5f7c2f03af4af8b60a9527e55bb18525d989b) fix: add support for Time-Stamping Protocol for Windows codesigning (fix [#3563](https://www.github.com/tauri-apps/tauri/pull/3563)) ([#3570](https://www.github.com/tauri-apps/tauri/pull/3570)) on 2022-03-07
- Revert the `clap` usage back to the version 3.0 API.
  - [2b554c38](https://www.github.com/tauri-apps/tauri/commit/2b554c38a5181b948d7674b6ef33e4049ca7d327) fix(core): revert to clap 3.0 API, allow deprecations, closes [#3549](https://www.github.com/tauri-apps/tauri/pull/3549) ([#3552](https://www.github.com/tauri-apps/tauri/pull/3552)) on 2022-02-24
- The `tauri::api::process::Command` API now properly reads stdout and stderr messages that ends with a carriage return (`\r`) instead of just a newline (`\n`).
  - [0a0de8ab](https://www.github.com/tauri-apps/tauri/commit/0a0de8ab6ed80b7722012e83636dff41a813b770) fix: read Command output ending with a carriage return, closes [#3508](https://www.github.com/tauri-apps/tauri/pull/3508) ([#3523](https://www.github.com/tauri-apps/tauri/pull/3523)) on 2022-02-24
- Fixes filesystem and asset scope stripping the first component of the allowed path.
  - [4d0e2ecc](https://www.github.com/tauri-apps/tauri/commit/4d0e2eccd9e0e1db16e6cc20613ffdc3dbe0474e) fix(core): scope should not strip the first path component, closes [#3592](https://www.github.com/tauri-apps/tauri/pull/3592) ([#3602](https://www.github.com/tauri-apps/tauri/pull/3602)) on 2022-03-03
- Ignore trailing slashes on path scope validation.
  - [929a83dd](https://www.github.com/tauri-apps/tauri/commit/929a83dd4d2614a119ab05122663669648e8c701) fix(core): ignore trailing slashes on scope validation, closes [#3580](https://www.github.com/tauri-apps/tauri/pull/3580) ([#3601](https://www.github.com/tauri-apps/tauri/pull/3601)) on 2022-03-03
- Fixes `Command::output` and `Command::status` deadlock when running on async commands.
  - [0163489e](https://www.github.com/tauri-apps/tauri/commit/0163489ed60ec0fe9486b5556ec0234499852a16) fix(core): `safe_block_on` usage on async contexts, closes [#3505](https://www.github.com/tauri-apps/tauri/pull/3505) ([#3513](https://www.github.com/tauri-apps/tauri/pull/3513)) on 2022-02-24
- Update tray menu id map when `SystemTrayHandle::set_menu` is called.
  - [da882431](https://www.github.com/tauri-apps/tauri/commit/da8824318affb50475f28b9c78c56da3fb048708) fix(core): update tray menu ids on `set_menu`, closes [#3608](https://www.github.com/tauri-apps/tauri/pull/3608) ([#3611](https://www.github.com/tauri-apps/tauri/pull/3611)) on 2022-03-04
- Allow absolute paths on the filesystem APIs as long as it does not include parent directory components.
  - [b744cd27](https://www.github.com/tauri-apps/tauri/commit/b744cd2758bbc5da39d6105fd82002dc6536dd16) feat: extend scopes with user selected paths, closes [#3591](https://www.github.com/tauri-apps/tauri/pull/3591) ([#3595](https://www.github.com/tauri-apps/tauri/pull/3595)) on 2022-03-03
- **Breaking change:** The `tauri::api::file::Extract` API is now available when the `fs-extract-api` feature is enabled.
  - [0f155898](https://www.github.com/tauri-apps/tauri/commit/0f1558980a0fb1d6c042988e173047f0590b6574) fix(core): docs.rs on Windows and macOS ([#3566](https://www.github.com/tauri-apps/tauri/pull/3566)) on 2022-03-02
- Allow listening to events on the filesystem and asset scopes.
  - [58070c1e](https://www.github.com/tauri-apps/tauri/commit/58070c1eb443c29b01638b7fc3f69aacdcc464a6) feat(core): filesystem and asset protocol scope events ([#3609](https://www.github.com/tauri-apps/tauri/pull/3609)) on 2022-03-04
- Allow configuring forbidden paths on the asset and filesystem scopes.
  - [983ccb81](https://www.github.com/tauri-apps/tauri/commit/983ccb815b1cb094ac681c2b36c69ec16f123b4e) feat(core): allow denying paths on the fs and asset scopes ([#3607](https://www.github.com/tauri-apps/tauri/pull/3607)) on 2022-03-03
- Extend the allowed patterns for the filesystem and asset protocol when the user selects a path (dialog open and save commands and file drop on the window).
  - [b744cd27](https://www.github.com/tauri-apps/tauri/commit/b744cd2758bbc5da39d6105fd82002dc6536dd16) feat: extend scopes with user selected paths, closes [#3591](https://www.github.com/tauri-apps/tauri/pull/3591) ([#3595](https://www.github.com/tauri-apps/tauri/pull/3595)) on 2022-03-03
- The HTTP scope now matches the entire URL using a glob pattern instead of only its path.
  - [944b124c](https://www.github.com/tauri-apps/tauri/commit/944b124ce04a15a415fcbb0afdfa78b0e900c97a) feat(core): enhance HTTP scope glob validation, closes [#3507](https://www.github.com/tauri-apps/tauri/pull/3507) ([#3515](https://www.github.com/tauri-apps/tauri/pull/3515)) on 2022-02-24
- Parse window icons at compile time.
  - [8c935872](https://www.github.com/tauri-apps/tauri/commit/8c9358725a17dcc2acaf4d10c3f654afdff586b0) refactor(core): move `png` and `ico` behind Cargo features ([#3588](https://www.github.com/tauri-apps/tauri/pull/3588)) on 2022-03-05
- **Breaking change:** Move `ico` and `png` parsing behind `icon-ico` and `icon-png` Cargo features.
  - [8c935872](https://www.github.com/tauri-apps/tauri/commit/8c9358725a17dcc2acaf4d10c3f654afdff586b0) refactor(core): move `png` and `ico` behind Cargo features ([#3588](https://www.github.com/tauri-apps/tauri/pull/3588)) on 2022-03-05
- Return an error when creating a window with an invalid label instead of panicking.
  - [64e00542](https://www.github.com/tauri-apps/tauri/commit/64e0054299c95f10ef5a1a9d3f914bbaeff3d73f) refactor(core): do not panic on invalid window labels,[#3544](https://www.github.com/tauri-apps/tauri/pull/3544) ([#3596](https://www.github.com/tauri-apps/tauri/pull/3596)) on 2022-03-03
- Allows the configuration CSP to be an object mapping a directive name to its source list.
  - [3fe0260f](https://www.github.com/tauri-apps/tauri/commit/3fe0260f4c7f47dba28b0d049c741d885bc6d356) feat(core): allow CSP configuration to be an object, ref [#3533](https://www.github.com/tauri-apps/tauri/pull/3533) ([#3603](https://www.github.com/tauri-apps/tauri/pull/3603)) on 2022-03-04
- Allow range in the form of `bytes=0-*` on the asset protocol.
  - [d06efc77](https://www.github.com/tauri-apps/tauri/commit/d06efc7704092a549c886be701122ad420db5543) fix(core): parse range `bytes=0-*`, closes [#3143](https://www.github.com/tauri-apps/tauri/pull/3143) ([#3516](https://www.github.com/tauri-apps/tauri/pull/3516)) on 2022-02-24
- Reimplement endpoint to read file as string for performance.
  - [834ccc51](https://www.github.com/tauri-apps/tauri/commit/834ccc51539401d36a7dfa1c0982623c9c446a4c) feat(core): reimplement `readTextFile` for performance ([#3631](https://www.github.com/tauri-apps/tauri/pull/3631)) on 2022-03-07
- **Breaking change:** Renamed the `command` Cargo feature to `process-command-api`.
  - [4e1af005](https://www.github.com/tauri-apps/tauri/commit/4e1af005a1fc80e911299b5fd6d6b0f3eb44f8e7) refactor(core): rename `command` feature to `process-command-api` ([#3594](https://www.github.com/tauri-apps/tauri/pull/3594)) on 2022-03-03
- Disabled the default features for the `zip` crate.
  - [5293445f](https://www.github.com/tauri-apps/tauri/commit/5293445f08f1484639ad9f56c45aad53039ba579) refactor(core): disable default features for the zip crate ([#3624](https://www.github.com/tauri-apps/tauri/pull/3624)) on 2022-03-06
- The `cmd` field is no longer required on the shell scope for sidecars.
  - [9b3b163b](https://www.github.com/tauri-apps/tauri/commit/9b3b163baa9b8eb2ef0bbc53f6303cc7a59a01af) feat(core): simplify scope definition for sidecars ([#3574](https://www.github.com/tauri-apps/tauri/pull/3574)) on 2022-03-02
- Fixes a regression on the `unlisten` command.
  - [76c791bd](https://www.github.com/tauri-apps/tauri/commit/76c791bd2b836d2055410e37e71716172a3f81ef) fix(core): regression on the unlisten function ([#3623](https://www.github.com/tauri-apps/tauri/pull/3623)) on 2022-03-06
- Run `AppHandle` cleanup code before restarting the application when a new update is installed.
  - [fce7d3bb](https://www.github.com/tauri-apps/tauri/commit/fce7d3bbae8d9e7e928a31b04a0a87dcaf4cd65f) feat(core): run app cleanup code before updater restart, closes [#3605](https://www.github.com/tauri-apps/tauri/pull/3605) ([#3616](https://www.github.com/tauri-apps/tauri/pull/3616)) on 2022-03-04
- Added a `WindowBuilder` type.
  - [141133a4](https://www.github.com/tauri-apps/tauri/commit/141133a414154631d42a0873c6abe2d76391b87d) feat(core): add WindowBuilder type ([#3598](https://www.github.com/tauri-apps/tauri/pull/3598)) on 2022-03-04
- Added `WindowBuilder::on_web_resource_request`, which allows customizing the tauri custom protocol response.
  - [3b13fda5](https://www.github.com/tauri-apps/tauri/commit/3b13fda56f515c708014c0396ed5ca295faaef84) feat(core): add `WindowBuilder::on_request`, closes [#3533](https://www.github.com/tauri-apps/tauri/pull/3533) ([#3618](https://www.github.com/tauri-apps/tauri/pull/3618)) on 2022-03-06

## \[1.0.0-rc.3]

- `tauri::plugin::Builder` closures are no longer required to implement `Sync`.
  - [fb7ee2c9](https://www.github.com/tauri-apps/tauri/commit/fb7ee2c987d9fca23f08bd470789069d9477c66e) drop Sync req from `setup` and `setup_with_config` ([#3471](https://www.github.com/tauri-apps/tauri/pull/3471)) on 2022-02-16
  - [b8e4d651](https://www.github.com/tauri-apps/tauri/commit/b8e4d651f9866b34bd3afcf2392812a18e1cee53) fix(core): drop all plugin builder Sync requirements ([#3490](https://www.github.com/tauri-apps/tauri/pull/3490)) on 2022-02-17
- Added context to the file system endpoint errors.
  - [06053833](https://www.github.com/tauri-apps/tauri/commit/060538331c138473159cf8fee0fcb7904ca33d3b) feat(core): add context to the filesystem APIs errors, closes [#3457](https://www.github.com/tauri-apps/tauri/pull/3457) ([#3480](https://www.github.com/tauri-apps/tauri/pull/3480)) on 2022-02-16
- Changed the default value for `tauri > bundle > macOS > minimumSystemVersion` to `10.13`.
  - [fce344b9](https://www.github.com/tauri-apps/tauri/commit/fce344b90b7227f8f5514853c2f885fb24d3648e) feat(core): set default value for `minimum_system_version` to 10.13 ([#3497](https://www.github.com/tauri-apps/tauri/pull/3497)) on 2022-02-17

## \[1.0.0-rc.2]

- Ease the requirements for plugin hooks. `setup` and `setup_with_config` can now be `FnOnce` and `on_webview_ready`, `on_event` and `on_page_load` can be `FnMut`.
  - [fd557e98](https://www.github.com/tauri-apps/tauri/commit/fd557e984db17335a7c2ff10e6fe9c07065a95ed) Ease plugin hook restrictions ([#3404](https://www.github.com/tauri-apps/tauri/pull/3404)) on 2022-02-13
- Fixes an issue with the updater when replacing the `{{target}}` and `{{current_version}}` variables due to percent-encoding on the `Url` that is parsed from the configuration.
  - [20f0477f](https://www.github.com/tauri-apps/tauri/commit/20f0477f95b836cdf03ca66dadd9c0d3c95b198a) fix(core): updater not replacing variables, closes [#3428](https://www.github.com/tauri-apps/tauri/pull/3428) ([#3432](https://www.github.com/tauri-apps/tauri/pull/3432)) on 2022-02-13

## \[1.0.0-rc.1]

- Change default value for the `freezePrototype` configuration to `false`.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

## \[1.0.0-rc.0]

- The dialog allowlist now includes flags for the `message`, `ask` and `confirm` APIs.
  - [d660cab3](https://www.github.com/tauri-apps/tauri/commit/d660cab38d7d703e8b2bb85a3e9462d9e28b086b) feat: enhance allowlist configuration \[TRI-027] ([#11](https://www.github.com/tauri-apps/tauri/pull/11)) on 2022-01-09
- - **Breaking change:** Renamed `tauri::Event`  to `tauri::RunEvent`
- Exported `tauri::Event` and `tauri::EventHandler` so you can define a function and pass it to `Window::listen`
- [15358b18](https://www.github.com/tauri-apps/tauri/commit/15358b1895487cb9c258a8ca4d2336b4215e2a8f) Expose event interface. fixes [#2733](https://www.github.com/tauri-apps/tauri/pull/2733) ([#3321](https://www.github.com/tauri-apps/tauri/pull/3321)) on 2022-02-04
- The `tauri::api` modules `http`, `notification`, `dialog`, and `process::Command` APIs are now hidden behind a feature flag, `http-api`, `notification`, `dialog` and `command`, respectively.
  - [6feb5a0c](https://www.github.com/tauri-apps/tauri/commit/6feb5a0c50852a4792a9cc80967eab0fcf18609e) refactor(core): api feature flags, documentation ([#26](https://www.github.com/tauri-apps/tauri/pull/26)) on 2022-01-09
- Add `title` option to file open/save dialogs.
  - [e1d6a6e6](https://www.github.com/tauri-apps/tauri/commit/e1d6a6e6445637723e2331ca799a662e720e15a8) Create api-file-dialog-title.md ([#3235](https://www.github.com/tauri-apps/tauri/pull/3235)) on 2022-01-16
  - [62c7a8ad](https://www.github.com/tauri-apps/tauri/commit/62c7a8ad30fd3031b8679960590e5ef3eef8e4da) chore(covector): prepare for `rc` release ([#3376](https://www.github.com/tauri-apps/tauri/pull/3376)) on 2022-02-10
- Added `any_thread()` to the `tauri::Builder` to run applications on any thread (only exposed on Linux and Windows).
  - [af44bf81](https://www.github.com/tauri-apps/tauri/commit/af44bf8168310cf77fbe102a53e7c433f11641a3) feat(core): allow app run on any thread on Linux & Windows, closes [#3172](https://www.github.com/tauri-apps/tauri/pull/3172) ([#3353](https://www.github.com/tauri-apps/tauri/pull/3353)) on 2022-02-07
- Enable CORS on the `asset` protocol.
  - [d28ac8aa](https://www.github.com/tauri-apps/tauri/commit/d28ac8aac0d19a70bb658f12e56330ec8ac4dda5) fix(core): enable CORS on the `asset` protocol, closes [#2965](https://www.github.com/tauri-apps/tauri/pull/2965) ([#2974](https://www.github.com/tauri-apps/tauri/pull/2974)) on 2021-12-09
- The `asset://` custom protocol is only defined when either the `api-all`, `protocol-all` or `protocol-asset` feature flags are enabled. These feature flags are accessible with the `tauri.conf.json` allowlist.
  - [7920ff14](https://www.github.com/tauri-apps/tauri/commit/7920ff14e6424079c48ea5645d9aa13e7a272b87) feat: scope the `fs` API and the `asset` protocol \[TRI-026] \[TRI-010] \[TRI-011] ([#10](https://www.github.com/tauri-apps/tauri/pull/10)) on 2022-01-09
- Expose the `asset_resolver` API on the `App` and `AppHandle` structs.
  - [7c6c7adc](https://www.github.com/tauri-apps/tauri/commit/7c6c7adcc4c1ddd7406dd2d3b31541d5b962b385) feat(core): add `asset_resolver` API ([#2879](https://www.github.com/tauri-apps/tauri/pull/2879)) on 2021-11-12
- **Breaking change:** Refactored the types returned from the `async_runtime` module.
  - [a3537078](https://www.github.com/tauri-apps/tauri/commit/a3537078ddeaac7d26250f111f071072cf328b0b) feat(core): allow running along another tokio runtime, closes [#2838](https://www.github.com/tauri-apps/tauri/pull/2838) ([#2973](https://www.github.com/tauri-apps/tauri/pull/2973)) on 2021-12-08
- Added `tauri::async_runtime::set` method, allowing to share your tokio runtime with Tauri.
  - [a3537078](https://www.github.com/tauri-apps/tauri/commit/a3537078ddeaac7d26250f111f071072cf328b0b) feat(core): allow running along another tokio runtime, closes [#2838](https://www.github.com/tauri-apps/tauri/pull/2838) ([#2973](https://www.github.com/tauri-apps/tauri/pull/2973)) on 2021-12-08
- Added `tauri::async_runtime::spawn_blocking` API.
  - [a3537078](https://www.github.com/tauri-apps/tauri/commit/a3537078ddeaac7d26250f111f071072cf328b0b) feat(core): allow running along another tokio runtime, closes [#2838](https://www.github.com/tauri-apps/tauri/pull/2838) ([#2973](https://www.github.com/tauri-apps/tauri/pull/2973)) on 2021-12-08
- The `callback` and `error` invoke fields, along with other `transformCallback` usages, are now validated to be numeric.
  - [a48b8b18](https://www.github.com/tauri-apps/tauri/commit/a48b8b18d428bcc404d489daa690bbefe1f57311) feat(core): validate callbacks and event names \[TRI-038] \[TRI-020] ([#21](https://www.github.com/tauri-apps/tauri/pull/21)) on 2022-01-09
- Change `Error::ParseCliArguments(clap::Error)` to `Error::ParseCliArguments(String)` because `clap::Error` is not `Send`.
  - [1f988535](https://www.github.com/tauri-apps/tauri/commit/1f98853573a837dd0cfc2161b206a5033ec2da5e) chore(deps) Update Tauri Core ([#2480](https://www.github.com/tauri-apps/tauri/pull/2480)) on 2021-08-24
- The `api::process::Command` APIs are now hidden behind the `command` feature flag.
  - [eed01728](https://www.github.com/tauri-apps/tauri/commit/eed017287fed2ade689af4268e8b63b9c9f2e585) feat(core): add `shell > sidecar` allowlist and `process` feature flag \[TRI-037] ([#18](https://www.github.com/tauri-apps/tauri/pull/18)) on 2021-10-24
- Add `tauri::api::path::log_dir` function to access the suggested log directory path.
  - [acbb3ae7](https://www.github.com/tauri-apps/tauri/commit/acbb3ae7bb0165846b9456aea103269f027fc548) feat: add Log directory ([#2736](https://www.github.com/tauri-apps/tauri/pull/2736)) on 2021-10-16
  - [62c7a8ad](https://www.github.com/tauri-apps/tauri/commit/62c7a8ad30fd3031b8679960590e5ef3eef8e4da) chore(covector): prepare for `rc` release ([#3376](https://www.github.com/tauri-apps/tauri/pull/3376)) on 2022-02-10
- The `process`, `path` and `updater` APIs now takes a `tauri::Env` argument, used to force environment variables load on startup to prevent env var update attacks.
  - [7209fdf7](https://www.github.com/tauri-apps/tauri/commit/7209fdf732ffe1893aaa9cd970ab6491a883d997) refactor(core): load APPIMAGE and APPDIR env vars on startup \[TRI-007] \[TRI-041] on 2022-01-09
- Now `resolve()`, `join()` and `normalize()` from the `path` module, won't throw errors if the path doesn't exist, which matches NodeJS behavior.
  - [fe381a0b](https://www.github.com/tauri-apps/tauri/commit/fe381a0bde86ebf4014007f6e21af4c1a9e58cef) fix: `join` no longer cares if path doesn't exist, closes [#2499](https://www.github.com/tauri-apps/tauri/pull/2499) ([#2548](https://www.github.com/tauri-apps/tauri/pull/2548)) on 2021-09-21
- **Breaking change:** Return `Window` on `App` and `AppHandle`'s `create_window` function.
  - [e15a8af8](https://www.github.com/tauri-apps/tauri/commit/e15a8af8434805da9716f3f840daa4e87b18b0a2) refactor(core): return `Window` on `create_window` API ([#3211](https://www.github.com/tauri-apps/tauri/pull/3211)) on 2022-01-13
  - [62c7a8ad](https://www.github.com/tauri-apps/tauri/commit/62c7a8ad30fd3031b8679960590e5ef3eef8e4da) chore(covector): prepare for `rc` release ([#3376](https://www.github.com/tauri-apps/tauri/pull/3376)) on 2022-02-10
- Apply `nonce` to `script` and `style` tags and set them on the `CSP` (`script-src` and `style-src` fetch directives).
  - [cf54dcf9](https://www.github.com/tauri-apps/tauri/commit/cf54dcf9c81730e42c9171daa9c8aa474c95b522) feat: improve `CSP` security with nonces and hashes, add `devCsp` \[TRI-004] ([#8](https://www.github.com/tauri-apps/tauri/pull/8)) on 2022-01-09
- The path returned from `tauri::api::process::current_binary` is now cached when loading the binary.
  - [7c3db7a3](https://www.github.com/tauri-apps/tauri/commit/7c3db7a3811fd4de3e71c78cfd00894fa51ab786) cache current binary path much sooner ([#45](https://www.github.com/tauri-apps/tauri/pull/45)) on 2022-02-01
- Added an API to use a custom invoke system to receive and respond to commands (`Builder#invoke_system`).
  - [15164d93](https://www.github.com/tauri-apps/tauri/commit/15164d930a64807054057d83a2b6dae8540bcd9a) feat(core): allow defining a custom invoke system ([#2899](https://www.github.com/tauri-apps/tauri/pull/2899)) on 2021-11-16
- Inject configured `CSP` on `data:` URLs.
  - [8259cd64](https://www.github.com/tauri-apps/tauri/commit/8259cd64c2087b23b694a78aa98c43866762faec) feat(core): inject `CSP` on data URLs \[TRI-049] ([#16](https://www.github.com/tauri-apps/tauri/pull/16)) on 2022-01-09
- Emit `tauri://*` events to Rust listeners.
  - [4c4ab1eb](https://www.github.com/tauri-apps/tauri/commit/4c4ab1eb8b0b8b98ded402d0afb9dbca7ffe08e8) fix(core): trigger `tauri://*` events to Rust listeners, closes [#2901](https://www.github.com/tauri-apps/tauri/pull/2901) ([#2902](https://www.github.com/tauri-apps/tauri/pull/2902)) on 2021-11-16
- Emit `tauri://window-created` event for windows created on the backend.
  - [1dbd887a](https://www.github.com/tauri-apps/tauri/commit/1dbd887a6e0fc77b56e1b4d1b96dae8ddea1d5ea) fix(core): emit tauri://window-created event for windows created on Rust ([#3299](https://www.github.com/tauri-apps/tauri/pull/3299)) on 2022-02-04
- Enable non-session cookie persistence on Linux.
  - [d7c02a30](https://www.github.com/tauri-apps/tauri/commit/d7c02a30a56de79100804969138b379e703f0e07) feat(core): persist non-session cookies on Linux ([#3052](https://www.github.com/tauri-apps/tauri/pull/3052)) on 2021-12-09
- Expose `tauri::api::ipc::{serialize_js_with, serialize_js}` functions.
  - [5a94200f](https://www.github.com/tauri-apps/tauri/commit/5a94200f65feb4fddc79f0c68f8bc0a4067ad51a) feat(core): expose functions to serialize `serde::Serialize` values to JS ([#3354](https://www.github.com/tauri-apps/tauri/pull/3354)) on 2022-02-07
- Resolve `asset` protocol HTTP request instead of panicking if the file does not exist or cannot be read.
  - [03fc92c8](https://www.github.com/tauri-apps/tauri/commit/03fc92c8304efe897a09de6e82da2fea7f69ecab) fix(core): resolve request instead of panicking on asset protocol ([#3347](https://www.github.com/tauri-apps/tauri/pull/3347)) on 2022-02-06
- Avoid `async_runtime::block_on` panics when used along another tokio runtime.
  - [a3537078](https://www.github.com/tauri-apps/tauri/commit/a3537078ddeaac7d26250f111f071072cf328b0b) feat(core): allow running along another tokio runtime, closes [#2838](https://www.github.com/tauri-apps/tauri/pull/2838) ([#2973](https://www.github.com/tauri-apps/tauri/pull/2973)) on 2021-12-08
- Prevent window closing if `tauri://close-requested` is listened on the JS layer. Users must call `appWindow.close()` manually when listening to that event.
  - [74dff536](https://www.github.com/tauri-apps/tauri/commit/74dff536d4757820e68ab05301be83c0870c22ed) fix(core): emit `tauri://close-requested` to JS, closes [#2996](https://www.github.com/tauri-apps/tauri/pull/2996) ([#3041](https://www.github.com/tauri-apps/tauri/pull/3041)) on 2021-12-09
- Fixes a deadlock when creating a window from a menu event handler.
  - [9c82006b](https://www.github.com/tauri-apps/tauri/commit/9c82006b2fe166d20510183e36cee099bf96e8d9) fix(core): deadlock when creating window from menu handler, closes [#3110](https://www.github.com/tauri-apps/tauri/pull/3110) ([#3126](https://www.github.com/tauri-apps/tauri/pull/3126)) on 2021-12-28
- Fixes the dialog `defaultPath` usage on Linux.
  - [2212bd5d](https://www.github.com/tauri-apps/tauri/commit/2212bd5d75146f5a2df27cc2157a057642f626da) fix: dialog default path on Linux, closes [#3091](https://www.github.com/tauri-apps/tauri/pull/3091) ([#3123](https://www.github.com/tauri-apps/tauri/pull/3123)) on 2021-12-27
- Fixes `WindowEvent::Focus` and `WindowEvent::Blur` events not firing.
  - [3b33d67a](https://www.github.com/tauri-apps/tauri/commit/3b33d67aa4f48dcf4e32b3b8a5f45e83808efc2d) fix: re-adding focus/blur events for linux and macos (fix [#2485](https://www.github.com/tauri-apps/tauri/pull/2485)) ([#2489](https://www.github.com/tauri-apps/tauri/pull/2489)) on 2021-08-24
- Fixes `tauri://focus` and `tauri://blur` events not firing.
  - [3b33d67a](https://www.github.com/tauri-apps/tauri/commit/3b33d67aa4f48dcf4e32b3b8a5f45e83808efc2d) fix: re-adding focus/blur events for linux and macos (fix [#2485](https://www.github.com/tauri-apps/tauri/pull/2485)) ([#2489](https://www.github.com/tauri-apps/tauri/pull/2489)) on 2021-08-24
- Use webview's inner_size instead of window's value to get the correct size on macOS.
  - [4c0c780e](https://www.github.com/tauri-apps/tauri/commit/4c0c780e00d8851be38cb1c22f636d9e4ed34a23) fix(core): window's inner_size usage, closes [#2187](https://www.github.com/tauri-apps/tauri/pull/2187) ([#2690](https://www.github.com/tauri-apps/tauri/pull/2690)) on 2021-09-29
- Fixes resource directory resolution on Linux.
  - [1a28904b](https://www.github.com/tauri-apps/tauri/commit/1a28904b8ebea92e143d5dc21ebd209e9edec531) fix(core): resource path resolution on Linux, closes [#2493](https://www.github.com/tauri-apps/tauri/pull/2493) on 2021-08-22
- Fixes the menu id mapping not reflecting the current window.
  - [ac37b56e](https://www.github.com/tauri-apps/tauri/commit/ac37b56ef43c9e97039967a5fd99f0d2dccb5b5a) fix(core): menu id map not reflecting the current window menu ([#2726](https://www.github.com/tauri-apps/tauri/pull/2726)) on 2021-10-08
- `Manager::once_global` and `Window::once` allow `FnOnce` callbacks.
  - [d5400a3d](https://www.github.com/tauri-apps/tauri/commit/d5400a3d62ff2a37ccad20987dfea309725975a6) `once_global` and `once` accept FnOnce callbacks ([#3383](https://www.github.com/tauri-apps/tauri/pull/3383)) on 2022-02-10
- Properly check if document is loaded before invoking commands.
  - [000d126e](https://www.github.com/tauri-apps/tauri/commit/000d126e0e7231fb666b9ef53d6a1479dca774f7) fix(core): properly check if document is loaded, closes [#2716](https://www.github.com/tauri-apps/tauri/pull/2716) ([#2900](https://www.github.com/tauri-apps/tauri/pull/2900)) on 2021-11-16
- Initialize system tray before windows so `tray_handle` can be accessed on command handlers.
  - [dbe0d21b](https://www.github.com/tauri-apps/tauri/commit/dbe0d21b6fdc0f7d4c7b5e099cef805b60ef203e) fix(core): initialize system tray before app windows on 2021-08-31
- Reimplement `remove_system_tray` on Windows to drop the `SystemTray` to run its cleanup code.
  - [a03b8554](https://www.github.com/tauri-apps/tauri/commit/a03b85545a4b0b61a598a43eabe96e03565dcaf0) fix(core): tray not closing on Windows ([#3351](https://www.github.com/tauri-apps/tauri/pull/3351)) on 2022-02-07
- Immediately listen to `tauri://window-created` event to catch it before the application triggers it.
  - [878b8b9a](https://www.github.com/tauri-apps/tauri/commit/878b8b9a1fc825b1ea34f955e053311409a1468d) fix(core): immediately listen to window-created, closes [#3297](https://www.github.com/tauri-apps/tauri/pull/3297) ([#3298](https://www.github.com/tauri-apps/tauri/pull/3298)) on 2022-02-04
- The `tauri::Window#emit` function now correctly sends the event to all windows that has a registered listener.
  **Breaking change:** `Window#emit_and_trigger` and `Window#emit` now requires the payload to be cloneable.
  - [9b340552](https://www.github.com/tauri-apps/tauri/commit/9b340552643a1d8b9311219101329ce23c9271ea) fix(core): window-specific event delivery, closes [#3302](https://www.github.com/tauri-apps/tauri/pull/3302) ([#3344](https://www.github.com/tauri-apps/tauri/pull/3344)) on 2022-02-06
- Allow using a fixed version for the Webview2 runtime via the `tauri > bundle > windows > webviewFixedRuntimePath` config option.
  - [85df94f2](https://www.github.com/tauri-apps/tauri/commit/85df94f2b0d40255812b42c5e32a70c4b45392df) feat(core): config for fixed webview2 runtime version path ([#27](https://www.github.com/tauri-apps/tauri/pull/27)) on 2021-11-02
- The updater `pubkey` is now a required field for security reasons. Sign your updates with the `tauri signer` command.
  - [d95cc831](https://www.github.com/tauri-apps/tauri/commit/d95cc83105dda52df7514e30e54f3676cdb374ee) feat: enforce updater public key \[TRI-015] ([#42](https://www.github.com/tauri-apps/tauri/pull/42)) on 2022-01-09
- `tauri::api::HttpRequestBuilder::new` now returns a `Result` to validate the url.
  - [0ad1c651](https://www.github.com/tauri-apps/tauri/commit/0ad1c6515f696fadefddbf133a9561836b3d5934) feat(core): add `http` allowlist scope \[TRI-008] ([#24](https://www.github.com/tauri-apps/tauri/pull/24)) on 2021-10-29
- Added the `isolation` pattern.
  - [d5d6d2ab](https://www.github.com/tauri-apps/tauri/commit/d5d6d2abc17cd89c3a079d2ce01581193469dbc0) Isolation Pattern ([#43](https://www.github.com/tauri-apps/tauri/pull/43)) Co-authored-by: Ngo Iok Ui (Wu Yu Wei) <wusyong9104@gmail.com> Co-authored-by: Lucas Fernandes Nogueira <lucas@tauri.app> on 2022-01-17
- Added `abort` method to `tauri::async_runtime::JoinHandle`.
  - [ad169759](https://www.github.com/tauri-apps/tauri/commit/ad16975938afc9e87747de5fdcb0f07fc2d24811) feat: Add JoinHandle::abort() ([#2877](https://www.github.com/tauri-apps/tauri/pull/2877)) on 2021-11-13
- Adds support for using JSON5 format for the `tauri.conf.json` file, along with also supporting the `.json5` extension.

Here is the logic flow that determines if JSON or JSON5 will be used to parse the config:

1. Check if `tauri.conf.json` exists
   a. Parse it with `serde_json`
   b. Parse it with `json5` if `serde_json` fails
   c. Return original `serde_json` error if all above steps failed
2. Check if `tauri.conf.json5` exists
   a. Parse it with `json5`
   b. Return error if all above steps failed
3. Return error if all above steps failed

- [995de57a](https://www.github.com/tauri-apps/tauri/commit/995de57a76cf51215277673e526d7ec32b86b564) Add seamless support for using JSON5 in the config file ([#47](https://www.github.com/tauri-apps/tauri/pull/47)) on 2022-02-03
- Allow preventing opening the default browser on a click on an `<a target="_blank">` element via `stopImmediatePropagation()`.
  - [10e3190f](https://www.github.com/tauri-apps/tauri/commit/10e3190fdd38893a720a51b7bfd06744e0306626) fix(core): do not use capture on \_blank link event handler, closes [#2791](https://www.github.com/tauri-apps/tauri/pull/2791) ([#3349](https://www.github.com/tauri-apps/tauri/pull/3349)) on 2022-02-07
- The `run_return` API is now available on Linux.
  - [8483fde9](https://www.github.com/tauri-apps/tauri/commit/8483fde975aac8833d2ce426e42fb40aeaeecba9) feat(core): expose `run_return` on Linux ([#3352](https://www.github.com/tauri-apps/tauri/pull/3352)) on 2022-02-07
- Allow window, global shortcut and clipboard APIs to be called on the main thread.
  - [2812c446](https://www.github.com/tauri-apps/tauri/commit/2812c4464b93a365ab955935d05b5cea8cb03aab) feat(core): window, shortcut and clipboard API calls on main thread ([#2659](https://www.github.com/tauri-apps/tauri/pull/2659)) on 2021-09-26
  - [d24fd8d1](https://www.github.com/tauri-apps/tauri/commit/d24fd8d10242da3da143a971d976b42ec4de6079) feat(tauri-runtime-wry): allow window creation and closing on the main thread ([#2668](https://www.github.com/tauri-apps/tauri/pull/2668)) on 2021-09-27
- Add `Menu::with_items` constructor, taking an iterator of `MenuEntry`.
  - [7cc95e10](https://www.github.com/tauri-apps/tauri/commit/7cc95e10ec66d8b155e9bb7f89cf73df56d1f107) feat(core): add `Menu::with_items`, closes [#2807](https://www.github.com/tauri-apps/tauri/pull/2807) ([#2966](https://www.github.com/tauri-apps/tauri/pull/2966)) on 2021-12-27
- The updater now expects signatures created with the latest CLI release.
  - [c2a6e8d7](https://www.github.com/tauri-apps/tauri/commit/c2a6e8d7e64284080129f09b3a0ae6cf93a87df0) chore(deps) Update Tauri Core ([#2746](https://www.github.com/tauri-apps/tauri/pull/2746)) on 2021-10-13
- Change event loop callbacks definition to allow callers to move in mutable values.
  - [bdbf905e](https://www.github.com/tauri-apps/tauri/commit/bdbf905e5d802b58693d2bd27582ce4269faf79c) Transformed event-loop callback to FnMut to allow mutable values ([#2667](https://www.github.com/tauri-apps/tauri/pull/2667)) on 2021-09-27
- Fixes `Notification.requestPermission()` deadlock.
  - [48f3768c](https://www.github.com/tauri-apps/tauri/commit/48f3768c41a2c68e2e097fcc1ef50e549c1dfb78) fix(core): `Notification.requestPermission()` deadlock regression on 2021-08-24
- Added `Window#open_devtools` API.
  - [55aa22de](https://www.github.com/tauri-apps/tauri/commit/55aa22de80c3de873e29bcffcb5b2fe236a637a6) feat(core): add `Window#open_devtools` API, closes [#1213](https://www.github.com/tauri-apps/tauri/pull/1213) ([#3350](https://www.github.com/tauri-apps/tauri/pull/3350)) on 2022-02-07
- Add a `plugin::Builder` struct to make plugin creation more convenient.
  - [9aed2996](https://www.github.com/tauri-apps/tauri/commit/9aed299621763a6e0a35d102cdf012da522dac35) feat: `plugin::Builder` closes [#2959](https://www.github.com/tauri-apps/tauri/pull/2959) ([#3005](https://www.github.com/tauri-apps/tauri/pull/3005)) on 2022-02-07
  - [62c7a8ad](https://www.github.com/tauri-apps/tauri/commit/62c7a8ad30fd3031b8679960590e5ef3eef8e4da) chore(covector): prepare for `rc` release ([#3376](https://www.github.com/tauri-apps/tauri/pull/3376)) on 2022-02-10
- Added `on_event` on the `Plugin` trait, which allows a plugin to react to the event loop.
  - [cc2f39a2](https://www.github.com/tauri-apps/tauri/commit/cc2f39a29fc8852724aa3954ff0d42a36484929b) feat(core): add `on_event` hook on the `Plugin` trait ([#2656](https://www.github.com/tauri-apps/tauri/pull/2656)) on 2021-09-26
- Prevent path traversal on the file system APIs.
  - [4d89f60d](https://www.github.com/tauri-apps/tauri/commit/4d89f60d77a2abe7f3358cec00e15ecacf5e1148) refactor(core): prevent path traversal \[TRI-012] ([#35](https://www.github.com/tauri-apps/tauri/pull/35)) on 2021-12-06
- **Breaking change:** Add `macos-private-api` feature flag, enabled via `tauri.conf.json > tauri > macOSPrivateApi`.
  - [6ac21b3c](https://www.github.com/tauri-apps/tauri/commit/6ac21b3cef7f14358df38cc69ea3d277011accaf) feat: add private api feature flag ([#7](https://www.github.com/tauri-apps/tauri/pull/7)) on 2022-01-09
- Add `raw_headers` to `tauri::api::http::ResponseData`.
  - [b7a2345b](https://www.github.com/tauri-apps/tauri/commit/b7a2345b06ca0306988b4ba3d3deadd449e65af9) feat(core): add raw headers to HTTP API, closes [#2695](https://www.github.com/tauri-apps/tauri/pull/2695) ([#3053](https://www.github.com/tauri-apps/tauri/pull/3053)) on 2022-01-07
- Implement `raw_window_handle::RawWindowHandle` for `tauri::Window` on `Windows` and `macOS`. The `tauri::api::dialog::window_parent` function was removed since now you can use the window directly.
  - [e98c1af4](https://www.github.com/tauri-apps/tauri/commit/e98c1af44279a5ff6c8a6f0a506ecc219c9f77af) feat(core): expose message dialog APIs, fix window.confirm, implement HasRawWindowHandle for Window, closes [#2535](https://www.github.com/tauri-apps/tauri/pull/2535) ([#2700](https://www.github.com/tauri-apps/tauri/pull/2700)) on 2021-10-02
- Refactor `create_tao_window` API to return `Weak<Window>` instead of `Arc<Window>`.
  - [c1494b35](https://www.github.com/tauri-apps/tauri/commit/c1494b353233c6a9552d7ace962fdf8d5b1f199a) refactor: return Weak<Window> on create_tao_window on 2021-08-31
- Added the `tauri::api::dialog::blocking` module.
  - [4818531a](https://www.github.com/tauri-apps/tauri/commit/4818531aba47e126af91253d5d0eae3972b27d4c) refactor(core): add blocking dialog APIs, improve docs, closes [#3255](https://www.github.com/tauri-apps/tauri/pull/3255) ([#3270](https://www.github.com/tauri-apps/tauri/pull/3270)) on 2022-02-05
- The notification endpoint now checks for the permission flag and requests if the value is not set.
  - [239bba56](https://www.github.com/tauri-apps/tauri/commit/239bba56666c96acce24f30851f2d0719c95fc2c) refactor(core): check notification permission on the Rust endpoint \[TRI-017] ([#23](https://www.github.com/tauri-apps/tauri/pull/23)) on 2022-01-09
- **Breaking change:** The `WindowEvent::CloseRequested` variant now includes `label` and `signal_tx` fields to allow preventing closing the window.
  - [74dff536](https://www.github.com/tauri-apps/tauri/commit/74dff536d4757820e68ab05301be83c0870c22ed) fix(core): emit `tauri://close-requested` to JS, closes [#2996](https://www.github.com/tauri-apps/tauri/pull/2996) ([#3041](https://www.github.com/tauri-apps/tauri/pull/3041)) on 2021-12-09
- **Breaking change:** Move `__currentWindow` and `__windows` values from `window.__TAURI__` to `window.__TAURI_METADATA__`.
  - [f5109e0c](https://www.github.com/tauri-apps/tauri/commit/f5109e0c962e3d25404995194968bade1be33b16) fix(api): window label null instead of actual value, closes [#3295](https://www.github.com/tauri-apps/tauri/pull/3295) ([#3332](https://www.github.com/tauri-apps/tauri/pull/3332)) on 2022-02-04
- Remove the `BaseDirectory::Current` enum variant for security reasons.
  - [696dca58](https://www.github.com/tauri-apps/tauri/commit/696dca58a9f8ee127a1cf857eb848e09f5845d18) refactor(core): remove `BaseDirectory::Current` variant on 2022-01-26
- **Breaking change:** Remove default webview window when `tauri.conf.json > tauri > windows` is not set.
  - [c119060e](https://www.github.com/tauri-apps/tauri/commit/c119060e3d9a5a824639fb6b3c45a87e7a62e4e2) refactor(core): empty default value for config > tauri > windows ([#3380](https://www.github.com/tauri-apps/tauri/pull/3380)) on 2022-02-10
- **Breaking change:** Renamed the `rpc` module to `ipc`.
  - [3420aa50](https://www.github.com/tauri-apps/tauri/commit/3420aa5031b3274a95c6c5fa0f8683ca13213396) refactor: IPC handler \[TRI-019] ([#9](https://www.github.com/tauri-apps/tauri/pull/9)) on 2022-01-09
- Expose `run_on_main_thread` APIs on `Window` and `AppHandle`.
  - [53fdfe52](https://www.github.com/tauri-apps/tauri/commit/53fdfe52bb30d52653c72ca9f42506c3863dcf4a) feat(core): expose `run_on_main_thread` API ([#2711](https://www.github.com/tauri-apps/tauri/pull/2711)) on 2021-10-04
- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22
- The minimum Rust version is now 1.57.
  - [d5d6d2ab](https://www.github.com/tauri-apps/tauri/commit/d5d6d2abc17cd89c3a079d2ce01581193469dbc0) Isolation Pattern ([#43](https://www.github.com/tauri-apps/tauri/pull/43)) Co-authored-by: Ngo Iok Ui (Wu Yu Wei) <wusyong9104@gmail.com> Co-authored-by: Lucas Fernandes Nogueira <lucas@tauri.app> on 2022-01-17
- Scopes the `filesystem` APIs from the webview access using `tauri.conf.json > tauri > allowlist > fs > scope`.
  Scopes the `asset` protocol access using `tauri.conf.json > tauri > allowlist > protocol > assetScope`.
  Scopes the `http` APIs from the webview access using `tauri.conf.json > tauri > allowlist > http > scope`.
  Scopes the `shell` execute API from the webview access using `tauri.conf.json > tauri > allowlist > shell > scope`. Additionally, check the `tauri.conf.json > tauri > bundle > externalBin` to prevent access to unknown sidecars.
  - [7920ff14](https://www.github.com/tauri-apps/tauri/commit/7920ff14e6424079c48ea5645d9aa13e7a272b87) feat: scope the `fs` API and the `asset` protocol \[TRI-026] \[TRI-010] \[TRI-011] ([#10](https://www.github.com/tauri-apps/tauri/pull/10)) on 2022-01-09
  - [0ad1c651](https://www.github.com/tauri-apps/tauri/commit/0ad1c6515f696fadefddbf133a9561836b3d5934) feat(core): add `http` allowlist scope \[TRI-008] ([#24](https://www.github.com/tauri-apps/tauri/pull/24)) on 2021-10-29
  - [d4db95e7](https://www.github.com/tauri-apps/tauri/commit/d4db95e7161e064d9463bc84d871a5391c3ce2d7) feat(core): shell execute API scope \[TRI-002] ([#36](https://www.github.com/tauri-apps/tauri/pull/36)) on 2021-12-01
- `Builder#setup` closure type changed from `Fn` to `FnOnce`.
  - [3f3599b9](https://www.github.com/tauri-apps/tauri/commit/3f3599b9cc238da06b36aff4c120a013a97fc3f0) refactor(core): change `setup` closure type to `FnOnce`, closes [#3061](https://www.github.com/tauri-apps/tauri/pull/3061) ([#3065](https://www.github.com/tauri-apps/tauri/pull/3065)) on 2021-12-27
- The `tauri::api::shell::open`'s `with` argument is now an enum value instead of any string.
  - [63921fad](https://www.github.com/tauri-apps/tauri/commit/63921fada436f010c33bb5e647bd67c6e549571c) refactor: change `tauri::api::open` `with` argument to an enum \[TRI-022] ([#19](https://www.github.com/tauri-apps/tauri/pull/19)) on 2022-01-09
- The `shell` allowlist now includes a `sidecar` flag, which enables the use of the `shell` API to execute sidecars.
  - [eed01728](https://www.github.com/tauri-apps/tauri/commit/eed017287fed2ade689af4268e8b63b9c9f2e585) feat(core): add `shell > sidecar` allowlist and `process` feature flag \[TRI-037] ([#18](https://www.github.com/tauri-apps/tauri/pull/18)) on 2021-10-24
- **Breaking change:** The sidecar's target triple suffix is now removed at build time.
  - [3035e458](https://www.github.com/tauri-apps/tauri/commit/3035e4581c161ec7f0bd6d9b42e9015cf1dd1d77) Remove target triple from sidecar bin paths, closes [#3355](https://www.github.com/tauri-apps/tauri/pull/3355) ([#3356](https://www.github.com/tauri-apps/tauri/pull/3356)) on 2022-02-07
- Fix streaming of small files using the `asset` protocol.
  - [151e629e](https://www.github.com/tauri-apps/tauri/commit/151e629ebf15ec5c068eb623e3dbc0ecdef1f816) fix(core): streaming of small files using `asset://`, closes [#2854](https://www.github.com/tauri-apps/tauri/pull/2854) ([#3039](https://www.github.com/tauri-apps/tauri/pull/3039)) on 2021-12-09
- Add `set_menu` API on `tauri::SystemTrayHandle`.
  - [0e4d12b5](https://www.github.com/tauri-apps/tauri/commit/0e4d12b541652d98c5a73de65034db8214e0363c) fix: [#2502](https://www.github.com/tauri-apps/tauri/pull/2502) Expose `set_menu` from tao through the TrayHandle struct ([#2532](https://www.github.com/tauri-apps/tauri/pull/2532)) on 2021-10-02
- Adds `unlisten` function to the `Window` struct.
  - [3a59f5f7](https://www.github.com/tauri-apps/tauri/commit/3a59f5f79e1b3ea08a224e6f2e0a56668d96cda3) Unlisten to an event on this window ([#2664](https://www.github.com/tauri-apps/tauri/pull/2664)) on 2021-09-28
- Force updater endpoint URL to use `https` on release builds.
  - [c077f449](https://www.github.com/tauri-apps/tauri/commit/c077f449270cffbf7956b1af81e1fb237ebf564a) feat: force endpoint URL to use https on release \[TRI-015] ([#41](https://www.github.com/tauri-apps/tauri/pull/41)) on 2022-01-09
- Validate the `std::env::current_exe` return value if `APPDIR` or `APPIMAGE` environment variables are set.
  - [6fbd6dba](https://www.github.com/tauri-apps/tauri/commit/6fbd6dba5290dc017ab0ba5a44cf4358b022836f) feat(core): validate `AppImage` execution when env vars are set \[TRI-041] ([#17](https://www.github.com/tauri-apps/tauri/pull/17)) on 2021-10-24
- The event name is now validated. On a IPC message, it returns an error if it fails validation; on the Rust side, it panics.
  It must include only alphanumeric characters, `-`, `/`, `:` and `_`.
  - [a48b8b18](https://www.github.com/tauri-apps/tauri/commit/a48b8b18d428bcc404d489daa690bbefe1f57311) feat(core): validate callbacks and event names \[TRI-038] \[TRI-020] ([#21](https://www.github.com/tauri-apps/tauri/pull/21)) on 2022-01-09
- The window label is now validated and must be alphanumeric, resulting in a panic if it isn't.
  - [680554de](https://www.github.com/tauri-apps/tauri/commit/680554de3ef6b7fccf87c441ad355cfef7aab6fe) feat: validate window label \[TRI-021] ([#13](https://www.github.com/tauri-apps/tauri/pull/13)) on 2021-10-23
- Allow `tauri.conf.json > package > version` to specify a path to a `package.json` file and pull the version from it.
  - [46f2eae8](https://www.github.com/tauri-apps/tauri/commit/46f2eae8aad7c6a228eaf48480d5603dae6454b4) feat: allow config's version to be a path to package.json, closes [#2967](https://www.github.com/tauri-apps/tauri/pull/2967) ([#2971](https://www.github.com/tauri-apps/tauri/pull/2971)) on 2022-01-07
- Added `clipboard` field on the `WebviewAttributes` struct, which must be set to `true` to enable clipboard access on the webview.
  - [d42ccfb3](https://www.github.com/tauri-apps/tauri/commit/d42ccfb34f71851dfeb22fe74c83a8bdbddb5550) feat: add `clipboard` flag to `WebviewAttributes` \[TRI-032] ([#12](https://www.github.com/tauri-apps/tauri/pull/12)) on 2021-10-23
- Replace all of the `winapi` crate references with the `windows` crate, and replace `webview2` and `webview2-sys` with `webview2-com` and `webview2-com-sys` built with the `windows` crate. This goes along with updates to the TAO and WRY `next` branches.
  - [bb00d5bd](https://www.github.com/tauri-apps/tauri/commit/bb00d5bd6c9dfcb6bdd0d308dadb70e6c6aafe5c) Replace winapi with windows crate and use webview2-com instead of webview2 ([#2615](https://www.github.com/tauri-apps/tauri/pull/2615)) on 2021-09-24
- Show `Ok/Cancel` buttons instead of `Yes/No` when executing `window.confirm`.
  - [e98c1af4](https://www.github.com/tauri-apps/tauri/commit/e98c1af44279a5ff6c8a6f0a506ecc219c9f77af) feat(core): expose message dialog APIs, fix window.confirm, implement HasRawWindowHandle for Window, closes [#2535](https://www.github.com/tauri-apps/tauri/pull/2535) ([#2700](https://www.github.com/tauri-apps/tauri/pull/2700)) on 2021-10-02
- Update the `windows` crate to 0.25.0, which comes with pre-built libraries. WRY and Tao can both reference the same types directly from the `windows` crate instead of sharing bindings in `webview2-com-sys`.
  - [34be6cf3](https://www.github.com/tauri-apps/tauri/commit/34be6cf37a98ee7cbd66623ebddae08e5a6520fd) Update webview2-com and windows crates ([#2875](https://www.github.com/tauri-apps/tauri/pull/2875)) on 2021-11-11

## \[1.0.0-beta.8]

- Fix missing asset protocol path.Now the protocol is `https://asset.localhost/path/to/file` on Windows. Lunix and macOS
  is still `asset://path/to/file`.
  - [994b5325](https://www.github.com/tauri-apps/tauri/commit/994b5325dd385f564b37fe1530c5d798dc925fff) fix: missing asset protocol path ([#2484](https://www.github.com/tauri-apps/tauri/pull/2484)) on 2021-08-23
- **Breaking change:** Removed `register_uri_scheme_protocol` from the `WebviewAttributes` struct and renamed `register_global_uri_scheme_protocol` to `register_uri_scheme_protocol` on the `Builder` struct, which now takes a `Fn(&AppHandle, &http::Request) -> http::Response` closure.
  - [539e4489](https://www.github.com/tauri-apps/tauri/commit/539e4489e0bac7029d86917e9982ea49e02fe489) refactor: custom protocol ([#2503](https://www.github.com/tauri-apps/tauri/pull/2503)) on 2021-08-23
- Migrate to latest custom protocol allowing `Partial content` streaming and Header parsing.
  - [539e4489](https://www.github.com/tauri-apps/tauri/commit/539e4489e0bac7029d86917e9982ea49e02fe489) refactor: custom protocol ([#2503](https://www.github.com/tauri-apps/tauri/pull/2503)) on 2021-08-23

## \[1.0.0-beta.7]

- Cleanup application on `AppHandle#exit`.
  - [a54bba6c](https://www.github.com/tauri-apps/tauri/commit/a54bba6c868508844d68a9f4ea9f5519a2b94d09) fix(core): cleanup app before exit, closes [#2464](https://www.github.com/tauri-apps/tauri/pull/2464) ([#2466](https://www.github.com/tauri-apps/tauri/pull/2466)) on 2021-08-17
- Fix `raw-window-handle` dependency declaration.
  - [aecdfaf7](https://www.github.com/tauri-apps/tauri/commit/aecdfaf76fcf0d2820d6ce6eb7590ebe399bfa04) fix(core): `raw-window-handle` dependency, closes [#2460](https://www.github.com/tauri-apps/tauri/pull/2460) ([#2465](https://www.github.com/tauri-apps/tauri/pull/2465)) on 2021-08-17

## \[1.0.0-beta.6]

- **Breaking change:** The `tauri::async_runtime::spawn` function now returns `tauri::async_runtime::JoinHandle<T>`.
  - [9aeb04fa](https://www.github.com/tauri-apps/tauri/commit/9aeb04faf40989e8fd9d3dcac1d430a9e8bd23a9) feat(core): `async_runtime` `handle` API, `spawn` returns `JoinHandle` ([#2399](https://www.github.com/tauri-apps/tauri/pull/2399)) on 2021-08-11

- **Breaking change:** Added `window_parent: Option<&Window>` as first argument to the `ask` and `message` APIs on the `tauri::api::dialog` module.
  - [c76f4b7d](https://www.github.com/tauri-apps/tauri/commit/c76f4b7d39a620c7710c2046bb13b140a4793881) feat(core): set parent window on ask and message dialog APIs ([#2454](https://www.github.com/tauri-apps/tauri/pull/2454)) on 2021-08-16

- Allow the `tauri::api::dialog` APIs to be executed on any secondary thread.
  **Breaking change:** All dialog APIs now takes a closure instead of returning the response on the function call.
  - [2088cd0f](https://www.github.com/tauri-apps/tauri/commit/2088cd0f24cd56ba427241136138c74bebee28f2) refactor(core): handle dialog threading internally, closes [#2223](https://www.github.com/tauri-apps/tauri/pull/2223) ([#2429](https://www.github.com/tauri-apps/tauri/pull/2429)) on 2021-08-14
  - [60b1e260](https://www.github.com/tauri-apps/tauri/commit/60b1e260f511f50bbebceb6367f412c11f8dcf11) chore: adjust change file on 2021-08-16

- **Breaking change:** The `Plugin` trait `initialize` method now takes an `AppHandle` reference instead of `App`.
  - [c17532f7](https://www.github.com/tauri-apps/tauri/commit/c17532f7412bdcc57ae850c1251052ad1421fd67) refactor(core): change Plugin `initialize` signature, move register t… ([#2347](https://www.github.com/tauri-apps/tauri/pull/2347)) on 2021-08-03

- **Breaking change:** Remove menu feature flag since there's no package dependency need to be installed on any platform anymore.
  - [f81ebddf](https://www.github.com/tauri-apps/tauri/commit/f81ebddfcc1aea0d4989706aef43538e8ea98bea) feat: remove menu feature flag ([#2415](https://www.github.com/tauri-apps/tauri/pull/2415)) on 2021-08-13

- Adds `set_activation_policy` API to the `tauri::App` struct (macOS only).
  - [4a031add](https://www.github.com/tauri-apps/tauri/commit/4a031add69014a1f3823f4ea19b172a2557f6794) feat(core): expose `set_activation_policy`, closes [#2258](https://www.github.com/tauri-apps/tauri/pull/2258) ([#2420](https://www.github.com/tauri-apps/tauri/pull/2420)) on 2021-08-13

- Add `handle` API to `tauri::async_runtime`.
  - [9aeb04fa](https://www.github.com/tauri-apps/tauri/commit/9aeb04faf40989e8fd9d3dcac1d430a9e8bd23a9) feat(core): `async_runtime` `handle` API, `spawn` returns `JoinHandle` ([#2399](https://www.github.com/tauri-apps/tauri/pull/2399)) on 2021-08-11

- Assets will now fallback to `<uri>/index.html` before `/index.html`, allowing anchor links to work as expected.
  - [d22da650](https://www.github.com/tauri-apps/tauri/commit/d22da650ef5b51ab7dd0e45dd9527a9c5a01f84d) fix(core): fallback to `{asset}/index.html` before `index.html`, closes [#2328](https://www.github.com/tauri-apps/tauri/pull/2328) ([#2329](https://www.github.com/tauri-apps/tauri/pull/2329)) on 2021-08-02

- Fix `data-tauri-drag-region` double-click, will now respect `resizable: false` and won't maximize.
  - [1a510066](https://www.github.com/tauri-apps/tauri/commit/1a510066732d5f61c88c0ceed1c5f5cc559faf7d) fix(core): `data-tauri-drag-region` didn't respect resizable, closes [#2314](https://www.github.com/tauri-apps/tauri/pull/2314) ([#2316](https://www.github.com/tauri-apps/tauri/pull/2316)) on 2021-08-02

- Fix `Notification.requestPermission()` throwing `Unhandled Promise Rejection: TypeError: undefined is not a function (near '...window.__TAURI__.invoke...')`
  - [cf9f6aa1](https://www.github.com/tauri-apps/tauri/commit/cf9f6aa148db31c6fd4e3571b301db18654a1249) fix(core): fix typo in notifications, closes [#2330](https://www.github.com/tauri-apps/tauri/pull/2330) ([#2331](https://www.github.com/tauri-apps/tauri/pull/2331)) on 2021-08-02

- Fix blur/focus events being incorrect on Windows.
  - [d832d575](https://www.github.com/tauri-apps/tauri/commit/d832d575d9b03a0ff78accabe4631cc638c08c3b) fix(windows): use webview events on windows ([#2277](https://www.github.com/tauri-apps/tauri/pull/2277)) on 2021-07-23

- Move items which `tauri::api` re-exports from `tauri-utils` to individual module `utils`. Because these items has their
  own Error/Result types which are not related to api module at all.
  - [12642a1a](https://www.github.com/tauri-apps/tauri/commit/12642a1ad42b23d8d4e31ae67cb8de287ba526c0) doc: update doc in tauri-utils and tauri ([#2435](https://www.github.com/tauri-apps/tauri/pull/2435)) on 2021-08-15
  - [cd55d671](https://www.github.com/tauri-apps/tauri/commit/cd55d671498d48cf83ae603c4241031b9827119b) doc: update tauri documentations ([#2446](https://www.github.com/tauri-apps/tauri/pull/2446)) on 2021-08-16

- Allow registering a plugin through an `AppHandle` instance using the `plugin` method.
  - [5b7be813](https://www.github.com/tauri-apps/tauri/commit/5b7be8133a6a861128ad53d1202d633f7ea8c2d2) feat(core): add plugin register API on the `Manager` trait ([#2340](https://www.github.com/tauri-apps/tauri/pull/2340)) on 2021-08-02
  - [c17532f7](https://www.github.com/tauri-apps/tauri/commit/c17532f7412bdcc57ae850c1251052ad1421fd67) refactor(core): change Plugin `initialize` signature, move register t… ([#2347](https://www.github.com/tauri-apps/tauri/pull/2347)) on 2021-08-03

- Embed Info.plist file contents on binary on dev.
  - [537ab1b6](https://www.github.com/tauri-apps/tauri/commit/537ab1b6d5a792c550a535619965c9e4126292e6) feat(core): inject src-tauri/Info.plist file on dev and merge on bundle, closes [#1570](https://www.github.com/tauri-apps/tauri/pull/1570) [#2338](https://www.github.com/tauri-apps/tauri/pull/2338) ([#2444](https://www.github.com/tauri-apps/tauri/pull/2444)) on 2021-08-15

- Add `ExitRequested` event that allows preventing the app from exiting when all windows are closed, and an `AppHandle.exit()` function to exit the app manually.
  - [892c63a0](https://www.github.com/tauri-apps/tauri/commit/892c63a0538f8d62680dce5848657128ad6b7af3) feat([#2287](https://www.github.com/tauri-apps/tauri/pull/2287)): Add `ExitRequested` event to let users prevent app from exiting ([#2293](https://www.github.com/tauri-apps/tauri/pull/2293)) on 2021-08-09

- Change `App.create_window()` and `AppHandle.create_window()` to accept an `Into<String>` type instead of `String`.
  - [8216cba1](https://www.github.com/tauri-apps/tauri/commit/8216cba13da10b272aae8e5058b1ec2cc937e196) `App.create_window()` to accept any `Into<String>` type (fix [#2290](https://www.github.com/tauri-apps/tauri/pull/2290)) ([#2291](https://www.github.com/tauri-apps/tauri/pull/2291)) on 2021-07-26

- Fixes `defaultPath` option on dialog API not setting the file name if it doesn't exist on Linux.
  - [8b2cc261](https://www.github.com/tauri-apps/tauri/commit/8b2cc2615d784100980807a520d16681819d83c7) fix(core): dialog's `defaultPath` behavior on Linux, closes [#2232](https://www.github.com/tauri-apps/tauri/pull/2232) ([#2382](https://www.github.com/tauri-apps/tauri/pull/2382)) on 2021-08-10

- Fix ES Module detection for default imports with relative paths or scoped packages and exporting of async functions.
  - [b2b36cfe](https://www.github.com/tauri-apps/tauri/commit/b2b36cfe8dfcccb341638a4cb6dc23a514c54148) fix(core): fixes ES Module detection for default imports with relative paths or scoped packages ([#2380](https://www.github.com/tauri-apps/tauri/pull/2380)) on 2021-08-10
  - [fbf8caf5](https://www.github.com/tauri-apps/tauri/commit/fbf8caf5c419cb4fc3d123be910e094a8e8c4bef) fix(core): ESM detection when using `export async function` ([#2425](https://www.github.com/tauri-apps/tauri/pull/2425)) on 2021-08-14

- Fix `listen` calls receiving past events.
  - [1ecb8651](https://www.github.com/tauri-apps/tauri/commit/1ecb8651a795aa53eded31e3fb357c857dcf2ab1) fix(core): `listen` receiving past events, closes [#2323](https://www.github.com/tauri-apps/tauri/pull/2323) ([#2371](https://www.github.com/tauri-apps/tauri/pull/2371)) on 2021-08-09

- Fixes file drop events being swapped (`file-drop-hover` on drop and `file-drop` on hover).
  - [c2b0fe1c](https://www.github.com/tauri-apps/tauri/commit/c2b0fe1ce58e54dbcfdb63162ad17d7e6d8774d9) fix(core): fix wrong file drop events ([#2300](https://www.github.com/tauri-apps/tauri/pull/2300)) on 2021-07-31

- Fixes `app.listen_global` not receiving events emitted in javascript.
  - [a8c1de55](https://www.github.com/tauri-apps/tauri/commit/a8c1de5547cd3968aa238a06c410cb936391db5d) fix listen_global not listening to events with a window label ([#2272](https://www.github.com/tauri-apps/tauri/pull/2272)) on 2021-07-23

- Fixes minimum window height being used as maximum height.
  - [e3f99165](https://www.github.com/tauri-apps/tauri/commit/e3f9916526b226866137cb663e5cafab2b6a0e01) fix(core) minHeight being used as maxHeight ([#2247](https://www.github.com/tauri-apps/tauri/pull/2247)) on 2021-07-19

- Fixes `unlisten` calls from JavaScript removing every registered event listener.
  - [aa498e72](https://www.github.com/tauri-apps/tauri/commit/aa498e72614f59afcdd1f637b4e3bdf6fe00b137) fix: unlisten removes all listeners, closes [#2264](https://www.github.com/tauri-apps/tauri/pull/2264) ([#2302](https://www.github.com/tauri-apps/tauri/pull/2302)) on 2021-07-29

- Use [`Url.join()`](https://docs.rs/url/2.2.2/url/struct.Url.html#method.join) when building webview URLs in
  `WindowManager`, to handle edge cases and leading/trailing slashes in paths and urls.
  - [31685c9f](https://www.github.com/tauri-apps/tauri/commit/31685c9f9d9c2f12d3debc3189ed67e8d669f3d8) fix([#2281](https://www.github.com/tauri-apps/tauri/pull/2281)): Prevent double slashes when joining URLs ([#2282](https://www.github.com/tauri-apps/tauri/pull/2282)) on 2021-07-23

- Fixes `fs-all` feature not requiring the `base64` crate.
  - [9b32b939](https://www.github.com/tauri-apps/tauri/commit/9b32b93996201190d83fcad4767709774e5a0780) fix(core): `fs-all` feature not including `base64` crate, closes [#2336](https://www.github.com/tauri-apps/tauri/pull/2336) ([#2368](https://www.github.com/tauri-apps/tauri/pull/2368)) on 2021-08-08

- Update gtk and its related libraries to v0.14. This also remove requirements of `clang` as build dependency.
  - [63ad3039](https://www.github.com/tauri-apps/tauri/commit/63ad303903bbee7c9a7382413b342e2a05d3ea75) chore(linux): bump gtk to v0.14 ([#2361](https://www.github.com/tauri-apps/tauri/pull/2361)) on 2021-08-07

- Use `HeaderValue::from_bytes` instead of `HeaderValue::from_str` and `HeaderValue#to_bytes` instead of `HeaderValue#to_str` to improve compatibility.
  - [1635798a](https://www.github.com/tauri-apps/tauri/commit/1635798a669ced825577941dc66f1334c5904b43) feat(core): improve HeaderValue compatibility, closes [#2162](https://www.github.com/tauri-apps/tauri/pull/2162) ([#2438](https://www.github.com/tauri-apps/tauri/pull/2438)) on 2021-08-15

- Implement `Debug` on public API structs and enums.
  - [fa9341ba](https://www.github.com/tauri-apps/tauri/commit/fa9341ba18ba227735341530900714dba0f27291) feat(core): implement `Debug` on public API structs/enums, closes [#2292](https://www.github.com/tauri-apps/tauri/pull/2292) ([#2387](https://www.github.com/tauri-apps/tauri/pull/2387)) on 2021-08-11

- Adds `Resumed` and `MainEventsCleared` variants to the `Event` enum.
  - [6be3f433](https://www.github.com/tauri-apps/tauri/commit/6be3f4339168651fe4e003b09f7d181fd12cd5a8) feat(core): add `Resumed` and `MainEventsCleared` events, closes [#2127](https://www.github.com/tauri-apps/tauri/pull/2127) ([#2439](https://www.github.com/tauri-apps/tauri/pull/2439)) on 2021-08-15

- Panic when a dispatcher getter method (`Window`, `GlobalShortcutHandle`, `ClipboardManager` and `MenuHandle` APIs) is called on the main thread.
  - [50ffdc06](https://www.github.com/tauri-apps/tauri/commit/50ffdc06fbde56aba32b4291fd130104935d1408) feat(core): panic when a dispatcher getter is used on the main thread ([#2455](https://www.github.com/tauri-apps/tauri/pull/2455)) on 2021-08-16

- Use `percent_encoding::percent_decode` on the `asset` custom protocol URL before reading the file.
  - [9acd8301](https://www.github.com/tauri-apps/tauri/commit/9acd83017f25b5b04e97e6e98af80f3076f8dbe3) fix(core): percent decode asset protocol URL ([#2427](https://www.github.com/tauri-apps/tauri/pull/2427)) on 2021-08-14

- Keep original value on `config > package > productName` on Linux (previously converted to kebab-case).
  - [3f039cb8](https://www.github.com/tauri-apps/tauri/commit/3f039cb8a308b0f18deaa37d7cfb1cc50d308d0e) fix: keep original `productName` for .desktop `Name` field, closes [#2295](https://www.github.com/tauri-apps/tauri/pull/2295) ([#2384](https://www.github.com/tauri-apps/tauri/pull/2384)) on 2021-08-10

- Inject the invoke key on regular `<script></script>` tags.
  - [d0142e87](https://www.github.com/tauri-apps/tauri/commit/d0142e87ddf5231fd46e2cbe4769bb16f3fe01e9) fix(core): invoke key injection on regular JS scripts, closes [#2342](https://www.github.com/tauri-apps/tauri/pull/2342) ([#2344](https://www.github.com/tauri-apps/tauri/pull/2344)) on 2021-08-03

- Remove salt-related APIs (no longer needed after the `__TAURI_INVOKE_KEY__` implementation).
  - [e2a0704c](https://www.github.com/tauri-apps/tauri/commit/e2a0704c6c7a447b628a95f8920f9bbe9feef229) refactor(core): remove salt APIs ([#2426](https://www.github.com/tauri-apps/tauri/pull/2426)) on 2021-08-14

- Update minimum Rust version to 1.54.0.
  - [a5394716](https://www.github.com/tauri-apps/tauri/commit/a53947160985a4f5b0ad1fbb4aa6865d6f852c66) chore: update rust to 1.54.0 ([#2434](https://www.github.com/tauri-apps/tauri/pull/2434)) on 2021-08-15

- Run the setup callback after preparing the system tray.
  - [1792c455](https://www.github.com/tauri-apps/tauri/commit/1792c45592cd4999af063fa89017f52a985553c1) fix(core): run setup after preparing system tray ([#2312](https://www.github.com/tauri-apps/tauri/pull/2312)) on 2021-07-28

- Fixes a consistency issue on the order of `tauri::process::Command` emitted events.
  - [737da872](https://www.github.com/tauri-apps/tauri/commit/737da87244cbdeb1158c93944bcb5e10bb383b31) fix(core): random shell command output order, closes [#2184](https://www.github.com/tauri-apps/tauri/pull/2184) ([#2376](https://www.github.com/tauri-apps/tauri/pull/2376)) on 2021-08-09

- Force data directory even on non-local window.
  - [70a19414](https://www.github.com/tauri-apps/tauri/commit/70a1941468f55f0dc09ac2e13802945891d766f4) fix(core): Force data_directory on Windows ([#2288](https://www.github.com/tauri-apps/tauri/pull/2288)) on 2021-07-23

- Allow creation of empty Window with `create_tao_window()` and management with `send_tao_window_event()` on the AppHandler.
  - [88080855](https://www.github.com/tauri-apps/tauri/commit/8808085541a629b8e22b612a06cef01cf9b3722e) feat(window): Allow creation of Window without `wry` ([#2321](https://www.github.com/tauri-apps/tauri/pull/2321)) on 2021-07-29
  - [15566cfd](https://www.github.com/tauri-apps/tauri/commit/15566cfd64f5072fa4980a6ce5b33259958e9021) feat(core): add API to send wry window message to the event loop ([#2339](https://www.github.com/tauri-apps/tauri/pull/2339)) on 2021-08-02

- Make `ClipboardManager` and `GlobalShortcutManager` public as they are exposed in the `AppHandle`.
  - [6e0dbf63](https://www.github.com/tauri-apps/tauri/commit/6e0dbf639ac2c79e00fee9270a2ca8e613dc1f98) fix(core): Expose `ClipboardManager` and `GlobalShortcutManager` ([#2263](https://www.github.com/tauri-apps/tauri/pull/2263)) on 2021-08-03

- - Support [macOS tray icon template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) to adjust automatically based on taskbar color.

- Images you mark as template images should consist of only black and clear colors. You can use the alpha channel in the image to adjust the opacity of black content, however.

- [426a6b49](https://www.github.com/tauri-apps/tauri/commit/426a6b49962de8faf061db2e820ac10fcbb300d6) feat(macOS): Implement tray icon template ([#2322](https://www.github.com/tauri-apps/tauri/pull/2322)) on 2021-07-29

- Add `Event::Ready` on the `run()` callback. Triggered once when the event loop is ready.
  - [28c6b7ad](https://www.github.com/tauri-apps/tauri/commit/28c6b7adfe98e701b158e936eafb7541ddc700e0) feat: add `Event::Ready` ([#2433](https://www.github.com/tauri-apps/tauri/pull/2433)) on 2021-08-15

- - Do not run the updater with UAC task if server don't tell us. (Allow toggling server-side)

- The updater expect a field named `with_elevated_task` with a `boolean` and will not run if the task is not installed first. (windows only)

- [c5761190](https://www.github.com/tauri-apps/tauri/commit/c576119013297f3731d76924a887c5c2a62c13ba) fix(updater): Run elevated task only if server tell us ([#2357](https://www.github.com/tauri-apps/tauri/pull/2357)) on 2021-08-08

- Add `try_state` API to the `Manager` trait.
  - [84a0e04c](https://www.github.com/tauri-apps/tauri/commit/84a0e04cbe242b2b7abb388da2d878fce10bc27d) feat(core): `try_state` API on the `Manager` trait ([#2341](https://www.github.com/tauri-apps/tauri/pull/2341)) on 2021-08-02

## \[1.0.0-beta.5]

- Allow preventing window close when the user requests it.
  - [8157a68a](https://www.github.com/tauri-apps/tauri/commit/8157a68af1d94de1b90a14aa44139bb123b3436b) feat(core): allow listening to event loop events & prevent window close ([#2131](https://www.github.com/tauri-apps/tauri/pull/2131)) on 2021-07-06
- Add `App#run` method with callback argument (event loop event handler).
  - [8157a68a](https://www.github.com/tauri-apps/tauri/commit/8157a68af1d94de1b90a14aa44139bb123b3436b) feat(core): allow listening to event loop events & prevent window close ([#2131](https://www.github.com/tauri-apps/tauri/pull/2131)) on 2021-07-06
- Fixes `data-tauri-drag-region` not firing its events.
  - [578610a2](https://www.github.com/tauri-apps/tauri/commit/578610a29d5cefb8df070606b7587318b14c397a) fix(core): fix drag-region not sending its events correctly ([#2196](https://www.github.com/tauri-apps/tauri/pull/2196)) on 2021-07-12
- Fix macOS `EXC_BAD_ACCESS` panic when app is code-signed.
  - [456a94f6](https://www.github.com/tauri-apps/tauri/commit/456a94f6637746800b9b85fc3922e82871603402) fix(macOS): updater `EXC_BAD_ACCESS` ([#2181](https://www.github.com/tauri-apps/tauri/pull/2181)) on 2021-07-12
- Fixes SVG loading on custom protocol.
  - [e663bdd5](https://www.github.com/tauri-apps/tauri/commit/e663bdd5938830ab4eba961e69c3985191b499dd) fix(core): svg mime type ([#2129](https://www.github.com/tauri-apps/tauri/pull/2129)) on 2021-06-30
- Expose `gtk_window` getter.
  - [e0a8e09c](https://www.github.com/tauri-apps/tauri/commit/e0a8e09cab6799eeb9ec524b5f7780d1e5a84299) feat(core): expose `gtk_window`, closes [#2083](https://www.github.com/tauri-apps/tauri/pull/2083) ([#2141](https://www.github.com/tauri-apps/tauri/pull/2141)) on 2021-07-02
- Inject invoke key on `script` tags with `type="module"`.
  - [f03eea9c](https://www.github.com/tauri-apps/tauri/commit/f03eea9c9b964709532afbc4d1dd343b3fd96081) feat(core): inject invoke key on `<script type="module">` ([#2120](https://www.github.com/tauri-apps/tauri/pull/2120)) on 2021-06-29
- Fix macOS high CPU usage.
  - [a280ee90](https://www.github.com/tauri-apps/tauri/commit/a280ee90af0749ce18d6d0b00939b06473717bc9) Fix high cpu usage on mac, fix [#2074](https://www.github.com/tauri-apps/tauri/pull/2074) ([#2125](https://www.github.com/tauri-apps/tauri/pull/2125)) on 2021-06-30
- Export `MenuHandle` and `MenuEvent` types on `tauri::window`.
  - [acb88929](https://www.github.com/tauri-apps/tauri/commit/acb88929c4dd1bd2ac94f7896a1e54afb9417768) fix(core): export `MenuHandle` and `MenuEvent` ([#2148](https://www.github.com/tauri-apps/tauri/pull/2148)) on 2021-07-03
- Use glib context for linux updater to prevent GTK panic.
  - [3389bd81](https://www.github.com/tauri-apps/tauri/commit/3389bd8180ed79d0f6778f7f41a88ad6d4598932) fix(linux): use glib main context for the updater on linux ([#2222](https://www.github.com/tauri-apps/tauri/pull/2222)) on 2021-07-16
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

## \[1.0.0-beta.4]

- Double clicking a `data-tauri-drag-region` element will toggle the window maximized state.
  - [8b7ac1ad](https://www.github.com/tauri-apps/tauri/commit/8b7ac1ad1432db1fb1b85b3f72d336b303414554) feat: double-click tauri-drag-region to maximize, closes [#1839](https://www.github.com/tauri-apps/tauri/pull/1839) ([#2106](https://www.github.com/tauri-apps/tauri/pull/2106)) on 2021-06-29
- Fixes `asset` protocol crashing application.
  - [99d96084](https://www.github.com/tauri-apps/tauri/commit/99d960841c4411c3805219d07640185b1d04c37a) fix(core): custom protocol regression ([#2115](https://www.github.com/tauri-apps/tauri/pull/2115)) on 2021-06-28

## \[1.0.0-beta.3]

- Fixes `api::process::Command` events not firing consistently.
  - [8c13344f](https://www.github.com/tauri-apps/tauri/commit/8c13344f8f97bc67b8fcde68ce14da438f7c66ba) fix(core): command events not firing consistently ([#2082](https://www.github.com/tauri-apps/tauri/pull/2082)) on 2021-06-27
- Detect ESM scripts and inject the invoke key directly instead of using an IIFE.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27
- Improve invoke key code injection performance time rewriting code at compile time.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27
- Enforce uniqueness of window label.
  - [d18b5367](https://www.github.com/tauri-apps/tauri/commit/d18b5367a91fd53d408510b456897630c70abcca) feat(core): enforce label uniqueness, closes [#2067](https://www.github.com/tauri-apps/tauri/pull/2067) ([#2097](https://www.github.com/tauri-apps/tauri/pull/2097)) on 2021-06-27
- `Window` is now `Send + Sync` on Windows.
  - [fe32afcc](https://www.github.com/tauri-apps/tauri/commit/fe32afcc933920d6282ae1d63b041b182278a031) fix(core): `Window` must be `Send + Sync` on Windows, closes [#2078](https://www.github.com/tauri-apps/tauri/pull/2078) ([#2093](https://www.github.com/tauri-apps/tauri/pull/2093)) on 2021-06-27

## \[1.0.0-beta.2]

- Remove anonymous lifetimes on examples.
  - [c1f8e113](https://www.github.com/tauri-apps/tauri/commit/c1f8e11342941b846479d73bf50cb0a328ac3574) chore: remove unnecessary anonymous lifetimes ([#1829](https://www.github.com/tauri-apps/tauri/pull/1829)) on 2021-05-14

- Moves `shell`, `dialog::FileDialogBuilder` and `process::Command` APIs behind their allowlist feature flags.
  - [aab3e1f1](https://www.github.com/tauri-apps/tauri/commit/aab3e1f18b74efd63fdc227e84382ee6e9e63817) refactor(core): move api modules behind allowlist feature flags ([#1864](https://www.github.com/tauri-apps/tauri/pull/1864)) on 2021-05-19

- Adds `create_window` API to the `AppHandle` struct.
  - [95d518af](https://www.github.com/tauri-apps/tauri/commit/95d518afa14fe613302d7091622b4b115543e1f2) feat(core): expose `AppHandle`, add `create_window` API ([#1855](https://www.github.com/tauri-apps/tauri/pull/1855)) on 2021-05-18

- Adds a `handle` function to the `App` struct, which returns a `Send` handle to the app instance.
  - [95d518af](https://www.github.com/tauri-apps/tauri/commit/95d518afa14fe613302d7091622b4b115543e1f2) feat(core): expose `AppHandle`, add `create_window` API ([#1855](https://www.github.com/tauri-apps/tauri/pull/1855)) on 2021-05-18

- Use `attohttpc` on the HTTP API by default for bundle size optimization. `reqwest` is implemented behind the `reqwest-client` feature flag.
  - [17c7c439](https://www.github.com/tauri-apps/tauri/commit/17c7c4396ff2d5e13fc8726c2965b4e810fad6b9) refactor(core): use `attohttpc` by default ([#1861](https://www.github.com/tauri-apps/tauri/pull/1861)) on 2021-05-19

- Kill child processes spawned with `tauri::api::process::Command` on `tauri::App` drop. Can be skipped with `tauri::Builder#skip_cleanup_on_drop`.
  - [4bdc4066](https://www.github.com/tauri-apps/tauri/commit/4bdc406679363f460e39079cb26319c39ab8cac8) feat(core): kill sidecar child processes on App drop, closes [#1896](https://www.github.com/tauri-apps/tauri/pull/1896) ([#1932](https://www.github.com/tauri-apps/tauri/pull/1932)) on 2021-06-01

- Adds `clipboard` APIs (write and read text).
  - [285bf64b](https://www.github.com/tauri-apps/tauri/commit/285bf64bf9569efb2df904c69c6df405ff0d62e2) feat(core): add clipboard writeText and readText APIs ([#2035](https://www.github.com/tauri-apps/tauri/pull/2035)) on 2021-06-21
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23

- Allow accessing an `AppHandle` instance on a command through dependency injection.
  - [59784c7e](https://www.github.com/tauri-apps/tauri/commit/59784c7e5587ff10d7f0c37ac414499da42d0707) feat(core): implement `CommandArg` for `AppHandle` ([#2037](https://www.github.com/tauri-apps/tauri/pull/2037)) on 2021-06-21

- Fixes child processes messages not arriving until the subprocess is terminated.
  - [df21ffc6](https://www.github.com/tauri-apps/tauri/commit/df21ffc61f42ffb4dfde50c74a01c07a59a76e3e) fix(core): command mpsc usage, closes [#1935](https://www.github.com/tauri-apps/tauri/pull/1935) ([#1936](https://www.github.com/tauri-apps/tauri/pull/1936)) on 2021-06-01

- Adds `config` and `package_info` getters to the `App` and `AppHandle` structs.
  - [70fc87a7](https://www.github.com/tauri-apps/tauri/commit/70fc87a7ff5a149c0cb22c13afdf90f25ddafde8) feat(core): add `config` and `package_info` getters on App and AppHandle ([#2016](https://www.github.com/tauri-apps/tauri/pull/2016)) on 2021-06-20

- Expose mutable getters for the rest of the public `Context` getters.

- `pub fn assets_mut(&mut self) -> &mut Arc<A>`

- `pub fn default_window_icon_mut(&mut self) -> &mut Option<Vec<u8>>`

- `pub fn system_tray_icon_mut(&mut self) -> &mut Option<Icon>`

- `pub fn package_info_mut(&mut self) -> &mut tauri::api::PackageInfo`

- [754c2e76](https://www.github.com/tauri-apps/tauri/commit/754c2e766abe25da053236ce311318aa1410d106) feat(core): finish mutable getters for `Context` ([#1814](https://www.github.com/tauri-apps/tauri/pull/1814)) on 2021-05-13

- Adds `request_user_attention` API to the `Window` struct.
  - [7dcca6e9](https://www.github.com/tauri-apps/tauri/commit/7dcca6e9281182b11ad3d4a79871f09b30b9b419) feat(core): add `request_user_attention` API, closes [#2023](https://www.github.com/tauri-apps/tauri/pull/2023) ([#2026](https://www.github.com/tauri-apps/tauri/pull/2026)) on 2021-06-20

- Adds `show`, `hide`, `is_visible` and `toggle` APIs to the `MenuHandle`.
  - [954460c5](https://www.github.com/tauri-apps/tauri/commit/954460c5205d57444ef4b1412051fbedf3e38676) feat(core): MenuHandle `show`, `hide`, `is_visible` and `toggle` APIs ([#1958](https://www.github.com/tauri-apps/tauri/pull/1958)) on 2021-06-15

- Allow `dev_path` and `dist_dir` to be an array of root files and directories to embed.
  - [6ec54c53](https://www.github.com/tauri-apps/tauri/commit/6ec54c53b504eec3873d326b1a45e450227d46ed) feat(core): allow `dev_path`, `dist_dir` as array of paths, fixes [#1897](https://www.github.com/tauri-apps/tauri/pull/1897) ([#1926](https://www.github.com/tauri-apps/tauri/pull/1926)) on 2021-05-31

- Validate `tauri.conf.json > build > devPath` and `tauri.conf.json > build > distDir` values.
  - [e97846aa](https://www.github.com/tauri-apps/tauri/commit/e97846aae933cad5cba284a2a133ae7aaee1107c) feat(core): validate `devPath` and `distDir` values ([#1848](https://www.github.com/tauri-apps/tauri/pull/1848)) on 2021-05-17

- Set the Tauri window as parent for dialogs.
  - [abf78c58](https://www.github.com/tauri-apps/tauri/commit/abf78c5860cdc52fbfd2bc5dbca29a864e2da8f9) fix(core): set parent window handle on dialogs, closes [#1876](https://www.github.com/tauri-apps/tauri/pull/1876) ([#1889](https://www.github.com/tauri-apps/tauri/pull/1889)) on 2021-05-21

- Fallback to `index.html` on asset loading so router with history mode works.
  - [8a7921e5](https://www.github.com/tauri-apps/tauri/commit/8a7921e5cbcbecea16d4ad4a29ef05d984ad3110) fix(core): fallback to index.html on asset loading, closes [#2020](https://www.github.com/tauri-apps/tauri/pull/2020) [#2021](https://www.github.com/tauri-apps/tauri/pull/2021) ([#2022](https://www.github.com/tauri-apps/tauri/pull/2022)) on 2021-06-20

- Fixes custom protocol asset loader not decoding the percent-encoded path.
  - [c021968e](https://www.github.com/tauri-apps/tauri/commit/c021968eb81f541599a3ce08eebb44e92604e39a) fix(core): asset loading not decoding percent-encoded path, closes [#1879](https://www.github.com/tauri-apps/tauri/pull/1879) ([#1938](https://www.github.com/tauri-apps/tauri/pull/1938)) on 2021-06-01

- As some frameworks automatically add "true" as the value of the attribute, we need to check if it exists instead.
  - [23707764](https://www.github.com/tauri-apps/tauri/commit/23707764dc54037e9aaba3023f2509459e3a9a8d) Drag region attribute check ([#1907](https://www.github.com/tauri-apps/tauri/pull/1907)) on 2021-05-30

- Fixes build without the dialog Cargo features.
  - [49fb3b72](https://www.github.com/tauri-apps/tauri/commit/49fb3b72255993ea6f92e8d475021adac0c914a7) fix(core): build without dialog Cargo features ([#1973](https://www.github.com/tauri-apps/tauri/pull/1973)) on 2021-06-09

- Allow disabling the webview file drop handler (required to use drag and drop on the frontend on Windows) using the `tauri.conf.json > tauri > windows > fileDropEnabled` flag or the `WebviewAttributes#disable_file_drop_handler` method.
  - [9cd10df4](https://www.github.com/tauri-apps/tauri/commit/9cd10df4d520de12f3b13fe88cc1c1a1b4bd48bf) feat(core): allow disabling file drop handler, closes [#2014](https://www.github.com/tauri-apps/tauri/pull/2014) ([#2030](https://www.github.com/tauri-apps/tauri/pull/2030)) on 2021-06-21

- Fixes the HTTP API binary response serialization.
  - [47f75584](https://www.github.com/tauri-apps/tauri/commit/47f7558417cc654bdb1d018127e8900bc4eac622) fix(core): resolve HTTP API on non-ok status code, fix binary response, closes [#2046](https://www.github.com/tauri-apps/tauri/pull/2046) ([#2053](https://www.github.com/tauri-apps/tauri/pull/2053)) on 2021-06-23

- The `http` APIs now resolve the returned promise when the API call finishes with an error status code.
  - [47f75584](https://www.github.com/tauri-apps/tauri/commit/47f7558417cc654bdb1d018127e8900bc4eac622) fix(core): resolve HTTP API on non-ok status code, fix binary response, closes [#2046](https://www.github.com/tauri-apps/tauri/pull/2046) ([#2053](https://www.github.com/tauri-apps/tauri/pull/2053)) on 2021-06-23

- Run the `notification.show()` method on a dedicated async task to prevent a panic on Windows.
  - [86d0aaa0](https://www.github.com/tauri-apps/tauri/commit/86d0aaa021f146529a65228de9e07d5f4df5b099) fix(core): notification panic on Windows, closes [#917](https://www.github.com/tauri-apps/tauri/pull/917) ([#2011](https://www.github.com/tauri-apps/tauri/pull/2011)) on 2021-06-19

- Fixes HTTP API headers being overwritten when using the `reqwest` client.
  - [1006c1cf](https://www.github.com/tauri-apps/tauri/commit/1006c1cf3be9be054d0650f0d9787b5aee045f5a) fix(core): HTTP headers being overwritten by reqwest, closes [#2032](https://www.github.com/tauri-apps/tauri/pull/2032) ([#2036](https://www.github.com/tauri-apps/tauri/pull/2036)) on 2021-06-21

- Remove closed window from the `window.__TAURI__.__windows` array, used by the `window.getAll` API from `@tauri-apps/api`.
  - [ebaa33cb](https://www.github.com/tauri-apps/tauri/commit/ebaa33cb47e045af75140d818565d211f45946b4) fix(core): remove closed window from `window.__TAURI__.__windows` ([#2057](https://www.github.com/tauri-apps/tauri/pull/2057)) on 2021-06-23

- Panic on window getters usage on the main thread when the event loop is not running and document it.
  - [ab3eb44b](https://www.github.com/tauri-apps/tauri/commit/ab3eb44bac7a3bf73a4985df38ccc2b87a913be7) fix(core): deadlock on window getters, fixes [#1893](https://www.github.com/tauri-apps/tauri/pull/1893) ([#1998](https://www.github.com/tauri-apps/tauri/pull/1998)) on 2021-06-16

- Adds `focus` API to the WindowBuilder.
  - [5f351622](https://www.github.com/tauri-apps/tauri/commit/5f351622c7812ad1bb56ddb37364ccaa4124c24b) feat(core): add focus API to the WindowBuilder and WindowOptions, [#1737](https://www.github.com/tauri-apps/tauri/pull/1737) on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23

- **Breaking change:** The global shortcut API is now managed by `tao` so it cannot be accessed globally, the manager is now exposed on the `App` and `AppHandle` structs.
  - [3280c4aa](https://www.github.com/tauri-apps/tauri/commit/3280c4aa91e50a8ccdd561a8b48a12a4a13ea8d5) refactor(core): global shortcut is now provided by `tao` ([#2031](https://www.github.com/tauri-apps/tauri/pull/2031)) on 2021-06-21

- Hide `phf` crate export (not public API).
  - [cd1a299a](https://www.github.com/tauri-apps/tauri/commit/cd1a299a7d5a9bd164063a32c87a27762b71e9a8) chore(core): hide phf, closes [#1961](https://www.github.com/tauri-apps/tauri/pull/1961) ([#1964](https://www.github.com/tauri-apps/tauri/pull/1964)) on 2021-06-09

- (internal): allow `wry` dependency to be optional again while keeping default args.
  code that wishes to expose a struct with a default arg should use the `crate::manager::default_args!` macro to declare
  the struct, so that it can automatically feature-gate `DefaultArgs` behind using `wry`.
  - [3d8dcbbf](https://www.github.com/tauri-apps/tauri/commit/3d8dcbbf8188b9e96d6b03dc984ca022eebf53e4) fix(core): allow wry to be an optional dep again (fix [#1841](https://www.github.com/tauri-apps/tauri/pull/1841)) ([#1854](https://www.github.com/tauri-apps/tauri/pull/1854)) on 2021-05-17

- Adds `is_decorated` getter on Window.
  - [f58a2114](https://www.github.com/tauri-apps/tauri/commit/f58a2114fbfd5307c349f05c88f2e08fd8baa8aa) feat(core): add `is_decorated` Window getter on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23

- Adds `is_resizable` getter on Window.
  - [1e8af280](https://www.github.com/tauri-apps/tauri/commit/1e8af280c27f381828d6209722b10e889082fa00) feat(core): add `is_resizable` Window getter on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23

- Adds `is_visible` getter on Window.
  - [36506c96](https://www.github.com/tauri-apps/tauri/commit/36506c967de82bc7ff453d11e6104ecf66d7a588) feat(core): add `is_visible` API on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23

- Read `tauri.conf.json > tauri > bundle > icons` and use the first `.png` icon as window icon on Linux. Defaults to `icon/icon.png` if a PNG icon is not configured.
  - [40b717ed](https://www.github.com/tauri-apps/tauri/commit/40b717edc57288a1393fad0529390e101ab903c1) feat(core): set window icon on Linux, closes [#1922](https://www.github.com/tauri-apps/tauri/pull/1922) ([#1937](https://www.github.com/tauri-apps/tauri/pull/1937)) on 2021-06-01

- Adds `accelerator` method to the `CustomMenuItem` struct to define a keyboard shortcut for the menu item.
  - [034c2601](https://www.github.com/tauri-apps/tauri/commit/034c26013bce0c7bbe6db067ea7fd24a53a5c998) feat(core): add `accelerator` method to `CustomMenuItem` ([#2043](https://www.github.com/tauri-apps/tauri/pull/2043)) on 2021-06-22

- **Breaking change:** The `menu` API was not designed to have all the new features: submenus, item updates, disabled state... so we broke it before going to stable.
  - [f7e9fe8f](https://www.github.com/tauri-apps/tauri/commit/f7e9fe8f3f7c83532713be6cc4ef84e8b127c208) refactor(core): new system tray and window menu APIs, closes [#1898](https://www.github.com/tauri-apps/tauri/pull/1898) ([#1944](https://www.github.com/tauri-apps/tauri/pull/1944)) on 2021-06-04

- Adds a `PathResolver` struct to simplify the usage of the `tauri::api::path::{app_dir, resource_dir}` APIs, accessible through the `App` and `AppHandle` `path_resolver` methods.
  - [5ca462f6](https://www.github.com/tauri-apps/tauri/commit/5ca462f6ccc6c970a6f2c8c6c1bc0e3343a52bfb) feat(core): add path resolver API to the App and AppHandle structs ([#2015](https://www.github.com/tauri-apps/tauri/pull/2015)) on 2021-06-19

- Removes `image` dependency. For now only `.ico` icons on Windows are supported, and we'll implement other types on demand to optimize bundle size.
  - [1be37a3f](https://www.github.com/tauri-apps/tauri/commit/1be37a3f30ff789d9396ec9009f9c0dd0bb928a7) refactor(core): remove `image` dependency ([#1859](https://www.github.com/tauri-apps/tauri/pull/1859)) on 2021-05-18

- Remove window object from the `Manager` internal `HashMap` on close. This fixes the behavior of using `[App|AppHandle|Window]#get_window` after the window is closed (now correctly returns `None`).
  - [08c161c5](https://www.github.com/tauri-apps/tauri/commit/08c161c5e85d9f6392be55723b573030fa70637b) fix(core): remove window from HashMap on close ([#2024](https://www.github.com/tauri-apps/tauri/pull/2024)) on 2021-06-20

- Improve RPC security by requiring a numeric code to invoke commands. The codes are generated by the Rust side and injected into the app's code using a closure, so external scripts can't access the backend. This change doesn't protect `withGlobalTauri` (`window.__TAURI__`) usage.
  - [160fb052](https://www.github.com/tauri-apps/tauri/commit/160fb0529fd31d755574ae30fbdf01fa221a2acb) feat(core): improve RPC security, closes [#814](https://www.github.com/tauri-apps/tauri/pull/814) ([#2047](https://www.github.com/tauri-apps/tauri/pull/2047)) on 2021-06-22

- Adds `run_iteration` API to the `App` and return the app instance on the `build` method of the `Builder`. The `run_iteration` method runs the window event loop step by step, allowing Tauri to be run along other applications.
  - [8c0d0739](https://www.github.com/tauri-apps/tauri/commit/8c0d0739eebf7286b64a5380e922746411eb52c6) feat(core): add `run_iteration`, `parent_window` and `owner_window` APIs, closes [#1872](https://www.github.com/tauri-apps/tauri/pull/1872) ([#1874](https://www.github.com/tauri-apps/tauri/pull/1874)) on 2021-05-21

- The `run_on_main_thread` API now uses WRY's UserEvent, so it wakes the event loop.
  - [9bf82f0d](https://www.github.com/tauri-apps/tauri/commit/9bf82f0d9261808f58bdb5b5dbd6a255e5dcd333) fix(core): `run_on_main_thread` now wakes the event loop ([#1949](https://www.github.com/tauri-apps/tauri/pull/1949)) on 2021-06-04

- Adds `set_focus` API on Window.
  - [bb6992f8](https://www.github.com/tauri-apps/tauri/commit/bb6992f888196ca7c87bb2fe74ad2bd8bf393e05) feat(core): add `set_focus` window API, fixes [#1737](https://www.github.com/tauri-apps/tauri/pull/1737) on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23

- Adds `set_skip_taskbar` API on Window.
  - [e06aa277](https://www.github.com/tauri-apps/tauri/commit/e06aa277384450cfef617c0e57b0d5d403bb1e7f) feat(core): add `set_skip_taskbar` API on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23

- Adds `skip_taskbar` API to the WindowBuilder.
  - [5525b03a](https://www.github.com/tauri-apps/tauri/commit/5525b03a78a2232c650043fbd9894ce1553cad41) feat(core): add `skip_taskbar` API to the WindowBuilder/WindowOptions on 2021-05-30
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23

- **Breaking change:** The `system_tray` and `on_system_tray_event` APIs were not designed to have all the new features: submenus, item updates, click events, positioning... so we broke it before going to stable.
  - [f7e9fe8f](https://www.github.com/tauri-apps/tauri/commit/f7e9fe8f3f7c83532713be6cc4ef84e8b127c208) refactor(core): new system tray and window menu APIs, closes [#1898](https://www.github.com/tauri-apps/tauri/pull/1898) ([#1944](https://www.github.com/tauri-apps/tauri/pull/1944)) on 2021-06-04

- Fix loading url containing URI fragment
  - [07fd9a92](https://www.github.com/tauri-apps/tauri/commit/07fd9a92dea695fc8178d34bb71e0745f21cd634) fix(core): add hash symbol in uri parse, closes [#1943](https://www.github.com/tauri-apps/tauri/pull/1943) ([#1947](https://www.github.com/tauri-apps/tauri/pull/1947)) on 2021-06-05

- Adds `Window#center` and `WindowBuilder#center` APIs.
  - [5cba6eb4](https://www.github.com/tauri-apps/tauri/commit/5cba6eb4d28d53f06855d60d4d0eae6b95233ccf) feat(core): add window `center` API, closes [#1822](https://www.github.com/tauri-apps/tauri/pull/1822) ([#1954](https://www.github.com/tauri-apps/tauri/pull/1954)) on 2021-06-05

- Adds window native handle getter (HWND on Windows).
  - [abf78c58](https://www.github.com/tauri-apps/tauri/commit/abf78c5860cdc52fbfd2bc5dbca29a864e2da8f9) fix(core): set parent window handle on dialogs, closes [#1876](https://www.github.com/tauri-apps/tauri/pull/1876) ([#1889](https://www.github.com/tauri-apps/tauri/pull/1889)) on 2021-05-21

## \[1.0.0-beta.1]

- Adds a mutable `config` getter on the `Context` struct.
  - [144d6b9](https://www.github.com/tauri-apps/tauri/commit/144d6b9d4d327debae13392715103a3208ce8a45) feat(core): add mutable `config` getter on the `Context` struct ([#1803](https://www.github.com/tauri-apps/tauri/pull/1803)) on 2021-05-12

## \[1.0.0-beta.0]

- **Breaking:** `api::path::resolve_path()` and `api::path::app_dir()` now takes the config as first argument and the `PackageInfo` as second argument.
  **Breaking:** `api::path::app_dir()` now resolves to `${configDir}/${config.tauri.bundle.identifier}`.
  - [428d50a](https://www.github.com/tauri-apps/tauri/commit/428d50add4da937325d189434dbaf3a02d745187) refactor(core): use bundle identifier on app dir, closes [#1693](https://www.github.com/tauri-apps/tauri/pull/1693) ([#1703](https://www.github.com/tauri-apps/tauri/pull/1703)) on 2021-05-04
  - [7bb7dda](https://www.github.com/tauri-apps/tauri/commit/7bb7dda7523bc1a81e890e0aeafffd35e3ed767f) refactor(core): resolve resource_dir using the package info ([#1762](https://www.github.com/tauri-apps/tauri/pull/1762)) on 2021-05-10

- Adds `manage` API to the `Builder` struct, which manages app state.
  - [8b6f3de](https://www.github.com/tauri-apps/tauri/commit/8b6f3de0ad47684e72a2ae5f884d8675acfaeeac) feat(core): add state management, closes [#1655](https://www.github.com/tauri-apps/tauri/pull/1655) ([#1665](https://www.github.com/tauri-apps/tauri/pull/1665)) on 2021-05-02

- **Breaking:** The `assets` field on the `tauri::Context` struct is now a `Arc<impl Assets>`.
  - [5110c70](https://www.github.com/tauri-apps/tauri/commit/5110c704be67e51d49fb83f3710afb593973dcf9) feat(core): allow users to access the Assets instance ([#1691](https://www.github.com/tauri-apps/tauri/pull/1691)) on 2021-05-03

- Only commands with a `async fn` are executed on a separate task. `#[command] fn command_name` runs on the main thread.
  - [bb8dafb](https://www.github.com/tauri-apps/tauri/commit/bb8dafbe1ea6edde7385631d41ac05e96a3309ef) feat(core): #\[command] return with autoref specialization workaround fix [#1672](https://www.github.com/tauri-apps/tauri/pull/1672) ([#1734](https://www.github.com/tauri-apps/tauri/pull/1734)) on 2021-05-09

- Renamed the `command` API module to `process`.
  - [b0bb796](https://www.github.com/tauri-apps/tauri/commit/b0bb796a42e2560233aea47ce6ced54ac238eb53) refactor: rename `command` mod to `process`, move restart_application ([#1667](https://www.github.com/tauri-apps/tauri/pull/1667)) on 2021-04-30

- Adds `options` argument to the shell command API (`env` and `cwd` configuration).
  - [721e98f](https://www.github.com/tauri-apps/tauri/commit/721e98f175567b360c86f30565ab1b9d08e7cf85) feat(core): add env, cwd to the command API, closes [#1634](https://www.github.com/tauri-apps/tauri/pull/1634) ([#1635](https://www.github.com/tauri-apps/tauri/pull/1635)) on 2021-04-28

- Improves support for commands returning `Result`.
  - [bb8dafb](https://www.github.com/tauri-apps/tauri/commit/bb8dafbe1ea6edde7385631d41ac05e96a3309ef) feat(core): #\[command] return with autoref specialization workaround fix [#1672](https://www.github.com/tauri-apps/tauri/pull/1672) ([#1734](https://www.github.com/tauri-apps/tauri/pull/1734)) on 2021-05-09

- Adds `status` and `output` APIs to the `tauri::api::process::Command` struct.
  - [d92fde7](https://www.github.com/tauri-apps/tauri/commit/d92fde75053d1f3fbac4f926c40a1e8cf29bf2a4) feat(core): add `output` and `status` APIs to the `Command` struct ([#1668](https://www.github.com/tauri-apps/tauri/pull/1668)) on 2021-05-02

- The `create_window` API callback now takes two arguments: the `WindowBuilder` and the `WebviewAttributes` and must return a tuple containing both values.
  - [c31f097](https://www.github.com/tauri-apps/tauri/commit/c31f0978c535f794fffb75a121e69a323e70b06e) refactor: update to wry 0.9 ([#1630](https://www.github.com/tauri-apps/tauri/pull/1630)) on 2021-04-28

- Reintroduce `csp` injection, configured on `tauri.conf.json > tauri > security > csp`.
  - [6132f3f](https://www.github.com/tauri-apps/tauri/commit/6132f3f4feb64488ef618f690a4f06adce864d91) feat(core): reintroduce CSP injection ([#1704](https://www.github.com/tauri-apps/tauri/pull/1704)) on 2021-05-04

- Adds the default types used with `Builder::default()` to items that expose `Params` in their type. This allows you to
  skip specifying a generic parameter to types like `Window<P>` if you use the default type.
  - [27a7810](https://www.github.com/tauri-apps/tauri/commit/27a78107673b63b6dad42fcf2bc8b7acd90b6ec4) feat(core): add default Args to all types exposing Params ([#1777](https://www.github.com/tauri-apps/tauri/pull/1777)) on 2021-05-11

- Change draggable region element detection from `drag-region` class to `data-tauri-drag-region` attribute.
  - [4f1e87f](https://www.github.com/tauri-apps/tauri/commit/4f1e87f87bbd4e094116b83edff448847da178ea) refactor(core): change drag element detection to data attr, fixes [#1656](https://www.github.com/tauri-apps/tauri/pull/1656) ([#1659](https://www.github.com/tauri-apps/tauri/pull/1659)) on 2021-04-29

- Emit `tauri://resize`, `tauri://move`, `tauri://close-requested`, `tauri://destroyed`, `tauri://focus`, `tauri://blur` and `tauri://scale-change` events to the window.
  - [9c10ccf](https://www.github.com/tauri-apps/tauri/commit/9c10ccf8d30af92cb90044a732e904c53507771a) feat(core) window events, closes [#1523](https://www.github.com/tauri-apps/tauri/pull/1523) ([#1726](https://www.github.com/tauri-apps/tauri/pull/1726)) on 2021-05-06

- The event `emit` function payload type is now `impl Serialize` instead of `Option<impl Serialize>`.
  - [4687538](https://www.github.com/tauri-apps/tauri/commit/4687538987af7638e8625342f2e3d065771c12c7) refactor(core): drop `Option` payload type on `event::emit` ([#1760](https://www.github.com/tauri-apps/tauri/pull/1760)) on 2021-05-09

- Update `tauri-hotkey` to v0.1.2, fixing a compilation issue on 32-bit platforms.
  - [92a01a7](https://www.github.com/tauri-apps/tauri/commit/92a01a7cab6d55f368b60a0d6bcc96d2847b2a81) chore(deps): bump tauri-hotkey to 0.1.2 ([#1701](https://www.github.com/tauri-apps/tauri/pull/1701)) on 2021-05-04

- Implemented window menus APIs.
  - [41d5d6a](https://www.github.com/tauri-apps/tauri/commit/41d5d6aff29750beca7263a9c186bf209388b8ee) feat(core): window menus ([#1745](https://www.github.com/tauri-apps/tauri/pull/1745)) on 2021-05-08

- Added the \`#\[non_exhaustive] attribute where appropriate.
  - [e087f0f](https://www.github.com/tauri-apps/tauri/commit/e087f0f9374355ac4b4a48f94727ef8b26b1c4cf) feat: add `#[non_exhaustive]` attribute ([#1725](https://www.github.com/tauri-apps/tauri/pull/1725)) on 2021-05-05

- `Notification.requestPermission()` now returns `"denied"` when not allowlisted.
  `IsNotificationPermissionGranted` returns `false` when not allowlisted.
  - [8941790](https://www.github.com/tauri-apps/tauri/commit/8941790f98206adce441d7bdc42374af39edc786) fix(core): notification permission check when !allowlisted, closes [#1666](https://www.github.com/tauri-apps/tauri/pull/1666) ([#1677](https://www.github.com/tauri-apps/tauri/pull/1677)) on 2021-05-02

- Refactored the `Plugin` trait `initialize` and `extend_api` signatures.
  `initialize` now takes the `App` as first argument, and `extend_api` takes an `Invoke` instead of `InvokeMessage`.
  This adds support to managed state on plugins.
  - [8b6f3de](https://www.github.com/tauri-apps/tauri/commit/8b6f3de0ad47684e72a2ae5f884d8675acfaeeac) feat(core): add state management, closes [#1655](https://www.github.com/tauri-apps/tauri/pull/1655) ([#1665](https://www.github.com/tauri-apps/tauri/pull/1665)) on 2021-05-02
  - [1d6f418](https://www.github.com/tauri-apps/tauri/commit/1d6f41812925142eb40f1908d2498880ab4a6266) refactor(core): merge invoke items into single struct, allow ? ([#1683](https://www.github.com/tauri-apps/tauri/pull/1683)) on 2021-05-02

- `window.print()` now works on all platforms.
  - [56e74cc](https://www.github.com/tauri-apps/tauri/commit/56e74ccf748ef24075b1095170d764dcfda4ddeb) feat(core): ensure `window.print()`works on macOS ([#1738](https://www.github.com/tauri-apps/tauri/pull/1738)) on 2021-05-07

- **Breaking:** `Context` fields are now private, and is expected to be created through `Context::new(...)`.
  All fields previously available through `Context` are now public methods.
  - [5542359](https://www.github.com/tauri-apps/tauri/commit/55423590ddbf560684dab6a0214acf95aadfa8d2) refactor(core): Context fields now private, Icon used on all platforms ([#1774](https://www.github.com/tauri-apps/tauri/pull/1774)) on 2021-05-11

- `Settings` is now serialized using `bincode`.
  - [455c550](https://www.github.com/tauri-apps/tauri/commit/455c550f347fe09581b4440bb868cacbbbbe2ad2) refactor(core): `Settings` serialization using `bincode` ([#1758](https://www.github.com/tauri-apps/tauri/pull/1758)) on 2021-05-09

- The window management API was refactored: removed `setX`, `setY`, `setWidth`, `setHeight` APIs, renamed `resize` to `setSize` and the size and position APIs now allow defining both logical and physical values.
  - [6bfac86](https://www.github.com/tauri-apps/tauri/commit/6bfac866a703f1499a64237fb29b2625703f4e22) refactor(core): add window getters, physical & logical sizes/positions ([#1723](https://www.github.com/tauri-apps/tauri/pull/1723)) on 2021-05-05

- Removed the `tcp` module from `tauri::api`.
  - [e087f0f](https://www.github.com/tauri-apps/tauri/commit/e087f0f9374355ac4b4a48f94727ef8b26b1c4cf) feat: add `#[non_exhaustive]` attribute ([#1725](https://www.github.com/tauri-apps/tauri/pull/1725)) on 2021-05-05

- Removes the `with_window` attribute on the `command` macro. Tauri now infers it using the `CommandArg` trait.
  - [8b6f3de](https://www.github.com/tauri-apps/tauri/commit/8b6f3de0ad47684e72a2ae5f884d8675acfaeeac) feat(core): add state management, closes [#1655](https://www.github.com/tauri-apps/tauri/pull/1655) ([#1665](https://www.github.com/tauri-apps/tauri/pull/1665)) on 2021-05-02
  - [1d6f418](https://www.github.com/tauri-apps/tauri/commit/1d6f41812925142eb40f1908d2498880ab4a6266) refactor(core): merge invoke items into single struct, allow ? ([#1683](https://www.github.com/tauri-apps/tauri/pull/1683)) on 2021-05-02

- Move `restart_application` API from `app` module to `process` module.
  - [b0bb796](https://www.github.com/tauri-apps/tauri/commit/b0bb796a42e2560233aea47ce6ced54ac238eb53) refactor: rename `command` mod to `process`, move restart_application ([#1667](https://www.github.com/tauri-apps/tauri/pull/1667)) on 2021-04-30

- `tauri-runtime` crate initial release.
  - [665ec1d](https://www.github.com/tauri-apps/tauri/commit/665ec1d4a199ad06e369221da187dc838da71cbf) refactor: move runtime to `tauri-runtime` crate ([#1751](https://www.github.com/tauri-apps/tauri/pull/1751)) on 2021-05-09
  - [45a7a11](https://www.github.com/tauri-apps/tauri/commit/45a7a111e0cf9d9956d713cc9a99fa7a5313eec7) feat(core): add `tauri-wry` crate ([#1756](https://www.github.com/tauri-apps/tauri/pull/1756)) on 2021-05-09

- The `setup` Error type must be `Send`.
  - [e087f0f](https://www.github.com/tauri-apps/tauri/commit/e087f0f9374355ac4b4a48f94727ef8b26b1c4cf) feat: add `#[non_exhaustive]` attribute ([#1725](https://www.github.com/tauri-apps/tauri/pull/1725)) on 2021-05-05

- Simplify usage of app event and window label types. The following functions now
  accept references the `Tag` can be borrowed as. This means an `&str` can now be
  accepted for functions like `Window::emit`. This is a breaking change for the
  following items, which now need to take a reference. Additionally, type inference
  for `&"event".into()` will no longer work, but `&"event".to_string()` will. The
  solution for this is to now just pass `"event"` because `Borrow<str>` is implemented
  for the default event type `String`.

- **Breaking:** `Window::emit` now accepts `Borrow` for the event.

- **Breaking:** `Window::emit_others` now accepts `Borrow` for the event

- **Breaking:** `Window::trigger` now accepts `Borrow` for the event.

- **Breaking:** `Manager::emit_all` now accepts `Borrow` for the event.

- **Breaking:** `Manager::emit_to` now accepts `Borrow` for both the event and window label.

- **Breaking:** `Manager::trigger_global` now accepts `Borrow` for the event.

- **Breaking:** `Manager::get_window` now accepts `Borrow` for the window label.

- *(internal):* `trait tauri::runtime::tag::TagRef` helper for accepting tag references.
  Any time you want to accept a tag reference, that trait will handle requiring the reference
  to have all the necessary bounds, and generate errors when the exposed function doesn't
  set a bound like `P::Event: Borrow<E>`.

- [181e132](https://www.github.com/tauri-apps/tauri/commit/181e132aee895da23c1b63deb41a52e9910910cc) refactor(core): simplify usage of app event and window label types ([#1623](https://www.github.com/tauri-apps/tauri/pull/1623)) on 2021-04-27

- [a755d23](https://www.github.com/tauri-apps/tauri/commit/a755d23e1bd0a3d6a2b6a85ff94feaf5a1a3a60d) refactor(core): more bounds for better errors from [#1623](https://www.github.com/tauri-apps/tauri/pull/1623) ([#1632](https://www.github.com/tauri-apps/tauri/pull/1632)) on 2021-04-27

- `tauri-runtime-wry` initial release.
  - [45a7a11](https://www.github.com/tauri-apps/tauri/commit/45a7a111e0cf9d9956d713cc9a99fa7a5313eec7) feat(core): add `tauri-wry` crate ([#1756](https://www.github.com/tauri-apps/tauri/pull/1756)) on 2021-05-09

- Adds system tray support.
  - [c090927](https://www.github.com/tauri-apps/tauri/commit/c0909270216983bed47453ddf5902abf5082fe42) feat(core): system tray, closes [#157](https://www.github.com/tauri-apps/tauri/pull/157) ([#1749](https://www.github.com/tauri-apps/tauri/pull/1749)) on 2021-05-09

- Rename `Attributes` to `WindowBuilder`.
  - [c31f097](https://www.github.com/tauri-apps/tauri/commit/c31f0978c535f794fffb75a121e69a323e70b06e) refactor: update to wry 0.9 ([#1630](https://www.github.com/tauri-apps/tauri/pull/1630)) on 2021-04-28

- The `Window#create_window` API now has the same signature as `App#create_window`.
  - [dbd9b07](https://www.github.com/tauri-apps/tauri/commit/dbd9b078aaa53663f61318153ba3d50c7e554ad8) refactor(core): `create_window` API signature on the Window struct ([#1746](https://www.github.com/tauri-apps/tauri/pull/1746)) on 2021-05-08

- Adds `on_window_event` API to the `Window` struct.
  - [9c10ccf](https://www.github.com/tauri-apps/tauri/commit/9c10ccf8d30af92cb90044a732e904c53507771a) feat(core) window events, closes [#1523](https://www.github.com/tauri-apps/tauri/pull/1523) ([#1726](https://www.github.com/tauri-apps/tauri/pull/1726)) on 2021-05-06

- Adds window getters.
  - [6bfac86](https://www.github.com/tauri-apps/tauri/commit/6bfac866a703f1499a64237fb29b2625703f4e22) refactor(core): add window getters, physical & logical sizes/positions ([#1723](https://www.github.com/tauri-apps/tauri/pull/1723)) on 2021-05-05

- Update `wry` to v0.9.
  - [c31f097](https://www.github.com/tauri-apps/tauri/commit/c31f0978c535f794fffb75a121e69a323e70b06e) refactor: update to wry 0.9 ([#1630](https://www.github.com/tauri-apps/tauri/pull/1630)) on 2021-04-28

## \[1.0.0-beta-rc.4]

- Update `tauri-macros` and `tauri-utils` to `1.0.0-beta-rc.1`.

## \[1.0.0-beta-rc.3]

- `tauri::error::CreateWebview` now has the error string message attached.
  - [7471e34](https://www.github.com/tauri-apps/tauri/commit/7471e347d3b23b7604c19040b0d989da8f48cb91) feat(core): add error message on `error::CreateWebview` ([#1602](https://www.github.com/tauri-apps/tauri/pull/1602)) on 2021-04-23
- If the dialog `defaultPath` is a file, use it as starting file path.
  - [aa7e273](https://www.github.com/tauri-apps/tauri/commit/aa7e2738ccafd8e4f5df866206f12888f6db8973) feat: use `rfd::FileDialog#set_file_name` if `default_path` is a file ([#1598](https://www.github.com/tauri-apps/tauri/pull/1598)) on 2021-04-23
- Validate dialog option `defaultPath` - it must exists.
  - [cfa74eb](https://www.github.com/tauri-apps/tauri/commit/cfa74ebf68de96cf46bc9471c61f9a84dd0be9ee) feat(core): validate dialog `default_path` (it must exist) ([#1599](https://www.github.com/tauri-apps/tauri/pull/1599)) on 2021-04-23
- Expose `async_runtime` module.
  - [d3fdeb4](https://www.github.com/tauri-apps/tauri/commit/d3fdeb45184d9aed8405ded53607a7cca979275e) feat(core): expose `async_runtime` module ([#1576](https://www.github.com/tauri-apps/tauri/pull/1576)) on 2021-04-21
- Expose `PageLoadPayload` struct.
  - [5e65b76](https://www.github.com/tauri-apps/tauri/commit/5e65b768e5930708695512260faf8c12d679c04e) fix(core): expose `PageLoadPayload` struct ([#1590](https://www.github.com/tauri-apps/tauri/pull/1590)) on 2021-04-22
- Fixes the Message `command` name value on plugin invoke handler.
  - [422dd5e](https://www.github.com/tauri-apps/tauri/commit/422dd5e2a0a03bb1556915c78f110bfab092c874) fix(core): command name on plugin invoke handler ([#1577](https://www.github.com/tauri-apps/tauri/pull/1577)) on 2021-04-21
- Allow `window.__TAURI__.invoke` to be moved to another variable.
  - [be65f04](https://www.github.com/tauri-apps/tauri/commit/be65f04db7d2fce23477156ebba368a897ceee3c) fix(core): make window.**TAURI**.invoke context free, fixes [#1547](https://www.github.com/tauri-apps/tauri/pull/1547) ([#1565](https://www.github.com/tauri-apps/tauri/pull/1565)) on 2021-04-21
- Make sure custom protocol is treated as secure content on macOS.
  - [5909c1e](https://www.github.com/tauri-apps/tauri/commit/5909c1e01437e10c45694c24f9037f4b176a03ec) Make sure custom protocol is handled as secure context on macOS ([#1551](https://www.github.com/tauri-apps/tauri/pull/1551)) on 2021-04-22
- Fixes macOS shortcut modifiers keycodes.
  - [ceadf2f](https://www.github.com/tauri-apps/tauri/commit/ceadf2f556f5f327b34f8fdd01e5e07969182b13) fix(core): macos shortcut modifiers, closes [#1542](https://www.github.com/tauri-apps/tauri/pull/1542) ([#1560](https://www.github.com/tauri-apps/tauri/pull/1560)) on 2021-04-21
- Adds APIs to determine global and webview-specific URI scheme handlers.
  - [938fb62](https://www.github.com/tauri-apps/tauri/commit/938fb624f5cc0f2a4499ea67cd30b014a18a6526) feat(core): expose custom protocol handler APIs ([#1553](https://www.github.com/tauri-apps/tauri/pull/1553)) on 2021-04-21
  - [a868cb7](https://www.github.com/tauri-apps/tauri/commit/a868cb71762268aa7b78af26622c900bddf3344c) refactor(core): clear `uri_scheme_protocol` registration function names ([#1617](https://www.github.com/tauri-apps/tauri/pull/1617)) on 2021-04-25
- The package info APIs now checks the `package` object on `tauri.conf.json`.
  - [8fd1baf](https://www.github.com/tauri-apps/tauri/commit/8fd1baf69b14bb81d7be9d31605ed7f02058b392) fix(core): pull package info from tauri.conf.json if set ([#1581](https://www.github.com/tauri-apps/tauri/pull/1581)) on 2021-04-22
- Change plugin trait `initialization` return type to `std::result::Result<(), Box<dyn std::error::Error>>`.
  - [508eddc](https://www.github.com/tauri-apps/tauri/commit/508eddc78458cd7ff51259ed733fe8e6f206e293) refactor(core): plugin initialization return value ([#1575](https://www.github.com/tauri-apps/tauri/pull/1575)) on 2021-04-21
- Fixes `sidecar` Command API.
  - [99307d0](https://www.github.com/tauri-apps/tauri/commit/99307d02c3c28ce10ba418873ac02ce267af4f4f) fix(core): sidecar command path ([#1584](https://www.github.com/tauri-apps/tauri/pull/1584)) on 2021-04-22
- Set LocalStorage and IndexedDB files path on Linux to `$HOME/.local/${bundleIdentifier}`.
  - [5f033db](https://www.github.com/tauri-apps/tauri/commit/5f033db41cf6b043d9d2b4debe8b10bdc4633c58) feat(core): use bundle identifier on user data path ([#1580](https://www.github.com/tauri-apps/tauri/pull/1580)) on 2021-04-22
- Use bundle identifier instead of `Tauri` for user data path on Windows.
  - [5f033db](https://www.github.com/tauri-apps/tauri/commit/5f033db41cf6b043d9d2b4debe8b10bdc4633c58) feat(core): use bundle identifier on user data path ([#1580](https://www.github.com/tauri-apps/tauri/pull/1580)) on 2021-04-22

## \[1.0.0-beta-rc.2]

- Prevent "once" events from being able to be called multiple times.

- `Window::trigger(/*...*/)` is now properly `pub` instead of `pub(crate)`.

- `Manager::once_global(/*...*/)` now returns an `EventHandler`.

- `Window::once(/*...*/)` now returns an `EventHandler`.

- (internal) `event::Listeners::trigger(/*...*/)` now handles removing "once" events.

- [ece243d](https://www.github.com/tauri-apps/tauri/commit/ece243d17c9c3ef8d2ba8e3b25b872aa6ea0b6ab) don't remove once listener on new thread ([#1506](https://www.github.com/tauri-apps/tauri/pull/1506)) on 2021-04-14

- Window and global events can now be nested inside event handlers. They will run as soon
  as the event handler closure is finished in the order they were called. Previously, calling
  events inside an event handler would produce a deadlock.

Note: The order that event handlers are called when triggered is still non-deterministic.

- [e447b8e](https://www.github.com/tauri-apps/tauri/commit/e447b8e0e6198c8972bae25625bb409850cb686b) allow event listeners to be nested ([#1513](https://www.github.com/tauri-apps/tauri/pull/1513)) on 2021-04-15

## \[1.0.0-beta-rc.1]

- Missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
  - Bumped due to a bump in api.
  - [b2569a7](https://www.github.com/tauri-apps/tauri/commit/b2569a729a3caa88bdba62abc31f0665e1323aaa) fix(js-api): dist ([#1498](https://www.github.com/tauri-apps/tauri/pull/1498)) on 2021-04-15

## \[1.0.0-beta-rc.0]

- internal refactoring of `Params` to allow for easier usage without a private trait with only 1 implementor.
  `ParamsPrivate` -> `ParamsBase`
  `ManagerPrivate` -> `ManagerBase`
  (new) `Args`, crate only. Now implements `Params`/`ParamsBase`.
  `App` and `Window` use `WindowManager` directly
- [ec27ca8](https://www.github.com/tauri-apps/tauri/commit/ec27ca81fe721e0b08ed574073547250b7a8153a) refactor(tauri): remove private params trait methods ([#1484](https://www.github.com/tauri-apps/tauri/pull/1484)) on 2021-04-14
- Updated `wry`, fixing an issue with the draggable region.
  - [f2d24ef](https://www.github.com/tauri-apps/tauri/commit/f2d24ef2fbd95ec7d3433ba651964f4aa3b7f48c) chore(deps): update wry ([#1482](https://www.github.com/tauri-apps/tauri/pull/1482)) on 2021-04-14
- Now Tauri commands always returns Promise<T>.
  - [ea73325](https://www.github.com/tauri-apps/tauri/commit/ea7332539d100bd63f93396101ffa01ff73c924b) refactor(core): all API are now promise based ([#1239](https://www.github.com/tauri-apps/tauri/pull/1239)) on 2021-02-16
- Rename macOS bundle settings from `osx` to `macOS`.
  - [080f639](https://www.github.com/tauri-apps/tauri/commit/080f6391bac4fd59e9e71b9785d7a2f835703805) refactor(bundler): specific settings on dedicated structs, update README ([#1380](https://www.github.com/tauri-apps/tauri/pull/1380)) on 2021-03-25
- The shell process spawning API was rewritten and now includes stream access.
  - [3713066](https://www.github.com/tauri-apps/tauri/commit/3713066e451bd30d0cc6f57bb437f08276f4c4ad) refactor(core): rewrite shell execute API, closes [#1229](https://www.github.com/tauri-apps/tauri/pull/1229) ([#1408](https://www.github.com/tauri-apps/tauri/pull/1408)) on 2021-03-31
- The Tauri files are now read on the app space instead of the `tauri` create.
  Also, the `AppBuilder` `build` function now returns a Result.
  - [e02c941](https://www.github.com/tauri-apps/tauri/commit/e02c9419cb8c66f4e43ed598d2fc74d4b19384ec) refactor(tauri): support for building without environmental variables ([#850](https://www.github.com/tauri-apps/tauri/pull/850)) on 2021-02-09
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Tauri now uses explicit Error variants with `thiserror` instead of relying on `anyhow`.
  - [156a0ad](https://www.github.com/tauri-apps/tauri/commit/156a0ad5cb0a152eaa0dd038a6b3dba68f03eb21) refactor(tauri): use explicit error types instead of anyhow ([#1209](https://www.github.com/tauri-apps/tauri/pull/1209)) on 2021-02-10
- Align HTTP API types with the [documentation](https://tauri.app/en/docs/api/js#http).
  - [2fc39fc](https://www.github.com/tauri-apps/tauri/commit/2fc39fc341771431078c20a95fa6b2affe5155c9) fix(api/http): correct types ([#1360](https://www.github.com/tauri-apps/tauri/pull/1360)) on 2021-03-17
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Replace `\` with `\\` in css assets that are lazy loaded. Since these are injected in a template literal, backslashes must be escaped. Backslashes are sometimes used for octal sequences in CSS.
  - [4491c70](https://www.github.com/tauri-apps/tauri/commit/4491c707907a6a931fd8c057c2baeb0b9e6db1d8) fix(tauri/asset): escape octal sequences in css ([#1166](https://www.github.com/tauri-apps/tauri/pull/1166)) on 2021-01-30
- Replaces the embedded-server mode with Wry's custom protocol feature. This allows assets to be transferred to the webview directly, instead of through a localhost server.
  - [0c691f4](https://www.github.com/tauri-apps/tauri/commit/0c691f40a338be184a4dd2c84d6e5d0b0ed6ee4b) feat(core): Use Wry custom protocol instead of embedded server ([#1296](https://www.github.com/tauri-apps/tauri/pull/1296)) on 2021-02-25
- The `message` and `ask` dialogs now use `tinyfiledialogs-rs` instead of `tauri-dialog-rs`.
  - [6eee355](https://www.github.com/tauri-apps/tauri/commit/6eee355a12ead3ac9cb4be0c98c1cfe5c0611291) refactor(core): use tinyfiledialogs-rs for message/confirmation dialogs ([#1255](https://www.github.com/tauri-apps/tauri/pull/1255)) on 2021-02-17
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Refactor the event callback payload and return an unlisten function on the `listen` API.
  - [b670ec5](https://www.github.com/tauri-apps/tauri/commit/b670ec55f2b7389b8a2f8c965d4fe1e0cb46e6dc) refactor(core): add `unlisten`, `once` APIs to the event system ([#1359](https://www.github.com/tauri-apps/tauri/pull/1359)) on 2021-03-16
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Adds `unlisten` and `once` APIs on the Rust event system.
  - [b670ec5](https://www.github.com/tauri-apps/tauri/commit/b670ec55f2b7389b8a2f8c965d4fe1e0cb46e6dc) refactor(core): add `unlisten`, `once` APIs to the event system ([#1359](https://www.github.com/tauri-apps/tauri/pull/1359)) on 2021-03-16
- The `tauri::event` module has been moved to a Webview manager API.
  - [07208df](https://www.github.com/tauri-apps/tauri/commit/07208dff6c1e8cff7c10780f4f7f8cee9de44a2e) feat(core): add mult-window support ([#1217](https://www.github.com/tauri-apps/tauri/pull/1217)) on 2021-02-11
- The file dialog API now uses [rfd](https://github.com/PolyMeilex/rfd). The filter option is now an array of `{ name: string, extensions: string[] }`.
  - [2326bcd](https://www.github.com/tauri-apps/tauri/commit/2326bcd399411f7f0eabdb7ade910be473adadae) refactor(core): use `nfd` for file dialogs, closes [#1251](https://www.github.com/tauri-apps/tauri/pull/1251) ([#1257](https://www.github.com/tauri-apps/tauri/pull/1257)) on 2021-02-18
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Tauri now emits events on file drops on the webview window.
  - [2db901e](https://www.github.com/tauri-apps/tauri/commit/2db901e744f51cd4296ddec4352d7a51c859b85b) feat(core): add file drop handler ([#1352](https://www.github.com/tauri-apps/tauri/pull/1352)) on 2021-03-12
- Fixes `resource_dir` resolution on AppImage.
  - [bd1df5d](https://www.github.com/tauri-apps/tauri/commit/bd1df5d80431f5de4cd905ffaf7f3f2628d6b8ab) fix: get correct resource dir in AppImge (fix [#1308](https://www.github.com/tauri-apps/tauri/pull/1308)) ([#1333](https://www.github.com/tauri-apps/tauri/pull/1333)) on 2021-03-12
- Fixed missing 'App' variant & string promise instead of void promise.
  - [44fc65c](https://www.github.com/tauri-apps/tauri/commit/44fc65c723f638f2a1b2ecafb79b32d509ed2f35) Fixing TS API typings ([#1451](https://www.github.com/tauri-apps/tauri/pull/1451)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The HTTP API was improved with client caching and better payload and response types.
  - [a7bc472](https://www.github.com/tauri-apps/tauri/commit/a7bc472e994730071f960d09a12ac85296a080ae) refactor(core): improve HTTP API, closes [#1098](https://www.github.com/tauri-apps/tauri/pull/1098) ([#1237](https://www.github.com/tauri-apps/tauri/pull/1237)) on 2021-02-15
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Added new JavaScript API to extract `name`, `version`, `tauri version` from the running application. We exposed `relaunch` and `exit` as well to control your application state.
  - [e511d39](https://www.github.com/tauri-apps/tauri/commit/e511d3991041a974273a2674a9bf60230b7519ee) feat(api): Expose application metadata and functions to JS api - fix [#1387](https://www.github.com/tauri-apps/tauri/pull/1387) ([#1445](https://www.github.com/tauri-apps/tauri/pull/1445)) on 2021-04-08
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- The event listener `once` kind was moved to a dedicated function.
  - [372036c](https://www.github.com/tauri-apps/tauri/commit/372036ce20ac7f103dea05bae7e8686858d096a4) refactor(api): move event's `once` to its own function ([#1276](https://www.github.com/tauri-apps/tauri/pull/1276)) on 2021-02-23
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Use ``JSON.parse(String.raw`{arg}`)`` for communicating serialized JSON objects and arrays < 1 GB to the Webview from Rust.

https://github.com/GoogleChromeLabs/json-parse-benchmark

- [eeb2030](https://www.github.com/tauri-apps/tauri/commit/eeb20308acdd83029abb6ce94fb5d0c896759060) Use JSON.parse instead of literal JS for callback formatting ([#1370](https://www.github.com/tauri-apps/tauri/pull/1370)) on 2021-04-07
- [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Added support to multiple windows.
  - [07208df](https://www.github.com/tauri-apps/tauri/commit/07208dff6c1e8cff7c10780f4f7f8cee9de44a2e) feat(core): add mult-window support ([#1217](https://www.github.com/tauri-apps/tauri/pull/1217)) on 2021-02-11
- Adds `productName` and `version` configs on `tauri.conf.json > package`.
  - [5b3d9b2](https://www.github.com/tauri-apps/tauri/commit/5b3d9b2c07da766f81981ba7c4961cd354d51340) feat(config): allow setting product name and version on tauri.conf.json ([#1358](https://www.github.com/tauri-apps/tauri/pull/1358)) on 2021-03-22
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Plugins are now configurable through a `tauri.conf.json > "plugins" > $pluginName` object.
  - [2058cc3](https://www.github.com/tauri-apps/tauri/commit/2058cc39c9ac9d9d442802db2c2f3be34a85acc4) feat(tauri): add plugin `initialize` (with config) API, run in parallel ([#1194](https://www.github.com/tauri-apps/tauri/pull/1194)) on 2021-02-10
- Renamed the `Plugin` trait `init_script` to `initialization_script`.
  - [5c5d8f8](https://www.github.com/tauri-apps/tauri/commit/5c5d8f811fc094cca1b441ff966f15c7bf5d2e90) refactor(tauri): rename `init_script` to `initialization_script` ([#1200](https://www.github.com/tauri-apps/tauri/pull/1200)) on 2021-02-10
- The plugin instance is now mutable and must be `Send`.
  - [fb607ee](https://www.github.com/tauri-apps/tauri/commit/fb607ee97a912d1e23f6d7dd6dd3c28aac9b4527) refactor(tauri): plugin trait with mutable references ([#1197](https://www.github.com/tauri-apps/tauri/pull/1197)) on 2021-02-10
  - [1318ffb](https://www.github.com/tauri-apps/tauri/commit/1318ffb47c5f2fb696d6323fbcee4f840396c6b3) refactor(core): remove async from app hooks, add InvokeMessage type ([#1392](https://www.github.com/tauri-apps/tauri/pull/1392)) on 2021-03-26
- Fixes the event system usage on the plugin `ready` hook.
  - [23132ac](https://www.github.com/tauri-apps/tauri/commit/23132acf765ab8b6a37b74151a4c175b68390657) fix(tauri): run plugin::ready without webview.dispatch ([#1164](https://www.github.com/tauri-apps/tauri/pull/1164)) on 2021-01-29
- The `allowlist` configuration now has one object per module.
  - [e0be59e](https://www.github.com/tauri-apps/tauri/commit/e0be59ea26df17fe2e31224759f21fb1d0cbdfd3) refactor(core): split allowlist configuration per module ([#1263](https://www.github.com/tauri-apps/tauri/pull/1263)) on 2021-02-20
- The Tauri script is now injected with the webview `init` API, so it is available after page changes.
  - [4412b7c](https://www.github.com/tauri-apps/tauri/commit/4412b7c438c2b10e519bf8b696e3ef827e9091f2) refactor(tauri): inject script with webview init API ([#1186](https://www.github.com/tauri-apps/tauri/pull/1186)) on 2021-02-05
  - [8bdd894](https://www.github.com/tauri-apps/tauri/commit/8bdd8949254d63bfc57ad67ce2592d40a0b44bf8) refactor(core): move bundle script to /tauri crate ([#1377](https://www.github.com/tauri-apps/tauri/pull/1377)) on 2021-03-23
- Removed the `no-server` mode, the `inliner`, the `dev` server proxy and the `loadAsset` API.
  - [84d7cda](https://www.github.com/tauri-apps/tauri/commit/84d7cdae632eeb02a66f8d1d7577adfa65917a34) refactor(core): remove `no-server` and its APIs ([#1215](https://www.github.com/tauri-apps/tauri/pull/1215)) on 2021-02-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Adds a global shortcut API.
  - [855effa](https://www.github.com/tauri-apps/tauri/commit/855effadd9ebfb6bc1a3555ac7fc733f6f766b7a) feat(core): globalShortcut API ([#1232](https://www.github.com/tauri-apps/tauri/pull/1232)) on 2021-02-14
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Added `async` support to the Tauri Rust core on commit [#a169b67](https://github.com/tauri-apps/tauri/commit/a169b67ef0277b958bdac97e33c6e4c41b6844c3).
  - [2bf55f8](https://www.github.com/tauri-apps/tauri/commit/2bf55f80564f5c31d89384bd4a82dd55307f8c75) chore: add changefile on 2021-02-03
  - [e02c941](https://www.github.com/tauri-apps/tauri/commit/e02c9419cb8c66f4e43ed598d2fc74d4b19384ec) refactor(tauri): support for building without environmental variables ([#850](https://www.github.com/tauri-apps/tauri/pull/850)) on 2021-02-09
- Alpha version of tauri-updater. Please refer to the `README` for more details.
  - [6d70c8e](https://www.github.com/tauri-apps/tauri/commit/6d70c8e1e256fe839c4a947375bb529d7b4f7301) feat(updater): Alpha version ([#643](https://www.github.com/tauri-apps/tauri/pull/643)) on 2021-04-05
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The Tauri integration with Webview was refactored to use traits, which allows custom implementations by developers and simplifies changes on the webview implementation.
  - [b9ce7b9](https://www.github.com/tauri-apps/tauri/commit/b9ce7b94c4eb027bcbbd4ee600b75a7407f108ca) refactor(tauri): Webview traits ([#1183](https://www.github.com/tauri-apps/tauri/pull/1183)) on 2021-02-05
- Added window management and window creation APIs.
  - [a3d6dff](https://www.github.com/tauri-apps/tauri/commit/a3d6dff2163c7a45842253edd81dbc62248dc65d) feat(core): window API ([#1225](https://www.github.com/tauri-apps/tauri/pull/1225)) on 2021-02-13
  - [641374b](https://www.github.com/tauri-apps/tauri/commit/641374b15343518cd835bd5ada811941c65dcf2e) feat(core): window creation at runtime ([#1249](https://www.github.com/tauri-apps/tauri/pull/1249)) on 2021-02-17
- Use [WRY](https://github.com/tauri-apps/wry) as Webview interface, thanks to @wusyong.
  - [99ecf7b](https://www.github.com/tauri-apps/tauri/commit/99ecf7bb3e8da6e611b57c6680a14b65179f8a35) feat(tauri): use WRY as webview engine ([#1190](https://www.github.com/tauri-apps/tauri/pull/1190)) on 2021-02-08

## \[0.11.1]

- Update webview-official dependency which fix compatibility on macOS.
  - [692312a](https://www.github.com/tauri-apps/tauri/commit/692312a0f51a05dd418d9cad553a695f3347b943) chore(deps) Update webview-official ([#1152](https://www.github.com/tauri-apps/tauri/pull/1152)) on 2021-01-24

## \[0.11.0]

- Match writeBinaryFile command name between js and rust
  - [486bd92](https://www.github.com/tauri-apps/tauri/commit/486bd920f899905bec0f690092aa1e3cac2c78f3) Fix: writeBinaryFile to call the correct command (fix [#1133](https://www.github.com/tauri-apps/tauri/pull/1133)) ([#1136](https://www.github.com/tauri-apps/tauri/pull/1136)) on 2021-01-06

## \[0.10.0]

- Adds missing APIs features from `allowlist` to the tauri crate's manifest file.
  - [2c0f09c](https://www.github.com/tauri-apps/tauri/commit/2c0f09c85c8a60c2fa304fb25174d5020663f0d7) fix(tauri) add missing API features, closes [#1023](https://www.github.com/tauri-apps/tauri/pull/1023) ([#1052](https://www.github.com/tauri-apps/tauri/pull/1052)) on 2020-10-17
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Adds a path resolution API (e.g. getting the download directory or resolving a path to the home directory).
  - [771e401](https://www.github.com/tauri-apps/tauri/commit/771e4019b8cfd1973015ffa632c9d6c6b82c5657) feat: Port path api to js ([#1006](https://www.github.com/tauri-apps/tauri/pull/1006)) on 2020-09-24
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Update minimum Rust version to 1.42.0 due to a dependency update.
  - [d13dcd9](https://www.github.com/tauri-apps/tauri/commit/d13dcd9fd8d30b1db147a78cecb878e924382274) chore(deps) Update Tauri Bundler ([#1045](https://www.github.com/tauri-apps/tauri/pull/1045)) on 2020-10-17
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Minimum Rust version updated to 1.47.0. Run `$ rustup update` to update to the latest version.
  - [b4544b6](https://www.github.com/tauri-apps/tauri/commit/b4544b63f268dc6f3f47a4bfbad5d72cceee8698) chore(deps) Update Tauri API ([#1072](https://www.github.com/tauri-apps/tauri/pull/1072)) on 2020-11-07

## \[0.9.2]

- Bump all deps as noted in #975, #976, #977, #978, and #979.
  - [06dd75b](https://www.github.com/tauri-apps/tauri/commit/06dd75b68a15d388808c51ae2bf50554ae761d9d) chore: bump all js/rust deps ([#983](https://www.github.com/tauri-apps/tauri/pull/983)) on 2020-08-20

## \[0.9.1]

- Adjust payload formatting to handle multibyte characters in front-end message
  payloads.
  \- [df70ca5](https://www.github.com/tauri-apps/tauri/commit/df70ca51965665952a74161cc6eb1ff19eae45e2) Fix [#912](https://www.github.com/tauri-apps/tauri/pull/912) multibyte character breaks message ([#914](https://www.github.com/tauri-apps/tauri/pull/914)) on 2020-08-01

## \[0.9.0]

- Make sure CSS content loaded with the `loadAsset` API is inside a template string and not injected raw.
  - [e3e2e39](https://www.github.com/tauri-apps/tauri/commit/e3e2e3920833627400ee7a5b000dc6e51d8d332b) fix(tauri) ensure css content is loaded inside a string ([#884](https://www.github.com/tauri-apps/tauri/pull/884)) on 2020-07-22
  - [b96b1fb](https://www.github.com/tauri-apps/tauri/commit/b96b1fb6b8a4f565fb946847bb9a29d9d939e2cb) inject css with template string to allow for line breaks ([#894](https://www.github.com/tauri-apps/tauri/pull/894)) on 2020-07-25
- Pin the `tauri-api` dependency on the `tauri` crate so updates doesn't crash the build.
  - [ad717c6](https://www.github.com/tauri-apps/tauri/commit/ad717c6f33b4d6e20fbb13cbe30e06946dbb74f6) chore(tauri) pin tauri-api dep version ([#872](https://www.github.com/tauri-apps/tauri/pull/872)) on 2020-07-21
- Fixes the Webview initialization on Windows.
  - [4abd12c](https://www.github.com/tauri-apps/tauri/commit/4abd12c2a42b5ace8527114ab64da38f4486754f) fix(tauri) webview initialization on windows, fixes [#879](https://www.github.com/tauri-apps/tauri/pull/879) ([#885](https://www.github.com/tauri-apps/tauri/pull/885)) on 2020-07-23

## \[0.8.0]

- Use native dialog on `window.alert` and `window.confirm`.
  Since every communication with the webview is asynchronous, the `window.confirm` returns a Promise resolving to a boolean value.
  \- [0245833](https://www.github.com/tauri-apps/tauri/commit/0245833bb56d5462a4e1249eb1e2f9f5e477592d) feat(tauri) make `window.alert` and `window.confirm` available, fix [#848](https://www.github.com/tauri-apps/tauri/pull/848) ([#854](https://www.github.com/tauri-apps/tauri/pull/854)) on 2020-07-18
  \- [dac0ae9](https://www.github.com/tauri-apps/tauri/commit/dac0ae976ea1b419ed5af078d00106b1476dee04) chore(changes) add tauri-api to JS dialogs changefile on 2020-07-19
- The notification's `body` is now optional, closes #793.
  - [dac1db3](https://www.github.com/tauri-apps/tauri/commit/dac1db39831ecbcf23c630351d5753af01ccd500) fix(tauri) notification body optional, requestPermission() regression, closes [#793](https://www.github.com/tauri-apps/tauri/pull/793) ([#844](https://www.github.com/tauri-apps/tauri/pull/844)) on 2020-07-16
- Fixes a regression on the storage of requestPermission response.
  ß
  \- [dac1db3](https://www.github.com/tauri-apps/tauri/commit/dac1db39831ecbcf23c630351d5753af01ccd500) fix(tauri) notification body optional, requestPermission() regression, closes [#793](https://www.github.com/tauri-apps/tauri/pull/793) ([#844](https://www.github.com/tauri-apps/tauri/pull/844)) on 2020-07-16
- Plugin system added. You can hook into the webview lifecycle (`created`, `ready`) and extend the API adding logic to the `invoke_handler` by implementing the `tauri::plugin::Plugin` trait.
  - [78afee9](https://www.github.com/tauri-apps/tauri/commit/78afee9725e0e372f9de7edeaac523011a1c02a3) feat(tauri) add plugin system for rust ([#494](https://www.github.com/tauri-apps/tauri/pull/494)) on 2020-07-12
- Renaming `whitelist` to `allowlist` (see #645).
  - [a6bb3b5](https://www.github.com/tauri-apps/tauri/commit/a6bb3b59059e08a844d7bb2b43f3d1192954d890) refactor(tauri) rename `whitelist` to `allowlist`, ref [#645](https://www.github.com/tauri-apps/tauri/pull/645) ([#858](https://www.github.com/tauri-apps/tauri/pull/858)) on 2020-07-19
- Moving the webview implementation to [webview](https://github.com/webview/webview), with the [official Rust binding](https://github.com/webview/webview_rust).
  This is a breaking change.
  IE support has been dropped, so the `edge` object on `tauri.conf.json > tauri` no longer exists and you need to remove it.
  `webview.handle()` has been replaced with `webview.as_mut()`.
  \- [cd5b401](https://www.github.com/tauri-apps/tauri/commit/cd5b401707d709bf8212b0a4c34623f902ae40f9) feature: import official webview rust binding ([#846](https://www.github.com/tauri-apps/tauri/pull/846)) on 2020-07-18

## \[0.7.5]

- Fixes Edge blank screen on Windows when running tauri dev (Tauri crashing window due to Edge reloading app because of missing Content-Type header).
  - Bumped due to a bump in tauri-api.
  - [fedee83](https://www.github.com/tauri-apps/tauri/commit/fedee835e36daa4363b91aabd43143e8033c9a5c) fix(tauri.js) windows Edge blank screen on tauri dev ([#808](https://www.github.com/tauri-apps/tauri/pull/808)) on 2020-07-11

## \[0.7.4]

- Ignoring non UTF-8 characters on the loopback command output.
  - [f340b29](https://www.github.com/tauri-apps/tauri/commit/f340b2914dc9c3a94ca8606f4663964fa87b95ea) fix(tauri) addition to the previous commit on 2020-07-10

## \[0.7.3]

- Properly run the loopback command on Windows.
- Properly ignore the ${distDir}/index.html asset from the asset embedding. Previously every asset with name matching /(.+)index.html$/g were ignored.

## \[0.7.2]

Bumped due to dependency.

## \[0.7.1]

- Fixes the assets embedding into the binary.

## \[0.7.0]

- The execute_promise and execute_promise_sync helpers now accepts any tauri::Result<T> where T: impl Serialize.
  This means that you do not need to serialize your response manually or deal with String quotes anymore.
  As part of this refactor, the event::emit function also supports impl Serialize instead of String.

## \[0.6.2]

- Fixes the Windows build with the latest Windows SDK.

## \[0.6.1] - (Not Published)

## \[0.6.0]

- Adds a command line interface option to tauri apps, configurable under tauri.conf.json > tauri > cli.
- Fixes no-server mode not running on another machine due to fs::read_to_string usage instead of the include_str macro.
  Build no longer fails when compiling without environment variables, now the app will show an error.
- Adds desktop notifications API.
- Properly reflect tauri.conf.json changes on app when running tauri dev.
