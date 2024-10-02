# Changelog

## \[2.0.1]

### What's Changed

- [`be683e2ac`](https://www.github.com/tauri-apps/tauri/commit/be683e2ac36df9c51a5c050d9d500247bd019090) ([#11199](https://www.github.com/tauri-apps/tauri/pull/11199) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Publish package with the latest NPM tag.

## \[2.0.0]

### What's Changed

- [`637285790`](https://www.github.com/tauri-apps/tauri/commit/6372857905ae9c0aedb7f482ddf6cf9f9836c9f2) Promote to v2 stable!

## \[2.0.0-rc.6]

### New Features

- [`9014a3f17`](https://www.github.com/tauri-apps/tauri/commit/9014a3f1765ca406ea5c3e5224267a79c52cd53d) ([#11066](https://www.github.com/tauri-apps/tauri/pull/11066) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `WebviewWindow.clearAllBrowsingData` and `Webview.clearAllBrowsingData` to clear the webview browsing data.
- [`95df53a2e`](https://www.github.com/tauri-apps/tauri/commit/95df53a2ed96873cd35a4b14a5e312d07e4e3004) ([#11143](https://www.github.com/tauri-apps/tauri/pull/11143) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Add the ability to set theme dynamically using `Window.setTheme` or `setTheme` function from the `app` module
- [`d9d2502b4`](https://www.github.com/tauri-apps/tauri/commit/d9d2502b41e39efde679e30c8955006e2ba9ea64) ([#11140](https://www.github.com/tauri-apps/tauri/pull/11140) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `Webview.hide` and `Webview.show` methods.
- [`de7414aab`](https://www.github.com/tauri-apps/tauri/commit/de7414aab935e45540594ea930eb60bae4dbc979) ([#11154](https://www.github.com/tauri-apps/tauri/pull/11154) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `Window::setEnabled` and `Window::isEnabled` methods

### Bug Fixes

- [`948772a65`](https://www.github.com/tauri-apps/tauri/commit/948772a657eb3caf20843628abac9109e3b67d41) ([#11114](https://www.github.com/tauri-apps/tauri/pull/11114) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Change the `button_state` tray event field to camelCase `buttonState`.

### Breaking Changes

- [`0b4495996`](https://www.github.com/tauri-apps/tauri/commit/0b4495996d3131a5ee80fbb2c71a28203e491ee7) ([#11121](https://www.github.com/tauri-apps/tauri/pull/11121) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Simplified emitted tray event JS value and updated `TrayIconEvent` type definition to match it.

## \[2.0.0-rc.5]

### New Features

- [`ddf69157b`](https://www.github.com/tauri-apps/tauri/commit/ddf69157b54249f3321ca72db6703812019f1ab9) ([#11031](https://www.github.com/tauri-apps/tauri/pull/11031) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `toPhysical` method on `LogicalPositon` and `LogicalSize` classes.

## \[2.0.0-rc.4]

### Enhancements

- [`f81929e25`](https://www.github.com/tauri-apps/tauri/commit/f81929e25104aa1091e464bd012c80649dedf9e5) ([#10799](https://www.github.com/tauri-apps/tauri/pull/10799) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added `PermissionState`, `checkPermissions` and `requestPermissions` base APIs to the core module, designed for plugin authors to extend.

### Bug Fixes

- [`fbe76a955`](https://www.github.com/tauri-apps/tauri/commit/fbe76a955a63af9fb33f66d5f747caf858cf179b) ([#10797](https://www.github.com/tauri-apps/tauri/pull/10797) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Uint8Arrays and ArrayBuffers are now properly serialized as an array of numbers.

## \[2.0.0-rc.3]

### What's Changed

- [`f4d5241b3`](https://www.github.com/tauri-apps/tauri/commit/f4d5241b377d0f7a1b58100ee19f7843384634ac) ([#10731](https://www.github.com/tauri-apps/tauri/pull/10731) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Update documentation icon path.

## \[2.0.0-rc.2]

### Bug Fixes

- [`c689521a7`](https://www.github.com/tauri-apps/tauri/commit/c689521a7674b6562b5dfd4f5cacd12138d99d85) ([#10681](https://www.github.com/tauri-apps/tauri/pull/10681) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix tslib path in dist.

## \[2.0.0-rc.1]

### Breaking Changes

- [`b6dca99ff`](https://www.github.com/tauri-apps/tauri/commit/b6dca99fff73816a39380b288c299b47b493cfdb) ([#10630](https://www.github.com/tauri-apps/tauri/pull/10630) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Changed `WebviewWindow.getAll`, `WebviewWindow.getByLabel`, `getAllWebviewWindows`,
  `Window.getAll`, `Window.getByLabel`, `getAllWindows`,
  `Webview.getAll`, `Webview.getByLabel`, `getAllWebviews`
  to be async so their return value are synchronized with the state from the Rust side,
  meaning new and destroyed windows are reflected.

## \[2.0.0-rc.0]

### Changes

- Promoted to RC!

## \[2.0.0-beta.16]

### New Features

- [`da25f7353`](https://www.github.com/tauri-apps/tauri/commit/da25f7353070477ba969851e974379d7666d6806) ([#10242](https://www.github.com/tauri-apps/tauri/pull/10242) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add APIs to enable setting window size constraints separately:

  - Added `WindowSizeConstraints` interface in `window` and `webviewWindow` modules.
  - Added `Window.setSizeConstraints` and `WebviewWindow.setSizeConstraints`

### Bug Fixes

- [`3c17fb64f`](https://www.github.com/tauri-apps/tauri/commit/3c17fb64fd822597d5cc16ee7e7b3f9e1023637b) ([#10277](https://www.github.com/tauri-apps/tauri/pull/10277) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Fix `Webview.reparent` pointing to `set_webview_focus` instead of `reparent` Rust API
- [`da25f7353`](https://www.github.com/tauri-apps/tauri/commit/da25f7353070477ba969851e974379d7666d6806) ([#10242](https://www.github.com/tauri-apps/tauri/pull/10242) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Apply `minWidth`, `minHieght`, `maxWidth` and `maxHeight` constraints separately, which fixes a long standing bug where these constraints were never applied unless width and height were constrained together.

## \[2.0.0-beta.15]

### New Features

- [`7bc6a2a1d`](https://www.github.com/tauri-apps/tauri/commit/7bc6a2a1d6d2c5406d91cac94d33bce76443c28f) ([#9788](https://www.github.com/tauri-apps/tauri/pull/9788) by [@pewsheen](https://www.github.com/tauri-apps/tauri/../../pewsheen)) Add a new method to set title bar style dynamically on macOS.

### Enhancements

- [`080b6e127`](https://www.github.com/tauri-apps/tauri/commit/080b6e12720b89d839c686d7067cc94d276ed7e4) ([#10246](https://www.github.com/tauri-apps/tauri/pull/10246) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Use `EventName` on `Window`, `Webview` and `WebviewWindow`'s `once` so you can get auto complete for tauri's built-in events

### Bug Fixes

- [`080b6e127`](https://www.github.com/tauri-apps/tauri/commit/080b6e12720b89d839c686d7067cc94d276ed7e4) ([#10246](https://www.github.com/tauri-apps/tauri/pull/10246) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Fix `once` doesn't detached after one callback if event handler throws

### Breaking Changes

- [`261c9f942`](https://www.github.com/tauri-apps/tauri/commit/261c9f942de9a598b5c6cc504de6bddd1306113b) ([#10170](https://www.github.com/tauri-apps/tauri/pull/10170) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Renamed drag and drop events in `TauriEvent` enum to better convey when they are triggered:

  - `TauriEvent.DRAG` -> `TauriEvent.DRAG_ENTER`
  - `TauriEvent.DROP_OVER` -> `TauriEvent.DRAG_OVER`
  - `TauriEvent.DROP` -> `TauriEvent.DRAG_DROP`
  - `TauriEvent.DROP_CANCELLED` -> `TauriEvent::DRAG_LEAVE`

  Also the `type` field values in `Window/Webview/WebviewWindow.onDropEvent` and `DragDropEvent` have changed:

  - `dragged` -> `enter`
  - `dragOver` -> `over`
  - `dropped` -> `drop`
  - `cancelled` -> `leave`
- [`2b1ceb40d`](https://www.github.com/tauri-apps/tauri/commit/2b1ceb40d345aef42dd79438fa69ca7989ee0194) ([#10229](https://www.github.com/tauri-apps/tauri/pull/10229) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Renamed the JS `getCurrent` and `getAll` functions to a clearer name to avoid ambiguity:

  - `getCurrent` in `window` module has been renamed to `getCurrentWindow`
  - `getCurrent` in `webview` module has been renamed to `getCurrentWebview`
  - `getCurrent` in `webviewWindow` module has been renamed to `getCurrentWebviewWindow`
  - `getAll` in `window` module has been renamed to `getAllWindows`
  - `getAll` in `webview` module has been renamed to `getAllWebviews`
  - `getAll` in `webviewWindow` module has been renamed to `getAllWebviewWindows`

## \[2.0.0-beta.14]

### New Features

- [`148f04887`](https://www.github.com/tauri-apps/tauri/commit/148f048871caee21498b236c058b8890f2b66cc7) ([#9979](https://www.github.com/tauri-apps/tauri/pull/9979)) Add `defaultWindowIcon` to the JS `app` module to retrieve the default window icon in JS.

### Bug Fixes

- [`c98f385cb`](https://www.github.com/tauri-apps/tauri/commit/c98f385cb5da4d72968df24b1fc0b58212d59653) ([#10044](https://www.github.com/tauri-apps/tauri/pull/10044)) Export `mocks` module in `@tauri-apps/api` npm package.

## \[2.0.0-beta.13]

### Breaking Changes

- [`c4410daa8`](https://www.github.com/tauri-apps/tauri/commit/c4410daa85616340e911c8243fdaa69e6906fd49)([#9777](https://www.github.com/tauri-apps/tauri/pull/9777)) This release contains breaking changes to the tray event structure because of newly added events:

  - Changed `TrayIconEvent` to be an enum instead of a struct.
  - Added `MouseButtonState` and `MouseButton` enums.
  - Removed `ClickType` enum and replaced it with `MouseButton` enum.
  - Added `MouseButtonState` enum.

## \[2.0.0-beta.12]

### New Features

- [`ec0e092ec`](https://www.github.com/tauri-apps/tauri/commit/ec0e092ecd23b547c756c7476f23a0d95be6db80)([#9770](https://www.github.com/tauri-apps/tauri/pull/9770)) Add `monitorFromPoint` function in `window` module to get the monitor from a given point.

## \[2.0.0-beta.11]

### Bug Fixes

- [`aa080696e`](https://www.github.com/tauri-apps/tauri/commit/aa080696e0952abff416dd9088d519eaf2587a3a)([#9618](https://www.github.com/tauri-apps/tauri/pull/9618)) Fix `isTauri` incorrect return type.

## \[2.0.0-beta.10]

### New Features

- [`477bb8cd4`](https://www.github.com/tauri-apps/tauri/commit/477bb8cd4ea88ade3f6c1f268ad1701a68150161)([#9297](https://www.github.com/tauri-apps/tauri/pull/9297)) Add `cursorPosition` function in `window` module to get the current cursor position.

## \[2.0.0-beta.9]

### New Features

- [`70c51371e`](https://www.github.com/tauri-apps/tauri/commit/70c51371e01184223312de3dba8030394a5a9406)([#9539](https://www.github.com/tauri-apps/tauri/pull/9539)) Add `isTauri` function in `core` module to check whether running inside tauri or not.

### Bug Fixes

- [`be7eab209`](https://www.github.com/tauri-apps/tauri/commit/be7eab209c60c45e140f7bcb4bab1037d62d4c03)([#9486](https://www.github.com/tauri-apps/tauri/pull/9486)) Set the `exports > types` package.json field.
- [`cf615e8e4`](https://www.github.com/tauri-apps/tauri/commit/cf615e8e4d5008ee1ac3f77e530ba26fb91e8977)([#9463](https://www.github.com/tauri-apps/tauri/pull/9463)) Fixes a bug when processing channel messages out of order.
- [`35b25f7e5`](https://www.github.com/tauri-apps/tauri/commit/35b25f7e5c0fe03af4ed3582e22a626863f035f0)([#9530](https://www.github.com/tauri-apps/tauri/pull/9530)) Do not use JS optional chaining to prevent script errors on older webviews such as macOS 10.14.

## \[2.0.0-beta.8]

### New Features

- [`58a7a552d`](https://www.github.com/tauri-apps/tauri/commit/58a7a552d739b77b71d61af11c53f7f2dc7a6e7e)([#9378](https://www.github.com/tauri-apps/tauri/pull/9378)) Added the `set_zoom` function to the webview API.
- [`58a7a552d`](https://www.github.com/tauri-apps/tauri/commit/58a7a552d739b77b71d61af11c53f7f2dc7a6e7e)([#9378](https://www.github.com/tauri-apps/tauri/pull/9378)) Add `zoom_hotkeys_enabled` to enable browser native zoom controls on creating webviews.

### Bug Fixes

- [`48a7a78f8`](https://www.github.com/tauri-apps/tauri/commit/48a7a78f8094d08e5e403e88050391642d29151b)([#9376](https://www.github.com/tauri-apps/tauri/pull/9376)) Fix `Window/Webview/WebviewWindow.setSize`, `Window/Webview/WebviewWindow.setPostion`, `Window/WebviewWindow.setMinSize`, `Window/WebviewWindow.setMaxSize`, `Window/WebviewWindow.setCursorPosition` and `Menu/Submenu.popup` methods failing with invalid args.

## \[2.0.0-beta.7]

### Bug Fixes

- [`c33f6e6cf`](https://www.github.com/tauri-apps/tauri/commit/c33f6e6cf35a0d34b5598875a2e5b642a01c8b38)([#9211](https://www.github.com/tauri-apps/tauri/pull/9211)) Re-added the `TauriEvent.WINDOW_CREATED` (`tauri://window-created`) event.

### Breaking Changes

- [`06833f4fa`](https://www.github.com/tauri-apps/tauri/commit/06833f4fa8e63ecc55fe3fc874a9e397e77a5709)([#9100](https://www.github.com/tauri-apps/tauri/pull/9100)) Rename `FileDrop` to `DragDrop` on structs, enums and enum variants. Also renamed `file_drop` to `drag_drop` on fields and function names.

## \[2.0.0-beta.6]

### New Features

- [`acdd76833`](https://www.github.com/tauri-apps/tauri/commit/acdd76833db6d81f4012418133d0042220de100b)([#9155](https://www.github.com/tauri-apps/tauri/pull/9155)) Add `TrayIcon.getById` and `TrayIcon.removeById` static methods.

### Enhancements

- [`ea0242db4`](https://www.github.com/tauri-apps/tauri/commit/ea0242db4aa6c127d2bb4a2e275000ba47c9e68c)([#9179](https://www.github.com/tauri-apps/tauri/pull/9179)) The `Image` constructor is now public (for internal use only).

### Bug Fixes

- [`379cc2b35`](https://www.github.com/tauri-apps/tauri/commit/379cc2b3547395474d4b66b4222679cf4538428d)([#9165](https://www.github.com/tauri-apps/tauri/pull/9165)) Fix `basename(path, 'ext')` JS API when removing all occurances of `ext` where it should only remove the last one.

### Breaking Changes

- [`ea0242db4`](https://www.github.com/tauri-apps/tauri/commit/ea0242db4aa6c127d2bb4a2e275000ba47c9e68c)([#9179](https://www.github.com/tauri-apps/tauri/pull/9179)) `Image::rgba()` now returns `Promise<Uint8Array>`.
- [`ea0242db4`](https://www.github.com/tauri-apps/tauri/commit/ea0242db4aa6c127d2bb4a2e275000ba47c9e68c)([#9179](https://www.github.com/tauri-apps/tauri/pull/9179)) Removed `width` and `height` methods on the JS `Image` class, use `size` instead.

## \[2.0.0-beta.5]

### Breaking Changes

- [`db0a24a97`](https://www.github.com/tauri-apps/tauri/commit/db0a24a973191752aeecfbd556faa254b0f17e79)([#9132](https://www.github.com/tauri-apps/tauri/pull/9132)) Remove the `Image.fromPngBytes` and `Image.fromIcoBytes` APIs. Use `Image.fromBytes` instead.

## \[2.0.0-beta.4]

### New Features

- [`d1e77acd8`](https://www.github.com/tauri-apps/tauri/commit/d1e77acd8dfdf554b90b542513a58a2de1ef2360)([#9011](https://www.github.com/tauri-apps/tauri/pull/9011)) Add a new `Image` type in Rust and JS.

### Enhancements

- [`e62ca4ee9`](https://www.github.com/tauri-apps/tauri/commit/e62ca4ee95f4308a6ad128d0f100c85634e28223)([#9070](https://www.github.com/tauri-apps/tauri/pull/9070)) Added a mechanism to preserve channel message order.

## \[2.0.0-beta.3]

### New Features

- [`fdcaf935`](https://www.github.com/tauri-apps/tauri/commit/fdcaf935fa75ecfa2806939c4faad4fe9e880386)([#8939](https://www.github.com/tauri-apps/tauri/pull/8939)) Added the `reparent` function to the webview API.

## \[2.0.0-beta.2]

### Breaking Changes

- [`361ec37f`](https://www.github.com/tauri-apps/tauri/commit/361ec37fd4a5caa5b6630b9563ef079f53c6c336)([#8932](https://www.github.com/tauri-apps/tauri/pull/8932)) Removed the `unityUri` option from the progress bar state, no longer required.

## \[2.0.0-beta.1]

### New Features

- [`16e550ec`](https://www.github.com/tauri-apps/tauri/commit/16e550ec1503765158cdc3bb2a20e70ec710e981)([#8844](https://www.github.com/tauri-apps/tauri/pull/8844)) Add a new `webviewWindow` module that exports `WebviewWindow` class and related methods such as `getCurrent` and `getAll`.
- [`16e550ec`](https://www.github.com/tauri-apps/tauri/commit/16e550ec1503765158cdc3bb2a20e70ec710e981)([#8844](https://www.github.com/tauri-apps/tauri/pull/8844)) Add `Window.onFileDropEvent` method.

### Breaking Changes

- [`16e550ec`](https://www.github.com/tauri-apps/tauri/commit/16e550ec1503765158cdc3bb2a20e70ec710e981)([#8844](https://www.github.com/tauri-apps/tauri/pull/8844)) Renamed the following enum variants of `TauriEvent` enum:

  - `TauriEvent.WEBVIEW_FILE_DROP` -> `TauriEvent.FILE_DROP`
  - `TauriEvent.WEBVIEW_FILE_DROP_HOVER` -> `TauriEvent.FILE_DROP_HOVER`
  - `TauriEvent.WEBVIEW_FILE_DROP_CANCELLED` -> `TauriEvent.FILE_DROP_CANCELLED`
- [`16e550ec`](https://www.github.com/tauri-apps/tauri/commit/16e550ec1503765158cdc3bb2a20e70ec710e981)([#8844](https://www.github.com/tauri-apps/tauri/pull/8844)) Move `WebviewWindow` class from `webview` module to a new `webviewWindow` module.

## \[2.0.0-beta.0]

### New Features

- [`74a2a603`](https://www.github.com/tauri-apps/tauri/commit/74a2a6036a5e57462f161d728cbd8a6f121028ca)([#8661](https://www.github.com/tauri-apps/tauri/pull/8661)) Implement access control list for IPC usage.
- [`a093682d`](https://www.github.com/tauri-apps/tauri/commit/a093682d2df7169b024bb4f736c7f1fd2ea8b327)([#8621](https://www.github.com/tauri-apps/tauri/pull/8621)) Added `emitTo` api to `event` module which is equivalent to the rust `emit_to` method. Also added `emitTo` method on `Window`, `Webivew` and `WebviewWindow` classes.
- [`a2fc3a63`](https://www.github.com/tauri-apps/tauri/commit/a2fc3a63579ca739646d696870cbecbb3a169d33)([#8657](https://www.github.com/tauri-apps/tauri/pull/8657)) Add `visibleOnAllWorkspaces` option when creating the window in JS and `Window.setVisibleOnAllWorkspaces` method.
- [`7f033f6d`](https://www.github.com/tauri-apps/tauri/commit/7f033f6dcd54c69a4193765a5c1584755ba92c61)([#8537](https://www.github.com/tauri-apps/tauri/pull/8537)) Add `Window.startResizeDragging`.
- [`9eaeb5a8`](https://www.github.com/tauri-apps/tauri/commit/9eaeb5a8cd95ae24b5e66205bdc2763cb7f965ce)([#8622](https://www.github.com/tauri-apps/tauri/pull/8622)) Add `parent` option when creating a window.
- [`af610232`](https://www.github.com/tauri-apps/tauri/commit/af6102327376884364b2075b468bdf08ee0d02aa)([#8710](https://www.github.com/tauri-apps/tauri/pull/8710)) Added `Window::destroy` to force close a window.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Added support to multiwebview via the new `window` and `webview` modules.

### Breaking Changes

- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Removed event callback's `windowLabel`.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) The event target is now an object so you can target either a window or a webview.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Moved webview-specific APIs from the `Window` class to the `Webview` class.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Renamed `TauriEvent.WINDOW_FILE_DROP` to `TauriEvent.WEBVIEW_FILE_DROP`, `TauriEvent.WINDOW_FILE_DROP_HOVER` to `TauriEvent.WEBVIEW_FILE_DROP_HOVER` and `TauriEvent.WINDOW_FILE_DROP_CANCELLED` to `TauriEvent.WEBVIEW_FILE_DROP_CANCELLED`.
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Added back the `WebviewWindow` API that exposes functionality of a window that hosts a single webview. The dedicated `Window` and `Webview` types are exposed for multiwebview features.
- [`af610232`](https://www.github.com/tauri-apps/tauri/commit/af6102327376884364b2075b468bdf08ee0d02aa)([#8710](https://www.github.com/tauri-apps/tauri/pull/8710)) `Window::close` now triggers a close requested event instead of forcing the window to be closed.

## \[2.0.0-alpha.14]

- [`97e33412`](https://www.github.com/tauri-apps/tauri/commit/97e334129956159bbd60e1c531b6acd3bc6139a6)([#8534](https://www.github.com/tauri-apps/tauri/pull/8534)) `mockIPC` and `mockWindows` no longer crash if `window.__TAURI_INTERNALS__` is undefined.

## \[2.0.0-alpha.13]

### New Features

- [`428ea652`](https://www.github.com/tauri-apps/tauri/commit/428ea6524c70545be33aac96d7c22b21f25caa4c)([#8370](https://www.github.com/tauri-apps/tauri/pull/8370)) Exposed `Resource` class which should be extended for Rust-backed resources created through `tauri::Manager::resources_table`.

### Bug Fixes

- [`ef21b681`](https://www.github.com/tauri-apps/tauri/commit/ef21b681e237a80592c9118b9c023c1d57231bac)([#8391](https://www.github.com/tauri-apps/tauri/pull/8391)) Fix a regression where typescript could not find types when using `"moduleResolution": "node"`
- [`46451aee`](https://www.github.com/tauri-apps/tauri/commit/46451aee1318f63a6cd861a12b63929b38c64eb6)([#8268](https://www.github.com/tauri-apps/tauri/pull/8268)) Add top-level `main`, `module` and `types` fields in `package.json` to be compliant with typescripts's `"moduleResolution": "node"`

### Breaking Changes

- [`c2ad4d28`](https://www.github.com/tauri-apps/tauri/commit/c2ad4d28c481b2d7ed643458db56210cd44a2e0c)([#8273](https://www.github.com/tauri-apps/tauri/pull/8273)) Changed former `tauri` module from `primitives` to `core`.

## \[2.0.0-alpha.12]

### New Features

- [`f93148ea`](https://www.github.com/tauri-apps/tauri/commit/f93148eac05a1428e038bd9351a8149b2464ff4c)([#7709](https://www.github.com/tauri-apps/tauri/pull/7709)) Add `tray` and `menu` modules to create and manage tray icons and menus from Javascript.

### Enhancements

- [`b7add750`](https://www.github.com/tauri-apps/tauri/commit/b7add750ef9f32d959de613ab35063ff240281c2)([#8204](https://www.github.com/tauri-apps/tauri/pull/8204)) Added `position` field to the `FileDropEvent` payload.

## \[2.0.0-alpha.11]

### Bug Fixes

- [`822bf15d`](https://www.github.com/tauri-apps/tauri/commit/822bf15d6b258556b689ca55ac2ac224897e913a)([#8130](https://www.github.com/tauri-apps/tauri/pull/8130)) Fix tslib missing in the distributed api package.

## \[2.0.0-alpha.10]

### Enhancements

- [`c6c59cf2`](https://www.github.com/tauri-apps/tauri/commit/c6c59cf2373258b626b00a26f4de4331765dd487) Pull changes from Tauri 1.5 release.

### Bug Fixes

- [`287066b2`](https://www.github.com/tauri-apps/tauri/commit/287066b279f503dd09bfd43d5da37d1f471451fb)([#8071](https://www.github.com/tauri-apps/tauri/pull/8071)) No longer crashing in tests without mocks when `clearMocks` is defined in `afterEach` hook.

## \[2.0.0-alpha.9]

### New Features

- [`c1ec0f15`](https://www.github.com/tauri-apps/tauri/commit/c1ec0f155118527361dd5645d920becbc8afd569)([#7933](https://www.github.com/tauri-apps/tauri/pull/7933)) Added `setAlwaysOnBottom` function on `Window` and the `alwaysOnBottom` option when creating a window.
- [`fb10b879`](https://www.github.com/tauri-apps/tauri/commit/fb10b87970a43320ef4d14564f45e7579b774eaf)([#8039](https://www.github.com/tauri-apps/tauri/pull/8039)) Add the `app` module back.
- [`ed32257d`](https://www.github.com/tauri-apps/tauri/commit/ed32257d044f90b5eb15053efd1667125def2d2b)([#7794](https://www.github.com/tauri-apps/tauri/pull/7794)) On Windows, add `Effect.Tabbed`,`Effect.TabbedDark` and `Effect.TabbedLight` effects.
- [`c9a9246c`](https://www.github.com/tauri-apps/tauri/commit/c9a9246c37bdf190661355c8ee406dac6c427344)([#8007](https://www.github.com/tauri-apps/tauri/pull/8007)) Add the `window` module back.
- [`c085adda`](https://www.github.com/tauri-apps/tauri/commit/c085addab58ba851398373c6fd13f9cb026d71e8)([#8009](https://www.github.com/tauri-apps/tauri/pull/8009)) Added the `setProgressBar` API on the `Window` class.

### What's Changed

- [`5c0eeb40`](https://www.github.com/tauri-apps/tauri/commit/5c0eeb40c1003583290ff3aebfa02e2b5f5b9c41)([#7638](https://www.github.com/tauri-apps/tauri/pull/7638)) Updated minimum Node.js version to 18.

### Breaking Changes

- [`a63e71f9`](https://www.github.com/tauri-apps/tauri/commit/a63e71f9799e9bbc82521d2f17b5238fbf690e89)([#7942](https://www.github.com/tauri-apps/tauri/pull/7942)) Changed `tauri` module to `primitives` and removed the undocumented `invoke` export from the root module.

## \[2.0.0-alpha.8]

### Breaking Changes

- [`d5074af5`](https://www.github.com/tauri-apps/tauri/commit/d5074af562b2b5cb6c5711442097c4058af32db6)([#7801](https://www.github.com/tauri-apps/tauri/pull/7801)) The custom protocol on Android now uses the `http` scheme instead of `https`.

## \[2.0.0-alpha.7]

### Breaking Changes

- [`4cb51a2d`](https://www.github.com/tauri-apps/tauri/commit/4cb51a2d56cfcae0749062c79ede5236bd8c02c2)([#7779](https://www.github.com/tauri-apps/tauri/pull/7779)) The custom protocol on Windows now uses the `http` scheme instead of `https`.

## \[2.0.0-alpha.6]

### New Features

- [`4af5c5a8`](https://www.github.com/tauri-apps/tauri/commit/4af5c5a8293263c16f8a65e8d232f2de52f41701)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Change the IPC call to align with the new format for the custom protocol based API.

## \[2.0.0-alpha.5]

### New Features

- [`e0f0dce2`](https://www.github.com/tauri-apps/tauri/commit/e0f0dce220730e2822fc202463aedf0166145de7)([#6442](https://www.github.com/tauri-apps/tauri/pull/6442)) Added the `windowEffects` option when creating a window and `setWindowEffects` method to change it at runtime.

### Enhancements

- [`9e3a18e0`](https://www.github.com/tauri-apps/tauri/commit/9e3a18e04672edad15d0ec654bd8632544871967)([#7132](https://www.github.com/tauri-apps/tauri/pull/7132)) Expose the window target option on event APIs.
- [`6d3f3138`](https://www.github.com/tauri-apps/tauri/commit/6d3f3138b9e2f41cda712c7d9caba0f0e65dfd3c)([#7160](https://www.github.com/tauri-apps/tauri/pull/7160)) Changed `sep` and `delimiter` from `path` module into functions to fix import in frameworks like `next.js`
- [`4652c446`](https://www.github.com/tauri-apps/tauri/commit/4652c446b361a801252bcf45e9da39813bf85482)([#7144](https://www.github.com/tauri-apps/tauri/pull/7144)) Add `tempDir` function to `path` module

## \[2.0.0-alpha.4]

- [`0ab5f40d`](https://www.github.com/tauri-apps/tauri/commit/0ab5f40d3a4207f20e4440587b41c4e78f91d233)([#6813](https://www.github.com/tauri-apps/tauri/pull/6813)) Add channel API for sending data across the IPC.
- [`3245d14b`](https://www.github.com/tauri-apps/tauri/commit/3245d14b9eb256a5c5675c7030bac7082855df47)([#6895](https://www.github.com/tauri-apps/tauri/pull/6895)) Moved the `app` feature to its own plugin in the plugins-workspace repository.
- [`09376af5`](https://www.github.com/tauri-apps/tauri/commit/09376af59424cc27803fa2820d2ac0d4cdc90a6d)([#6704](https://www.github.com/tauri-apps/tauri/pull/6704)) Moved the `cli` feature to its own plugin in the plugins-workspace repository.
- [`2d5378bf`](https://www.github.com/tauri-apps/tauri/commit/2d5378bfc1ba817ee2f331b41738a90e5997e5e8)([#6717](https://www.github.com/tauri-apps/tauri/pull/6717)) Moved the dialog APIs to its own plugin in the plugins-workspace repository.
- [`39f1b04f`](https://www.github.com/tauri-apps/tauri/commit/39f1b04f7be4966488484829cd54c8ce72a04200)([#6943](https://www.github.com/tauri-apps/tauri/pull/6943)) Moved the `event` JS APIs to a plugin.
- [`fc4d687e`](https://www.github.com/tauri-apps/tauri/commit/fc4d687ef0ef2ea069ed73c40916da733b5dcb8f)([#6716](https://www.github.com/tauri-apps/tauri/pull/6716)) Moved the file system APIs to its own plugin in the plugins-workspace repository.
- [`f78a3783`](https://www.github.com/tauri-apps/tauri/commit/f78a378344bbec48533641661d865920a8f46f8f)([#6742](https://www.github.com/tauri-apps/tauri/pull/6742)) Moved the `http` feature to its own plugin in the plugins-workspace repository.
- [`29ce9ce2`](https://www.github.com/tauri-apps/tauri/commit/29ce9ce2ce7dfb260d556d5cffd075e8fe06660c)([#6902](https://www.github.com/tauri-apps/tauri/pull/6902)) Moved the `os` feature to its own plugin in the plugins-workspace repository.
- [`60cf9ed2`](https://www.github.com/tauri-apps/tauri/commit/60cf9ed2fcd7be4df41e86cf18735efe9b6cb254)([#6905](https://www.github.com/tauri-apps/tauri/pull/6905)) Moved the `process` feature to its own plugin in the plugins-workspace repository.
- [`96639ca2`](https://www.github.com/tauri-apps/tauri/commit/96639ca239c9e4f75142fc07868ac46822111cff)([#6749](https://www.github.com/tauri-apps/tauri/pull/6749)) Moved the `shell` functionality to its own plugin in the plugins-workspace repository.
- [`b072daa3`](https://www.github.com/tauri-apps/tauri/commit/b072daa3bd3e38b808466666619ddb885052c5b2)([#6919](https://www.github.com/tauri-apps/tauri/pull/6919)) Moved the `updater` feature to its own plugin in the plugins-workspace repository.
- [`cebd7526`](https://www.github.com/tauri-apps/tauri/commit/cebd75261ac71b98976314a450cb292eeeec1515)([#6728](https://www.github.com/tauri-apps/tauri/pull/6728)) Moved the `clipboard` feature to its own plugin in the plugins-workspace repository.
- [`3f17ee82`](https://www.github.com/tauri-apps/tauri/commit/3f17ee82f6ff21108806edb7b00500b8512b8dc7)([#6737](https://www.github.com/tauri-apps/tauri/pull/6737)) Moved the `global-shortcut` feature to its own plugin in the plugins-workspace repository.
- [`9a79dc08`](https://www.github.com/tauri-apps/tauri/commit/9a79dc085870e0c1a5df13481ff271b8c6cc3b78)([#6947](https://www.github.com/tauri-apps/tauri/pull/6947)) Moved the `window` JS APIs to its own plugin in the plugins-workspace repository.

## \[2.0.0-alpha.3]

- Overload the dialog `open` function to have better TS result types.
  - [1eacd51d](https://www.github.com/tauri-apps/tauri/commit/1eacd51d185ba69a3c3cb2cc93c792e2d5929843) overloaded the open function for convenient type inference ([#5619](https://www.github.com/tauri-apps/tauri/pull/5619)) on 2023-04-07

## \[2.0.0-alpha.2]

- Added `raw` encoding option to read stdout and stderr raw bytes.
  - [f992e7f5](https://www.github.com/tauri-apps/tauri/commit/f992e7f58bf975c654a3daf36780b31a32bac064) chore(changes): readd change file on 2023-04-03
- Removed shell's `Command` constructor and added the `Command.create` static function instead.
  - [509d4678](https://www.github.com/tauri-apps/tauri/commit/509d4678b12816c1dd08a9a5efa71ba556d91c27) Support sending raw byte data to the "data" event for child command's stdout and stderr ([#5789](https://www.github.com/tauri-apps/tauri/pull/5789)) on 2023-03-31

## \[2.0.0-alpha.1]

- Added the `shadow` option when creating a window and `setShadow` function.
  - [a81750d7](https://www.github.com/tauri-apps/tauri/commit/a81750d779bc72f0fdb7de90b7fbddfd8049b328) feat(core): add shadow APIs ([#6206](https://www.github.com/tauri-apps/tauri/pull/6206)) on 2023-02-08

## \[2.0.0-alpha.0]

- First mobile alpha release!
  - [fa3a1098](https://www.github.com/tauri-apps/tauri/commit/fa3a10988a03aed1b66fb17d893b1a9adb90f7cd) feat(ci): prepare 2.0.0-alpha.0 ([#5786](https://www.github.com/tauri-apps/tauri/pull/5786)) on 2022-12-08

## \[1.5.3]

### Bug Fixes

- [`1c582a94`](https://www.github.com/tauri-apps/tauri/commit/1c582a942e345a066b65620e4db9f688ec142bb9)([#8392](https://www.github.com/tauri-apps/tauri/pull/8392)) Fix a regression where typescript could not find types when using `"moduleResolution": "node"`

## \[1.5.2]

### Bug Fixes

- [`50462702`](https://www.github.com/tauri-apps/tauri/commit/504627027303ef5a0e855aab2abea64c6964223b)([#8267](https://www.github.com/tauri-apps/tauri/pull/8267)) Add top-level `main`, `module` and `types` fields in `package.json` to be compliant with typescripts's `"moduleResolution": "node"`
- [`14544e4b`](https://www.github.com/tauri-apps/tauri/commit/14544e4b87269c06c89fed3647d80f492e0a1d34)([#8219](https://www.github.com/tauri-apps/tauri/pull/8219)) Avoid crashing in `clearMocks`

## \[1.5.1]

### New Features

- [`2b0212af`](https://www.github.com/tauri-apps/tauri/commit/2b0212af49c386e52bb2357381813d6d435ec4af)([#7961](https://www.github.com/tauri-apps/tauri/pull/7961)) Add `mockConvertFileSrc` in `mocks` module, to mock `convertFileSrc` function.

## \[1.5.0]

### New Features

- [`6c408b73`](https://www.github.com/tauri-apps/tauri/commit/6c408b736c7aa2a0a91f0a40d45a2b7a7dedfe78)([#7269](https://www.github.com/tauri-apps/tauri/pull/7269)) Add option to specify notification sound.

### Enhancements

- [`58d6b899`](https://www.github.com/tauri-apps/tauri/commit/58d6b899e21d37bb42810890d289deb57f2273bd)([#7636](https://www.github.com/tauri-apps/tauri/pull/7636)) Add `append` option to `FsOptions` in the `fs` JS module, used in `writeTextFile` and `writeBinaryFile`, to be able to append to existing files instead of overwriting it.

### Bug Fixes

- [`2eab1505`](https://www.github.com/tauri-apps/tauri/commit/2eab1505632ff71431d4c31c49b5afc78fa5b9dd)([#7394](https://www.github.com/tauri-apps/tauri/pull/7394)) Fix `Body.form` static not reading and sending entries of type `Blob` (including subclasses such as `File`)

## \[1.4.0]

### New Features

- [`359058ce`](https://www.github.com/tauri-apps/tauri/commit/359058cecca44a9c30b65140c44a8bb3a6dd3be8)([#5939](https://www.github.com/tauri-apps/tauri/pull/5939)) Add `locale` function in the `os` module to get the system locale.
- [`c4d6fb4b`](https://www.github.com/tauri-apps/tauri/commit/c4d6fb4b1ea8acf02707a9fe5dcab47c1c5bae7b)([#2353](https://www.github.com/tauri-apps/tauri/pull/2353)) Added the `maximizable`, `minimizable` and `closable` fields on `WindowOptions`.
- [`c4d6fb4b`](https://www.github.com/tauri-apps/tauri/commit/c4d6fb4b1ea8acf02707a9fe5dcab47c1c5bae7b)([#2353](https://www.github.com/tauri-apps/tauri/pull/2353)) Added the `setMaximizable`, `setMinimizable`, `setClosable`, `isMaximizable`, `isMinimizable` and `isClosable` methods.
- [`000104bc`](https://www.github.com/tauri-apps/tauri/commit/000104bc3bc0c9ff3d20558ab9cf2080f126e9e0)([#6472](https://www.github.com/tauri-apps/tauri/pull/6472)) Add `WebviewWindow.is_focused` and `WebviewWindow.getFocusedWindow` getters.

## \[1.3.0]

- Return correct type for `event.payload` in `onResized` and `onMoved` window event handlers.
  - [0b46637e](https://www.github.com/tauri-apps/tauri/commit/0b46637ebaba54403afa32a1cb466f09df2db999) fix(api): construct correct object for onResized and onMoved, closes [#6507](https://www.github.com/tauri-apps/tauri/pull/6507) ([#6509](https://www.github.com/tauri-apps/tauri/pull/6509)) on 2023-04-03
- Added the `WindowOptions::contentProtected` option and `WebviewWindow#setContentProtected` to change it at runtime.
  - [4ab5545b](https://www.github.com/tauri-apps/tauri/commit/4ab5545b7a831c549f3c65e74de487ede3ab7ce5) feat: add content protection api, closes [#5132](https://www.github.com/tauri-apps/tauri/pull/5132) ([#5513](https://www.github.com/tauri-apps/tauri/pull/5513)) on 2022-12-13
- Allow setting the text of the dialog buttons.
  - [00e1efaa](https://www.github.com/tauri-apps/tauri/commit/00e1efaa9b33876d41dd360624b69971e70d3856) feat: customize button texts of message dialog ([#4383](https://www.github.com/tauri-apps/tauri/pull/4383)) on 2022-12-28
- Add `is_minimized()` window method.
  - [62144ef3](https://www.github.com/tauri-apps/tauri/commit/62144ef3be63b237869e511826edfb938e2c7174) feat: add is_minimized (fix [#3878](https://www.github.com/tauri-apps/tauri/pull/3878)) ([#5618](https://www.github.com/tauri-apps/tauri/pull/5618)) on 2022-12-13
- Add `title` getter on window.
  - [233e43b0](https://www.github.com/tauri-apps/tauri/commit/233e43b0c34fada1ca025378533a0b76931a6540) feat: add `title` getter on window, closes [#5023](https://www.github.com/tauri-apps/tauri/pull/5023) ([#5515](https://www.github.com/tauri-apps/tauri/pull/5515)) on 2022-12-13

## \[1.2.0]

- Added the `acceptFirstMouse` window option.
  - [95f467ad](https://www.github.com/tauri-apps/tauri/commit/95f467add51448319983c54e2f382c7c09fb72d6) feat(core): add window `accept_first_mouse` option, closes [#5347](https://www.github.com/tauri-apps/tauri/pull/5347) ([#5374](https://www.github.com/tauri-apps/tauri/pull/5374)) on 2022-10-17
- Fix incorrect return type on `fs/exists`
  - [ca3cd8b3](https://www.github.com/tauri-apps/tauri/commit/ca3cd8b3d11beb9b6102da40b7d27f6dbe6cd2d0) fix(api): fs/exists return type previously set to void when it should be boolean ([#5252](https://www.github.com/tauri-apps/tauri/pull/5252)) on 2022-09-29
- Initialize `Monitor` instances with the correct classes for `position` and `size` fields instead of plain object.
  - [6f41a271](https://www.github.com/tauri-apps/tauri/commit/6f41a2712445ac41a5ed84bbcd40af3b76c8b1d8) fix(api.js): fix `Monitor` initialization, closes [#4672](https://www.github.com/tauri-apps/tauri/pull/4672) ([#5314](https://www.github.com/tauri-apps/tauri/pull/5314)) on 2022-09-30
- **Breaking change:** Node.js v12 is no longer supported.
  - [1129f4f5](https://www.github.com/tauri-apps/tauri/commit/1129f4f575dd02f746abe8e66472c88c8f9fe63d) refactor: simplify api.js bundling ([#4277](https://www.github.com/tauri-apps/tauri/pull/4277)) on 2022-10-04
- Add new app-specific `BaseDirectory` enum variants `AppConfig`, `AppData`, `AppLocalData`, `AppCache` and `AppLog` along with equivalent functions in `path` module and deprecated ambiguous variants `Log` and `App` along with their equivalent functions in `path` module.
  - [5d89905e](https://www.github.com/tauri-apps/tauri/commit/5d89905e39ce0e6eaaec50a693679335449edb32) feat(api): add app-specific directory APIs, closes [#5263](https://www.github.com/tauri-apps/tauri/pull/5263) ([#5272](https://www.github.com/tauri-apps/tauri/pull/5272)) on 2022-09-28
- Fix `dialog.save` return type
  - [8357ce5b](https://www.github.com/tauri-apps/tauri/commit/8357ce5b2efdd6f92c7944822542e48ba0e303ce) Fix dialog.save return type ([#5373](https://www.github.com/tauri-apps/tauri/pull/5373)) on 2022-10-08
- Added support to `FormData` on the `Body.form` function.
  - [aa119f28](https://www.github.com/tauri-apps/tauri/commit/aa119f28364f8ffbc64c6bcdfc77483613076a20) feat(api): add FormData support on Body.form, closes [#5545](https://www.github.com/tauri-apps/tauri/pull/5545) ([#5546](https://www.github.com/tauri-apps/tauri/pull/5546)) on 2022-11-04
- Added `show` and `hide` methods on the `app` module.
  - [39bf895b](https://www.github.com/tauri-apps/tauri/commit/39bf895b73ec6b53f5758815396ba85dda6b9c67) feat(macOS): Add application `show` and `hide` methods ([#3689](https://www.github.com/tauri-apps/tauri/pull/3689)) on 2022-10-03
- Added `tabbingIdentifier` window option for macOS.
  - [4137ab44](https://www.github.com/tauri-apps/tauri/commit/4137ab44a81d739556cbc7583485887e78952bf1) feat(macos): add `tabbing_identifier` option, closes [#2804](https://www.github.com/tauri-apps/tauri/pull/2804), [#3912](https://www.github.com/tauri-apps/tauri/pull/3912) ([#5399](https://www.github.com/tauri-apps/tauri/pull/5399)) on 2022-10-19
- Added `tabbing_identifier` to the window builder on macOS.
  - [4137ab44](https://www.github.com/tauri-apps/tauri/commit/4137ab44a81d739556cbc7583485887e78952bf1) feat(macos): add `tabbing_identifier` option, closes [#2804](https://www.github.com/tauri-apps/tauri/pull/2804), [#3912](https://www.github.com/tauri-apps/tauri/pull/3912) ([#5399](https://www.github.com/tauri-apps/tauri/pull/5399)) on 2022-10-19
- Added the `user_agent` option when creating a window.
  - [a6c94119](https://www.github.com/tauri-apps/tauri/commit/a6c94119d8545d509723b147c273ca5edfe3729f) feat(core): expose user_agent to window config ([#5317](https://www.github.com/tauri-apps/tauri/pull/5317)) on 2022-10-02

## \[1.1.0]

- Update `mockIPC()` handler signature to allow async handler functions.
  - [4fa968dc](https://www.github.com/tauri-apps/tauri/commit/4fa968dc0e74b5206bfcd54e704d180c16b67b08) fix(api): add async `mockIPC()` handler signature ([#5056](https://www.github.com/tauri-apps/tauri/pull/5056)) on 2022-08-26
- Improve shell's `Command`, `Command.stdout` and `Command.stderr` events with new `once`, `off`, `listenerCount`, `prependListener`, `prependOnceListener` and `removeAllListeners` functions.
  - [aa9f1243](https://www.github.com/tauri-apps/tauri/commit/aa9f1243e6c1629972a82e469f20c8399741740e) Improved EventEmitter for tauri api shell ([#4697](https://www.github.com/tauri-apps/tauri/pull/4697)) on 2022-07-26
- Added the `encoding` option to the `Command` options.
  - [d8cf9f9f](https://www.github.com/tauri-apps/tauri/commit/d8cf9f9fcd617ac24fa418952fd4a32c08804f5c) Command support for specified character encoding, closes [#4644](https://www.github.com/tauri-apps/tauri/pull/4644) ([#4772](https://www.github.com/tauri-apps/tauri/pull/4772)) on 2022-07-28
- Add `exists` function to the fs module.
  - [3c62dbc9](https://www.github.com/tauri-apps/tauri/commit/3c62dbc902c904d35a7472ce72a969084c95fbbe) feat(api): Add `exists` function to the fs module. ([#5060](https://www.github.com/tauri-apps/tauri/pull/5060)) on 2022-09-15

## \[1.0.2]

- Added helper functions to listen to updater and window events.
  - [b02fc90f](https://www.github.com/tauri-apps/tauri/commit/b02fc90f450ff9e9d8a35ee55dc1beced4957869) feat(api): add abstractions to updater and window event listeners ([#4569](https://www.github.com/tauri-apps/tauri/pull/4569)) on 2022-07-05
- Add support to `ArrayBuffer` in `Body.bytes` and `writeBinaryFile`.
  - [92aca55a](https://www.github.com/tauri-apps/tauri/commit/92aca55a6f1f899d5c0c3a6aae9ac9cb0a7e9a86) feat(api): add support to ArrayBuffer ([#4579](https://www.github.com/tauri-apps/tauri/pull/4579)) on 2022-07-05
- Use `toString()` on message/confirm/ask dialogs title and message values.
  - [b8cd2a79](https://www.github.com/tauri-apps/tauri/commit/b8cd2a7993cd2aa5b71b30c545b3307245d254bf) feat(api): call `toString()` on dialog title and message, closes [#4583](https://www.github.com/tauri-apps/tauri/pull/4583) ([#4588](https://www.github.com/tauri-apps/tauri/pull/4588)) on 2022-07-04
- Remove the `type-fest` dependency, changing the OS types to the specific enum instead of allowing any string.
  - [d5e910eb](https://www.github.com/tauri-apps/tauri/commit/d5e910ebcc6c8d7f055ab0691286722b140ffcd4) chore(api): remove `type-fest` ([#4605](https://www.github.com/tauri-apps/tauri/pull/4605)) on 2022-07-06

## \[1.0.1]

- Fixes the `writeBinaryFile` sending an empty file contents when only the first argument is passed.
  - [ea43cf52](https://www.github.com/tauri-apps/tauri/commit/ea43cf52db8541d20a6397ef3ecd40f0f2bd6113) fix(api): `writeBinaryFile` sends an empty contents with only one arg ([#4368](https://www.github.com/tauri-apps/tauri/pull/4368)) on 2022-06-16

## \[1.0.0]

- Allow choosing multiple folders in `dialog.open`.
  - [4e51dce6](https://www.github.com/tauri-apps/tauri/commit/4e51dce6ca21c7664de779bc78a04be1051371f7) fix: dialog open supports multiple dirs, fixes [#4091](https://www.github.com/tauri-apps/tauri/pull/4091) ([#4354](https://www.github.com/tauri-apps/tauri/pull/4354)) on 2022-06-15
- Upgrade to `stable`!
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[1.0.0-rc.7]

- Fix `FilePart` usage in `http.Body.form` by renaming the `value` property to `file`.
  - [55f89d5f](https://www.github.com/tauri-apps/tauri/commit/55f89d5f9d429252ad3fd557b1d6233b256495e0) fix(api): Rename FormPart `value` to `file` to match docs and endpoint ([#4307](https://www.github.com/tauri-apps/tauri/pull/4307)) on 2022-06-09
- Fixes a memory leak in the command system.
  - [f72cace3](https://www.github.com/tauri-apps/tauri/commit/f72cace36821dc675a6d25268ae85a21bdbd6296) fix: never remove ipc callback & mem never be released ([#4274](https://www.github.com/tauri-apps/tauri/pull/4274)) on 2022-06-05
- The notification's `isPermissionGranted` function now returns `boolean` instead of `boolean | null`. The response is never `null` because we won't check the permission for now, always returning `true` instead.
  - [f482b094](https://www.github.com/tauri-apps/tauri/commit/f482b0942276e9402ab3725957535039bacb4fef) fix: remove notification permission prompt ([#4302](https://www.github.com/tauri-apps/tauri/pull/4302)) on 2022-06-09
- Added the `resolveResource` API to the path module.
  - [7bba8db8](https://www.github.com/tauri-apps/tauri/commit/7bba8db83ead92e9bd9c4be7863742e71ac47513) feat(api): add `resolveResource` API to the path module ([#4234](https://www.github.com/tauri-apps/tauri/pull/4234)) on 2022-05-29
- Renamed `writeFile` to `writeTextFile` but kept the original function for backwards compatibility.
  - [3f998ca2](https://www.github.com/tauri-apps/tauri/commit/3f998ca29445a349489078a74dd068e157a4d68e) feat(api): add `writeTextFile` and `(path, contents, options)` overload ([#4228](https://www.github.com/tauri-apps/tauri/pull/4228)) on 2022-05-29
- Added `(path, contents[, options])` overload to the `writeTextFile` and `writeBinaryFile` APIs.
  - [3f998ca2](https://www.github.com/tauri-apps/tauri/commit/3f998ca29445a349489078a74dd068e157a4d68e) feat(api): add `writeTextFile` and `(path, contents, options)` overload ([#4228](https://www.github.com/tauri-apps/tauri/pull/4228)) on 2022-05-29

## \[1.0.0-rc.6]

- Expose option to set the dialog type.
  - [f46175d5](https://www.github.com/tauri-apps/tauri/commit/f46175d5d46fa3eae66ad2415a0eb1efb7d31da2) feat(core): expose option to set dialog type, closes [#4183](https://www.github.com/tauri-apps/tauri/pull/4183) ([#4187](https://www.github.com/tauri-apps/tauri/pull/4187)) on 2022-05-21
- Expose `title` option in the message dialog API.
  - [ae99f991](https://www.github.com/tauri-apps/tauri/commit/ae99f991674d77c322a2240d10ed4b78ed2f4d4b) feat(core): expose message dialog's title option, ref [#4183](https://www.github.com/tauri-apps/tauri/pull/4183) ([#4186](https://www.github.com/tauri-apps/tauri/pull/4186)) on 2022-05-21

## \[1.0.0-rc.5]

- Fixes the type of `http > connectTimeout`.
  - [f3c5ca89](https://www.github.com/tauri-apps/tauri/commit/f3c5ca89e79d429183c4e15a9e7cebada2b493a0) fix(core): http api `connect_timeout` deserialization, closes [#4004](https://www.github.com/tauri-apps/tauri/pull/4004) ([#4006](https://www.github.com/tauri-apps/tauri/pull/4006)) on 2022-04-29

## \[1.0.0-rc.4]

- Encode the file path in the `convertFileSrc` function.
  - [42e8d9cf](https://www.github.com/tauri-apps/tauri/commit/42e8d9cf925089e9ad591198ee04b0cc0a0eed48) fix(api): encode file path in `convertFileSrc` function, closes [#3841](https://www.github.com/tauri-apps/tauri/pull/3841) ([#3846](https://www.github.com/tauri-apps/tauri/pull/3846)) on 2022-04-02
- Added `theme` getter to `WebviewWindow`.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21
- Added `theme` field to `WindowOptions`.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21
- Added the `setCursorGrab`, `setCursorVisible`, `setCursorIcon` and `setCursorPosition` methods to the `WebviewWindow` class.
  - [c54ddfe9](https://www.github.com/tauri-apps/tauri/commit/c54ddfe9338e7eb90b4d5b02dfde687d432d5bc1) feat: expose window cursor APIs, closes [#3888](https://www.github.com/tauri-apps/tauri/pull/3888) [#3890](https://www.github.com/tauri-apps/tauri/pull/3890) ([#3935](https://www.github.com/tauri-apps/tauri/pull/3935)) on 2022-04-21
- **Breaking change:** The process Command API stdio lines now includes the trailing `\r`.
  - [b5622882](https://www.github.com/tauri-apps/tauri/commit/b5622882cf3748e1e5a90915f415c0cd922aaaf8) fix(cli): exit on non-compilation Cargo errors, closes [#3930](https://www.github.com/tauri-apps/tauri/pull/3930) ([#3942](https://www.github.com/tauri-apps/tauri/pull/3942)) on 2022-04-22
- Added the `tauri://theme-changed` event.
  - [4cebcf6d](https://www.github.com/tauri-apps/tauri/commit/4cebcf6da7cad1953e0f01b426afac3b5ef1f81e) feat: expose theme APIs, closes [#3903](https://www.github.com/tauri-apps/tauri/pull/3903) ([#3937](https://www.github.com/tauri-apps/tauri/pull/3937)) on 2022-04-21

## \[1.0.0-rc.3]

- Properly define the `appWindow` type.
  - [1deeb03e](https://www.github.com/tauri-apps/tauri/commit/1deeb03ef6c7cbea8cf585864424a3d66f184a02) fix(api.js): appWindow shown as type `any`, fixes [#3747](https://www.github.com/tauri-apps/tauri/pull/3747) ([#3772](https://www.github.com/tauri-apps/tauri/pull/3772)) on 2022-03-24
- Added `Temp` to the `BaseDirectory` enum.
  - [266156a0](https://www.github.com/tauri-apps/tauri/commit/266156a0b08150b21140dd552c8bc252fe413cdd) feat(core): add `BaseDirectory::Temp` and `$TEMP` variable ([#3763](https://www.github.com/tauri-apps/tauri/pull/3763)) on 2022-03-24

## \[1.0.0-rc.2]

- Do not crash if `__TAURI_METADATA__` is not set, log an error instead.
  - [9cb1059a](https://www.github.com/tauri-apps/tauri/commit/9cb1059aa3f81521ccc6da655243acfe0327cd98) fix(api): do not throw an exception if **TAURI_METADATA** is not set, fixes [#3554](https://www.github.com/tauri-apps/tauri/pull/3554) ([#3572](https://www.github.com/tauri-apps/tauri/pull/3572)) on 2022-03-03
- Reimplement endpoint to read file as string for performance.
  - [834ccc51](https://www.github.com/tauri-apps/tauri/commit/834ccc51539401d36a7dfa1c0982623c9c446a4c) feat(core): reimplement `readTextFile` for performance ([#3631](https://www.github.com/tauri-apps/tauri/pull/3631)) on 2022-03-07
- Fixes a regression on the `unlisten` command.
  - [76c791bd](https://www.github.com/tauri-apps/tauri/commit/76c791bd2b836d2055410e37e71716172a3f81ef) fix(core): regression on the unlisten function ([#3623](https://www.github.com/tauri-apps/tauri/pull/3623)) on 2022-03-06

## \[1.0.0-rc.1]

- Provide functions to mock IPC calls during testing and static site generation.
  - [7e04c072](https://www.github.com/tauri-apps/tauri/commit/7e04c072c4ee2278c648f44575c6c4710ac047f3) feat: add mock functions for testing and SSG ([#3437](https://www.github.com/tauri-apps/tauri/pull/3437)) on 2022-02-14
  - [6f5ed2e6](https://www.github.com/tauri-apps/tauri/commit/6f5ed2e69cb7ffa0d5c8eb5a744fbf94ed6010d4) fix: change file on 2022-02-14

## \[1.0.0-rc.0]

- Add `fileDropEnabled` property to `WindowOptions` so you can now disable it when creating windows from js.

  - [1bfc32a3](https://www.github.com/tauri-apps/tauri/commit/1bfc32a3b2f31b962ce8a5c611b60cb008360923) fix(api.js): add `fileDropEnabled` to `WindowOptions`, closes [#2968](https://www.github.com/tauri-apps/tauri/pull/2968) ([#2989](https://www.github.com/tauri-apps/tauri/pull/2989)) on 2021-12-09

- Add `logDir` function to the `path` module to access the suggested log directory.
  Add `BaseDirectory.Log` to the `fs` module.

  - [acbb3ae7](https://www.github.com/tauri-apps/tauri/commit/acbb3ae7bb0165846b9456aea103269f027fc548) feat: add Log directory ([#2736](https://www.github.com/tauri-apps/tauri/pull/2736)) on 2021-10-16
  - [62c7a8ad](https://www.github.com/tauri-apps/tauri/commit/62c7a8ad30fd3031b8679960590e5ef3eef8e4da) chore(covector): prepare for `rc` release ([#3376](https://www.github.com/tauri-apps/tauri/pull/3376)) on 2022-02-10

- Expose `ask`, `message` and `confirm` APIs on the dialog module.

  - [e98c1af4](https://www.github.com/tauri-apps/tauri/commit/e98c1af44279a5ff6c8a6f0a506ecc219c9f77af) feat(core): expose message dialog APIs, fix window.confirm, implement HasRawWindowHandle for Window, closes [#2535](https://www.github.com/tauri-apps/tauri/pull/2535) ([#2700](https://www.github.com/tauri-apps/tauri/pull/2700)) on 2021-10-02

- Event `emit` now automatically serialize non-string types.

  - [06000996](https://www.github.com/tauri-apps/tauri/commit/060009969627890fa9018e2f1105bad13299394c) feat(api): support unknown types for event emit payload, closes [#2929](https://www.github.com/tauri-apps/tauri/pull/2929) ([#2964](https://www.github.com/tauri-apps/tauri/pull/2964)) on 2022-01-07

- Fix `http.fetch` throwing error if the response is successful but the body is empty.

  - [50c63900](https://www.github.com/tauri-apps/tauri/commit/50c63900c7313064037e2ceb798a6432fcd1bcda) fix(api.js): fix `http.fetch` throwing error if response body is empty, closes [#2831](https://www.github.com/tauri-apps/tauri/pull/2831) ([#3008](https://www.github.com/tauri-apps/tauri/pull/3008)) on 2021-12-09

- Add `title` option to file open/save dialogs.

  - [e1d6a6e6](https://www.github.com/tauri-apps/tauri/commit/e1d6a6e6445637723e2331ca799a662e720e15a8) Create api-file-dialog-title.md ([#3235](https://www.github.com/tauri-apps/tauri/pull/3235)) on 2022-01-16
  - [62c7a8ad](https://www.github.com/tauri-apps/tauri/commit/62c7a8ad30fd3031b8679960590e5ef3eef8e4da) chore(covector): prepare for `rc` release ([#3376](https://www.github.com/tauri-apps/tauri/pull/3376)) on 2022-02-10

- Fix `os.platform` returning `macos` and `windows` instead of `darwin` and `win32`.

  - [3924c3d8](https://www.github.com/tauri-apps/tauri/commit/3924c3d85365df30b376a1ec6c2d933460d66af0) fix(api.js): fix `os.platform` return on macos and windows, closes [#2698](https://www.github.com/tauri-apps/tauri/pull/2698) ([#2699](https://www.github.com/tauri-apps/tauri/pull/2699)) on 2021-10-02

- The `formatCallback` helper function now returns a number instead of a string.

  - [a48b8b18](https://www.github.com/tauri-apps/tauri/commit/a48b8b18d428bcc404d489daa690bbefe1f57311) feat(core): validate callbacks and event names \[TRI-038] \[TRI-020] ([#21](https://www.github.com/tauri-apps/tauri/pull/21)) on 2022-01-09

- Added `rawHeaders` to `http > Response`.

  - [b7a2345b](https://www.github.com/tauri-apps/tauri/commit/b7a2345b06ca0306988b4ba3d3deadd449e65af9) feat(core): add raw headers to HTTP API, closes [#2695](https://www.github.com/tauri-apps/tauri/pull/2695) ([#3053](https://www.github.com/tauri-apps/tauri/pull/3053)) on 2022-01-07

- Removed the `currentDir` API from the `path` module.

  - [a08509c6](https://www.github.com/tauri-apps/tauri/commit/a08509c641f43695e25944a2dd47697b18cd83e2) fix(api): remove `currentDir` API from the `path` module on 2022-02-04

- Remove `.ts` files on the published package.

  - [0f321ac0](https://www.github.com/tauri-apps/tauri/commit/0f321ac08d56412edd5bc9d166201fbc95d887d8) fix(api): do not ship TS files, closes [#2598](https://www.github.com/tauri-apps/tauri/pull/2598) ([#2645](https://www.github.com/tauri-apps/tauri/pull/2645)) on 2021-09-23

- **Breaking change:** Replaces all usages of `number[]` with `Uint8Array` to be closer aligned with the wider JS ecosystem.

  - [9b19a805](https://www.github.com/tauri-apps/tauri/commit/9b19a805aa8efa64b22f2dfef193a144b8e0cee3) fix(api.js) Replace `number[]`with `Uint8Array`. fixes [#3306](https://www.github.com/tauri-apps/tauri/pull/3306) ([#3305](https://www.github.com/tauri-apps/tauri/pull/3305)) on 2022-02-05

- `WindowManager` methods `innerPosition` `outerPosition` now correctly return instance of `PhysicalPosition`.
  `WindowManager` methods `innerSize` `outerSize` now correctly return instance of `PhysicalSize`.

  - [cc8b1468](https://www.github.com/tauri-apps/tauri/commit/cc8b1468c821df53ceb771061c919409a9c80978) Fix(api): Window size and position returning wrong class (fix: [#2599](https://www.github.com/tauri-apps/tauri/pull/2599)) ([#2621](https://www.github.com/tauri-apps/tauri/pull/2621)) on 2021-09-22

- Change the `event` field of the `Event` interface to type `EventName` instead of `string`.

  - [b5d9bcb4](https://www.github.com/tauri-apps/tauri/commit/b5d9bcb402380abc86ae1fa1a77c629af2275f9d) Consistent event name usage ([#3228](https://www.github.com/tauri-apps/tauri/pull/3228)) on 2022-01-15
  - [62c7a8ad](https://www.github.com/tauri-apps/tauri/commit/62c7a8ad30fd3031b8679960590e5ef3eef8e4da) chore(covector): prepare for `rc` release ([#3376](https://www.github.com/tauri-apps/tauri/pull/3376)) on 2022-02-10

- Now `resolve()`, `join()` and `normalize()` from the `path` module, won't throw errors if the path doesn't exist, which matches NodeJS behavior.

  - [fe381a0b](https://www.github.com/tauri-apps/tauri/commit/fe381a0bde86ebf4014007f6e21af4c1a9e58cef) fix: `join` no longer cares if path doesn't exist, closes [#2499](https://www.github.com/tauri-apps/tauri/pull/2499) ([#2548](https://www.github.com/tauri-apps/tauri/pull/2548)) on 2021-09-21

- Fixes the dialog `defaultPath` usage on Linux.

  - [2212bd5d](https://www.github.com/tauri-apps/tauri/commit/2212bd5d75146f5a2df27cc2157a057642f626da) fix: dialog default path on Linux, closes [#3091](https://www.github.com/tauri-apps/tauri/pull/3091) ([#3123](https://www.github.com/tauri-apps/tauri/pull/3123)) on 2021-12-27

- Fixes `window.label` property returning null instead of the actual label.

  - [f5109e0c](https://www.github.com/tauri-apps/tauri/commit/f5109e0c962e3d25404995194968bade1be33b16) fix(api): window label null instead of actual value, closes [#3295](https://www.github.com/tauri-apps/tauri/pull/3295) ([#3332](https://www.github.com/tauri-apps/tauri/pull/3332)) on 2022-02-04

- Remove the `BaseDirectory::Current` enum variant for security reasons.

  - [696dca58](https://www.github.com/tauri-apps/tauri/commit/696dca58a9f8ee127a1cf857eb848e09f5845d18) refactor(core): remove `BaseDirectory::Current` variant on 2022-01-26

- Change `WindowLabel` type to `string`.

  - [f68603ae](https://www.github.com/tauri-apps/tauri/commit/f68603aee4e16500dff9e385b217f5dd8b1b39e8) chore(docs): simplify event system documentation on 2021-09-27

- When building Universal macOS Binaries through the virtual target `universal-apple-darwin`:

- Expect a universal binary to be created by the user

- Ensure that binary is bundled and accessed correctly at runtime

- [3035e458](https://www.github.com/tauri-apps/tauri/commit/3035e4581c161ec7f0bd6d9b42e9015cf1dd1d77) Remove target triple from sidecar bin paths, closes [#3355](https://www.github.com/tauri-apps/tauri/pull/3355) ([#3356](https://www.github.com/tauri-apps/tauri/pull/3356)) on 2022-02-07

## \[1.0.0-beta.8]

- Revert target back to ES5.
  - [657c7dac](https://www.github.com/tauri-apps/tauri/commit/657c7dac734661956b87d021ff531ba530dd92a3) fix(api): revert ES2021 target on 2021-08-23

## \[1.0.0-beta.7]

- Fix missing asset protocol path.Now the protocol is `https://asset.localhost/path/to/file` on Windows. Linux and macOS
  is still `asset://path/to/file`.
  - [994b5325](https://www.github.com/tauri-apps/tauri/commit/994b5325dd385f564b37fe1530c5d798dc925fff) fix: missing asset protocol path ([#2484](https://www.github.com/tauri-apps/tauri/pull/2484)) on 2021-08-23

## \[1.0.0-beta.6]

- `bundle` now exports `clipboard` module so you can `import { clipboard } from "@tauri-apps/api"`.
  - [4f88c3fb](https://www.github.com/tauri-apps/tauri/commit/4f88c3fb94286f3daafb906e3513c9210ecfa76b) fix(api.js): `bundle` now exports `clipboard` mod, closes [#2243](https://www.github.com/tauri-apps/tauri/pull/2243) ([#2244](https://www.github.com/tauri-apps/tauri/pull/2244)) on 2021-07-19
- Fix double window creation
  - [9fbcc024](https://www.github.com/tauri-apps/tauri/commit/9fbcc024542d87f71afd364acdcf2302cf82912c) fix(api.js): fix double window creation, closes [#2284](https://www.github.com/tauri-apps/tauri/pull/2284) ([#2285](https://www.github.com/tauri-apps/tauri/pull/2285)) on 2021-07-23
- Add `os` module which exports `EOL`, `platform()`, `version()`, `type()`, `arch()`, `tempdir()`
  - [05e679a6](https://www.github.com/tauri-apps/tauri/commit/05e679a6d2aca5642c780052bcf1384c49a462de) feat(api.js): add `os` module ([#2299](https://www.github.com/tauri-apps/tauri/pull/2299)) on 2021-07-28
- - Add new nodejs-inspired functions which are `join`, `resolve`, `normalize`, `dirname`, `basename` and `extname`.
- Add `sep` and `delimiter` constants.
- Removed `resolvePath` API, use `resolve` instead.
- [05b9d81e](https://www.github.com/tauri-apps/tauri/commit/05b9d81ee6bcc920defca76cff00178b301fffe8) feat(api.js): add nodejs-inspired functions in `path` module ([#2310](https://www.github.com/tauri-apps/tauri/pull/2310)) on 2021-08-02
- Change target to ES2021.
  - [97bc52ee](https://www.github.com/tauri-apps/tauri/commit/97bc52ee03dec0b67cc1cced23305a4c53e9eb62) Tooling: \[API] Changed target in tsconfig to es6 ([#2362](https://www.github.com/tauri-apps/tauri/pull/2362)) on 2021-08-09
- Add `toggleMaximize()` function to the `WebviewWindow` class.
  - [1a510066](https://www.github.com/tauri-apps/tauri/commit/1a510066732d5f61c88c0ceed1c5f5cc559faf7d) fix(core): `data-tauri-drag-region` didn't respect resizable, closes [#2314](https://www.github.com/tauri-apps/tauri/pull/2314) ([#2316](https://www.github.com/tauri-apps/tauri/pull/2316)) on 2021-08-02
- Fix `@ts-expect` error usage
  - [dd52e738](https://www.github.com/tauri-apps/tauri/commit/dd52e738f1fd323bd8d185d6e650f412eb031200) fix(api.js): fix `@ts-expect-error` usage, closes [#2249](https://www.github.com/tauri-apps/tauri/pull/2249) ([#2250](https://www.github.com/tauri-apps/tauri/pull/2250)) on 2021-07-20
- Fixes file drop events being swapped (`file-drop-hover` on drop and `file-drop` on hover).
  - [c2b0fe1c](https://www.github.com/tauri-apps/tauri/commit/c2b0fe1ce58e54dbcfdb63162ad17d7e6d8774d9) fix(core): fix wrong file drop events ([#2300](https://www.github.com/tauri-apps/tauri/pull/2300)) on 2021-07-31
- Fixes the global bundle UMD code.
  - [268450b1](https://www.github.com/tauri-apps/tauri/commit/268450b1329a4b55f2043890c565a8563f890c3a) fix(api): global bundle broken code, closes [#2289](https://www.github.com/tauri-apps/tauri/pull/2289) ([#2297](https://www.github.com/tauri-apps/tauri/pull/2297)) on 2021-07-26
- - Fixes monitor api not working.
- Fixes window.print() not working on macOS.
- [0f63f5e7](https://www.github.com/tauri-apps/tauri/commit/0f63f5e757873f1787a1ae07ca531340d0d45ec3) fix(api): Fix monitor functions, closes [#2294](https://www.github.com/tauri-apps/tauri/pull/2294) ([#2301](https://www.github.com/tauri-apps/tauri/pull/2301)) on 2021-07-29
- Improve `EventName` type using `type-fest`'s `LiteralUnion`.
  - [8e480297](https://www.github.com/tauri-apps/tauri/commit/8e48029790857b38988da4d291aa7458f51bb265) feat(api): improve `EventName` type definition ([#2379](https://www.github.com/tauri-apps/tauri/pull/2379)) on 2021-08-10
- Update protocol url path with wry 0.12.1 on Windows.
  - [88382fe1](https://www.github.com/tauri-apps/tauri/commit/88382fe147ebcb3f59308cc529e5562a04970876) chore(api): update protocol url path with wry 0.12.1 on Windows ([#2409](https://www.github.com/tauri-apps/tauri/pull/2409)) on 2021-08-13

## \[1.0.0-beta.5]

- Adds `convertFileSrc` helper to the `tauri` module, simplifying the process of using file paths as webview source (`img`, `video`, etc).
  - [51a5cfe4](https://www.github.com/tauri-apps/tauri/commit/51a5cfe4b5e9890fb6f639c9c929657fd747a595) feat(api): add `convertFileSrc` helper ([#2138](https://www.github.com/tauri-apps/tauri/pull/2138)) on 2021-07-02
- You can now use `emit`, `listen` and `once` using the `appWindow` exported by the window module.
  - [5d7626f8](https://www.github.com/tauri-apps/tauri/commit/5d7626f89781a6ebccceb9ab3b2e8335aa7a0392) feat(api): WindowManager extends WebviewWindowHandle, add events docs ([#2146](https://www.github.com/tauri-apps/tauri/pull/2146)) on 2021-07-03
- Allow manipulating a spawned window directly using `WebviewWindow`, which now extends `WindowManager`.
  - [d69b1cf6](https://www.github.com/tauri-apps/tauri/commit/d69b1cf6d7c13297073073d753e30fe1a22a09cb) feat(api): allow managing windows created on JS ([#2154](https://www.github.com/tauri-apps/tauri/pull/2154)) on 2021-07-05

## \[1.0.0-beta.4]

- Add asset custom protocol to access local file system.
  - [ee60e424](https://www.github.com/tauri-apps/tauri/commit/ee60e424221559d3d725716b0003c5566ef2b5cd) feat: asset custom protocol to access local file system ([#2104](https://www.github.com/tauri-apps/tauri/pull/2104)) on 2021-06-28

## \[1.0.0-beta.3]

- Export `Response` and `ResponseType` as value instead of type.
  - [394b6e05](https://www.github.com/tauri-apps/tauri/commit/394b6e0572e7a0a92e103e462a7f603f7d569319) fix(api): http `ResponseType` export type error ([#2065](https://www.github.com/tauri-apps/tauri/pull/2065)) on 2021-06-24

## \[1.0.0-beta.2]

- Export `BaseDirectory` in `path` module
  - [277f5ca5](https://www.github.com/tauri-apps/tauri/commit/277f5ca5a8ae227bbdccee1ad52bdd88b4a5b11b) feat(api): export `BaseDirectory` in `path` module ([#1885](https://www.github.com/tauri-apps/tauri/pull/1885)) on 2021-05-30
- Use `export type` to export TS types, enums and interfaces.
  - [9a662d26](https://www.github.com/tauri-apps/tauri/commit/9a662d2601b01d712c6bd205f8db1b674f56dfa7) fix: Monitor if --isolatedModules is enabled ([#1825](https://www.github.com/tauri-apps/tauri/pull/1825)) on 2021-05-13
  - [612cd8ec](https://www.github.com/tauri-apps/tauri/commit/612cd8ecb8e02954f3696b9e138cbc7d2c228fad) feat(api): finalize `export type` usage ([#1847](https://www.github.com/tauri-apps/tauri/pull/1847)) on 2021-05-17
- Adds `focus?: boolean` to the WindowOptions interface.
  - [5f351622](https://www.github.com/tauri-apps/tauri/commit/5f351622c7812ad1bb56ddb37364ccaa4124c24b) feat(core): add focus API to the WindowBuilder and WindowOptions, [#1737](https://www.github.com/tauri-apps/tauri/pull/1737) on 2021-05-30
- Adds `isDecorated` getter on the window API.
  - [f58a2114](https://www.github.com/tauri-apps/tauri/commit/f58a2114fbfd5307c349f05c88f2e08fd8baa8aa) feat(core): add `is_decorated` Window getter on 2021-05-30
- Adds `isResizable` getter on the window API.
  - [1e8af280](https://www.github.com/tauri-apps/tauri/commit/1e8af280c27f381828d6209722b10e889082fa00) feat(core): add `is_resizable` Window getter on 2021-05-30
- Adds `isVisible` getter on the window API.
  - [36506c96](https://www.github.com/tauri-apps/tauri/commit/36506c967de82bc7ff453d11e6104ecf66d7a588) feat(core): add `is_visible` API on 2021-05-30
- Adds `requestUserAttention` API to the `window` module.
  - [7dcca6e9](https://www.github.com/tauri-apps/tauri/commit/7dcca6e9281182b11ad3d4a79871f09b30b9b419) feat(core): add `request_user_attention` API, closes [#2023](https://www.github.com/tauri-apps/tauri/pull/2023) ([#2026](https://www.github.com/tauri-apps/tauri/pull/2026)) on 2021-06-20
- Adds `setFocus` to the window API.
  - [bb6992f8](https://www.github.com/tauri-apps/tauri/commit/bb6992f888196ca7c87bb2fe74ad2bd8bf393e05) feat(core): add `set_focus` window API, fixes [#1737](https://www.github.com/tauri-apps/tauri/pull/1737) on 2021-05-30
- Adds `setSkipTaskbar` to the window API.
  - [e06aa277](https://www.github.com/tauri-apps/tauri/commit/e06aa277384450cfef617c0e57b0d5d403bb1e7f) feat(core): add `set_skip_taskbar` API on 2021-05-30
- Adds `skipTaskbar?: boolean` to the WindowOptions interface.
  - [5525b03a](https://www.github.com/tauri-apps/tauri/commit/5525b03a78a2232c650043fbd9894ce1553cad41) feat(core): add `skip_taskbar` API to the WindowBuilder/WindowOptions on 2021-05-30
- Adds `center?: boolean` to `WindowOptions` and `center()` API to the `appWindow`.
  - [5cba6eb4](https://www.github.com/tauri-apps/tauri/commit/5cba6eb4d28d53f06855d60d4d0eae6b95233ccf) feat(core): add window `center` API, closes [#1822](https://www.github.com/tauri-apps/tauri/pull/1822) ([#1954](https://www.github.com/tauri-apps/tauri/pull/1954)) on 2021-06-05
- Adds `clipboard` APIs (write and read text).
  - [285bf64b](https://www.github.com/tauri-apps/tauri/commit/285bf64bf9569efb2df904c69c6df405ff0d62e2) feat(core): add clipboard writeText and readText APIs ([#2035](https://www.github.com/tauri-apps/tauri/pull/2035)) on 2021-06-21
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- The `http` APIs now resolve the returned promise when the API call finishes with an error status code.
  - [47f75584](https://www.github.com/tauri-apps/tauri/commit/47f7558417cc654bdb1d018127e8900bc4eac622) fix(core): resolve HTTP API on non-ok status code, fix binary response, closes [#2046](https://www.github.com/tauri-apps/tauri/pull/2046) ([#2053](https://www.github.com/tauri-apps/tauri/pull/2053)) on 2021-06-23
- Improve RPC security by requiring a numeric code to invoke commands. The codes are generated by the Rust side and injected into the app's code using a closure, so external scripts can't access the backend. This change doesn't protect `withGlobalTauri` (`window.__TAURI__`) usage.
  - [160fb052](https://www.github.com/tauri-apps/tauri/commit/160fb0529fd31d755574ae30fbdf01fa221a2acb) feat(core): improve RPC security, closes [#814](https://www.github.com/tauri-apps/tauri/pull/814) ([#2047](https://www.github.com/tauri-apps/tauri/pull/2047)) on 2021-06-22
- Mark the `WebviewWindow` constructor as public.
  - [4aeb936e](https://www.github.com/tauri-apps/tauri/commit/4aeb936e9b60b895d383597dc698ee5d638436f9) fix(api): `WebviewWindow` constructor is public ([#1888](https://www.github.com/tauri-apps/tauri/pull/1888)) on 2021-05-21
- Validate arguments on the window `setLocation`, `setSize`, `setMinSize` and `setMaxSize` API.
  - [7616e6cc](https://www.github.com/tauri-apps/tauri/commit/7616e6cc7bcd49f688b0d00fdc33c94b7b93713d) feat(api): validate window API `size` and `location` arguments ([#1846](https://www.github.com/tauri-apps/tauri/pull/1846)) on 2021-05-17

## \[1.0.0-beta.1]

- Adds `package.json` to the `exports` object.
  - [ab1ea96](https://www.github.com/tauri-apps/tauri/commit/ab1ea964786e1781c922582b059c555b6072f1a0) chore(api): add `package.json` to the `exports` field ([#1807](https://www.github.com/tauri-apps/tauri/pull/1807)) on 2021-05-12

## \[1.0.0-beta.0]

- CommonJS chunks are now properly exported with `.cjs` extension
  - [ddcd923](https://www.github.com/tauri-apps/tauri/commit/ddcd9233bd6f499aa7f22484d6c151b01778bc1b) fix(api): export commonjs chunks with `.cjs` extension, fix [#1625](https://www.github.com/tauri-apps/tauri/pull/1625) ([#1627](https://www.github.com/tauri-apps/tauri/pull/1627)) on 2021-04-26
- Adds `transparent?: boolean` to the `WindowOptions` interface.
  - [08c1c5c](https://www.github.com/tauri-apps/tauri/commit/08c1c5ca5c0ebe17ea98689a5fe3b7e47a98e955) fix(api): missing `transparent` flag on `WindowOptions` ([#1764](https://www.github.com/tauri-apps/tauri/pull/1764)) on 2021-05-10
- Adds `options` argument to the shell command API (`env` and `cwd` configuration).
  - [721e98f](https://www.github.com/tauri-apps/tauri/commit/721e98f175567b360c86f30565ab1b9d08e7cf85) feat(core): add env, cwd to the command API, closes [#1634](https://www.github.com/tauri-apps/tauri/pull/1634) ([#1635](https://www.github.com/tauri-apps/tauri/pull/1635)) on 2021-04-28
- Adds `startDragging` API on the window module.
  - [c31f097](https://www.github.com/tauri-apps/tauri/commit/c31f0978c535f794fffb75a121e69a323e70b06e) refactor: update to wry 0.9 ([#1630](https://www.github.com/tauri-apps/tauri/pull/1630)) on 2021-04-28
- Move `exit` and `relaunch` APIs from `app` to `process` module.
  - [b0bb796](https://www.github.com/tauri-apps/tauri/commit/b0bb796a42e2560233aea47ce6ced54ac238eb53) refactor: rename `command` mod to `process`, move restart_application ([#1667](https://www.github.com/tauri-apps/tauri/pull/1667)) on 2021-04-30
- The window management API was refactored: removed `setX`, `setY`, `setWidth`, `setHeight` APIs, renamed `resize` to `setSize` and the size and position APIs now allow defining both logical and physical values.
  - [6bfac86](https://www.github.com/tauri-apps/tauri/commit/6bfac866a703f1499a64237fb29b2625703f4e22) refactor(core): add window getters, physical & logical sizes/positions ([#1723](https://www.github.com/tauri-apps/tauri/pull/1723)) on 2021-05-05
- Adds window getters.
  - [6bfac86](https://www.github.com/tauri-apps/tauri/commit/6bfac866a703f1499a64237fb29b2625703f4e22) refactor(core): add window getters, physical & logical sizes/positions ([#1723](https://www.github.com/tauri-apps/tauri/pull/1723)) on 2021-05-05

## \[1.0.0-beta-rc.3]

- Fixes distribution of the `@tauri-apps/api` package for older bundlers.
  - [7f998d0](https://www.github.com/tauri-apps/tauri/commit/7f998d08e3ab8823c99190fa283bdfa2c4f2749b) fix(api): distribution ([#1582](https://www.github.com/tauri-apps/tauri/pull/1582)) on 2021-04-22
- Update minimum Node.js version to v12.13.0
  - [1f089fb](https://www.github.com/tauri-apps/tauri/commit/1f089fb4f964c673dcab5784bdf1da2833487a7c) chore: update minimum nodejs version to 12.13.0 ([#1562](https://www.github.com/tauri-apps/tauri/pull/1562)) on 2021-04-21

## \[1.0.0-beta-rc.2]

- TS was wrongly re-exporting the module.
  - [fcb3b48](https://www.github.com/tauri-apps/tauri/commit/fcb3b4857efa17d2a3717f32457e88b24520cc9b) fix: [#1512](https://www.github.com/tauri-apps/tauri/pull/1512) ([#1517](https://www.github.com/tauri-apps/tauri/pull/1517)) on 2021-04-19
  - [ae14a3f](https://www.github.com/tauri-apps/tauri/commit/ae14a3ff51a742b6ab6f76bbfc21f385310f1dc6) fix: [#1517](https://www.github.com/tauri-apps/tauri/pull/1517) had the wrong package reference in the changefile ([#1538](https://www.github.com/tauri-apps/tauri/pull/1538)) on 2021-04-19

## \[1.0.0-beta-rc.1]

- Missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
  - [b2569a7](https://www.github.com/tauri-apps/tauri/commit/b2569a729a3caa88bdba62abc31f0665e1323aaa) fix(js-api): dist ([#1498](https://www.github.com/tauri-apps/tauri/pull/1498)) on 2021-04-15

## \[1.0.0-beta-rc.0]

- Add current working directory to the path api module.
  - [52c2baf](https://www.github.com/tauri-apps/tauri/commit/52c2baf940773cf7c51647fb6f20d0f7df126115) feat: add current working directory to path api module ([#1375](https://www.github.com/tauri-apps/tauri/pull/1375)) on 2021-03-23
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- The shell process spawning API was rewritten and now includes stream access.
  - [3713066](https://www.github.com/tauri-apps/tauri/commit/3713066e451bd30d0cc6f57bb437f08276f4c4ad) refactor(core): rewrite shell execute API, closes [#1229](https://www.github.com/tauri-apps/tauri/pull/1229) ([#1408](https://www.github.com/tauri-apps/tauri/pull/1408)) on 2021-03-31
- The file dialog API now uses [rfd](https://github.com/PolyMeilex/rfd). The filter option is now an array of `{ name: string, extensions: string[] }`.
  - [2326bcd](https://www.github.com/tauri-apps/tauri/commit/2326bcd399411f7f0eabdb7ade910be473adadae) refactor(core): use `nfd` for file dialogs, closes [#1251](https://www.github.com/tauri-apps/tauri/pull/1251) ([#1257](https://www.github.com/tauri-apps/tauri/pull/1257)) on 2021-02-18
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- The HTTP API was improved with client caching and better payload and response types.
  - [a7bc472](https://www.github.com/tauri-apps/tauri/commit/a7bc472e994730071f960d09a12ac85296a080ae) refactor(core): improve HTTP API, closes [#1098](https://www.github.com/tauri-apps/tauri/pull/1098) ([#1237](https://www.github.com/tauri-apps/tauri/pull/1237)) on 2021-02-15
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Use secure RNG on callback function names.
  - [c8992bb](https://www.github.com/tauri-apps/tauri/commit/c8992bb0bfb8eaeae8ebed444719f9c9372d39d4) refactor(api): use secure RNG, closes [#1356](https://www.github.com/tauri-apps/tauri/pull/1356) ([#1398](https://www.github.com/tauri-apps/tauri/pull/1398)) on 2021-03-30
- The invoke function can now be called with the cmd as the first parameter and the args as the second.
  - [427d170](https://www.github.com/tauri-apps/tauri/commit/427d170930ab711fd0ca82f7a73b524d6fdc222f) feat(api/invoke): separate cmd arg ([#1321](https://www.github.com/tauri-apps/tauri/pull/1321)) on 2021-03-04
- Adds a global shortcut API.
  - [855effa](https://www.github.com/tauri-apps/tauri/commit/855effadd9ebfb6bc1a3555ac7fc733f6f766b7a) feat(core): globalShortcut API ([#1232](https://www.github.com/tauri-apps/tauri/pull/1232)) on 2021-02-14
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Added window management and window creation APIs.
  - [a3d6dff](https://www.github.com/tauri-apps/tauri/commit/a3d6dff2163c7a45842253edd81dbc62248dc65d) feat(core): window API ([#1225](https://www.github.com/tauri-apps/tauri/pull/1225)) on 2021-02-13
  - [641374b](https://www.github.com/tauri-apps/tauri/commit/641374b15343518cd835bd5ada811941c65dcf2e) feat(core): window creation at runtime ([#1249](https://www.github.com/tauri-apps/tauri/pull/1249)) on 2021-02-17
