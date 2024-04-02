# Changelog

## \[2.0.0-beta.11]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.11`
- Upgraded to `tauri-codegen@2.0.0-beta.11`

## \[2.0.0-beta.10]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.10`
- Upgraded to `tauri-codegen@2.0.0-beta.10`

## \[2.0.0-beta.9]

### New Features

- [`ba0206d8a`](https://www.github.com/tauri-apps/tauri/commit/ba0206d8a30a9b43ec5090dcaabd1a23baa1420c)([#9141](https://www.github.com/tauri-apps/tauri/pull/9141)) The `Context` codegen now accepts a `assets` input to define a custom `tauri::Assets` implementation.

### Dependencies

- Upgraded to `tauri-codegen@2.0.0-beta.9`
- Upgraded to `tauri-utils@2.0.0-beta.9`

## \[2.0.0-beta.8]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.8`
- Upgraded to `tauri-codegen@2.0.0-beta.8`

## \[2.0.0-beta.7]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.7`
- Upgraded to `tauri-codegen@2.0.0-beta.7`

## \[2.0.0-beta.6]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.6`
- Upgraded to `tauri-codegen@2.0.0-beta.6`

## \[2.0.0-beta.5]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.5`
- Upgraded to `tauri-codegen@2.0.0-beta.5`

## \[2.0.0-beta.4]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.4`
- Upgraded to `tauri-codegen@2.0.0-beta.4`

## \[2.0.0-beta.3]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.3`
- Upgraded to `tauri-codegen@2.0.0-beta.3`

## \[2.0.0-beta.2]

### Enhancements

- [`8d16a80d`](https://www.github.com/tauri-apps/tauri/commit/8d16a80d2fb2468667e7987d0cc99dbc7e3b9d0a)([#8802](https://www.github.com/tauri-apps/tauri/pull/8802)) The `generate_context` proc macro now accepts a `capabilities` attribute where the value is an array of file paths that can be [conditionally compiled](https://doc.rust-lang.org/reference/conditional-compilation.html). These capabilities are added to the application along the capabilities defined in the Tauri configuration file.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.2`
- Upgraded to `tauri-codegen@2.0.0-beta.2`

## \[2.0.0-beta.1]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.1`
- Upgraded to `tauri-codegen@2.0.0-beta.1`

## \[2.0.0-beta.0]

### New Features

- [`74a2a603`](https://www.github.com/tauri-apps/tauri/commit/74a2a6036a5e57462f161d728cbd8a6f121028ca)([#8661](https://www.github.com/tauri-apps/tauri/pull/8661)) Implement access control list for IPC usage.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.0`
- Upgraded to `tauri-codegen@2.0.0-beta.0`

## \[2.0.0-alpha.13]

### Enhancements

- [`d621d343`](https://www.github.com/tauri-apps/tauri/commit/d621d3437ce3947175eecf345b2c6d1c4c7ce020)([#8607](https://www.github.com/tauri-apps/tauri/pull/8607)) Added tracing for window startup, plugins, `Window::eval`, events, IPC, updater and custom protocol request handlers behind the `tracing` feature flag.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.13`
- Upgraded to `tauri-codegen@2.0.0-alpha.13`

## \[2.0.0-alpha.12]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.12`
- Upgraded to `tauri-codegen@2.0.0-alpha.12`

## \[2.0.0-alpha.11]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.11`
- Upgraded to `tauri-codegen@2.0.0-alpha.11`
- [`b6ed4ecb`](https://www.github.com/tauri-apps/tauri/commit/b6ed4ecb3730c346c973f1b34aa0de1b7d43ac44)([#7634](https://www.github.com/tauri-apps/tauri/pull/7634)) Update to syn v2.

## \[2.0.0-alpha.10]

### Enhancements

- [`c6c59cf2`](https://www.github.com/tauri-apps/tauri/commit/c6c59cf2373258b626b00a26f4de4331765dd487) Pull changes from Tauri 1.5 release.

### Dependencies

- Upgraded to `tauri-codegen@2.0.0-alpha.10`
- Upgraded to `tauri-utils@2.0.0-alpha.10`

## \[2.0.0-alpha.9]

### New Features

- [`880266a7`](https://www.github.com/tauri-apps/tauri/commit/880266a7f697e1fe58d685de3bb6836ce5251e92)([#8031](https://www.github.com/tauri-apps/tauri/pull/8031)) Bump the MSRV to 1.70.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.9`
- Upgraded to `tauri-codegen@2.0.0-alpha.9`

### Breaking Changes

- [`ebcc21e4`](https://www.github.com/tauri-apps/tauri/commit/ebcc21e4b95f4e8c27639fb1bca545b432f52d5e)([#8057](https://www.github.com/tauri-apps/tauri/pull/8057)) Renamed the beforeDevCommand, beforeBuildCommand and beforeBundleCommand hooks environment variables from `TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG` to `TAURI_ENV_PLATFORM, TAURI_ENV_ARCH, TAURI_ENV_FAMILY, TAURI_ENV_PLATFORM_VERSION, TAURI_ENV_PLATFORM_TYPE and TAURI_ENV_DEBUG` to differentiate the prefix with other CLI environment variables.

## \[2.0.0-alpha.8]

### Enhancements

- [`100d9ede`](https://www.github.com/tauri-apps/tauri/commit/100d9ede35995d9db21d2087dd5606adfafb89a5)([#7802](https://www.github.com/tauri-apps/tauri/pull/7802)) Use `Target` enum from `tauri_utils::platform`.

### Dependencies

- Upgraded to `tauri-codegen@2.0.0-alpha.8`
- Upgraded to `tauri-utils@2.0.0-alpha.8`

## \[2.0.0-alpha.7]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.7`
- Upgraded to `tauri-codegen@2.0.0-alpha.7`

### Breaking Changes

- [`fbeb5b91`](https://www.github.com/tauri-apps/tauri/commit/fbeb5b9185baeda19e865228179e3e44c165f1d9)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Moved `tauri::api::ipc` to `tauri::ipc` and refactored all types.

## \[2.0.0-alpha.6]

### Dependencies

- Updated to latest `tauri-utils`

## \[2.0.0-alpha.5]

- [`7a4b1fb9`](https://www.github.com/tauri-apps/tauri/commit/7a4b1fb96da475053c61960f362bbecf18cd00d4)([#6839](https://www.github.com/tauri-apps/tauri/pull/6839)) Added support to attibutes for each command path in the `generate_handler` macro.
- [`96639ca2`](https://www.github.com/tauri-apps/tauri/commit/96639ca239c9e4f75142fc07868ac46822111cff)([#6749](https://www.github.com/tauri-apps/tauri/pull/6749)) Moved the `shell` functionality to its own plugin in the plugins-workspace repository.
- [`3188f376`](https://www.github.com/tauri-apps/tauri/commit/3188f3764978c6d1452ee31d5a91469691e95094)([#6883](https://www.github.com/tauri-apps/tauri/pull/6883)) Bump the MSRV to 1.65.
- [`9a79dc08`](https://www.github.com/tauri-apps/tauri/commit/9a79dc085870e0c1a5df13481ff271b8c6cc3b78)([#6947](https://www.github.com/tauri-apps/tauri/pull/6947)) Removed the module command macros.

## \[2.0.0-alpha.4]

- Added `android` configuration object under `tauri > bundle`.
  - Bumped due to a bump in tauri-utils.
  - [db4c9dc6](https://www.github.com/tauri-apps/tauri/commit/db4c9dc655e07ee2184fe04571f500f7910890cd) feat(core): add option to configure Android's minimum SDK version ([#6651](https://www.github.com/tauri-apps/tauri/pull/6651)) on 2023-04-07

## \[2.0.0-alpha.3]

- Pull changes from Tauri 1.3 release.
  - [](https://www.github.com/tauri-apps/tauri/commit/undefined)  on undefined

## \[2.0.0-alpha.2]

- Resolve Android package name from single word bundle identifiers.
  - [60a8b07d](https://www.github.com/tauri-apps/tauri/commit/60a8b07dc7c56c9c45331cb57d9afb410e7eadf3) fix: handle single word bundle identifier when resolving Android domain ([#6313](https://www.github.com/tauri-apps/tauri/pull/6313)) on 2023-02-19
- Return `bool` in the invoke handler.
  - [05dad087](https://www.github.com/tauri-apps/tauri/commit/05dad0876842e2a7334431247d49365cee835d3e) feat: initial work for iOS plugins ([#6205](https://www.github.com/tauri-apps/tauri/pull/6205)) on 2023-02-11
- Refactored the implementation of the `mobile_entry_point` macro.
  - [9feab904](https://www.github.com/tauri-apps/tauri/commit/9feab904bf08b5c168d4779c21d0419409a68d30) feat(core): add API to call Android plugin ([#6239](https://www.github.com/tauri-apps/tauri/pull/6239)) on 2023-02-10

## \[2.0.0-alpha.1]

- Refactor mobile environment variables.
  - [dee9460f](https://www.github.com/tauri-apps/tauri/commit/dee9460f9c9bc92e9c638e7691e616849ac2085b) feat: keep CLI alive when iOS app exits, show logs, closes [#5855](https://www.github.com/tauri-apps/tauri/pull/5855) ([#5902](https://www.github.com/tauri-apps/tauri/pull/5902)) on 2022-12-27
- Bump the MSRV to 1.64.
  - [7eb9aa75](https://www.github.com/tauri-apps/tauri/commit/7eb9aa75cfd6a3176d3f566fdda02d88aa529b0f) Update gtk to 0.16 ([#6155](https://www.github.com/tauri-apps/tauri/pull/6155)) on 2023-01-30
- Removed mobile logging initialization, which will be handled by `tauri-plugin-log`.
  - [](https://www.github.com/tauri-apps/tauri/commit/undefined)  on undefined

## \[2.0.0-alpha.0]

- Added the `mobile_entry_point` macro.
  - [98904863](https://www.github.com/tauri-apps/tauri/commit/9890486321c9c79ccfb7c547fafee85b5c3ffa71) feat(core): add `mobile_entry_point` macro ([#4983](https://www.github.com/tauri-apps/tauri/pull/4983)) on 2022-08-21
- First mobile alpha release!
  - [fa3a1098](https://www.github.com/tauri-apps/tauri/commit/fa3a10988a03aed1b66fb17d893b1a9adb90f7cd) feat(ci): prepare 2.0.0-alpha.0 ([#5786](https://www.github.com/tauri-apps/tauri/pull/5786)) on 2022-12-08

## \[1.4.3]

### Dependencies

- Upgraded to `tauri-utils@1.5.2`
- Upgraded to `tauri-codegen@1.4.2`

## \[1.4.2]

### Enhancements

- [`5e05236b`](https://www.github.com/tauri-apps/tauri/commit/5e05236b4987346697c7caae0567d3c50714c198)([#8289](https://www.github.com/tauri-apps/tauri/pull/8289)) Added tracing for window startup, plugins, `Window::eval`, events, IPC, updater and custom protocol request handlers behind the `tracing` feature flag.

## \[1.4.1]

### Dependencies

- Upgraded to `tauri-utils@1.5.0`
- Upgraded to `tauri-codegen@1.4.1`

## \[1.4.0]

### Enhancements

- [`d68a25e3`](https://www.github.com/tauri-apps/tauri/commit/d68a25e32e012e57a9e5225b589b9ecbea70a887)([#6124](https://www.github.com/tauri-apps/tauri/pull/6124)) Improve compiler error message when generating an async command that has a reference input and don't return a Result.

## \[1.3.0]

- Bump minimum supported Rust version to 1.60.
  - [5fdc616d](https://www.github.com/tauri-apps/tauri/commit/5fdc616df9bea633810dcb814ac615911d77222c) feat: Use the zbus-backed of notify-rust ([#6332](https://www.github.com/tauri-apps/tauri/pull/6332)) on 2023-03-31

## \[1.2.1]

- Fix `allowlist > app > show/hide` always disabled when `allowlist > app > all: false`.
  - Bumped due to a bump in tauri-utils.
  - [bb251087](https://www.github.com/tauri-apps/tauri/commit/bb2510876d0bdff736d36bf3a465cdbe4ad2b90c) fix(core): extend allowlist with `app`'s allowlist, closes [#5650](https://www.github.com/tauri-apps/tauri/pull/5650) ([#5652](https://www.github.com/tauri-apps/tauri/pull/5652)) on 2022-11-18

## \[1.2.0]

- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28

## \[1.1.1]

- Add missing allowlist config for `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position` APIs.
  - Bumped due to a bump in tauri-utils.
  - [c764408d](https://www.github.com/tauri-apps/tauri/commit/c764408da7fae123edd41115bda42fa75a4731d2) fix: Add missing allowlist config for cursor apis, closes [#5207](https://www.github.com/tauri-apps/tauri/pull/5207) ([#5211](https://www.github.com/tauri-apps/tauri/pull/5211)) on 2022-09-16

## \[1.1.0]

- Added support to configuration files in TOML format (Tauri.toml file).
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02

## \[1.0.4]

- Adjust command imports to fix `items_after_statements` Clippy warning.
  - [d3e19e34](https://www.github.com/tauri-apps/tauri/commit/d3e19e3420a023cddc46173a2d1f1e6c5a390a1b) fix(macros): `items_after_statements` Clippy warning, closes [#4639](https://www.github.com/tauri-apps/tauri/pull/4639) on 2022-07-11
- Remove raw identifier (`r#`) prefix from command arguments.
  - [ac72800f](https://www.github.com/tauri-apps/tauri/commit/ac72800fb630738e2502569935ecdc83e3e78055) fix(macros): strip `r#` from command arguments, closes [#4654](https://www.github.com/tauri-apps/tauri/pull/4654) ([#4657](https://www.github.com/tauri-apps/tauri/pull/4657)) on 2022-07-12

## \[1.0.3]

- Add `#[doc(hidden)]` attribute to the `#[command]` generated macro.
  - [d4cdf807](https://www.github.com/tauri-apps/tauri/commit/d4cdf80781a7955ac620fe6d394e82d067178c4d) feat(macros): hide command macro from docs, closes [#4550](https://www.github.com/tauri-apps/tauri/pull/4550) ([#4556](https://www.github.com/tauri-apps/tauri/pull/4556)) on 2022-07-01

## \[1.0.2]

- Expose `platform::windows_version` function.
  - Bumped due to a bump in tauri-utils.
  - [bf764e83](https://www.github.com/tauri-apps/tauri/commit/bf764e83e01e7443e6cc54572001e1c98c357465) feat(utils): expose `windows_version` function ([#4534](https://www.github.com/tauri-apps/tauri/pull/4534)) on 2022-06-30

## \[1.0.1]

- Set the bundle name and app metadata in the Info.plist file in development mode.
  - [38f5db6e](https://www.github.com/tauri-apps/tauri/commit/38f5db6e6a8b496b50d486db6fd8204266de3a69) feat(codegen): fill app metadata in development Info.plist on 2022-06-21
- Set the application icon in development mode on macOS.
  - [307c2ebf](https://www.github.com/tauri-apps/tauri/commit/307c2ebfb68238dacab6088f9c6ba310c727c68c) feat(core): set macOS app icon in development ([#4385](https://www.github.com/tauri-apps/tauri/pull/4385)) on 2022-06-19

## \[1.0.0]

- Upgrade to `stable`!
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[1.0.0-rc.11]

- Read the tray icon path relatively to the config directory.
  - Bumped due to a bump in tauri-codegen.
  - [562e8ca2](https://www.github.com/tauri-apps/tauri/commit/562e8ca23facf1a8e5fa6c8cdf872357d3523a78) fix(codegen): tray icon path is relative to the config directory on 2022-06-15

## \[1.0.0-rc.10]

- **Breaking change:** The `TrayIcon` enum has been removed and now `Icon` is used instead.
  This allows you to use more image formats and use embedded icons on Linux.
  - Bumped due to a bump in tauri-codegen.
  - [4ce8e228](https://www.github.com/tauri-apps/tauri/commit/4ce8e228134cd3f22973b74ef26ca0d165fbbbd9) refactor(core): use `Icon` for tray icons ([#4342](https://www.github.com/tauri-apps/tauri/pull/4342)) on 2022-06-14

## \[1.0.0-rc.9]

- Added a config flag to bundle the media framework used by webkit2gtk `tauri.conf.json > tauri > bundle > appimage > bundleMediaFramework`.
  - Bumped due to a bump in tauri-utils.
  - [d335fae9](https://www.github.com/tauri-apps/tauri/commit/d335fae92cdcbb0ee18aad4e54558914afa3e778) feat(bundler): bundle additional gstreamer files, closes [#4092](https://www.github.com/tauri-apps/tauri/pull/4092) ([#4271](https://www.github.com/tauri-apps/tauri/pull/4271)) on 2022-06-10

## \[1.0.0-rc.8]

- **Breaking change:** `PackageInfo::version` is now a `semver::Version` instead of a `String`.
  - Bumped due to a bump in tauri-utils.
  - [2badbd2d](https://www.github.com/tauri-apps/tauri/commit/2badbd2d7ed51bf33c1b547b4c837b600574bd4a) refactor: force semver versions, change updater `should_install` sig ([#4215](https://www.github.com/tauri-apps/tauri/pull/4215)) on 2022-05-25
  - [a7388e23](https://www.github.com/tauri-apps/tauri/commit/a7388e23c3b9019d48b078cae00a75c74d74d11b) fix(ci): adjust change file to include tauri-utils and tauri-codegen on 2022-05-27

## \[1.0.0-rc.7]

- Allow configuring the display options for the MSI execution allowing quieter updates.
  - Bumped due to a bump in tauri-utils.
  - [9f2c3413](https://www.github.com/tauri-apps/tauri/commit/9f2c34131952ea83c3f8e383bc3cec7e1450429f) feat(core): configure msiexec display options, closes [#3951](https://www.github.com/tauri-apps/tauri/pull/3951) ([#4061](https://www.github.com/tauri-apps/tauri/pull/4061)) on 2022-05-15

## \[1.0.0-rc.6]

- Added `$schema` support to `tauri.conf.json`.
  - Bumped due to a bump in tauri-utils.
  - [715cbde3](https://www.github.com/tauri-apps/tauri/commit/715cbde3842a916c4ebeab2cab348e1774b5c192) feat(config): add `$schema` to `tauri.conf.json`, closes [#3464](https://www.github.com/tauri-apps/tauri/pull/3464) ([#4031](https://www.github.com/tauri-apps/tauri/pull/4031)) on 2022-05-03
- The `dangerous_allow_asset_csp_modification` configuration value has been changed to allow a list of CSP directives to disable.
  - Bumped due to a bump in tauri-utils.
  - [164078c0](https://www.github.com/tauri-apps/tauri/commit/164078c0b719ccbc12e956fecf8a7d4a3c5044e1) feat: allow limiting dangerousDisableAssetCspModification, closes [#3831](https://www.github.com/tauri-apps/tauri/pull/3831) ([#4021](https://www.github.com/tauri-apps/tauri/pull/4021)) on 2022-05-02

## \[1.0.0-rc.5]

- Read platform-specific configuration files when generating code without the `TAURI_CONFIG` env var.
  - Bumped due to a bump in tauri-codegen.
  - [edf85bc1](https://www.github.com/tauri-apps/tauri/commit/edf85bc1d18450c92aee17f7f99c163abe432ebd) fix(codegen): read platform-specific config file ([#3966](https://www.github.com/tauri-apps/tauri/pull/3966)) on 2022-04-25

## \[1.0.0-rc.4]

- Replace multiple dependencies who's C code compiled concurrently and caused
  the other ones to bloat compile time significantly.

- `zstd` -> `brotli`

- `blake3` -> a vendored version of the blake3 reference

- `ring` -> `getrandom`

See https://github.com/tauri-apps/tauri/pull/3773 for more information about
these specific choices.

- [8661e3e2](https://www.github.com/tauri-apps/tauri/commit/8661e3e24d96c399bfbcdee5d8e9d6beba2265a7) replace dependencies with long build times when used together (closes [#3571](https://www.github.com/tauri-apps/tauri/pull/3571)) ([#3773](https://www.github.com/tauri-apps/tauri/pull/3773)) on 2022-03-27

## \[1.0.0-rc.3]

- Parse window icons at compile time.
  - Bumped due to a bump in tauri-codegen.
  - [8c935872](https://www.github.com/tauri-apps/tauri/commit/8c9358725a17dcc2acaf4d10c3f654afdff586b0) refactor(core): move `png` and `ico` behind Cargo features ([#3588](https://www.github.com/tauri-apps/tauri/pull/3588)) on 2022-03-05

## \[1.0.0-rc.2]

- Changed the default value for `tauri > bundle > macOS > minimumSystemVersion` to `10.13`.
  - Bumped due to a bump in tauri-utils.
  - [fce344b9](https://www.github.com/tauri-apps/tauri/commit/fce344b90b7227f8f5514853c2f885fb24d3648e) feat(core): set default value for `minimum_system_version` to 10.13 ([#3497](https://www.github.com/tauri-apps/tauri/pull/3497)) on 2022-02-17

## \[1.0.0-rc.1]

- Change default value for the `freezePrototype` configuration to `false`.
  - Bumped due to a bump in tauri-utils.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

## \[1.0.0-rc.0]

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
- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22

## \[1.0.0-beta.5]

- Embed Info.plist file contents on binary on dev.
  - Bumped due to a bump in tauri-codegen.
  - [537ab1b6](https://www.github.com/tauri-apps/tauri/commit/537ab1b6d5a792c550a535619965c9e4126292e6) feat(core): inject src-tauri/Info.plist file on dev and merge on bundle, closes [#1570](https://www.github.com/tauri-apps/tauri/pull/1570) [#2338](https://www.github.com/tauri-apps/tauri/pull/2338) ([#2444](https://www.github.com/tauri-apps/tauri/pull/2444)) on 2021-08-15
- Fix ES Module detection for default imports with relative paths or scoped packages and exporting of async functions.
  - Bumped due to a bump in tauri-codegen.
  - [b2b36cfe](https://www.github.com/tauri-apps/tauri/commit/b2b36cfe8dfcccb341638a4cb6dc23a514c54148) fix(core): fixes ES Module detection for default imports with relative paths or scoped packages ([#2380](https://www.github.com/tauri-apps/tauri/pull/2380)) on 2021-08-10
  - [fbf8caf5](https://www.github.com/tauri-apps/tauri/commit/fbf8caf5c419cb4fc3d123be910e094a8e8c4bef) fix(core): ESM detection when using `export async function` ([#2425](https://www.github.com/tauri-apps/tauri/pull/2425)) on 2021-08-14

## \[1.0.0-beta.4]

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

## \[1.0.0-beta.3]

- Detect ESM scripts and inject the invoke key directly instead of using an IIFE.
  - Bumped due to a bump in tauri-codegen.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27
- Improve invoke key code injection performance time rewriting code at compile time.
  - Bumped due to a bump in tauri-codegen.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27

## \[1.0.0-beta.2]

- internal: Refactor all macro code that expects specific bindings to be passed Idents
  - [39f8f269](https://www.github.com/tauri-apps/tauri/commit/39f8f269164d2fda3d5b614a193b12bb266e4b4b) refactor(macros): explicitly pass idents ([#1812](https://www.github.com/tauri-apps/tauri/pull/1812)) on 2021-05-13

## \[1.0.0-beta.1]

- Fixes a name collision when the command function is named `invoke`.
  - [7862ec5](https://www.github.com/tauri-apps/tauri/commit/7862ec562fa70e3733263ce1f690d6cd2943c0b4) fix(macros): change invoke binding in generate handler ([#1804](https://www.github.com/tauri-apps/tauri/pull/1804)) on 2021-05-12
- Fixes a name collision when the command function is named `message` or `resolver`.
  - [0b87532](https://www.github.com/tauri-apps/tauri/commit/0b875327067ca825ff6f6f26c9b2ce6fcb001e79) fix(macros): fix rest of command collisions ([#1805](https://www.github.com/tauri-apps/tauri/pull/1805)) on 2021-05-12
- Fixes a name collision when the command function is named `cmd`.
  - [d36b726](https://www.github.com/tauri-apps/tauri/commit/d36b7269261d329dd7d7fcd4d5098f3fca167364) fix(macros): collision when command is named `cmd` ([#1802](https://www.github.com/tauri-apps/tauri/pull/1802)) on 2021-05-12

## \[1.0.0-beta.0]

- Only commands with a `async fn` are executed on a separate task. `#[command] fn command_name` runs on the main thread.
  - [bb8dafb](https://www.github.com/tauri-apps/tauri/commit/bb8dafbe1ea6edde7385631d41ac05e96a3309ef) feat(core): #\[command] return with autoref specialization workaround fix [#1672](https://www.github.com/tauri-apps/tauri/pull/1672) ([#1734](https://www.github.com/tauri-apps/tauri/pull/1734)) on 2021-05-09
- `#[command]` now generates a macro instead of a function to allow passing through `Params` and other generics.
  `generate_handler!` has been changed to consume the generated `#[command]` macro
  - [1453d4b](https://www.github.com/tauri-apps/tauri/commit/1453d4bf842ed6891ec604e0635344c930282189) feat(core): support generics (especially Param) in #\[command] ([#1622](https://www.github.com/tauri-apps/tauri/pull/1622)) on 2021-05-05
- Improves support for commands returning `Result`.
  - [bb8dafb](https://www.github.com/tauri-apps/tauri/commit/bb8dafbe1ea6edde7385631d41ac05e96a3309ef) feat(core): #\[command] return with autoref specialization workaround fix [#1672](https://www.github.com/tauri-apps/tauri/pull/1672) ([#1734](https://www.github.com/tauri-apps/tauri/pull/1734)) on 2021-05-09
- Adds support to command state, triggered when a command argument is `arg: State<'_, StateType>`.
  - [8b6f3de](https://www.github.com/tauri-apps/tauri/commit/8b6f3de0ad47684e72a2ae5f884d8675acfaeeac) feat(core): add state management, closes [#1655](https://www.github.com/tauri-apps/tauri/pull/1655) ([#1665](https://www.github.com/tauri-apps/tauri/pull/1665)) on 2021-05-02

## \[1.0.0-beta-rc.1]

- Fixes the Message `command` name value on plugin invoke handler.
  - [422dd5e](https://www.github.com/tauri-apps/tauri/commit/422dd5e2a0a03bb1556915c78f110bfab092c874) fix(core): command name on plugin invoke handler ([#1577](https://www.github.com/tauri-apps/tauri/pull/1577)) on 2021-04-21
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25

## \[1.0.0-beta-rc.0]

- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Added new macros to simplify the creation of commands that can be called by the webview.
  - [1f2e7a3](https://www.github.com/tauri-apps/tauri/commit/1f2e7a3226ccf0ee3e30ae0cba3c67f7e219d1f2) feat(core): improved command matching with macros, fixes [#1157](https://www.github.com/tauri-apps/tauri/pull/1157) ([#1301](https://www.github.com/tauri-apps/tauri/pull/1301)) on 2021-02-28
