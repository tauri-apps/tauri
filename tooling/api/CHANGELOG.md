# Changelog

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

- Return correct type for ` event.payload  ` in `onResized` and `onMoved` window event handlers.
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
  - [394b6e05](https://www.github.com/tauri-apps/tauri/commit/394b6e0572e7a0a92e103e462a7f603f7d569319) fix(api): http  `ResponseType` export type error ([#2065](https://www.github.com/tauri-apps/tauri/pull/2065)) on 2021-06-24

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
