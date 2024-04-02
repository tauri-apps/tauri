# Changelog

## \[2.0.0-beta.11]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.11`

## \[2.0.0-beta.10]

### New Features

- [`e227fe02f`](https://www.github.com/tauri-apps/tauri/commit/e227fe02f986e145c0731a64693e1c830a9eb5b0)([#9156](https://www.github.com/tauri-apps/tauri/pull/9156)) Allow plugins to define (at compile time) JavaScript that are initialized when `withGlobalTauri` is true.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.10`

## \[2.0.0-beta.9]

### New Features

- [`ba0206d8a`](https://www.github.com/tauri-apps/tauri/commit/ba0206d8a30a9b43ec5090dcaabd1a23baa1420c)([#9141](https://www.github.com/tauri-apps/tauri/pull/9141)) The `Context` codegen now accepts a `assets` input to define a custom `tauri::Assets` implementation.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.9`

## \[2.0.0-beta.8]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.8`

### Breaking Changes

- [`ed48e2b3c`](https://www.github.com/tauri-apps/tauri/commit/ed48e2b3c7fa914e4c9af432c02b8154f872c68a)([#9122](https://www.github.com/tauri-apps/tauri/pull/9122)) Expose `tauri::image` module to export the `JsImage` type and removed the `Image` root re-export.

## \[2.0.0-beta.7]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.7`

### Breaking Changes

- [`d1e77acd8`](https://www.github.com/tauri-apps/tauri/commit/d1e77acd8dfdf554b90b542513a58a2de1ef2360)([#9011](https://www.github.com/tauri-apps/tauri/pull/9011)) Change the generated context code to use the new `Image` type in tauri.

## \[2.0.0-beta.6]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.6`

### Breaking Changes

- [`3657ad82`](https://www.github.com/tauri-apps/tauri/commit/3657ad82f88ce528551d032d521c52eed3f396b4)([#9008](https://www.github.com/tauri-apps/tauri/pull/9008)) Allow defining permissions for the application commands via `tauri_build::Attributes::app_manifest`.

## \[2.0.0-beta.5]

### Enhancements

- [`bc5b5e67`](https://www.github.com/tauri-apps/tauri/commit/bc5b5e671a546512f823f1c157421b4c3311dfc0)([#8984](https://www.github.com/tauri-apps/tauri/pull/8984)) Do not include a CSP tag in the application HTML and rely on the custom protocol response header instead.

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

- [`83a68deb`](https://www.github.com/tauri-apps/tauri/commit/83a68deb5676d39cd4728d2e140f6b46d5f787ed)([#8797](https://www.github.com/tauri-apps/tauri/pull/8797)) Added a new configuration option `tauri.conf.json > app > security > capabilities` to reference existing capabilities and inline new ones. If it is empty, all capabilities are still included preserving the current behavior.
- [`8d16a80d`](https://www.github.com/tauri-apps/tauri/commit/8d16a80d2fb2468667e7987d0cc99dbc7e3b9d0a)([#8802](https://www.github.com/tauri-apps/tauri/pull/8802)) The `generate_context` proc macro now accepts a `capabilities` attribute where the value is an array of file paths that can be [conditionally compiled](https://doc.rust-lang.org/reference/conditional-compilation.html). These capabilities are added to the application along the capabilities defined in the Tauri configuration file.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.2`

## \[2.0.0-beta.1]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.1`

## \[2.0.0-beta.0]

### New Features

- [`74a2a603`](https://www.github.com/tauri-apps/tauri/commit/74a2a6036a5e57462f161d728cbd8a6f121028ca)([#8661](https://www.github.com/tauri-apps/tauri/pull/8661)) Implement access control list for IPC usage.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.0`

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

## \[2.0.0-alpha.13]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.13`

## \[2.0.0-alpha.12]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.12`

## \[2.0.0-alpha.11]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.11`

## \[2.0.0-alpha.10]

### Enhancements

- [`c6c59cf2`](https://www.github.com/tauri-apps/tauri/commit/c6c59cf2373258b626b00a26f4de4331765dd487) Pull changes from Tauri 1.5 release.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.10`

## \[2.0.0-alpha.9]

### New Features

- [`880266a7`](https://www.github.com/tauri-apps/tauri/commit/880266a7f697e1fe58d685de3bb6836ce5251e92)([#8031](https://www.github.com/tauri-apps/tauri/pull/8031)) Bump the MSRV to 1.70.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.9`

### Breaking Changes

- [`ebcc21e4`](https://www.github.com/tauri-apps/tauri/commit/ebcc21e4b95f4e8c27639fb1bca545b432f52d5e)([#8057](https://www.github.com/tauri-apps/tauri/pull/8057)) Renamed the beforeDevCommand, beforeBuildCommand and beforeBundleCommand hooks environment variables from `TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG` to `TAURI_ENV_PLATFORM, TAURI_ENV_ARCH, TAURI_ENV_FAMILY, TAURI_ENV_PLATFORM_VERSION, TAURI_ENV_PLATFORM_TYPE and TAURI_ENV_DEBUG` to differentiate the prefix with other CLI environment variables.

## \[2.0.0-alpha.8]

### Enhancements

- [`100d9ede`](https://www.github.com/tauri-apps/tauri/commit/100d9ede35995d9db21d2087dd5606adfafb89a5)([#7802](https://www.github.com/tauri-apps/tauri/pull/7802)) Use `Target` enum from `tauri_utils::platform`.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.8`

## \[2.0.0-alpha.7]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.7`

## \[2.0.0-alpha.6]

### Dependencies

- Updated to latest `tauri-utils`

## \[2.0.0-alpha.5]

- [`96639ca2`](https://www.github.com/tauri-apps/tauri/commit/96639ca239c9e4f75142fc07868ac46822111cff)([#6749](https://www.github.com/tauri-apps/tauri/pull/6749)) Moved the `shell` functionality to its own plugin in the plugins-workspace repository.
- [`3188f376`](https://www.github.com/tauri-apps/tauri/commit/3188f3764978c6d1452ee31d5a91469691e95094)([#6883](https://www.github.com/tauri-apps/tauri/pull/6883)) Bump the MSRV to 1.65.
- [`ae102980`](https://www.github.com/tauri-apps/tauri/commit/ae102980fcdde3f55effdc0623ea425b48d07dd1)([#6719](https://www.github.com/tauri-apps/tauri/pull/6719)) Refactor the `Context` conditional fields and only parse the tray icon on desktop.

## \[2.0.0-alpha.4]

- Added `android` configuration object under `tauri > bundle`.
  - Bumped due to a bump in tauri-utils.
  - [db4c9dc6](https://www.github.com/tauri-apps/tauri/commit/db4c9dc655e07ee2184fe04571f500f7910890cd) feat(core): add option to configure Android's minimum SDK version ([#6651](https://www.github.com/tauri-apps/tauri/pull/6651)) on 2023-04-07

## \[2.0.0-alpha.3]

- Pull changes from Tauri 1.3 release.
  - [](https://www.github.com/tauri-apps/tauri/commit/undefined)  on undefined

## \[2.0.0-alpha.2]

- Return `bool` in the invoke handler.
  - [05dad087](https://www.github.com/tauri-apps/tauri/commit/05dad0876842e2a7334431247d49365cee835d3e) feat: initial work for iOS plugins ([#6205](https://www.github.com/tauri-apps/tauri/pull/6205)) on 2023-02-11

## \[2.0.0-alpha.1]

- Bump the MSRV to 1.64.
  - [7eb9aa75](https://www.github.com/tauri-apps/tauri/commit/7eb9aa75cfd6a3176d3f566fdda02d88aa529b0f) Update gtk to 0.16 ([#6155](https://www.github.com/tauri-apps/tauri/pull/6155)) on 2023-01-30
- Added `crate_name` field on `PackageInfo`.
  - [630a7f4b](https://www.github.com/tauri-apps/tauri/commit/630a7f4b18cef169bfd48673609306fec434e397) refactor: remove mobile log initialization, ref [#6049](https://www.github.com/tauri-apps/tauri/pull/6049) ([#6081](https://www.github.com/tauri-apps/tauri/pull/6081)) on 2023-01-17

## \[2.0.0-alpha.0]

- Change `devPath` URL to use the local IP address on iOS and Android.
  - [6f061504](https://www.github.com/tauri-apps/tauri/commit/6f0615044d09ec58393a7ebca5e45bb175e20db3) feat(cli): add `android dev` and `ios dev` commands ([#4982](https://www.github.com/tauri-apps/tauri/pull/4982)) on 2022-08-20
- First mobile alpha release!
  - [fa3a1098](https://www.github.com/tauri-apps/tauri/commit/fa3a10988a03aed1b66fb17d893b1a9adb90f7cd) feat(ci): prepare 2.0.0-alpha.0 ([#5786](https://www.github.com/tauri-apps/tauri/pull/5786)) on 2022-12-08

## \[1.4.2]

### Dependencies

- Upgraded to `tauri-utils@1.5.2`

## \[1.4.1]

### Dependencies

- Upgraded to `tauri-utils@1.5.0`

## \[1.4.0]

### Enhancements

- [`17d5a4f5`](https://www.github.com/tauri-apps/tauri/commit/17d5a4f51f244d3ff42014b5d1b075fad7c636a5)([#6706](https://www.github.com/tauri-apps/tauri/pull/6706)) Early panic if the PNG icon is not RGBA.
- [`d2710e9d`](https://www.github.com/tauri-apps/tauri/commit/d2710e9d2e8fd93975ef6494512370faa8cb3b7e)([#6944](https://www.github.com/tauri-apps/tauri/pull/6944)) Unpin `time`, `ignore`, and `winnow` crate versions. Developers now have to pin crates if needed themselves. A list of crates that need pinning to adhere to Tauri's MSRV will be visible in Tauri's GitHub workflow: https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/test-core.yml#L85.

## \[1.3.0]

- Bump minimum supported Rust version to 1.60.
  - [5fdc616d](https://www.github.com/tauri-apps/tauri/commit/5fdc616df9bea633810dcb814ac615911d77222c) feat: Use the zbus-backed of notify-rust ([#6332](https://www.github.com/tauri-apps/tauri/pull/6332)) on 2023-03-31
- Pin `time` to `0.3.15`.
  - [3d16461b](https://www.github.com/tauri-apps/tauri/commit/3d16461b68583ba7db037fbc217786e79b46ddf2) fix(core): pin time to 0.3.15 ([#6312](https://www.github.com/tauri-apps/tauri/pull/6312)) on 2023-02-19

## \[1.2.1]

- Fix `allowlist > app > show/hide` always disabled when `allowlist > app > all: false`.
  - Bumped due to a bump in tauri-utils.
  - [bb251087](https://www.github.com/tauri-apps/tauri/commit/bb2510876d0bdff736d36bf3a465cdbe4ad2b90c) fix(core): extend allowlist with `app`'s allowlist, closes [#5650](https://www.github.com/tauri-apps/tauri/pull/5650) ([#5652](https://www.github.com/tauri-apps/tauri/pull/5652)) on 2022-11-18

## \[1.2.0]

- Properly serialize HTML template tags.
  - [aec5537d](https://www.github.com/tauri-apps/tauri/commit/aec5537de0205f62b2ae5c89da04d21930a6fc2e) fix(codegen): serialize template tags, closes [#4410](https://www.github.com/tauri-apps/tauri/pull/4410) ([#5247](https://www.github.com/tauri-apps/tauri/pull/5247)) on 2022-09-28
- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28

## \[1.1.1]

- Add missing allowlist config for `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position` APIs.
  - Bumped due to a bump in tauri-utils.
  - [c764408d](https://www.github.com/tauri-apps/tauri/commit/c764408da7fae123edd41115bda42fa75a4731d2) fix: Add missing allowlist config for cursor apis, closes [#5207](https://www.github.com/tauri-apps/tauri/pull/5207) ([#5211](https://www.github.com/tauri-apps/tauri/pull/5211)) on 2022-09-16

## \[1.1.0]

- Use `TARGET` environment variable for code generation inside build scripts.
  - [6ba99689](https://www.github.com/tauri-apps/tauri/commit/6ba99689aa7ed0ffa9072a1c8ab5db12c9bf95af) feat(codegen): use TARGET environment variable if set ([#4921](https://www.github.com/tauri-apps/tauri/pull/4921)) on 2022-08-12
- Added support to configuration files in TOML format (Tauri.toml file).
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02
- Improve tray icon read error message.
  - [52f0c8bb](https://www.github.com/tauri-apps/tauri/commit/52f0c8bb836c6d50b7ce2393161394f4ce78f5ae) feat(core): improve tray icon read error messages ([#4850](https://www.github.com/tauri-apps/tauri/pull/4850)) on 2022-08-03
- Fix relative paths in `version` field of `tauri.config.json` not being correctly parsed by `generate_context!()`.
  - [accbc5e8](https://www.github.com/tauri-apps/tauri/commit/accbc5e8806a32efc555d019634fc2fa14d17f0a) fix(codegen): fix relative paths in `version` field of `tauri.config.json`, closes [#4723](https://www.github.com/tauri-apps/tauri/pull/4723) ([#4725](https://www.github.com/tauri-apps/tauri/pull/4725)) on 2022-07-24
- Only rewrite temporary icon files when the content change, avoid needless rebuilds.
  - [f957cbb5](https://www.github.com/tauri-apps/tauri/commit/f957cbb56ccbd8d1c047a978b4579946252a6fd2) fix(codegen): write output file when contents change ([#4889](https://www.github.com/tauri-apps/tauri/pull/4889)) on 2022-08-09

## \[1.0.4]

- Validate `__TAURI_ISOLATION_HOOK__` being set by a file in the isolation application.
  - [3b4ed970](https://www.github.com/tauri-apps/tauri/commit/3b4ed970e663f5bffbfe0358610f9c3f157c513f) feat(codegen): validate `__TAURI_ISOLATION_HOOK__` is referenced ([#4631](https://www.github.com/tauri-apps/tauri/pull/4631)) on 2022-07-11

## \[1.0.3]

- The `TAURI_CONFIG` environment variable now represents the configuration to be merged instead of the entire JSON.
  - [fa028ebf](https://www.github.com/tauri-apps/tauri/commit/fa028ebf3c8ca7b43a70d283a01dbea86217594f) refactor: do not pass entire config from CLI to core, send patch instead ([#4598](https://www.github.com/tauri-apps/tauri/pull/4598)) on 2022-07-06

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
  - [562e8ca2](https://www.github.com/tauri-apps/tauri/commit/562e8ca23facf1a8e5fa6c8cdf872357d3523a78) fix(codegen): tray icon path is relative to the config directory on 2022-06-15

## \[1.0.0-rc.10]

- **Breaking change:** The `TrayIcon` enum has been removed and now `Icon` is used instead.
  This allows you to use more image formats and use embedded icons on Linux.
  - [4ce8e228](https://www.github.com/tauri-apps/tauri/commit/4ce8e228134cd3f22973b74ef26ca0d165fbbbd9) refactor(core): use `Icon` for tray icons ([#4342](https://www.github.com/tauri-apps/tauri/pull/4342)) on 2022-06-14

## \[1.0.0-rc.9]

- Added a config flag to bundle the media framework used by webkit2gtk `tauri.conf.json > tauri > bundle > appimage > bundleMediaFramework`.
  - Bumped due to a bump in tauri-utils.
  - [d335fae9](https://www.github.com/tauri-apps/tauri/commit/d335fae92cdcbb0ee18aad4e54558914afa3e778) feat(bundler): bundle additional gstreamer files, closes [#4092](https://www.github.com/tauri-apps/tauri/pull/4092) ([#4271](https://www.github.com/tauri-apps/tauri/pull/4271)) on 2022-06-10

## \[1.0.0-rc.8]

- **Breaking change:** `PackageInfo::version` is now a `semver::Version` instead of a `String`.
  - [2badbd2d](https://www.github.com/tauri-apps/tauri/commit/2badbd2d7ed51bf33c1b547b4c837b600574bd4a) refactor: force semver versions, change updater `should_install` sig ([#4215](https://www.github.com/tauri-apps/tauri/pull/4215)) on 2022-05-25
  - [a7388e23](https://www.github.com/tauri-apps/tauri/commit/a7388e23c3b9019d48b078cae00a75c74d74d11b) fix(ci): adjust change file to include tauri-utils and tauri-codegen on 2022-05-27

## \[1.0.0-rc.7]

- Allow configuring the display options for the MSI execution allowing quieter updates.
  - Bumped due to a bump in tauri-utils.
  - [9f2c3413](https://www.github.com/tauri-apps/tauri/commit/9f2c34131952ea83c3f8e383bc3cec7e1450429f) feat(core): configure msiexec display options, closes [#3951](https://www.github.com/tauri-apps/tauri/pull/3951) ([#4061](https://www.github.com/tauri-apps/tauri/pull/4061)) on 2022-05-15

## \[1.0.0-rc.6]

- The `dangerous_allow_asset_csp_modification` configuration value has been changed to allow a list of CSP directives to disable.
  - [164078c0](https://www.github.com/tauri-apps/tauri/commit/164078c0b719ccbc12e956fecf8a7d4a3c5044e1) feat: allow limiting dangerousDisableAssetCspModification, closes [#3831](https://www.github.com/tauri-apps/tauri/pull/3831) ([#4021](https://www.github.com/tauri-apps/tauri/pull/4021)) on 2022-05-02

## \[1.0.0-rc.5]

- Read platform-specific configuration files when generating code without the `TAURI_CONFIG` env var.
  - [edf85bc1](https://www.github.com/tauri-apps/tauri/commit/edf85bc1d18450c92aee17f7f99c163abe432ebd) fix(codegen): read platform-specific config file ([#3966](https://www.github.com/tauri-apps/tauri/pull/3966)) on 2022-04-25

## \[1.0.0-rc.4]

- Added an option to disable the CSP injection of distributable assets nonces and hashes.
  - [f6e32ee1](https://www.github.com/tauri-apps/tauri/commit/f6e32ee1880eb364ed76beb937c9d12e14d54910) feat(core): add dangerous option to disable compile time CSP injection ([#3775](https://www.github.com/tauri-apps/tauri/pull/3775)) on 2022-03-28

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

- Apply `nonce` to `script` and `style` tags and set them on the `CSP` (`script-src` and `style-src` fetch directives).
  - [cf54dcf9](https://www.github.com/tauri-apps/tauri/commit/cf54dcf9c81730e42c9171daa9c8aa474c95b522) feat: improve `CSP` security with nonces and hashes, add `devCsp` \[TRI-004] ([#8](https://www.github.com/tauri-apps/tauri/pull/8)) on 2022-01-09
- Added the `isolation` pattern.
  - [d5d6d2ab](https://www.github.com/tauri-apps/tauri/commit/d5d6d2abc17cd89c3a079d2ce01581193469dbc0) Isolation Pattern ([#43](https://www.github.com/tauri-apps/tauri/pull/43)) Co-authored-by: Ngo Iok Ui (Wu Yu Wei) <wusyong9104@gmail.com> Co-authored-by: Lucas Fernandes Nogueira <lucas@tauri.app> on 2022-01-17
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

## \[1.0.0-beta.4]

- Embed Info.plist file contents on binary on dev.
  - [537ab1b6](https://www.github.com/tauri-apps/tauri/commit/537ab1b6d5a792c550a535619965c9e4126292e6) feat(core): inject src-tauri/Info.plist file on dev and merge on bundle, closes [#1570](https://www.github.com/tauri-apps/tauri/pull/1570) [#2338](https://www.github.com/tauri-apps/tauri/pull/2338) ([#2444](https://www.github.com/tauri-apps/tauri/pull/2444)) on 2021-08-15
- Fix ES Module detection for default imports with relative paths or scoped packages and exporting of async functions.
  - [b2b36cfe](https://www.github.com/tauri-apps/tauri/commit/b2b36cfe8dfcccb341638a4cb6dc23a514c54148) fix(core): fixes ES Module detection for default imports with relative paths or scoped packages ([#2380](https://www.github.com/tauri-apps/tauri/pull/2380)) on 2021-08-10
  - [fbf8caf5](https://www.github.com/tauri-apps/tauri/commit/fbf8caf5c419cb4fc3d123be910e094a8e8c4bef) fix(core): ESM detection when using `export async function` ([#2425](https://www.github.com/tauri-apps/tauri/pull/2425)) on 2021-08-14

## \[1.0.0-beta.3]

- Improve ESM detection with regexes.
  - [4b0ec018](https://www.github.com/tauri-apps/tauri/commit/4b0ec0188078a8fefd4119fe5e19ebc30191f802) fix(core): improve JS ESM detection ([#2139](https://www.github.com/tauri-apps/tauri/pull/2139)) on 2021-07-02
- Inject invoke key on `script` tags with `type="module"`.
  - [f03eea9c](https://www.github.com/tauri-apps/tauri/commit/f03eea9c9b964709532afbc4d1dd343b3fd96081) feat(core): inject invoke key on `<script type="module">` ([#2120](https://www.github.com/tauri-apps/tauri/pull/2120)) on 2021-06-29

## \[1.0.0-beta.2]

- Detect ESM scripts and inject the invoke key directly instead of using an IIFE.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27
- Improve invoke key code injection performance time rewriting code at compile time.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27

## \[1.0.0-beta.1]

- Allow `dev_path` and `dist_dir` to be an array of root files and directories to embed.
  - [6ec54c53](https://www.github.com/tauri-apps/tauri/commit/6ec54c53b504eec3873d326b1a45e450227d46ed) feat(core): allow `dev_path`, `dist_dir` as array of paths, fixes [#1897](https://www.github.com/tauri-apps/tauri/pull/1897) ([#1926](https://www.github.com/tauri-apps/tauri/pull/1926)) on 2021-05-31
- Validate `tauri.conf.json > build > devPath` and `tauri.conf.json > build > distDir` values.
  - [e97846aa](https://www.github.com/tauri-apps/tauri/commit/e97846aae933cad5cba284a2a133ae7aaee1107c) feat(core): validate `devPath` and `distDir` values ([#1848](https://www.github.com/tauri-apps/tauri/pull/1848)) on 2021-05-17
- Read `tauri.conf.json > tauri > bundle > icons` and use the first `.png` icon as window icon on Linux. Defaults to `icon/icon.png` if a PNG icon is not configured.
  - [40b717ed](https://www.github.com/tauri-apps/tauri/commit/40b717edc57288a1393fad0529390e101ab903c1) feat(core): set window icon on Linux, closes [#1922](https://www.github.com/tauri-apps/tauri/pull/1922) ([#1937](https://www.github.com/tauri-apps/tauri/pull/1937)) on 2021-06-01

## \[1.0.0-beta.0]

- **Breaking:** The `assets` field on the `tauri::Context` struct is now a `Arc<impl Assets>`.
  - [5110c70](https://www.github.com/tauri-apps/tauri/commit/5110c704be67e51d49fb83f3710afb593973dcf9) feat(core): allow users to access the Assets instance ([#1691](https://www.github.com/tauri-apps/tauri/pull/1691)) on 2021-05-03
- Reintroduce `csp` injection, configured on `tauri.conf.json > tauri > security > csp`.
  - [6132f3f](https://www.github.com/tauri-apps/tauri/commit/6132f3f4feb64488ef618f690a4f06adce864d91) feat(core): reintroduce CSP injection ([#1704](https://www.github.com/tauri-apps/tauri/pull/1704)) on 2021-05-04
- Added the \`#\[non_exhaustive] attribute where appropriate.
  - [e087f0f](https://www.github.com/tauri-apps/tauri/commit/e087f0f9374355ac4b4a48f94727ef8b26b1c4cf) feat: add `#[non_exhaustive]` attribute ([#1725](https://www.github.com/tauri-apps/tauri/pull/1725)) on 2021-05-05

## \[1.0.0-beta-rc.1]

- The package info APIs now checks the `package` object on `tauri.conf.json`.
  - [8fd1baf](https://www.github.com/tauri-apps/tauri/commit/8fd1baf69b14bb81d7be9d31605ed7f02058b392) fix(core): pull package info from tauri.conf.json if set ([#1581](https://www.github.com/tauri-apps/tauri/pull/1581)) on 2021-04-22
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25

## \[1.0.0-beta-rc.0]

- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
