# Changelog

## \[2.5.2]

### Dependencies

- Upgraded to `tauri-utils@2.8.1`

## \[2.5.1]

### Bug Fixes

- [`4b6b8690a`](https://www.github.com/tauri-apps/tauri/commit/4b6b8690ab886ebdf1307951cffbe03e31280baa) ([#14347](https://www.github.com/tauri-apps/tauri/pull/14347) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed an issue that caused docs.rs builds to fail. No user facing changes.

## \[2.5.0]

### Dependencies

- Upgraded to `tauri-utils@2.8.0`
- [`6aa7f2d85`](https://www.github.com/tauri-apps/tauri/commit/6aa7f2d852870aeba1d4dd0e07f8be2bc9b66286) Upgraded to `tauri-utils@2.8.0`

## \[2.4.0]

### New Features

- [`a0113a8c6`](https://www.github.com/tauri-apps/tauri/commit/a0113a8c6471eccc43b3202cc009901a04e41baa) ([#13888](https://www.github.com/tauri-apps/tauri/pull/13888) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added `build::mobile::update_info_plist` to allow a plugin to update the iOS project Info.plist file.

### Dependencies

- Upgraded to `tauri-utils@2.7.0`

## \[2.3.1]

### Dependencies

- Upgraded to `tauri-utils@2.6.0`

## \[2.2.1]

### Dependencies

- Upgraded to `tauri-utils@2.5.0`

## \[2.2.0]

### Dependencies

- Upgraded to `tauri-utils@2.4.0`
- [`48b12b440`](https://www.github.com/tauri-apps/tauri/commit/48b12b440478937c46fdfef9f9d95194be117020) Update to `tauri-utils@2.4.0`

## \[2.1.1]

### Dependencies

- Upgraded to `tauri-utils@2.3.1`

## \[2.1.0]

### New Features

- [`013f8f652`](https://www.github.com/tauri-apps/tauri/commit/013f8f652302f2d49c5ec0a075582033d8b074fb) ([#12890](https://www.github.com/tauri-apps/tauri/pull/12890) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Added `build > removeUnusedCommands` to trigger the build scripts and macros to remove unused commands based on the capabilities you defined. Note this won't be accounting for dynamically added ACLs so make sure to check it when using this.

### Performance Improvements

- [`1cd8f55ee`](https://www.github.com/tauri-apps/tauri/commit/1cd8f55eed326d61860fee62ba2d2f4464bdcfcc) ([#13033](https://www.github.com/tauri-apps/tauri/pull/13033) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Don't ship global `bundle.global.js` if `app > withGlobalTauri` is set to false

### Dependencies

- Upgraded to `tauri-utils@2.3.0`

## \[2.0.5]

### Dependencies

- Upgraded to `tauri-utils@2.2.0`

## \[2.0.4]

### Dependencies

- Upgraded to `tauri-utils@2.1.1`

## \[2.0.3]

### Dependencies

- Upgraded to `tauri-utils@2.1.0`

## \[2.0.2]

### Dependencies

- Upgraded to `tauri-utils@2.0.2`

## \[2.0.1]

### What's Changed

- [`0ab2b3306`](https://www.github.com/tauri-apps/tauri/commit/0ab2b330644b6419f6cee1d5377bfb5cdda2ccf9) ([#11205](https://www.github.com/tauri-apps/tauri/pull/11205) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

### Dependencies

- Upgraded to `tauri-utils@2.0.1`

## \[2.0.0]

### What's Changed

- [`382ed482b`](https://www.github.com/tauri-apps/tauri/commit/382ed482bd08157c39e62f9a0aaad8802f1092cb) Bump MSRV to 1.78.
- [`637285790`](https://www.github.com/tauri-apps/tauri/commit/6372857905ae9c0aedb7f482ddf6cf9f9836c9f2) Promote to v2 stable!

### Dependencies

- Upgraded to `tauri-utils@2.0.0`

## \[2.0.0-rc.13]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.13`

## \[2.0.0-rc.12]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.12`

## \[2.0.0-rc.11]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.11`

## \[2.0.0-rc.10]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.10`

## \[2.0.0-rc.9]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.9`

## \[2.0.0-rc.8]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.8`

## \[2.0.0-rc.7]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.7`

## \[2.0.0-rc.6]

### What's Changed

- [`f4d5241b3`](https://www.github.com/tauri-apps/tauri/commit/f4d5241b377d0f7a1b58100ee19f7843384634ac) ([#10731](https://www.github.com/tauri-apps/tauri/pull/10731) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Update documentation icon path.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.6`

## \[2.0.0-rc.5]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.5`

## \[2.0.0-rc.4]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.4`

## \[2.0.0-rc.3]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.3`

## \[2.0.0-rc.2]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.2`

## \[2.0.0-rc.1]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.1`

## \[2.0.0-rc.0]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-rc.0`

### Breaking Changes

- [`758d28c8a`](https://www.github.com/tauri-apps/tauri/commit/758d28c8a2d5c9567158e339326b765f72da983e) ([#10390](https://www.github.com/tauri-apps/tauri/pull/10390)) Core plugin permissions are now prefixed with `core:`, the `core:default` permission set can now be used and the `core` plugin name is reserved.
  The `tauri migrate` tool will automate the migration process, which involves prefixing all `app`, `event`, `image`, `menu`, `path`, `resources`, `tray`, `webview` and `window` permissions with `core:`.

## \[2.0.0-beta.19]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.19`

## \[2.0.0-beta.18]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.18`

## \[2.0.0-beta.17]

### New Features

- [`8a1ae2dea`](https://www.github.com/tauri-apps/tauri/commit/8a1ae2deaf3086e531ada25b1627f900e2e421fb)([#9843](https://www.github.com/tauri-apps/tauri/pull/9843)) Added an option to use a Xcode project for the iOS plugin instead of a plain SwiftPM project.

### What's Changed

- [`9ac930380`](https://www.github.com/tauri-apps/tauri/commit/9ac930380a5df3fe700e68e75df8684d261ca292)([#9850](https://www.github.com/tauri-apps/tauri/pull/9850)) Emit `cargo:rustc-check-cfg` instruction so Cargo validates custom cfg attributes on Rust 1.80 (or nightly-2024-05-05).

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.17`

## \[2.0.0-beta.16]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.16`

## \[2.0.0-beta.15]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.15`

## \[2.0.0-beta.14]

### Enhancements

- [`bf2635ab6`](https://www.github.com/tauri-apps/tauri/commit/bf2635ab6241a5b82569eafc939046d6e245f3ad)([#9632](https://www.github.com/tauri-apps/tauri/pull/9632)) Improve the error message that is shown when the `links` property is missing from a Tauri Plugin.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.14`

## \[2.0.0-beta.13]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.13`

## \[2.0.0-beta.12]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.12`

## \[2.0.0-beta.11]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.11`

## \[2.0.0-beta.10]

### New Features

- [`e227fe02f`](https://www.github.com/tauri-apps/tauri/commit/e227fe02f986e145c0731a64693e1c830a9eb5b0)([#9156](https://www.github.com/tauri-apps/tauri/pull/9156)) Allow plugins to define (at compile time) JavaScript that are initialized when `withGlobalTauri` is true.
- [`e227fe02f`](https://www.github.com/tauri-apps/tauri/commit/e227fe02f986e145c0731a64693e1c830a9eb5b0)([#9156](https://www.github.com/tauri-apps/tauri/pull/9156)) Added `Builder::global_api_script_path` to define a JavaScript file containing the initialization script for the plugin API bindings when `withGlobalTauri` is used.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.10`

## \[2.0.0-beta.9]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.9`

## \[2.0.0-beta.8]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.8`

## \[2.0.0-beta.7]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.7`

## \[2.0.0-beta.6]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.6`

### Breaking Changes

- [`3657ad82`](https://www.github.com/tauri-apps/tauri/commit/3657ad82f88ce528551d032d521c52eed3f396b4)([#9008](https://www.github.com/tauri-apps/tauri/pull/9008)) Allow defining permissions for the application commands via `tauri_build::Attributes::app_manifest`.

## \[2.0.0-beta.5]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.5`

## \[2.0.0-beta.4]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.4`

## \[2.0.0-beta.3]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.3`

## \[2.0.0-beta.2]

### Enhancements

- [`dd7571a7`](https://www.github.com/tauri-apps/tauri/commit/dd7571a7808676c8063a4983b9c6687dfaf03a09)([#8815](https://www.github.com/tauri-apps/tauri/pull/8815)) Do not generate JSON schema and markdown reference file if the plugin does not define any permissions and delete those files if they exist.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.2`

## \[2.0.0-beta.1]

### Bug Fixes

- [`4e101f80`](https://www.github.com/tauri-apps/tauri/commit/4e101f801657e7d01ce8c22f9c6468067d0caab2)([#8756](https://www.github.com/tauri-apps/tauri/pull/8756)) Rerun build script when a new permission is added.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.1`

## \[2.0.0-beta.0]

### New Features

- [`74a2a603`](https://www.github.com/tauri-apps/tauri/commit/74a2a6036a5e57462f161d728cbd8a6f121028ca)([#8661](https://www.github.com/tauri-apps/tauri/pull/8661)) Implement access control list for IPC usage.
