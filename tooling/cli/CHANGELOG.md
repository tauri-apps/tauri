# Changelog

## \[2.0.0-beta.12]

### Dependencies

- Upgraded to `tauri-bundler@2.1.0-beta.0`
- Upgraded to `tauri-utils@2.0.0-beta.11`

## \[2.0.0-beta.11]

### Enhancements

- [`ac76a22f3`](https://www.github.com/tauri-apps/tauri/commit/ac76a22f383028d9bacdedebeb41d3fca5ec9dac)([#9183](https://www.github.com/tauri-apps/tauri/pull/9183)) Allow empty responses for `devUrl`, `beforeDevCommand` and `beforeBuildCommands` questions in `tauri init`.
- [`b525ddadf`](https://www.github.com/tauri-apps/tauri/commit/b525ddadf7e7588c3e195cf0f821c9862c545d06)([#9237](https://www.github.com/tauri-apps/tauri/pull/9237)) `openssl` is no longer a required dependency on macOS.

### Dependencies

- Upgraded to `tauri-bundler@2.0.1-beta.7`

## \[2.0.0-beta.10]

### New Features

- [`7213b9e47`](https://www.github.com/tauri-apps/tauri/commit/7213b9e47242bef814aa7257e0bf84631bf5fe7e)([#9124](https://www.github.com/tauri-apps/tauri/pull/9124)) Add default permission for a plugin to capabilities when using `tauri add <plugin>`.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.10`
- Upgraded to `tauri-bundler@2.0.1-beta.6`

## \[2.0.0-beta.9]

### Bug Fixes

- [`c3ea3a2b7`](https://www.github.com/tauri-apps/tauri/commit/c3ea3a2b7d2fe3085f05b63dd1feb962beb4b7b3)([#9126](https://www.github.com/tauri-apps/tauri/pull/9126)) Fix bundling when `plugins > updater > windows > installerArgs` are set in `tauri.conf.json`

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.9`
- Upgraded to `tauri-bundler@2.0.1-beta.5`

## \[2.0.0-beta.8]

### Enhancements

- [`3e472d0af`](https://www.github.com/tauri-apps/tauri/commit/3e472d0afcd67545dd6d9f18d304580a3b2759a8)([#9115](https://www.github.com/tauri-apps/tauri/pull/9115)) Changed the permission and capability platforms to be optional.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.8`
- Upgraded to `tauri-bundler@2.0.1-beta.4`

## \[2.0.0-beta.7]

### Enhancements

- [`c68218b36`](https://www.github.com/tauri-apps/tauri/commit/c68218b362c417b62e56c7a2b5b32c13fe035a83)([#8990](https://www.github.com/tauri-apps/tauri/pull/8990)) Add `--no-bundle` flag for `tauri build` command to skip bundling. Previously `none` was used to skip bundling, it will now be treated as invalid format and a warning will be emitted instead.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.7`
- Upgraded to `tauri-bundler@2.0.1-beta.3`
- [`4f7894176`](https://www.github.com/tauri-apps/tauri/commit/4f789417630b8a32dcf9c0daec448ea8182daca1)([#9034](https://www.github.com/tauri-apps/tauri/pull/9034)) Update dependencies, fix `log` compilation issue.

## \[2.0.0-beta.6]

### Bug Fixes

- [`f5f3ed5f`](https://www.github.com/tauri-apps/tauri/commit/f5f3ed5f6faa0b51e83244acc15e9006299a03ba)([#9009](https://www.github.com/tauri-apps/tauri/pull/9009)) Fixes Android and iOS project initialization when the Tauri CLI is on a different disk partition.
- [`d7d03c71`](https://www.github.com/tauri-apps/tauri/commit/d7d03c7197212f3a5bebe08c929417d60927eb89)([#9017](https://www.github.com/tauri-apps/tauri/pull/9017)) Fixes dev watcher on mobile dev.
- [`b658ded6`](https://www.github.com/tauri-apps/tauri/commit/b658ded614cfc169228cb22ad5bfc64478dfe161)([#9015](https://www.github.com/tauri-apps/tauri/pull/9015)) Fixes truncation of existing BuildTask.kt when running `tauri android init`.

### What's Changed

- [`3657ad82`](https://www.github.com/tauri-apps/tauri/commit/3657ad82f88ce528551d032d521c52eed3f396b4)([#9008](https://www.github.com/tauri-apps/tauri/pull/9008)) Updates to new ACL manifest path.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.6`
- Upgraded to `tauri-bundler@2.0.1-beta.2`

## \[2.0.0-beta.5]

### New Features

- [`06d63d67`](https://www.github.com/tauri-apps/tauri/commit/06d63d67a061459dd533ddcae755922427a6dfc5)([#8827](https://www.github.com/tauri-apps/tauri/pull/8827)) Add new subcommands for managing permissions and cababilities:

  - `tauri permission new`
  - `tauri permission add`
  - `tauri permission rm`
  - `tauri permission ls`
  - `tauri capability new`

### Breaking Changes

- [`b9e6a018`](https://www.github.com/tauri-apps/tauri/commit/b9e6a01879d9233040f3d3fab11c59e70563da7e)([#8937](https://www.github.com/tauri-apps/tauri/pull/8937)) The `custom-protocol` Cargo feature is no longer required on your application and is now ignored. To check if running on production, use `#[cfg(not(dev))]` instead of `#[cfg(feature = "custom-protocol")]`.

### Enhancements

- [`9be314f0`](https://www.github.com/tauri-apps/tauri/commit/9be314f07a4ca5d14433d41919492f3e91b5536a)([#8951](https://www.github.com/tauri-apps/tauri/pull/8951)) Add plugins to `Cargo.toml` when using `tauri migrate`

### Bug Fixes

- [`cbd9755e`](https://www.github.com/tauri-apps/tauri/commit/cbd9755e0926a7e47e59deb50f4bb93d621791a5)([#8977](https://www.github.com/tauri-apps/tauri/pull/8977)) Fixes process logs not showing on `ios dev`.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.5`
- Upgraded to `tauri-bundler@2.0.1-beta.1`

## \[2.0.0-beta.4]

### Bug Fixes

- [`e538ba58`](https://www.github.com/tauri-apps/tauri/commit/e538ba586c5b8b50955586c8ef2704adb5d7cc43)([#8949](https://www.github.com/tauri-apps/tauri/pull/8949)) Fixes android and iOS process spawning not working on Node.js.

### Dependencies

- Upgraded to `tauri-bundler@2.0.1-beta.0`
- Upgraded to `tauri-utils@2.0.0-beta.4`

### Breaking Changes

- [`a76fb118`](https://www.github.com/tauri-apps/tauri/commit/a76fb118ce2de22e1bdb4216bf0ac01dfc3e5799)([#8950](https://www.github.com/tauri-apps/tauri/pull/8950)) Changed the capability format to allow configuring both `remote: { urls: Vec<String> }` and `local: bool (default: true)` instead of choosing one on the `context` field.

## \[2.0.0-beta.3]

### Enhancements

- [`a029b9f7`](https://www.github.com/tauri-apps/tauri/commit/a029b9f77e432533a403c292940fa3efba68692c)([#8910](https://www.github.com/tauri-apps/tauri/pull/8910)) Setting up code signing is no longer required on iOS when using the simulator.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.3`
- Upgraded to `tauri-bundler@2.0.0-beta.3`

## \[2.0.0-beta.2]

### Enhancements

- [`83a68deb`](https://www.github.com/tauri-apps/tauri/commit/83a68deb5676d39cd4728d2e140f6b46d5f787ed)([#8797](https://www.github.com/tauri-apps/tauri/pull/8797)) Update app template following capabilities configuration change.

### Bug Fixes

- [`aa06a053`](https://www.github.com/tauri-apps/tauri/commit/aa06a0534cf224038866e0ddd6910ea873b2574d)([#8810](https://www.github.com/tauri-apps/tauri/pull/8810)) Fix `tauri plugin android init` printing invalid code that has a missing closing `"`.
- [`3cee26a5`](https://www.github.com/tauri-apps/tauri/commit/3cee26a58ab44639a12c7816f4096655daa327a4)([#8865](https://www.github.com/tauri-apps/tauri/pull/8865)) On Windows, fixed `tauri info` fails to detect the build tool when the system language is CJK.
- [`052e8b43`](https://www.github.com/tauri-apps/tauri/commit/052e8b4311d9f0f963a2866163be27bfd8f70c60)([#8838](https://www.github.com/tauri-apps/tauri/pull/8838)) Downgrade minisign dependency fixing updater signing key bug and prevent it from happening in the future.
- [`fb0d9971`](https://www.github.com/tauri-apps/tauri/commit/fb0d997117367e3387896bcd0fba004579475c40)([#8783](https://www.github.com/tauri-apps/tauri/pull/8783)) Fixes a regression on the `--config` argument not accepting file paths.
- [`baca704d`](https://www.github.com/tauri-apps/tauri/commit/baca704d4b5fae239fc320d10140f35bd705bfbb)([#8768](https://www.github.com/tauri-apps/tauri/pull/8768)) Do not migrate updater configuration if the active flag is set to false.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.2`
- Upgraded to `tauri-bundler@2.0.0-beta.2`

## \[2.0.0-beta.1]

### Enhancements

- [`4e101f80`](https://www.github.com/tauri-apps/tauri/commit/4e101f801657e7d01ce8c22f9c6468067d0caab2)([#8756](https://www.github.com/tauri-apps/tauri/pull/8756)) Moved the capability JSON schema to the `src-tauri/gen` folder so it's easier to track changes on the `capabilities` folder.
- [`4e101f80`](https://www.github.com/tauri-apps/tauri/commit/4e101f801657e7d01ce8c22f9c6468067d0caab2)([#8756](https://www.github.com/tauri-apps/tauri/pull/8756)) Update app and plugin templates following generated files change from tauri-build and tauri-plugin.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.1`
- Upgraded to `tauri-bundler@2.0.0-beta.1`

## \[2.0.0-beta.0]

### New Features

- [`7fcc0bcd`](https://www.github.com/tauri-apps/tauri/commit/7fcc0bcd3482bbc8771e330942ef6cd78cc8ec35)([#8490](https://www.github.com/tauri-apps/tauri/pull/8490)) Add plugin initialization rust code when using `tauri add`
- [`1878766f`](https://www.github.com/tauri-apps/tauri/commit/1878766f7f81a03b0f0b87ec33ee113d7aa7a902)([#8667](https://www.github.com/tauri-apps/tauri/pull/8667)) Migrate the allowlist config to the new capability file format.

### Enhancements

- [`d6c7568c`](https://www.github.com/tauri-apps/tauri/commit/d6c7568c27445653edf570f3969163bc358ba2ba)([#8720](https://www.github.com/tauri-apps/tauri/pull/8720)) Add `files` option to the AppImage Configuration.
- [`b3209bb2`](https://www.github.com/tauri-apps/tauri/commit/b3209bb28bb379d5046d577c7e42319d6e76ced0)([#8688](https://www.github.com/tauri-apps/tauri/pull/8688)) Ignore global `.gitignore` when searching for tauri directory.
- [`e691208e`](https://www.github.com/tauri-apps/tauri/commit/e691208e7b39bb8e3ffc9bf66cff731a5025ef16)([#7837](https://www.github.com/tauri-apps/tauri/pull/7837)) Prevent unneeded double Cargo.toml rewrite on `dev` and `build`.
- [`f492efd7`](https://www.github.com/tauri-apps/tauri/commit/f492efd7144fdd8d25cac0c4d2389a95ab75fb02)([#8666](https://www.github.com/tauri-apps/tauri/pull/8666)) Update app and plugin template following the new access control permission model.

### Bug Fixes

- [`9cb9aa79`](https://www.github.com/tauri-apps/tauri/commit/9cb9aa7978f231f7da238b33d6ab33fdd2d2c842)([#8672](https://www.github.com/tauri-apps/tauri/pull/8672)) Allow license field in Cargo.toml to be `{ workspace = true }`

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.0`
- Upgraded to `tauri-bundler@2.0.0-beta.0`

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

## \[2.0.0-alpha.21]

### New Features

- [`27bad32d`](https://www.github.com/tauri-apps/tauri/commit/27bad32d4d4acca8155b20225d529d540fb9aaf4)([#7798](https://www.github.com/tauri-apps/tauri/pull/7798)) Add `files` object on the `tauri > bundle > macOS` configuration option.
- [`0ec28c39`](https://www.github.com/tauri-apps/tauri/commit/0ec28c39f478de7199a66dd75e8642e1aa1344e6)([#8529](https://www.github.com/tauri-apps/tauri/pull/8529)) Include tauri-build on the migration script.

### Enhancements

- [`091100ac`](https://www.github.com/tauri-apps/tauri/commit/091100acbb507b51de39fb1446f685926f888fd2)([#5202](https://www.github.com/tauri-apps/tauri/pull/5202)) Add RPM packaging

### Bug Fixes

- [`4f73057e`](https://www.github.com/tauri-apps/tauri/commit/4f73057e6fd4c137bc112367fb91f5fc0c8a39f6)([#8486](https://www.github.com/tauri-apps/tauri/pull/8486)) Prevent `Invalid target triple` warnings and correctly set `TAURI_ENV_` vars when target triple contains 4 components.

### Dependencies

- Upgraded to `tauri-bundler@2.0.0-alpha.14`
- Upgraded to `tauri-utils@2.0.0-alpha.13`

### Breaking Changes

- [`4f73057e`](https://www.github.com/tauri-apps/tauri/commit/4f73057e6fd4c137bc112367fb91f5fc0c8a39f6)([#8486](https://www.github.com/tauri-apps/tauri/pull/8486)) Removed `TAURI_ENV_PLATFORM_TYPE` which will not be set for CLI hook commands anymore, use `TAURI_ENV_PLATFORM` instead. Also Changed value of `TAURI_ENV_PLATFORM` and `TAURI_ENV_ARCH` values to match the target triple more accurately:

  - `darwin` and `androideabi` are no longer replaced with `macos` and `android` in `TAURI_ENV_PLATFORM`.
  - `i686` and `i586` are no longer replaced with `x86` in `TAURI_ENV_ARCH`.

## \[2.0.0-alpha.20]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.12`
- Upgraded to `tauri-bundler@2.0.0-alpha.13`

## \[2.0.0-alpha.19]

### Enhancements

- [`803c3a79`](https://www.github.com/tauri-apps/tauri/commit/803c3a794d96c1f0b3361ca98f56e8bcf5038ede)([#8327](https://www.github.com/tauri-apps/tauri/pull/8327)) Read the following env vars when using the `tauri signer sign` command to make it easier to use in CI.

  - `TAURI_PRIVATE_KEY`
  - `TAURI_PRIVATE_KEY_PASSWORD`
  - `TAURI_PRIVATE_KEY_PATH`

## \[2.0.0-alpha.18]

### New Features

- [`50f7ccbb`](https://www.github.com/tauri-apps/tauri/commit/50f7ccbbf3467f33cc7dd1cca53125fec6eda1c6)([#6444](https://www.github.com/tauri-apps/tauri/pull/6444)) Add suport to SVG input image for the `tauri icon` command.
- [`25e5f91d`](https://www.github.com/tauri-apps/tauri/commit/25e5f91dae7fe2bbc1ba4317d5d829402bfd1d50)([#8200](https://www.github.com/tauri-apps/tauri/pull/8200)) Merge `src-tauri/Info.plist` and `src-tauri/Info.ios.plist` with the iOS project plist file.

### Enhancements

- [`01a7a983`](https://www.github.com/tauri-apps/tauri/commit/01a7a983aba2946b455a608b8a6a4b08cb25fc11)([#8128](https://www.github.com/tauri-apps/tauri/pull/8128)) Transform paths to relative to the mobile project for the IDE script runner script.

### Bug Fixes

- [`88dac86f`](https://www.github.com/tauri-apps/tauri/commit/88dac86f3b301d1919df6473a9e20f46b560f29b)([#8149](https://www.github.com/tauri-apps/tauri/pull/8149)) Ensure `tauri add` prints `rust_code` with plugin name in snake case.
- [`977d0e52`](https://www.github.com/tauri-apps/tauri/commit/977d0e52f14b1ad01c86371765ef25b36572459e)([#8202](https://www.github.com/tauri-apps/tauri/pull/8202)) Fixes `android build --open` and `ios build --open` IDE failing to read CLI options.
- [`bfbbefdb`](https://www.github.com/tauri-apps/tauri/commit/bfbbefdb9e13ed1f42f6db7fa9ceaa84db1267e9)([#8161](https://www.github.com/tauri-apps/tauri/pull/8161)) Fix invalid plugin template.
- [`92b50a3a`](https://www.github.com/tauri-apps/tauri/commit/92b50a3a398c9d55b6992a8f5c2571e4d72bdaaf)([#8209](https://www.github.com/tauri-apps/tauri/pull/8209)) Added support to Xcode's archive. This requires regenerating the Xcode project.

### Dependencies

- Upgraded to `tauri-bundler@2.0.0-alpha.12`
- Upgraded to `tauri-utils@2.0.0-alpha.11`

## \[2.0.0-alpha.17]

### Enhancements

- [`c6c59cf2`](https://www.github.com/tauri-apps/tauri/commit/c6c59cf2373258b626b00a26f4de4331765dd487) Pull changes from Tauri 1.5 release.

### Dependencies

- Upgraded to `tauri-bundler@2.0.0-alpha.11`
- Upgraded to `tauri-utils@2.0.0-alpha.10`

### Breaking Changes

- [`198abe3c`](https://www.github.com/tauri-apps/tauri/commit/198abe3c2cae06dacab860b3a93f715dcf529a95)([#8076](https://www.github.com/tauri-apps/tauri/pull/8076)) Updated the mobile plugin templates following the tauri v2.0.0-alpha.17 changes.

## \[2.0.0-alpha.16]

### New Features

- [`8b166e9b`](https://www.github.com/tauri-apps/tauri/commit/8b166e9bf82e69ddb3200a3a825614980bd8d433)([#7949](https://www.github.com/tauri-apps/tauri/pull/7949)) Add `--no-dev-server-wait` option to skip waiting for the dev server to start when using `tauri dev`.
- [`880266a7`](https://www.github.com/tauri-apps/tauri/commit/880266a7f697e1fe58d685de3bb6836ce5251e92)([#8031](https://www.github.com/tauri-apps/tauri/pull/8031)) Bump the MSRV to 1.70.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.9`
- Upgraded to `tauri-bundler@2.0.0-alpha.10`

### Breaking Changes

- [`8b166e9b`](https://www.github.com/tauri-apps/tauri/commit/8b166e9bf82e69ddb3200a3a825614980bd8d433)([#7949](https://www.github.com/tauri-apps/tauri/pull/7949)) Changed a number of environment variables used by tauri CLI for consistency and clarity:

  - `TAURI_PRIVATE_KEY` -> `TAURI_SIGNING_PRIVATE_KEY`
  - `TAURI_KEY_PASSWORD` -> `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
  - `TAURI_SKIP_DEVSERVER_CHECK` -> `TAURI_CLI_NO_DEV_SERVER_WAIT`
  - `TAURI_DEV_SERVER_PORT` -> `TAURI_CLI_PORT`
  - `TAURI_PATH_DEPTH` -> `TAURI_CLI_CONFIG_DEPTH`
  - `TAURI_FIPS_COMPLIANT` -> `TAURI_BUNDLER_WIX_FIPS_COMPLIANT`
  - `TAURI_DEV_WATCHER_IGNORE_FILE` -> `TAURI_CLI_WATCHER_IGNORE_FILENAME`
  - `TAURI_TRAY` -> `TAURI_LINUX_AYATANA_APPINDICATOR`
  - `TAURI_APPLE_DEVELOPMENT_TEAM` -> `APPLE_DEVELOPMENT_TEAM`
- [`4caa1cca`](https://www.github.com/tauri-apps/tauri/commit/4caa1cca990806f2c2ef32d5dabaf56e82f349e6)([#7990](https://www.github.com/tauri-apps/tauri/pull/7990)) The `tauri plugin` subcommand is receving a couple of consitency and quality of life improvements:

  - Renamed `tauri plugin android/ios add` command to `tauri plugin android/ios init` to match the `tauri plugin init` command.
  - Removed the `-n/--name` argument from the `tauri plugin init`, `tauri plugin android/ios init`, and is now parsed from the first positional argument.
  - Added `tauri plugin new` to create a plugin in a new directory.
  - Changed `tauri plugin init` to initalize a plugin in an existing directory (defaults to current directory) instead of creating a new one.
  - Changed `tauri plugin init` to NOT generate mobile projects by default, you can opt-in to generate them using `--android` and `--ios` flags or `--mobile` flag or initalize them later using `tauri plugin android/ios init`.
- [`8b166e9b`](https://www.github.com/tauri-apps/tauri/commit/8b166e9bf82e69ddb3200a3a825614980bd8d433)([#7949](https://www.github.com/tauri-apps/tauri/pull/7949)) Removed checking for a new version of the CLI.
- [`ebcc21e4`](https://www.github.com/tauri-apps/tauri/commit/ebcc21e4b95f4e8c27639fb1bca545b432f52d5e)([#8057](https://www.github.com/tauri-apps/tauri/pull/8057)) Renamed the beforeDevCommand, beforeBuildCommand and beforeBundleCommand hooks environment variables from `TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG` to `TAURI_ENV_PLATFORM, TAURI_ENV_ARCH, TAURI_ENV_FAMILY, TAURI_ENV_PLATFORM_VERSION, TAURI_ENV_PLATFORM_TYPE and TAURI_ENV_DEBUG` to differentiate the prefix with other CLI environment variables.

## \[2.0.0-alpha.15]

### New Features

- [`b2f17723`](https://www.github.com/tauri-apps/tauri/commit/b2f17723a415f04c2620132a6305eb138d7cb47f)([#7971](https://www.github.com/tauri-apps/tauri/pull/7971)) Use `devicectl` to connect to iOS 17+ devices on macOS 14+.

### Bug Fixes

- [`100d9ede`](https://www.github.com/tauri-apps/tauri/commit/100d9ede35995d9db21d2087dd5606adfafb89a5)([#7802](https://www.github.com/tauri-apps/tauri/pull/7802)) Properly read platform-specific configuration files for mobile targets.
- [`228e5a4c`](https://www.github.com/tauri-apps/tauri/commit/228e5a4c76ad5f97409c912d07699b49ba4bb162)([#7902](https://www.github.com/tauri-apps/tauri/pull/7902)) Fixes `icon` command not writing files to the correct Android project folders.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.8`
- Upgraded to `tauri-bundler@2.0.0-alpha.9`

## \[2.0.0-alpha.14]

### Breaking Changes

- [`d5074af5`](https://www.github.com/tauri-apps/tauri/commit/d5074af562b2b5cb6c5711442097c4058af32db6)([#7801](https://www.github.com/tauri-apps/tauri/pull/7801)) The custom protocol on Android now uses the `http` scheme instead of `https`.

## \[2.0.0-alpha.13]

### Breaking Changes

- [`4cb51a2d`](https://www.github.com/tauri-apps/tauri/commit/4cb51a2d56cfcae0749062c79ede5236bd8c02c2)([#7779](https://www.github.com/tauri-apps/tauri/pull/7779)) The custom protocol on Windows now uses the `http` scheme instead of `https`.
- [`974e38b4`](https://www.github.com/tauri-apps/tauri/commit/974e38b4ddc198530aa977ec77d513b76013b9f3)([#7744](https://www.github.com/tauri-apps/tauri/pull/7744)) Renamed the `plugin add` command to `add`.

## \[2.0.0-alpha.12]

### Bug Fixes

- [`b75a1210`](https://www.github.com/tauri-apps/tauri/commit/b75a1210bed589187678861d7314ae6279bf7c87)([#7762](https://www.github.com/tauri-apps/tauri/pull/7762)) Fixes a regression on alpha.11 where iOS logs aren't being displayed when using `ios dev` with a real device.
- [`8faa5a4a`](https://www.github.com/tauri-apps/tauri/commit/8faa5a4a1238a44ca7b54d2084aaed553ac2a1ba)([#7765](https://www.github.com/tauri-apps/tauri/pull/7765)) Ensure asset directory exists on the iOS project.

### Dependencies

- Upgraded to `tauri-bundler@2.0.0-alpha.8`

## \[2.0.0-alpha.11]

### New Features

- [`522de0e7`](https://www.github.com/tauri-apps/tauri/commit/522de0e78891d0bdf6387a5118985fc41a11baeb)([#7447](https://www.github.com/tauri-apps/tauri/pull/7447)) Expose an environment variable `TAURI_${PLUGIN_NAME}_PLUGIN_CONFIG` for each defined plugin configuration object.
- [`c7dacca4`](https://www.github.com/tauri-apps/tauri/commit/c7dacca4661c6ddf937c1a3dd3ace896d5baf40c)([#7446](https://www.github.com/tauri-apps/tauri/pull/7446)) Expose the `TAURI_IOS_PROJECT_PATH` and `TAURI_IOS_APP_NAME` environment variables when using `ios` commands.
- [`aa94f719`](https://www.github.com/tauri-apps/tauri/commit/aa94f7197e4345a7cab1617272b10895859674f9)([#7445](https://www.github.com/tauri-apps/tauri/pull/7445)) Generate empty entitlements file for the iOS project.
- [`d010bc07`](https://www.github.com/tauri-apps/tauri/commit/d010bc07b81116bef769b64cdc19b23dff762d48)([#7554](https://www.github.com/tauri-apps/tauri/pull/7554)) Set the iOS project PRODUCT_NAME value to the string under `tauri.conf.json > package > productName` if it is set.
- [`8af24974`](https://www.github.com/tauri-apps/tauri/commit/8af2497496f11ee481472dfdc3125757346c1a3e)([#7561](https://www.github.com/tauri-apps/tauri/pull/7561)) The `migrate` command now automatically reads all JavaScript files and updates `@tauri-apps/api` import paths and install the missing plugins.

### Enhancements

- [`fbeb5b91`](https://www.github.com/tauri-apps/tauri/commit/fbeb5b9185baeda19e865228179e3e44c165f1d9)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Update migrate command to update the configuration CSP to include `ipc:` on the `connect-src` directive, needed by the new IPC using custom protocols.

### Bug Fixes

- [`5eb85543`](https://www.github.com/tauri-apps/tauri/commit/5eb85543313eaf7ca2289898652b1b11dc776608)([#7282](https://www.github.com/tauri-apps/tauri/pull/7282)) Fix `tauri info` failing when there is no available iOS code signing certificate.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.7`
- Upgraded to `tauri-bundler@2.0.0-alpha.7`

## \[2.0.0-alpha.10]

### New Features

- [`7e5905ae`](https://www.github.com/tauri-apps/tauri/commit/7e5905ae1d56b920de0e821be28036cbbe302518)([#7023](https://www.github.com/tauri-apps/tauri/pull/7023)) Added `tauri plugin add` command to add a plugin to the Tauri project.
- [`b0f94775`](https://www.github.com/tauri-apps/tauri/commit/b0f947752a315b7b89c5979de50157f997f1dd6e)([#7008](https://www.github.com/tauri-apps/tauri/pull/7008)) Added `migrate` command.

### Enhancements

- [`aa6c9164`](https://www.github.com/tauri-apps/tauri/commit/aa6c9164e63b5316d690f25b1c118f1b12310570)([#7007](https://www.github.com/tauri-apps/tauri/pull/7007)) Don't build library files when building desktop targets.
- [`a28fdf7e`](https://www.github.com/tauri-apps/tauri/commit/a28fdf7ec71bf6db2498569004de83318b6d25ac)([#7044](https://www.github.com/tauri-apps/tauri/pull/7044)) Skip Rust target installation if they are already installed.
- [`735db1ce`](https://www.github.com/tauri-apps/tauri/commit/735db1ce839a16ba998c9e6786c441e3bf6c90b3)([#7044](https://www.github.com/tauri-apps/tauri/pull/7044)) Add `--skip-targets-install` flag for `tauri android init` and `tauri ios init` to skip installing needed rust targets vie rustup.

### Bug Fixes

- [`3f4c4ce8`](https://www.github.com/tauri-apps/tauri/commit/3f4c4ce88b071e4e59f03887beb4dfe76f66e11b)([#7028](https://www.github.com/tauri-apps/tauri/pull/7028)) Fix `--split-per-abi` not building any targets unless specified by `--target` flag.
- [`1ed2600d`](https://www.github.com/tauri-apps/tauri/commit/1ed2600da67715908af857255305eaeb293d8791)([#6771](https://www.github.com/tauri-apps/tauri/pull/6771)) Set current directory to tauri directory before reading config file.
- [`4847b87b`](https://www.github.com/tauri-apps/tauri/commit/4847b87b1067dd8c6e73986059f51e6eee1f1121)([#7209](https://www.github.com/tauri-apps/tauri/pull/7209)) Fix `tauri (android|ios) (dev|build)` failing when using `npx tauri`
- [`655c714e`](https://www.github.com/tauri-apps/tauri/commit/655c714e4100f69d4265c5ea3f08f5bc11709446)([#7240](https://www.github.com/tauri-apps/tauri/pull/7240)) Fixes panic when exiting the `ios dev` command with Ctrl + C.
- [`6252380f`](https://www.github.com/tauri-apps/tauri/commit/6252380f4447c66913b0f06611e5949005b1eec2)([#7241](https://www.github.com/tauri-apps/tauri/pull/7241)) Exit `beforeDevCommand` process if the android or iOS `dev` command fails.

## \[2.0.0-alpha.9]

- [`19cd0e49`](https://www.github.com/tauri-apps/tauri/commit/19cd0e49603ad3500cd2180bfa16e1649e3a771a)([#6811](https://www.github.com/tauri-apps/tauri/pull/6811)) Add `key.properties` file to android's `.gitignore`.
- [`124d5c5a`](https://www.github.com/tauri-apps/tauri/commit/124d5c5adf67f0b68d2e41c7ddb07d9cb63f1996)([#6788](https://www.github.com/tauri-apps/tauri/pull/6788)) On mobile, fix regression introduced in `tauri-cli` version `2.0.0-alpha.3` where library not found error was thrown.
- [`31444ac1`](https://www.github.com/tauri-apps/tauri/commit/31444ac196add770f2ad18012d7c18bce7538f22)([#6725](https://www.github.com/tauri-apps/tauri/pull/6725)) Update mobile template to `wry@0.28`
- [`6d1fa49f`](https://www.github.com/tauri-apps/tauri/commit/6d1fa49fce3a03965ce7c656390e682ce5b776e3)([#6881](https://www.github.com/tauri-apps/tauri/pull/6881)) Clear Android plugin JSON file before building Rust library to ensure removed plugins are propagated to the Android project.
- [`5a9307d1`](https://www.github.com/tauri-apps/tauri/commit/5a9307d11c1643221bc2a280feb00024f8fa6030)([#6890](https://www.github.com/tauri-apps/tauri/pull/6890)) Update android template to gradle 8.0
- [`73c803a5`](https://www.github.com/tauri-apps/tauri/commit/73c803a561181137f20366f5d52511392a619f2b)([#6837](https://www.github.com/tauri-apps/tauri/pull/6837)) Inject Tauri configuration in the Android assets.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Moved the updater configuration to the `BundleConfig`.
- [`3188f376`](https://www.github.com/tauri-apps/tauri/commit/3188f3764978c6d1452ee31d5a91469691e95094)([#6883](https://www.github.com/tauri-apps/tauri/pull/6883)) Bump the MSRV to 1.65.
- [`2969d1cb`](https://www.github.com/tauri-apps/tauri/commit/2969d1cbba39301f9cc611d9f7d7051d80eef846)([#6773](https://www.github.com/tauri-apps/tauri/pull/6773)) Use absolute path to each Android plugin project instead of copying the files to enhance developer experience.
- [`d48aaa15`](https://www.github.com/tauri-apps/tauri/commit/d48aaa150a1ceeb65ec0ba18f1e3795f70c838e3)([#6894](https://www.github.com/tauri-apps/tauri/pull/6894)) Add Cargo manifest files for the plugin example templates.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Removed the allowlist configuration.

## \[2.0.0-alpha.8]

- Do not gitignore the Android project's `buildSrc` folder by default since we removed absolute paths from it.
  - [ee2d3b97](https://www.github.com/tauri-apps/tauri/commit/ee2d3b971df6d3630b8d935394fb4febcfa3a909) fix(cli): remove buildSrc from Android project gitignored paths ([#6702](https://www.github.com/tauri-apps/tauri/pull/6702)) on 2023-04-13
- Fixes iOS build script using the wrong path for the app library file.
  - [abc5f91f](https://www.github.com/tauri-apps/tauri/commit/abc5f91fa3569efc9dfdee46d1c501eda8755944) fix(cli): iOS Xcode script using incorrect library path ([#6699](https://www.github.com/tauri-apps/tauri/pull/6699)) on 2023-04-13

## \[2.0.0-alpha.7]

- Add `--release` flag for `tauri android dev` however you will need to sign your Android app, see https://next--tauri.netlify.app/next/guides/distribution/sign-android
  - [63f088e5](https://www.github.com/tauri-apps/tauri/commit/63f088e5fc9701fd7fb329dad7ffb27a2d8fd5aa) feat(cli): add `--release` for `android dev` ([#6638](https://www.github.com/tauri-apps/tauri/pull/6638)) on 2023-04-05
- Build only specified rust targets for `tauri android build` instead of all.
  - [d03e47d1](https://www.github.com/tauri-apps/tauri/commit/d03e47d141c3917520975be9081775dbc4e9d4fd) fix: only build specified rust targets for aab/apk build ([#6625](https://www.github.com/tauri-apps/tauri/pull/6625)) on 2023-04-05
- Use local ip address for built-in dev server on mobile.
  - [7fec0f08](https://www.github.com/tauri-apps/tauri/commit/7fec0f083c932dc63ccb8716080d97e2ab985b25) fix(cli): use local ip addr for built-in server on mobile, closes [#6454](https://www.github.com/tauri-apps/tauri/pull/6454) ([#6631](https://www.github.com/tauri-apps/tauri/pull/6631)) on 2023-04-04
- Change minimum Android SDK version to 21 for the plugin library.
  - [db4c9dc6](https://www.github.com/tauri-apps/tauri/commit/db4c9dc655e07ee2184fe04571f500f7910890cd) feat(core): add option to configure Android's minimum SDK version ([#6651](https://www.github.com/tauri-apps/tauri/pull/6651)) on 2023-04-07
- Readd the Cargo.toml file to the plugin template.
  - [5288a386](https://www.github.com/tauri-apps/tauri/commit/5288a386f1bf8ac11f991350463c3f5c20983f43) fix(cli): readd Cargo.toml to the plugin template ([#6637](https://www.github.com/tauri-apps/tauri/pull/6637)) on 2023-04-04

## \[2.0.0-alpha.6]

- Use Ubuntu 20.04 to compile the CLI for cargo-binstall, increasing the minimum libc required.
- Automatically enable the `rustls-tls` tauri feature on mobile and `native-tls` on desktop if `rustls-tls` is not enabled.
  - [cfdee00f](https://www.github.com/tauri-apps/tauri/commit/cfdee00f2b1455a9719bc44823fdaeabbe4c1cb2) refactor(core): fix tls features, use rustls on mobile ([#6591](https://www.github.com/tauri-apps/tauri/pull/6591)) on 2023-03-30

## \[2.0.0-alpha.5]

- Fixes the iOS project script to build the Rust library.
  - [6e3e4c22](https://www.github.com/tauri-apps/tauri/commit/6e3e4c22be51500bec7856d90dcb2e40ef7fe1b4) fix(cli): use correct variable on script to build Rust iOS code ([#6581](https://www.github.com/tauri-apps/tauri/pull/6581)) on 2023-03-29
- Fix `tauri android build/dev` crashing when used with standalone `pnpm` executable on Windows.
  - [39df2c98](https://www.github.com/tauri-apps/tauri/commit/39df2c982e5e2ee8617b40f829a2f2e4abfce412) fix(cli/android): fallback to `${program}.cmd` ([#6576](https://www.github.com/tauri-apps/tauri/pull/6576)) on 2023-03-29

## \[2.0.0-alpha.4]

- Fix android project build crashing when using `pnpm` caused by extra `--`.
  - [c787f749](https://www.github.com/tauri-apps/tauri/commit/c787f749de01b79d891615aad8c37b23037fff4c) fix(cli): only add `--` to generated android template for npm ([#6508](https://www.github.com/tauri-apps/tauri/pull/6508)) on 2023-03-21
- Fixes the Android build gradle plugin implementation on Windows.
  - [00241fa9](https://www.github.com/tauri-apps/tauri/commit/00241fa92d104870068a701519340633cc35b716) fix(cli): append .cmd on the gradle plugin binary on Windows, fix [#6502](https://www.github.com/tauri-apps/tauri/pull/6502) ([#6503](https://www.github.com/tauri-apps/tauri/pull/6503)) on 2023-03-21

## \[2.0.0-alpha.3]

- Added `plugin android add` and `plugin ios add` commands to add mobile plugin functionality to existing projects.
  - [14d03d42](https://www.github.com/tauri-apps/tauri/commit/14d03d426e86d966950a790926c04560c76203b3) refactor(cli): enhance plugin commands for mobile ([#6289](https://www.github.com/tauri-apps/tauri/pull/6289)) on 2023-02-16
- Add `--port` to specify the port used for static files dev server. It can also be specified through `TAURI_DEV_SERVER_PORT` env var.
  - [b7a2ce2c](https://www.github.com/tauri-apps/tauri/commit/b7a2ce2c633c8383851ec9ec3c2cafda39f19745) feat(cli): add --port, closes [#6186](https://www.github.com/tauri-apps/tauri/pull/6186) ([#6283](https://www.github.com/tauri-apps/tauri/pull/6283)) on 2023-03-16
- Auto select an external IP for mobile development and fallback to prompting the user. Use `--force-ip-prompt` to force prompting.
  - [ec007ef0](https://www.github.com/tauri-apps/tauri/commit/ec007ef0d0852d8ee6e3247049916285c98a945f) feat: use `local_ip()` and fallback to prompt ([#6290](https://www.github.com/tauri-apps/tauri/pull/6290)) on 2023-02-16
  - [4d090744](https://www.github.com/tauri-apps/tauri/commit/4d09074454068ae282ccd4670ae0b1df30510a3a) feat(cli): add `--force-ip-prompt` ([#6406](https://www.github.com/tauri-apps/tauri/pull/6406)) on 2023-03-16
- Add commands to add native Android and iOS functionality to plugins.
  - [05dad087](https://www.github.com/tauri-apps/tauri/commit/05dad0876842e2a7334431247d49365cee835d3e) feat: initial work for iOS plugins ([#6205](https://www.github.com/tauri-apps/tauri/pull/6205)) on 2023-02-11
- In mobile commands, correctly detect when nodejs binary has the version in its name, for example `node-18`
  - [58d4709f](https://www.github.com/tauri-apps/tauri/commit/58d4709f7eba07e77d3afd59c0df47c085df9e2c) fix: update nodejs detection in mobile commands ([#6451](https://www.github.com/tauri-apps/tauri/pull/6451)) on 2023-03-16
- Use temp file instead of environment variable to pass CLI IPC websocket address to the IDE.
  - [894a8d06](https://www.github.com/tauri-apps/tauri/commit/894a8d060c12a482a0fc5b3714f3848189b809de) refactor(cli): use temp file to communicate IPC websocket address ([#6219](https://www.github.com/tauri-apps/tauri/pull/6219)) on 2023-02-08
- Change the Android template to enable minification on release and pull ProGuard rules from proguard-tauri.pro.
  - [bef4ef51](https://www.github.com/tauri-apps/tauri/commit/bef4ef51bc2c633b88db121c2087a38dddb7d6bf) feat(android): enable minify on release, add proguard rules ([#6257](https://www.github.com/tauri-apps/tauri/pull/6257)) on 2023-02-13
- Print an error if the Android project was generated with an older bundle identifier or package name.
  - [79eb0542](https://www.github.com/tauri-apps/tauri/commit/79eb054292b04bb089a0e4df401a5986b33b691e) feat(cli): handle Android package identifier change ([#6314](https://www.github.com/tauri-apps/tauri/pull/6314)) on 2023-02-19
- Fixes the generated mobile build script when using an NPM runner.
  - [62f15265](https://www.github.com/tauri-apps/tauri/commit/62f152659204ce1218178596f463f0bcfbd4e6dc) fix(cli): generate build script using NPM runner if it was used ([#6233](https://www.github.com/tauri-apps/tauri/pull/6233)) on 2023-02-10
- Resolve Android package name from single word bundle identifiers.
  - [60a8b07d](https://www.github.com/tauri-apps/tauri/commit/60a8b07dc7c56c9c45331cb57d9afb410e7eadf3) fix: handle single word bundle identifier when resolving Android domain ([#6313](https://www.github.com/tauri-apps/tauri/pull/6313)) on 2023-02-19
- Update Android project template with fix to crash on orientation change.
  - [947eb391](https://www.github.com/tauri-apps/tauri/commit/947eb391ca41cebdb11abd9ffaec642baffbf44a) fix(android): crash on orientation change due to activity recreation ([#6261](https://www.github.com/tauri-apps/tauri/pull/6261)) on 2023-02-13
- Added `--ios-color` option to the `tauri icon` command.
  - [67755425](https://www.github.com/tauri-apps/tauri/commit/677554257e40e05b1af0dd61c982d6be8a8a033c) feat(cli): add `--ios-color` option to set iOS icon background color ([#6247](https://www.github.com/tauri-apps/tauri/pull/6247)) on 2023-02-12
- Fixes HMR on mobile when devPath is configured to load a filesystem path.
  - [4a82da29](https://www.github.com/tauri-apps/tauri/commit/4a82da2919e0564ec993b2005dc65b5b49407b36) fix(cli): use local ip address for reload ([#6285](https://www.github.com/tauri-apps/tauri/pull/6285)) on 2023-02-16
- Ignore the `gen` folder on the dev watcher.
  - [cab4ff95](https://www.github.com/tauri-apps/tauri/commit/cab4ff95b98aeac88401c1fed2d8b8940e4180cb) fix(cli): ignore the `gen` folder on the dev watcher ([#6232](https://www.github.com/tauri-apps/tauri/pull/6232)) on 2023-02-09
- Correctly pass arguments from `npm run` to `tauri`.
  - [1b343bd1](https://www.github.com/tauri-apps/tauri/commit/1b343bd11686f47f24a87298d8192097c66250f6) fix(cli): use `npm run tauri -- foo` for correctly passing args to tauri ([#6448](https://www.github.com/tauri-apps/tauri/pull/6448)) on 2023-03-16
- Changed the `--api` flag on `plugin init` to `--no-api`.
  - [14d03d42](https://www.github.com/tauri-apps/tauri/commit/14d03d426e86d966950a790926c04560c76203b3) refactor(cli): enhance plugin commands for mobile ([#6289](https://www.github.com/tauri-apps/tauri/pull/6289)) on 2023-02-16

## \[2.0.0-alpha.2]

- Fixes `TAURI_*` environment variables for hook scripts on mobile commands.
  - [1af9be90](https://www.github.com/tauri-apps/tauri/commit/1af9be904a309138b9f79dc741391000b1652c75) feat(cli): properly fill target for TAURI\_ env vars on mobile ([#6116](https://www.github.com/tauri-apps/tauri/pull/6116)) on 2023-01-23
- Force colored logs on mobile commands.
  - [2c4a0bbd](https://www.github.com/tauri-apps/tauri/commit/2c4a0bbd1fbe15d7500264e6490772397e1917ed) feat(cli): force colored logs on mobile commands ([#5934](https://www.github.com/tauri-apps/tauri/pull/5934)) on 2022-12-28
- Keep the process alive even when the iOS application is closed.
  - [dee9460f](https://www.github.com/tauri-apps/tauri/commit/dee9460f9c9bc92e9c638e7691e616849ac2085b) feat: keep CLI alive when iOS app exits, show logs, closes [#5855](https://www.github.com/tauri-apps/tauri/pull/5855) ([#5902](https://www.github.com/tauri-apps/tauri/pull/5902)) on 2022-12-27
- Show all application logs on iOS.
  - [dee9460f](https://www.github.com/tauri-apps/tauri/commit/dee9460f9c9bc92e9c638e7691e616849ac2085b) feat: keep CLI alive when iOS app exits, show logs, closes [#5855](https://www.github.com/tauri-apps/tauri/pull/5855) ([#5902](https://www.github.com/tauri-apps/tauri/pull/5902)) on 2022-12-27
- Print log output for all tags on Android development.
  - [8cc11149](https://www.github.com/tauri-apps/tauri/commit/8cc111494d74161e489152e52191e1442dd99759) fix(cli): print Android logs for all tags on 2023-01-17
- Add support to custom and kebab case library names for mobile apps.
  - [50f6dd87](https://www.github.com/tauri-apps/tauri/commit/50f6dd87b1ac2c99f8794b055f1acba4ef7d34d3) feat: improvements to support hyphens in crate name ([#5989](https://www.github.com/tauri-apps/tauri/pull/5989)) on 2023-01-06
- Bump the MSRV to 1.64.
  - [7eb9aa75](https://www.github.com/tauri-apps/tauri/commit/7eb9aa75cfd6a3176d3f566fdda02d88aa529b0f) Update gtk to 0.16 ([#6155](https://www.github.com/tauri-apps/tauri/pull/6155)) on 2023-01-30
- Fix target directory detection when compiling for Android.
  - [e873bae0](https://www.github.com/tauri-apps/tauri/commit/e873bae09f0f27517f720a753f51c1dcb903f883) fix(cli): Cargo target dir detection on Android, closes [#5865](https://www.github.com/tauri-apps/tauri/pull/5865) ([#5932](https://www.github.com/tauri-apps/tauri/pull/5932)) on 2022-12-28

## \[2.0.0-alpha.1]

- Fixes running on device using Xcode 14.
  - [1e4a6758](https://www.github.com/tauri-apps/tauri/commit/1e4a675843c486bddc11292d09fb766e98758514) fix(cli): run on iOS device on Xcode 14 ([#5807](https://www.github.com/tauri-apps/tauri/pull/5807)) on 2022-12-12
- Improve local IP address detection with user selection.
  - [76204b89](https://www.github.com/tauri-apps/tauri/commit/76204b893846a04552f8f8b87ad2c9b55e1b417f) feat(cli): improve local IP detection ([#5817](https://www.github.com/tauri-apps/tauri/pull/5817)) on 2022-12-12

## \[2.0.0-alpha.0]

- Added `android build` command.
  - [4c9ea450](https://www.github.com/tauri-apps/tauri/commit/4c9ea450c3b47c6b8c825ba32e9837909945ccd7) feat(cli): add `android build` command ([#4999](https://www.github.com/tauri-apps/tauri/pull/4999)) on 2022-08-22
- Added `ios build` command.
  - [403859d4](https://www.github.com/tauri-apps/tauri/commit/403859d47e1a9bf978b353fa58e4b971e66337a3) feat(cli): add `ios build` command ([#5002](https://www.github.com/tauri-apps/tauri/pull/5002)) on 2022-08-22
- Added `android dev` and `ios dev` commands.
  - [6f061504](https://www.github.com/tauri-apps/tauri/commit/6f0615044d09ec58393a7ebca5e45bb175e20db3) feat(cli): add `android dev` and `ios dev` commands ([#4982](https://www.github.com/tauri-apps/tauri/pull/4982)) on 2022-08-20
- Added `android init` and `ios init` commands.
  - [d44f67f7](https://www.github.com/tauri-apps/tauri/commit/d44f67f7afd30a81d53a973ec603b2a253150bde) feat: add `android init` and `ios init` commands ([#4942](https://www.github.com/tauri-apps/tauri/pull/4942)) on 2022-08-15
- Added `android open` and `ios open` commands.
  - [a9c8e565](https://www.github.com/tauri-apps/tauri/commit/a9c8e565c6495961940877df7090f307be16b554) feat: add `android open` and `ios open` commands ([#4946](https://www.github.com/tauri-apps/tauri/pull/4946)) on 2022-08-15
- First mobile alpha release!
  - [fa3a1098](https://www.github.com/tauri-apps/tauri/commit/fa3a10988a03aed1b66fb17d893b1a9adb90f7cd) feat(ci): prepare 2.0.0-alpha.0 ([#5786](https://www.github.com/tauri-apps/tauri/pull/5786)) on 2022-12-08

## \[1.5.10]

### New Features

- [`89911296`](https://www.github.com/tauri-apps/tauri/commit/89911296e475d5c36f3486b9b75232505846e767)([#8259](https://www.github.com/tauri-apps/tauri/pull/8259)) On macOS, support for signing nested .dylib, .app, .xpc and .framework under predefined directories inside the bundled frameworks ("MacOS", "Frameworks", "Plugins", "Helpers", "XPCServices" and "Libraries").

### Bug Fixes

- [`b0f27814`](https://www.github.com/tauri-apps/tauri/commit/b0f27814b90ded2f1ed44b7852080eedbff0d9e4)([#8776](https://www.github.com/tauri-apps/tauri/pull/8776)) Fix `fail to rename app` when using `--profile dev`.
- [`0bff8c32`](https://www.github.com/tauri-apps/tauri/commit/0bff8c325d004fdead2023f58e0f5fd73a9c22ba)([#8697](https://www.github.com/tauri-apps/tauri/pull/8697)) Fix the built-in dev server failing to serve files when URL had queries `?` and other url components.
- [`67d7877f`](https://www.github.com/tauri-apps/tauri/commit/67d7877f27f265c133a70d48a46c83ffff31d571)([#8520](https://www.github.com/tauri-apps/tauri/pull/8520)) The cli now also watches cargo workspace members if the tauri folder is the workspace root.

### Dependencies

- Upgraded to `tauri-bundler@1.5.0`

## \[1.5.9]

### Bug Fixes

- [`0a2175ea`](https://www.github.com/tauri-apps/tauri/commit/0a2175eabb736b2a4cd01ab682e08be0b5ebb2b9)([#8439](https://www.github.com/tauri-apps/tauri/pull/8439)) Expand glob patterns in workspace member paths so the CLI would watch all matching pathhs.

### Dependencies

- Upgraded to `tauri-bundler@1.4.8`
- Upgraded to `tauri-utils@1.5.2`

## \[1.5.8]

### Dependencies

- Upgraded to `tauri-bundler@1.4.7`

## \[1.5.7]

### Bug Fixes

- [`1d5aa38a`](https://www.github.com/tauri-apps/tauri/commit/1d5aa38ae418ea31f593590b6d32cf04d3bfd8c1)([#8162](https://www.github.com/tauri-apps/tauri/pull/8162)) Fixes errors on command output, occuring when the output stream contains an invalid UTF-8 character, or ends with a multi-bytes UTF-8 character.
- [`f26d9f08`](https://www.github.com/tauri-apps/tauri/commit/f26d9f0884f63f61b9f4d4fac15e6b251163793e)([#8263](https://www.github.com/tauri-apps/tauri/pull/8263)) Fixes an issue in the NSIS installer which caused the uninstallation to leave empty folders on the system if the `resources` feature was used.
- [`92bc7d0e`](https://www.github.com/tauri-apps/tauri/commit/92bc7d0e16157434330a1bcf1eefda6f0f1e5f85)([#8233](https://www.github.com/tauri-apps/tauri/pull/8233)) Fixes an issue in the NSIS installer which caused the installation to take much longer than expected when many `resources` were added to the bundle.

### Dependencies

- Upgraded to `tauri-bundler@1.4.6`

## \[1.5.6]

### Bug Fixes

- [`5264e41d`](https://www.github.com/tauri-apps/tauri/commit/5264e41db3763e4c2eb0c3c21bd423fb7bece3e2)([#8082](https://www.github.com/tauri-apps/tauri/pull/8082)) Downgraded `rust-minisign` to `0.7.3` to fix signing updater bundles with empty passwords.

### Dependencies

- Upgraded to `tauri-bundler@1.4.5`

## \[1.5.5]

### Enhancements

- [`9bead42d`](https://www.github.com/tauri-apps/tauri/commit/9bead42dbca0fb6dd7ea0b6bfb2f2308a5c5f992)([#8059](https://www.github.com/tauri-apps/tauri/pull/8059)) Allow rotating the updater private key.

### Bug Fixes

- [`be8e5aa3`](https://www.github.com/tauri-apps/tauri/commit/be8e5aa3071d9bc5d0bd24647e8168f312d11c8d)([#8042](https://www.github.com/tauri-apps/tauri/pull/8042)) Fixes duplicated newlines on command outputs.

### Dependencies

- Upgraded to `tauri-bundler@1.4.4`

## \[1.5.4]

### Dependencies

- Upgraded to `tauri-bundler@1.4.3`

## \[1.5.3]

### Dependencies

- Upgraded to `tauri-bundler@1.4.2`

## \[1.5.2]

### Dependencies

- Upgraded to `tauri-bundler@1.4.1`

## \[1.5.1]

### Bug Fixes

- [`d6eb46cf`](https://www.github.com/tauri-apps/tauri/commit/d6eb46cf1116d147121f6b6db9d390b5e2fb238d)([#7934](https://www.github.com/tauri-apps/tauri/pull/7934)) On macOS, fix the `apple-id` option name when using `notarytools submit`.

## \[1.5.0]

### New Features

- [`e1526626`](https://www.github.com/tauri-apps/tauri/commit/e152662687ece7a62d383923a50751cc0dd34331)([#7723](https://www.github.com/tauri-apps/tauri/pull/7723)) Support Bun package manager in CLI

### Enhancements

- [`13279917`](https://www.github.com/tauri-apps/tauri/commit/13279917d4cae071d0ce3a686184d48af079f58a)([#7713](https://www.github.com/tauri-apps/tauri/pull/7713)) Add version of Rust Tauri CLI installed with Cargo to `tauri info` command.

### Bug Fixes

- [`dad4f54e`](https://www.github.com/tauri-apps/tauri/commit/dad4f54eec9773d2ea6233a7d9fd218741173823)([#7277](https://www.github.com/tauri-apps/tauri/pull/7277)) Removed the automatic version check of the CLI that ran after `tauri` commands which caused various issues.

### Dependencies

- Upgraded to `tauri-bundler@1.4.0`
- Upgraded to `tauri-utils@1.5.0`

## \[1.4.0]

### New Features

- [`0ddbb3a1`](https://www.github.com/tauri-apps/tauri/commit/0ddbb3a1dc1961ba5c6c1a60081513c1380c8af1)([#7015](https://www.github.com/tauri-apps/tauri/pull/7015)) Provide prebuilt CLIs for Windows ARM64 targets.
- [`35cd751a`](https://www.github.com/tauri-apps/tauri/commit/35cd751adc6fef1f792696fa0cfb471b0bf99374)([#5176](https://www.github.com/tauri-apps/tauri/pull/5176)) Added the `desktop_template` option on `tauri.conf.json > tauri > bundle > deb`.
- [`6c5ade08`](https://www.github.com/tauri-apps/tauri/commit/6c5ade08d97844bb685789d30e589400bbe3e04c)([#4537](https://www.github.com/tauri-apps/tauri/pull/4537)) Added `tauri completions` to generate shell completions scripts.
- [`29488205`](https://www.github.com/tauri-apps/tauri/commit/2948820579d20dfaa0861c2f0a58bd7737a7ffd1)([#6867](https://www.github.com/tauri-apps/tauri/pull/6867)) Allow specifying custom language files of Tauri's custom messages for the NSIS installer
- [`e092f799`](https://www.github.com/tauri-apps/tauri/commit/e092f799469ff32c7d1595d0f07d06fd2dab5c29)([#6887](https://www.github.com/tauri-apps/tauri/pull/6887)) Add `nsis > template` option to specify custom NSIS installer template.

### Enhancements

- [`d75c1b82`](https://www.github.com/tauri-apps/tauri/commit/d75c1b829bd96d9e3a672bcc79120597d5ada4a0)([#7181](https://www.github.com/tauri-apps/tauri/pull/7181)) Print a useful error when `updater` bundle target is specified without an updater-enabled target.
- [`52474e47`](https://www.github.com/tauri-apps/tauri/commit/52474e479d695865299d8c8d868fb98b99731020)([#7141](https://www.github.com/tauri-apps/tauri/pull/7141)) Enhance injection of Cargo features.
- [`2659ca1a`](https://www.github.com/tauri-apps/tauri/commit/2659ca1ab4799a5bda65c229c149e98bd01eb1ee)([#6900](https://www.github.com/tauri-apps/tauri/pull/6900)) Add `rustls` as default Cargo feature.
- [`c7056d1b`](https://www.github.com/tauri-apps/tauri/commit/c7056d1b202927d025aa479818c2fb20bf5d6113)([#6982](https://www.github.com/tauri-apps/tauri/pull/6982)) Improve Visual Studio installation detection in `tauri info` command to check for the necessary components instead of whole workloads. This also fixes the detection of minimal installations and auto-installations done by `rustup`.

### Bug Fixes

- [`3cb7a3e6`](https://www.github.com/tauri-apps/tauri/commit/3cb7a3e642bb10ee90dc1d24daa48b8c8c15c9ce)([#6997](https://www.github.com/tauri-apps/tauri/pull/6997)) Fix built-in devserver adding hot-reload code to non-html files.
- [`fd3b5a16`](https://www.github.com/tauri-apps/tauri/commit/fd3b5a16b13d62f62c850e68763401916dde26f2)([#6954](https://www.github.com/tauri-apps/tauri/pull/6954)) Fix building with a custom cargo profile
- [`1253bbf7`](https://www.github.com/tauri-apps/tauri/commit/1253bbf7ae11a87887e0b3bd98cc26dbb98c8130)([#7013](https://www.github.com/tauri-apps/tauri/pull/7013)) Fixes Cargo.toml feature rewriting.

## \[1.3.1]

- Correctly escape XML for resource files in WiX bundler.
  - Bumped due to a bump in tauri-bundler.
  - [6a6b1388](https://www.github.com/tauri-apps/tauri/commit/6a6b1388ea5787aea4c0e0b0701a4772087bbc0f) fix(bundler): correctly escape resource xml, fixes [#6853](https://www.github.com/tauri-apps/tauri/pull/6853) ([#6855](https://www.github.com/tauri-apps/tauri/pull/6855)) on 2023-05-04

- Added the following languages to the NSIS bundler:

- `Spanish`

- `SpanishInternational`

- Bumped due to a bump in tauri-bundler.

- [422b4817](https://www.github.com/tauri-apps/tauri/commit/422b48179856504e980a156500afa8e22c44d357) Add Spanish and SpanishInternational languages ([#6871](https://www.github.com/tauri-apps/tauri/pull/6871)) on 2023-05-06

- Correctly escape arguments in NSIS script to fix bundling apps that use non-default WebView2 install modes.
  - Bumped due to a bump in tauri-bundler.
  - [2915bd06](https://www.github.com/tauri-apps/tauri/commit/2915bd068ed40dc01a363b69212c6b6f2d3ec01e) fix(bundler): Fix webview install modes in NSIS bundler ([#6854](https://www.github.com/tauri-apps/tauri/pull/6854)) on 2023-05-04

## \[1.3.0]

- Look for available port when using the built-in dev server for static files.
  - [a7ee5ca7](https://www.github.com/tauri-apps/tauri/commit/a7ee5ca7c348d33bdfdc1213b6850bcf5c39d6e6) fix(cli): look for available ports for built-in dev server, closes [#6511](https://www.github.com/tauri-apps/tauri/pull/6511) ([#6514](https://www.github.com/tauri-apps/tauri/pull/6514)) on 2023-03-31
- Add `--port` to specify the port used for static files dev server. It can also be specified through `TAURI_DEV_SERVER_PORT` env var.
  - [b7a2ce2c](https://www.github.com/tauri-apps/tauri/commit/b7a2ce2c633c8383851ec9ec3c2cafda39f19745) feat(cli): add --port, closes [#6186](https://www.github.com/tauri-apps/tauri/pull/6186) ([#6283](https://www.github.com/tauri-apps/tauri/pull/6283)) on 2023-03-16
- Fix `tauri info` panicking when parsing crates version on a newly created project without a `Cargo.lock` file.
  - [c2608423](https://www.github.com/tauri-apps/tauri/commit/c2608423b6eec5eb91d0ffc861714c011ad3988b) fix(cli): don't panic when a crate version couldn't be parsed ([#5873](https://www.github.com/tauri-apps/tauri/pull/5873)) on 2022-12-26
- Improve the error message when `rustc` couldn't be found.
  - [7aab3e20](https://www.github.com/tauri-apps/tauri/commit/7aab3e2076272c14c78a563e288a1b04ed3cfd41) fix(cli.rs): improve `rustc` not found error msg ([#6021](https://www.github.com/tauri-apps/tauri/pull/6021)) on 2023-01-17
- Add `--ci` flag and respect the `CI` environment variable on the `signer generate` command. In this case the default password will be an empty string and the CLI will not prompt for a value.
  - [8fb1df8a](https://www.github.com/tauri-apps/tauri/commit/8fb1df8aa65a52cdb4a7e1bb9dda9b912a7a2895) feat(cli): add `--ci` flag to `signer generate`, closes [#6089](https://www.github.com/tauri-apps/tauri/pull/6089) ([#6097](https://www.github.com/tauri-apps/tauri/pull/6097)) on 2023-01-19
- Fix Outdated Github Actions in the Plugin Templates `with-api` and `backend`
  - [a926b49a](https://www.github.com/tauri-apps/tauri/commit/a926b49a01925ca757d391994bfac3beea29599b) Fix Github Actions of Tauri Plugin with-api template ([#6603](https://www.github.com/tauri-apps/tauri/pull/6603)) on 2023-04-03
- Do not crash on Cargo.toml watcher.
  - [e8014a7f](https://www.github.com/tauri-apps/tauri/commit/e8014a7f612a1094461ddad63aacc498a2682ff5) fix(cli): do not crash on watcher ([#6303](https://www.github.com/tauri-apps/tauri/pull/6303)) on 2023-02-17
- On Windows, printing consistent paths on Windows with backslashs only.
  - [9da99607](https://www.github.com/tauri-apps/tauri/commit/9da996073ff07d4b59668a5315d40e9bc578e340) fix(cli): fix printing paths on Windows ([#6137](https://www.github.com/tauri-apps/tauri/pull/6137)) on 2023-01-26
- Add `--png` option for the `icon` command to generate custom icon sizes.
  - [9d214412](https://www.github.com/tauri-apps/tauri/commit/9d2144128fc5fad67d8404bce95f82297ebb0e4a) feat(cli): add option to make custom icon sizes, closes [#5121](https://www.github.com/tauri-apps/tauri/pull/5121) ([#5246](https://www.github.com/tauri-apps/tauri/pull/5246)) on 2022-12-27
- Skip the password prompt on the build command when `TAURI_KEY_PASSWORD` environment variable is empty and the `--ci` argument is provided or the `CI` environment variable is set.
  - [d4f89af1](https://www.github.com/tauri-apps/tauri/commit/d4f89af18d69fd95a4d8a1ede8442547c6a6d0ee) feat: skip password prompt on the build command if CI is set fixes [#6089](https://www.github.com/tauri-apps/tauri/pull/6089) on 2023-01-18
- Fix `default-run` not deserialized.
  - [57c6bf07](https://www.github.com/tauri-apps/tauri/commit/57c6bf07bb380847abdf27c3fff9891d99c1c98c) fix(cli): fix default-run not deserialized ([#6584](https://www.github.com/tauri-apps/tauri/pull/6584)) on 2023-03-30
- Fixes HTML serialization removing template tags on the dev server.
  - [314f0e21](https://www.github.com/tauri-apps/tauri/commit/314f0e212fd2b9e452bfe3424cdce2b0bf37b5d7) fix(cli): web_dev_server html template serialization (fix [#6165](https://www.github.com/tauri-apps/tauri/pull/6165)) ([#6166](https://www.github.com/tauri-apps/tauri/pull/6166)) on 2023-01-29
- Use escaping on Handlebars templates.
  - [6d6b6e65](https://www.github.com/tauri-apps/tauri/commit/6d6b6e653ea70fc02794f723092cdc860995c259) feat: configure escaping on handlebars templates ([#6678](https://www.github.com/tauri-apps/tauri/pull/6678)) on 2023-05-02
- Fix building apps with unicode characters in their `productName`.
  - [72621892](https://www.github.com/tauri-apps/tauri/commit/72621892fe8195bad67b4237467ebd7e89f6af7f) fix(cli): use `unicode` feature for `heck` crate, closes [#5860](https://www.github.com/tauri-apps/tauri/pull/5860) ([#5872](https://www.github.com/tauri-apps/tauri/pull/5872)) on 2022-12-26
- Bump minimum supported Rust version to 1.60.
  - [5fdc616d](https://www.github.com/tauri-apps/tauri/commit/5fdc616df9bea633810dcb814ac615911d77222c) feat: Use the zbus-backed of notify-rust ([#6332](https://www.github.com/tauri-apps/tauri/pull/6332)) on 2023-03-31
- Add initial support for building `nsis` bundles on non-Windows platforms.
  - [60e6f6c3](https://www.github.com/tauri-apps/tauri/commit/60e6f6c3f1605f3064b5bb177992530ff788ccf0) feat(bundler): Add support for creating NSIS bundles on unix hosts ([#5788](https://www.github.com/tauri-apps/tauri/pull/5788)) on 2023-01-19
- Add `nsis` bundle target
  - [c94e1326](https://www.github.com/tauri-apps/tauri/commit/c94e1326a7c0767a13128a8b1d327a00156ece12) feat(bundler): add `nsis`, closes [#4450](https://www.github.com/tauri-apps/tauri/pull/4450), closes [#2319](https://www.github.com/tauri-apps/tauri/pull/2319) ([#4674](https://www.github.com/tauri-apps/tauri/pull/4674)) on 2023-01-03
- Remove default features from Cargo.toml template.
  - [b08ae637](https://www.github.com/tauri-apps/tauri/commit/b08ae637a0f58b38cbce9b8a1fa0b6c5dc0cfd05) fix(cli): remove default features from template ([#6074](https://www.github.com/tauri-apps/tauri/pull/6074)) on 2023-01-17
- Added support for Cargo's workspace inheritance for package information. The cli now also detects inherited `tauri` and `tauri-build` dependencies and disables manifest rewrites accordingly.
  - [cd8c074a](https://www.github.com/tauri-apps/tauri/commit/cd8c074ae6592303d3f6844a4fb6d262eae913b2) feat(cli): add support for Cargo's workspace inheritance for the package version, closes [#5070](https://www.github.com/tauri-apps/tauri/pull/5070) ([#5775](https://www.github.com/tauri-apps/tauri/pull/5775)) on 2022-12-14
  - [d20a7288](https://www.github.com/tauri-apps/tauri/commit/d20a728892eee1858ab525ab6216cd721f473ab5) feat: Further improve workspace inheritance, closes [#6122](https://www.github.com/tauri-apps/tauri/pull/6122), [#5070](https://www.github.com/tauri-apps/tauri/pull/5070) ([#6144](https://www.github.com/tauri-apps/tauri/pull/6144)) on 2023-01-26
- Use Ubuntu 20.04 to compile the CLI for cargo-binstall, increasing the minimum libc required.

## \[1.2.3]

- Pin `ignore` to `=0.4.18`.
  - [adcb082b](https://www.github.com/tauri-apps/tauri/commit/adcb082b1651ecb2a6208b093e12f4185aa3fc98) chore(deps): pin `ignore` to =0.4.18 on 2023-01-17

## \[1.2.2]

- Detect SvelteKit and Vite for the init and info commands.
  - [9d872ab8](https://www.github.com/tauri-apps/tauri/commit/9d872ab8728b1b121909af434adcd5936e5afb7d) feat(cli): detect SvelteKit and Vite ([#5742](https://www.github.com/tauri-apps/tauri/pull/5742)) on 2022-12-02
- Detect SolidJS and SolidStart for the init and info commands.
  - [9e7ce0a8](https://www.github.com/tauri-apps/tauri/commit/9e7ce0a8eef4bf3536645976e3e09162fbf772ab) feat(cli): detect SolidJS and SolidStart ([#5758](https://www.github.com/tauri-apps/tauri/pull/5758)) on 2022-12-08
- Use older icon types to work around a macOS bug resulting in corrupted 16x16px and 32x32px icons in bundled apps.
  - [2d545eff](https://www.github.com/tauri-apps/tauri/commit/2d545eff58734ec70f23f11a429d35435cdf090e) fix(cli): corrupted icons in bundled macOS icons ([#5698](https://www.github.com/tauri-apps/tauri/pull/5698)) on 2022-11-28
- Add `--no-dev-server` flag to the cli to disable the dev server for static files in dev mode.
  - [c0989848](https://www.github.com/tauri-apps/tauri/commit/c0989848b9421fb19070ae652a89a5d5675deab8) feat(cli/dev): add `--no-dev-server`, ref [#5708](https://www.github.com/tauri-apps/tauri/pull/5708) ([#5722](https://www.github.com/tauri-apps/tauri/pull/5722)) on 2022-11-30

## \[1.2.1]

- Fixes injection of Cargo features defined in the configuration file.
  - [1ecaeb29](https://www.github.com/tauri-apps/tauri/commit/1ecaeb29aa798f591f6488dc6c3a7a8d22f6073e) fix(cli): inject config feature flags when features arg is not provided on 2022-11-18

## \[1.2.0]

- Keep `tauri dev` watcher alive when the configuration is invalid.
  - [cc186c7a](https://www.github.com/tauri-apps/tauri/commit/cc186c7a0eab1c364f8b58101f86979ae4ed3d03) fix(cli): keep dev watcher alive if config is incorrect, closes [#5173](https://www.github.com/tauri-apps/tauri/pull/5173) ([#5495](https://www.github.com/tauri-apps/tauri/pull/5495)) on 2022-10-28
- Ignore workspace members in dev watcher if they are ignored by `.taurignore`
  - [9417ce40](https://www.github.com/tauri-apps/tauri/commit/9417ce401c4985e97245ce02d3b7cc31fb4bf59e) fix(cli): apply `.taurignore` rules to workspace members, closes [#5355](https://www.github.com/tauri-apps/tauri/pull/5355) ([#5460](https://www.github.com/tauri-apps/tauri/pull/5460)) on 2022-10-28
- Detect JSON5 and TOML configuration files in the dev watcher.
  - [e7ccbd85](https://www.github.com/tauri-apps/tauri/commit/e7ccbd8573f6b9124e80c0b67fa2365729c3c196) feat(cli): detect JSON5 and TOML configuration files in the dev watcher ([#5439](https://www.github.com/tauri-apps/tauri/pull/5439)) on 2022-10-19
- Fix cli passing `--no-default-features` to the app instead of the runner (Cargo).
  - [a3a70218](https://www.github.com/tauri-apps/tauri/commit/a3a70218f3cc438b4875a046a182ca44dab357ae) fix(cli): pass `--no-default-features` to runner instead of app, closes [#5415](https://www.github.com/tauri-apps/tauri/pull/5415) ([#5474](https://www.github.com/tauri-apps/tauri/pull/5474)) on 2022-10-25
- Validate `package > productName` in the tauri config and produce errors if it contains one of the following characters `/\:*?\"<>|`
  - [b9316a64](https://www.github.com/tauri-apps/tauri/commit/b9316a64eaa9348c79efafb8b94960d9b4d5b27a) fix(cli): validate `productName` in config, closes [#5233](https://www.github.com/tauri-apps/tauri/pull/5233) ([#5262](https://www.github.com/tauri-apps/tauri/pull/5262)) on 2022-09-28
- Hot-reload the frontend when `tauri.conf.json > build > devPath` points to a directory.
  - [54c337e0](https://www.github.com/tauri-apps/tauri/commit/54c337e06f3bc624c4780cf002bc54790f446c90) feat(cli): hotreload support for frontend static files, closes [#2173](https://www.github.com/tauri-apps/tauri/pull/2173) ([#5256](https://www.github.com/tauri-apps/tauri/pull/5256)) on 2022-09-28
- Expose `TAURI_TARGET_TRIPLE` to `beforeDevCommand`, `beforeBuildCommand` and `beforeBundleCommand`
  - [a4aec9f0](https://www.github.com/tauri-apps/tauri/commit/a4aec9f0a864ec6d7712db8bd50989d9c2e2fd2e) feat(cli): expose `TAURI_TARGET_TRIPLE` to before\*Commands, closes [#5091](https://www.github.com/tauri-apps/tauri/pull/5091) ([#5101](https://www.github.com/tauri-apps/tauri/pull/5101)) on 2022-10-03
- Log dev watcher file change detection.
  - [9076d5d2](https://www.github.com/tauri-apps/tauri/commit/9076d5d2e76d432aef475ba403e9ab5bd3b9d2b0) feat(cli): add prompt information when file changing detected, closes [#5417](https://www.github.com/tauri-apps/tauri/pull/5417) ([#5428](https://www.github.com/tauri-apps/tauri/pull/5428)) on 2022-10-19
- Set `TAURI_PLATFORM_TYPE`, `TAURI_FAMILY`, `TAURI_ARCH` and `TAURI_PLATFORM` env vars for hook commands to based on the app not the cli.
  - [a4aec9f0](https://www.github.com/tauri-apps/tauri/commit/a4aec9f0a864ec6d7712db8bd50989d9c2e2fd2e) feat(cli): expose `TAURI_TARGET_TRIPLE` to before\*Commands, closes [#5091](https://www.github.com/tauri-apps/tauri/pull/5091) ([#5101](https://www.github.com/tauri-apps/tauri/pull/5101)) on 2022-10-03
- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28
- Add `tauri.conf.json > bundle > publisher` field to specify the app publisher.
  - [628285c1](https://www.github.com/tauri-apps/tauri/commit/628285c1cf43f03ed62378f3b6cc0c991317526f) feat(bundler): add `publisher` field, closes [#5273](https://www.github.com/tauri-apps/tauri/pull/5273) ([#5283](https://www.github.com/tauri-apps/tauri/pull/5283)) on 2022-09-28
- Changed the project template to not enable all APIs by default.
  - [582c25a0](https://www.github.com/tauri-apps/tauri/commit/582c25a0f0fa2725d786ec4edd0defe7811ad6e8) refactor(cli): disable api-all on templates ([#5538](https://www.github.com/tauri-apps/tauri/pull/5538)) on 2022-11-03

## \[1.1.1]

- Fix wrong cli metadata that caused new projects (created through `tauri init`) fail to build
  - [db26aaf2](https://www.github.com/tauri-apps/tauri/commit/db26aaf2b44ce5335c9223c571ef2b2175e0cd6d) fix: fix wrong cli metadata ([#5214](https://www.github.com/tauri-apps/tauri/pull/5214)) on 2022-09-16

## \[1.1.0]

- Allow adding `build > beforeBundleCommand` in tauri.conf.json to run a shell command before the bundling phase.
  - [57ab9847](https://www.github.com/tauri-apps/tauri/commit/57ab9847eb2d8c9a5da584b873b7c072e9ee26bf) feat(cli): add `beforeBundleCommand`, closes [#4879](https://www.github.com/tauri-apps/tauri/pull/4879) ([#4893](https://www.github.com/tauri-apps/tauri/pull/4893)) on 2022-08-09
- Change `before_dev_command` and `before_build_command` config value to allow configuring the current working directory.
  - [d6f7d3cf](https://www.github.com/tauri-apps/tauri/commit/d6f7d3cfe8a7ec8d68c8341016c4e0a3103da587) Add cwd option to `before` commands, add wait option to dev [#4740](https://www.github.com/tauri-apps/tauri/pull/4740) [#3551](https://www.github.com/tauri-apps/tauri/pull/3551) ([#4834](https://www.github.com/tauri-apps/tauri/pull/4834)) on 2022-08-02
- Allow configuring the `before_dev_command` to force the CLI to wait for the command to finish before proceeding.
  - [d6f7d3cf](https://www.github.com/tauri-apps/tauri/commit/d6f7d3cfe8a7ec8d68c8341016c4e0a3103da587) Add cwd option to `before` commands, add wait option to dev [#4740](https://www.github.com/tauri-apps/tauri/pull/4740) [#3551](https://www.github.com/tauri-apps/tauri/pull/3551) ([#4834](https://www.github.com/tauri-apps/tauri/pull/4834)) on 2022-08-02
- Check if the default build target is set in the Cargo configuration.
  - [436f3d8d](https://www.github.com/tauri-apps/tauri/commit/436f3d8d66727f5b64165522f0b55f4ab54bd1ae) feat(cli): load Cargo configuration to check default build target ([#4990](https://www.github.com/tauri-apps/tauri/pull/4990)) on 2022-08-21
- Add support to cargo-binstall.
  - [90d5929f](https://www.github.com/tauri-apps/tauri/commit/90d5929fea6df575a2aa3c0a749374358f1ddb9b) feat(cli.rs): add support to cargo-binstall, closes [#4651](https://www.github.com/tauri-apps/tauri/pull/4651) ([#4817](https://www.github.com/tauri-apps/tauri/pull/4817)) on 2022-08-02
- Use `cargo metadata` to detect the workspace root and target directory.
  - [fea70eff](https://www.github.com/tauri-apps/tauri/commit/fea70effad219c0794d919f8834fa1a1ffd204c7) refactor(cli): Use `cargo metadata` to detect the workspace root and target directory, closes [#4632](https://www.github.com/tauri-apps/tauri/pull/4632), [#4928](https://www.github.com/tauri-apps/tauri/pull/4928). ([#4932](https://www.github.com/tauri-apps/tauri/pull/4932)) on 2022-08-21
- Prompt for `beforeDevCommand` and `beforeBuildCommand` in `tauri init`.
  - [6d4945c9](https://www.github.com/tauri-apps/tauri/commit/6d4945c9f06cd1f7018e1c48686ba682aae817df) feat(cli): prompt for before\*Command, closes [#4691](https://www.github.com/tauri-apps/tauri/pull/4691) ([#4721](https://www.github.com/tauri-apps/tauri/pull/4721)) on 2022-07-25
- Add `icon` command to generate icons.
  - [12e9d811](https://www.github.com/tauri-apps/tauri/commit/12e9d811e69813e7e9db5344b422101ddc09589f) feat(cli): Add `icon` command (tauricon) ([#4992](https://www.github.com/tauri-apps/tauri/pull/4992)) on 2022-09-03
- Added support to configuration files in TOML format (Tauri.toml file).
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02
- Automatically use any `.taurignore` file as ignore rules for dev watcher and app path finder.
  - [596fa08d](https://www.github.com/tauri-apps/tauri/commit/596fa08d48e371c7bd29e1ef799119ac8fca0d0b) feat(cli): automatically use `.taurignore`, ref [#4617](https://www.github.com/tauri-apps/tauri/pull/4617) ([#4623](https://www.github.com/tauri-apps/tauri/pull/4623)) on 2022-07-28
- Enable WiX FIPS compliance when the `TAURI_FIPS_COMPLIANT` environment variable is set to `true`.
  - [d88b9de7](https://www.github.com/tauri-apps/tauri/commit/d88b9de7aaeaaa2e42e4795dbc2b8642b5ae7a50) feat(core): add `fips_compliant` wix config option, closes [#4541](https://www.github.com/tauri-apps/tauri/pull/4541) ([#4843](https://www.github.com/tauri-apps/tauri/pull/4843)) on 2022-08-04
- Fixes dev watcher incorrectly exiting the CLI when sequential file updates are detected.
  - [47fab680](https://www.github.com/tauri-apps/tauri/commit/47fab6809a1e23b3b9a84695e2d91ff0826ba79a) fix(cli): dev watcher incorrectly killing process on multiple file write ([#4684](https://www.github.com/tauri-apps/tauri/pull/4684)) on 2022-07-25
- Set the `MACOSX_DEPLOYMENT_TARGET` environment variable with the configuration `minimum_system_version` value.
  - [fa23310f](https://www.github.com/tauri-apps/tauri/commit/fa23310f23cb9e6a02ec2524f1ef394a5b42990e) fix(cli): set MACOSX_DEPLOYMENT_TARGET env var, closes [#4704](https://www.github.com/tauri-apps/tauri/pull/4704) ([#4842](https://www.github.com/tauri-apps/tauri/pull/4842)) on 2022-08-02
- Added `--no-watch` argument to the `dev` command to disable the file watcher.
  - [0983d7ce](https://www.github.com/tauri-apps/tauri/commit/0983d7ce7f24ab43f9ae7b5e1177ff244d8885a8) feat(cli): add `--no-watch` argument to the dev command, closes [#4617](https://www.github.com/tauri-apps/tauri/pull/4617) ([#4793](https://www.github.com/tauri-apps/tauri/pull/4793)) on 2022-07-29
- Validate updater signature matches configured public key.
  - [b2a8930b](https://www.github.com/tauri-apps/tauri/commit/b2a8930b3c4b72c50ce72e73575f42c9cbe91bad) feat(cli): validate updater private key when signing ([#4754](https://www.github.com/tauri-apps/tauri/pull/4754)) on 2022-07-25

## \[1.0.5]

- Correctly fill the architecture when building Debian packages targeting ARM64 (aarch64).
  - Bumped due to a bump in tauri-bundler.
  - [635f23b8](https://www.github.com/tauri-apps/tauri/commit/635f23b88adbb8726d628f67840709cd870836dc) fix(bundler): correctly set debian architecture for aarch64 ([#4700](https://www.github.com/tauri-apps/tauri/pull/4700)) on 2022-07-17

## \[1.0.4]

- Do not capture and force colors of `cargo build` output.
  - [c635a0da](https://www.github.com/tauri-apps/tauri/commit/c635a0dad437860d54109adffaf245b7c21bc684) refactor(cli): do not capture and force colors of cargo build output ([#4627](https://www.github.com/tauri-apps/tauri/pull/4627)) on 2022-07-12
- Reduce the amount of allocations when converting cases.
  - [bc370e32](https://www.github.com/tauri-apps/tauri/commit/bc370e326810446e15b1f50fb962b980114ba16b) feat: reduce the amount of `heck`-related allocations ([#4634](https://www.github.com/tauri-apps/tauri/pull/4634)) on 2022-07-11

## \[1.0.3]

- Changed the app template to not set the default app menu as it is now set automatically on macOS which is the platform that needs a menu to function properly.
  - [91055883](https://www.github.com/tauri-apps/tauri/commit/9105588373cc8401bd9ad79bdef26f509b2d76b7) feat: add implicit default menu for macOS only, closes [#4551](https://www.github.com/tauri-apps/tauri/pull/4551) ([#4570](https://www.github.com/tauri-apps/tauri/pull/4570)) on 2022-07-04
- Improved bundle identifier validation showing the exact source of the configuration value.
  - [8e3e7fc6](https://www.github.com/tauri-apps/tauri/commit/8e3e7fc64641afc7a6833bc93205e6f525562545) feat(cli): improve bundle identifier validation, closes [#4589](https://www.github.com/tauri-apps/tauri/pull/4589) ([#4596](https://www.github.com/tauri-apps/tauri/pull/4596)) on 2022-07-05
- Improve configuration deserialization error messages.
  - [9170c920](https://www.github.com/tauri-apps/tauri/commit/9170c9207044fa561535f624916dfdbaa41ff79d) feat(core): improve config deserialization error messages ([#4607](https://www.github.com/tauri-apps/tauri/pull/4607)) on 2022-07-06
- Skip the static link of the `vcruntime140.dll` if the `STATIC_VCRUNTIME` environment variable is set to `false`.
  - [2e61abaa](https://www.github.com/tauri-apps/tauri/commit/2e61abaa9ae5d7a41ca1fa6505b5d6c368625ce5) feat(cli): allow dynamic link vcruntime, closes [#4565](https://www.github.com/tauri-apps/tauri/pull/4565) ([#4601](https://www.github.com/tauri-apps/tauri/pull/4601)) on 2022-07-06
- The `TAURI_CONFIG` environment variable now represents the configuration to be merged instead of the entire JSON.
  - [fa028ebf](https://www.github.com/tauri-apps/tauri/commit/fa028ebf3c8ca7b43a70d283a01dbea86217594f) refactor: do not pass entire config from CLI to core, send patch instead ([#4598](https://www.github.com/tauri-apps/tauri/pull/4598)) on 2022-07-06
- Watch for Cargo workspace members in the `dev` file watcher.
  - [dbb8c87b](https://www.github.com/tauri-apps/tauri/commit/dbb8c87b96dec9942b1bf877b29bafb8246514d4) feat(cli): watch Cargo workspaces in the dev command, closes [#4222](https://www.github.com/tauri-apps/tauri/pull/4222) ([#4572](https://www.github.com/tauri-apps/tauri/pull/4572)) on 2022-07-03

## \[1.0.2]

- Fixes a crash on the `signer sign` command.
  - [8e808fec](https://www.github.com/tauri-apps/tauri/commit/8e808fece95f2e506acf2c446d37b9913fd67d50) fix(cli.rs): conflicts_with arg doesn't exist closes  ([#4538](https://www.github.com/tauri-apps/tauri/pull/4538)) on 2022-06-30

## \[1.0.1]

- No longer adds the `pkg-config` dependency to `.deb` packages when the `systemTray` is used.
  This only works with recent versions of `libappindicator-sys` (including https://github.com/tauri-apps/libappindicator-rs/pull/38),
  so a `cargo update` may be necessary if you create `.deb` bundles and use the tray feature.
  - [0e6edeb1](https://www.github.com/tauri-apps/tauri/commit/0e6edeb14f379af1e02a7cebb4e3a5c9e87ebf7f) fix(cli): Don't add `pkg-config` to `deb` ([#4508](https://www.github.com/tauri-apps/tauri/pull/4508)) on 2022-06-29
- AppImage bundling will now prefer bundling correctly named appindicator library (including `.1` version suffix). With a symlink for compatibility with the old naming.
  - [bf45ca1d](https://www.github.com/tauri-apps/tauri/commit/bf45ca1df6691c05bdf72c5716cc01e89a7791d4) fix(cli,bundler): prefer AppImage libraries with ABI version ([#4505](https://www.github.com/tauri-apps/tauri/pull/4505)) on 2022-06-29
- Improve error message when `cargo` is not installed.
  - [e0e5f772](https://www.github.com/tauri-apps/tauri/commit/e0e5f772430f6349ec99ba891e601331e376e3c7) feat(cli): improve `cargo not found` error message, closes [#4428](https://www.github.com/tauri-apps/tauri/pull/4428) ([#4430](https://www.github.com/tauri-apps/tauri/pull/4430)) on 2022-06-21
- The app template now only sets the default menu on macOS.
  - [5105b428](https://www.github.com/tauri-apps/tauri/commit/5105b428c4726b2179cd4b3244350d1a1ee73734) feat(cli): change app template to only set default menu on macOS ([#4518](https://www.github.com/tauri-apps/tauri/pull/4518)) on 2022-06-29
- Warn if updater is enabled but not in the bundle target list.
  - [31c15cd2](https://www.github.com/tauri-apps/tauri/commit/31c15cd2bd94dbe39fb94982a15cbe02ac5d8925) docs(config): enhance documentation for bundle targets, closes [#3251](https://www.github.com/tauri-apps/tauri/pull/3251) ([#4418](https://www.github.com/tauri-apps/tauri/pull/4418)) on 2022-06-21
- Check if target exists and is installed on dev and build commands.
  - [13b8a240](https://www.github.com/tauri-apps/tauri/commit/13b8a2403d1353a8c3a643fbc6b6e862af68376e) feat(cli): validate target argument ([#4458](https://www.github.com/tauri-apps/tauri/pull/4458)) on 2022-06-24
- Fixes the covector configuration on the plugin templates.
  - [b8a64d01](https://www.github.com/tauri-apps/tauri/commit/b8a64d01bab11f955b7bbdf323d0afa1a3db4b64) fix(cli): add prepublish scripts to the plugin templates on 2022-06-19
- Set the binary name to the product name in development.
  - [b025b9f5](https://www.github.com/tauri-apps/tauri/commit/b025b9f581ac1a6ae0a26789c2be1e9928fb0282) refactor(cli): set binary name on dev ([#4447](https://www.github.com/tauri-apps/tauri/pull/4447)) on 2022-06-23
- Allow registering a `.gitignore` file to skip watching some project files and directories via the `TAURI_DEV_WATCHER_IGNORE_FILE` environment variable.
  - [83186dd8](https://www.github.com/tauri-apps/tauri/commit/83186dd89768407984db35fb67c3cc51f50ea8f5) Read extra ignore file for dev watcher, closes [#4406](https://www.github.com/tauri-apps/tauri/pull/4406) ([#4409](https://www.github.com/tauri-apps/tauri/pull/4409)) on 2022-06-20
- Fix shebang for `kill-children.sh`.
  - [35dd51db](https://www.github.com/tauri-apps/tauri/commit/35dd51db6826ec1eed7b90082b9eb6b2a699b627) fix(cli): add shebang for kill-children.sh, closes [#4262](https://www.github.com/tauri-apps/tauri/pull/4262) ([#4416](https://www.github.com/tauri-apps/tauri/pull/4416)) on 2022-06-22
- Update plugin templates to use newer `tauri-apps/create-pull-request` GitHub action.
  - [07f90795](https://www.github.com/tauri-apps/tauri/commit/07f9079532a42f3517d96faeaf46cad6176b31ac) chore(cli): update plugin template tauri-apps/create-pull-request on 2022-06-19
- Use UNIX path separator on the init `$schema` field.
  - [01053045](https://www.github.com/tauri-apps/tauri/commit/010530459ef62c48eed68ca965f2688accabcf69) chore(cli): use unix path separator on $schema ([#4384](https://www.github.com/tauri-apps/tauri/pull/4384)) on 2022-06-19
- The `info` command now can check the Cargo lockfile on workspaces.
  - [12f65219](https://www.github.com/tauri-apps/tauri/commit/12f65219ea75a51ebd38659ddce1563e015a036c) fix(cli): read lockfile from workspace on the info command, closes [#4232](https://www.github.com/tauri-apps/tauri/pull/4232) ([#4423](https://www.github.com/tauri-apps/tauri/pull/4423)) on 2022-06-21
- Preserve the `Cargo.toml` formatting when the features array is not changed.
  - [6650e5d6](https://www.github.com/tauri-apps/tauri/commit/6650e5d6720c215530ca1fdccd19bd2948dd6ca3) fix(cli): preserve Cargo manifest formatting when possible ([#4431](https://www.github.com/tauri-apps/tauri/pull/4431)) on 2022-06-21
- Change the updater signature metadata to include the file name instead of its full path.
  - [094b3eb3](https://www.github.com/tauri-apps/tauri/commit/094b3eb352bcf5de28414015e7c44290d619ea8c) fix(cli): file name instead of path on updater sig comment, closes [#4467](https://www.github.com/tauri-apps/tauri/pull/4467) ([#4484](https://www.github.com/tauri-apps/tauri/pull/4484)) on 2022-06-27
- Validate bundle identifier as it must only contain alphanumeric characters, hyphens and periods.
  - [0674a801](https://www.github.com/tauri-apps/tauri/commit/0674a80129d7c31bc93257849afc0a5069129fed) fix: assert config.bundle.identifier to be only alphanumeric, hyphens or dots. closes [#4359](https://www.github.com/tauri-apps/tauri/pull/4359) ([#4363](https://www.github.com/tauri-apps/tauri/pull/4363)) on 2022-06-17

## \[1.0.0]

- Upgrade to `stable`!
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[1.0.0-rc.16]

- Use the default window menu in the app template.
  - [4c4acc30](https://www.github.com/tauri-apps/tauri/commit/4c4acc3094218dd9cee0f1ad61810c979e0b41fa) feat: implement `Default` for `Menu`, closes [#2398](https://www.github.com/tauri-apps/tauri/pull/2398) ([#4291](https://www.github.com/tauri-apps/tauri/pull/4291)) on 2022-06-15

## \[1.0.0-rc.15]

- Removed the tray icon from the Debian and AppImage bundles since they are embedded in the binary now.
  - [4ce8e228](https://www.github.com/tauri-apps/tauri/commit/4ce8e228134cd3f22973b74ef26ca0d165fbbbd9) refactor(core): use `Icon` for tray icons ([#4342](https://www.github.com/tauri-apps/tauri/pull/4342)) on 2022-06-14

## \[1.0.0-rc.14]

- Set the `TRAY_LIBRARY_PATH` environment variable to make the bundle copy the appindicator library to the AppImage.
  - [34552444](https://www.github.com/tauri-apps/tauri/commit/3455244436578003a5fbb447b039e5c8971152ec) feat(cli): bundle appindicator library in the AppImage, closes [#3859](https://www.github.com/tauri-apps/tauri/pull/3859) ([#4267](https://www.github.com/tauri-apps/tauri/pull/4267)) on 2022-06-07
- Set the `APPIMAGE_BUNDLE_GSTREAMER` environment variable to make the bundler copy additional gstreamer files to the AppImage.
  - [d335fae9](https://www.github.com/tauri-apps/tauri/commit/d335fae92cdcbb0ee18aad4e54558914afa3e778) feat(bundler): bundle additional gstreamer files, closes [#4092](https://www.github.com/tauri-apps/tauri/pull/4092) ([#4271](https://www.github.com/tauri-apps/tauri/pull/4271)) on 2022-06-10
- Configure the AppImage bundler to copy the `/usr/bin/xdg-open` binary if it exists and the shell `open` API is enabled.
  - [2322ac11](https://www.github.com/tauri-apps/tauri/commit/2322ac11cf6290c6bf65413048a049c8072f863b) fix(bundler): bundle `/usr/bin/xdg-open` in appimage if open API enabled ([#4265](https://www.github.com/tauri-apps/tauri/pull/4265)) on 2022-06-04
- Fixes multiple occurrences handling of the `bundles` and `features` arguments.
  - [f685df39](https://www.github.com/tauri-apps/tauri/commit/f685df399a5a05480b6e4f5d92da71f3b87895ef) fix(cli): parsing of arguments with multiple values, closes [#4231](https://www.github.com/tauri-apps/tauri/pull/4231) ([#4233](https://www.github.com/tauri-apps/tauri/pull/4233)) on 2022-05-29
- Log command output in real time instead of waiting for it to finish.
  - [76d1eaae](https://www.github.com/tauri-apps/tauri/commit/76d1eaaebda5c8f6b0d41bf6587945e98cd441f3) feat(cli): debug command output in real time ([#4318](https://www.github.com/tauri-apps/tauri/pull/4318)) on 2022-06-12
- Configure the `STATIC_VCRUNTIME` environment variable so `tauri-build` statically links it on the build command.
  - [d703d27a](https://www.github.com/tauri-apps/tauri/commit/d703d27a707edc028f13b35603205da1133fcc2b) fix(build): statically link VC runtime only on `tauri build` ([#4292](https://www.github.com/tauri-apps/tauri/pull/4292)) on 2022-06-07
- Use the `TAURI_TRAY` environment variable to determine which package should be added to the Debian `depends` section. Possible values are `ayatana` and `gtk`.
  - [6216eb49](https://www.github.com/tauri-apps/tauri/commit/6216eb49e72863bfb6d4c9edb8827b21406ac393) refactor(core): drop `ayatana-tray` and `gtk-tray` Cargo features ([#4247](https://www.github.com/tauri-apps/tauri/pull/4247)) on 2022-06-02

## \[1.0.0-rc.13]

- Check if `$CWD/src-tauri/tauri.conf.json` exists before walking through the file tree to find the tauri dir in case the whole project is gitignored.
  - [bd8f3e29](https://www.github.com/tauri-apps/tauri/commit/bd8f3e298a0cb71809f2e93cc3ebc8e6e5b6a626) fix(cli): manual config lookup to handle gitignored folders, fixes [#3527](https://www.github.com/tauri-apps/tauri/pull/3527) ([#4224](https://www.github.com/tauri-apps/tauri/pull/4224)) on 2022-05-26
- Statically link the Visual C++ runtime instead of using a merge module on the installer.
  - [bb061509](https://www.github.com/tauri-apps/tauri/commit/bb061509fb674bef86ecbc1de3aa8f3e367a9907) refactor(core): statically link vcruntime, closes [#4122](https://www.github.com/tauri-apps/tauri/pull/4122) ([#4227](https://www.github.com/tauri-apps/tauri/pull/4227)) on 2022-05-27

## \[1.0.0-rc.12]

- Properly fetch the NPM dependency information when using Yarn 2+.
  - [cdfa6255](https://www.github.com/tauri-apps/tauri/commit/cdfa62551115586725bd3e4c04f12c5256f20790) fix(cli): properly read info when using yarn 2+, closes [#4106](https://www.github.com/tauri-apps/tauri/pull/4106) ([#4193](https://www.github.com/tauri-apps/tauri/pull/4193)) on 2022-05-23

## \[1.0.0-rc.11]

- Allow configuring the display options for the MSI execution allowing quieter updates.
  - [9f2c3413](https://www.github.com/tauri-apps/tauri/commit/9f2c34131952ea83c3f8e383bc3cec7e1450429f) feat(core): configure msiexec display options, closes [#3951](https://www.github.com/tauri-apps/tauri/pull/3951) ([#4061](https://www.github.com/tauri-apps/tauri/pull/4061)) on 2022-05-15

## \[1.0.0-rc.10]

- Resolve binary file extension from target triple instead of compile-time checks to allow cross compilation.
  - [4562e671](https://www.github.com/tauri-apps/tauri/commit/4562e671e4795e9386429348bf738f7078706945) fix(build): append .exe binary based on target triple instead of running OS, closes [#3870](https://www.github.com/tauri-apps/tauri/pull/3870) ([#4032](https://www.github.com/tauri-apps/tauri/pull/4032)) on 2022-05-03
- Fixes text overflow on `tauri dev` on Windows.
  - [094534d1](https://www.github.com/tauri-apps/tauri/commit/094534d138a9286e4746b61adff2da616e3b6a61) fix(cli): dev command stderr text overflow on Windows, closes [#3995](https://www.github.com/tauri-apps/tauri/pull/3995) ([#4000](https://www.github.com/tauri-apps/tauri/pull/4000)) on 2022-04-29
- Improve CLI's logging output, making use of the standard rust `log` system.
  - [35f21471](https://www.github.com/tauri-apps/tauri/commit/35f2147161e6697cbd2824681eeaf870b5a991c2) feat(cli): Improve CLI logging ([#4060](https://www.github.com/tauri-apps/tauri/pull/4060)) on 2022-05-07
- Don't override the default keychain on macOS while code signing.
  - [a4fcaf1d](https://www.github.com/tauri-apps/tauri/commit/a4fcaf1d04aafc3b4d42186f0fb386797d959a9d) fix: don't override default keychain, closes [#4008](https://www.github.com/tauri-apps/tauri/pull/4008) ([#4053](https://www.github.com/tauri-apps/tauri/pull/4053)) on 2022-05-05
- - Remove startup delay in `tauri dev` caused by checking for a newer cli version. The check is now done upon process exit.
- Add `TAURI_SKIP_UPDATE_CHECK` env variable to skip checking for a newer CLI version.
- [bbabc8cd](https://www.github.com/tauri-apps/tauri/commit/bbabc8cd1ea2c1f6806610fd2d533c99305d320c) fix(cli.rs): remove startup delay in `tauri dev` ([#3999](https://www.github.com/tauri-apps/tauri/pull/3999)) on 2022-04-29
- Fix `tauri info` panic when a package isn't installed.
  - [4f0f3187](https://www.github.com/tauri-apps/tauri/commit/4f0f3187c9e69262ef28350331b368c831ab930a) fix(cli.rs): fix `tauri info` panic when a package isn't installed, closes [#3985](https://www.github.com/tauri-apps/tauri/pull/3985) ([#3996](https://www.github.com/tauri-apps/tauri/pull/3996)) on 2022-04-29
- Added `$schema` support to `tauri.conf.json`.
  - [715cbde3](https://www.github.com/tauri-apps/tauri/commit/715cbde3842a916c4ebeab2cab348e1774b5c192) feat(config): add `$schema` to `tauri.conf.json`, closes [#3464](https://www.github.com/tauri-apps/tauri/pull/3464) ([#4031](https://www.github.com/tauri-apps/tauri/pull/4031)) on 2022-05-03
- **Breaking change:** The `dev` command now reads the custom config file from CWD instead of the Tauri folder.
  - [a1929c6d](https://www.github.com/tauri-apps/tauri/commit/a1929c6dacccd00af4cdbcc4d29cfb98d8428f55) fix(cli): always read custom config file from CWD, closes [#4067](https://www.github.com/tauri-apps/tauri/pull/4067) ([#4074](https://www.github.com/tauri-apps/tauri/pull/4074)) on 2022-05-07
- Fixes a Powershell crash when sending SIGINT to the dev command.
  - [32048486](https://www.github.com/tauri-apps/tauri/commit/320484866b83ecabb01eb58d158e0fedd9dd08be) fix(cli): powershell crashing on SIGINT, closes [#3997](https://www.github.com/tauri-apps/tauri/pull/3997) ([#4007](https://www.github.com/tauri-apps/tauri/pull/4007)) on 2022-04-29
- Prevent building when the bundle identifier is the default `com.tauri.dev`.
  - [95726ebb](https://www.github.com/tauri-apps/tauri/commit/95726ebb6180d371be44bff9f16ca1eee049006a) feat(cli): prevent default bundle identifier from building, closes [#4041](https://www.github.com/tauri-apps/tauri/pull/4041) ([#4042](https://www.github.com/tauri-apps/tauri/pull/4042)) on 2022-05-04

## \[1.0.0-rc.9]

- Exit CLI when Cargo returns a non-compilation error in `tauri dev`.
  - [b5622882](https://www.github.com/tauri-apps/tauri/commit/b5622882cf3748e1e5a90915f415c0cd922aaaf8) fix(cli): exit on non-compilation Cargo errors, closes [#3930](https://www.github.com/tauri-apps/tauri/pull/3930) ([#3942](https://www.github.com/tauri-apps/tauri/pull/3942)) on 2022-04-22
- Notify CLI update when running `tauri dev`.
  - [a649aad7](https://www.github.com/tauri-apps/tauri/commit/a649aad7ad26d4578699370d6e63d80edeca1f97) feat(cli): check and notify about updates on `tauri dev`, closes [#3789](https://www.github.com/tauri-apps/tauri/pull/3789) ([#3960](https://www.github.com/tauri-apps/tauri/pull/3960)) on 2022-04-25
- The CLI will not automatically run `strip` on release binaries anymore. Use the \[`strip`]\[strip] profile setting stabilized with Cargo 1.59.

[`strip`]: https://doc.rust-lang.org/cargo/reference/profiles.html#strip

- [62106224](https://www.github.com/tauri-apps/tauri/commit/621062246d065f21800d340126cf58315177f97e) refactor: drop strip from build command. closes [#3559](https://www.github.com/tauri-apps/tauri/pull/3559) ([#3863](https://www.github.com/tauri-apps/tauri/pull/3863)) on 2022-04-06
- Kill the `beforeDevCommand` and app processes if the dev command returns an error.
  - [485c9743](https://www.github.com/tauri-apps/tauri/commit/485c97438ac956d86bcf3794ceaa626bef968a4e) fix(cli): kill beforeDevCommand if dev code returns an error ([#3907](https://www.github.com/tauri-apps/tauri/pull/3907)) on 2022-04-19
- Fix `info` command showing outdated text for latest versions.
  - [73a4b74a](https://www.github.com/tauri-apps/tauri/commit/73a4b74aea8544e6fda51c1f6697630b0768072c) fix(cli.rs/info):  don't show outdated text for latest versions ([#3829](https://www.github.com/tauri-apps/tauri/pull/3829)) on 2022-04-02
- **Breaking change:** Enable default Cargo features except `tauri/custom-protocol` on the dev command.
  - [f2a30d8b](https://www.github.com/tauri-apps/tauri/commit/f2a30d8bc54fc3ba49e16f69a413eca5f61a9b1f) refactor(core): use ayatana appindicator by default, keep option to use gtk ([#3916](https://www.github.com/tauri-apps/tauri/pull/3916)) on 2022-04-19
- Kill the `beforeDevCommand` process recursively on Unix.
  - [e251e1b0](https://www.github.com/tauri-apps/tauri/commit/e251e1b0991d26ab10aea33cfb228f3e7f0f85b5) fix(cli): kill before dev command recursively on Unix, closes [#2794](https://www.github.com/tauri-apps/tauri/pull/2794) ([#3848](https://www.github.com/tauri-apps/tauri/pull/3848)) on 2022-04-03

## \[1.0.0-rc.8]

- Allows the `tauri.conf.json` file to be git ignored on the path lookup function.
  - [cc7c2d77](https://www.github.com/tauri-apps/tauri/commit/cc7c2d77da2e4a39ec2a97b080d41a719e6d0161) feat(cli): allow conf path to be gitignored, closes [#3636](https://www.github.com/tauri-apps/tauri/pull/3636) ([#3683](https://www.github.com/tauri-apps/tauri/pull/3683)) on 2022-03-13
- Remove `minimumSystemVersion: null` from the application template configuration.
  - [c81534eb](https://www.github.com/tauri-apps/tauri/commit/c81534ebd873c358e0346c7949aeb171803149a5) feat(cli): use default macOS minimum system version when it is empty ([#3658](https://www.github.com/tauri-apps/tauri/pull/3658)) on 2022-03-13
- Improve readability of the `info` subcommand output.
  - [49d2f13f](https://www.github.com/tauri-apps/tauri/commit/49d2f13fc07d763d5de9bf4b19d00c901776c11d) feat(cli): colorful cli ([#3635](https://www.github.com/tauri-apps/tauri/pull/3635)) on 2022-03-08
- Properly terminate the `beforeDevCommand` process.
  - [94d78efb](https://www.github.com/tauri-apps/tauri/commit/94d78efbe542e7be3c00e3b2355c22803816715f) fix(cli.rs): terminate the beforeDevCommand, closes [#2794](https://www.github.com/tauri-apps/tauri/pull/2794) ([#2883](https://www.github.com/tauri-apps/tauri/pull/2883)) on 2022-03-27
- Fixes DMG bundling on macOS 12.3.
  - [348a1ab5](https://www.github.com/tauri-apps/tauri/commit/348a1ab59d2697478a594016016f1fccbf1ac054) fix(bundler): DMG bundling on macOS 12.3 cannot use bless, closes [#3719](https://www.github.com/tauri-apps/tauri/pull/3719) ([#3721](https://www.github.com/tauri-apps/tauri/pull/3721)) on 2022-03-18
- Fixes resources bundling on Windows when the path is on the root of the Tauri folder.
  - [4c84559e](https://www.github.com/tauri-apps/tauri/commit/4c84559e1f3019e7aa2666b10a1a0bd97bb09d24) fix(cli): root resource bundling on Windows, closes [#3539](https://www.github.com/tauri-apps/tauri/pull/3539) ([#3685](https://www.github.com/tauri-apps/tauri/pull/3685)) on 2022-03-13

## \[1.0.0-rc.7]

- Added `tsp` config option under `tauri > bundle > windows`, which enables Time-Stamp Protocol (RFC 3161) for the timestamping
  server under code signing on Windows if set to `true`.
  - [bdd5f7c2](https://www.github.com/tauri-apps/tauri/commit/bdd5f7c2f03af4af8b60a9527e55bb18525d989b) fix: add support for Time-Stamping Protocol for Windows codesigning (fix [#3563](https://www.github.com/tauri-apps/tauri/pull/3563)) ([#3570](https://www.github.com/tauri-apps/tauri/pull/3570)) on 2022-03-07
- Change the `plugin init` templates to use the new `tauri::plugin::Builder` syntax.
  - [f7acb061](https://www.github.com/tauri-apps/tauri/commit/f7acb061e4d1ecdbfe182793587632d7ba6d8eff) feat(cli): use plugin::Builder syntax on the plugin template ([#3606](https://www.github.com/tauri-apps/tauri/pull/3606)) on 2022-03-03

## \[1.0.0-rc.6]

- Improve "waiting for your dev server to start" message.
  - [5999379f](https://www.github.com/tauri-apps/tauri/commit/5999379fb06052a115f04f99274ab46d1eefd659) chore(cli): improve "waiting for dev server" message, closes [#3491](https://www.github.com/tauri-apps/tauri/pull/3491) ([#3504](https://www.github.com/tauri-apps/tauri/pull/3504)) on 2022-02-18
- Do not panic if the updater private key password is wrong.
  - [17f17a80](https://www.github.com/tauri-apps/tauri/commit/17f17a80f818bcc20c387583a6ff00a8e07ec533) fix(cli): do not panic if private key password is wrong, closes [#3449](https://www.github.com/tauri-apps/tauri/pull/3449) ([#3495](https://www.github.com/tauri-apps/tauri/pull/3495)) on 2022-02-17
- Check the current folder before checking the directories on the app and tauri dir path lookup function.
  - [a06de376](https://www.github.com/tauri-apps/tauri/commit/a06de3760184caa71acfe7a2fe2189a033b565f5) fix(cli): path lookup should not check subfolder before the current one ([#3465](https://www.github.com/tauri-apps/tauri/pull/3465)) on 2022-02-15
- Fixes the signature of the `signer sign` command to not have duplicated short flags.
  - [a9755514](https://www.github.com/tauri-apps/tauri/commit/a975551461f3698db3f3b6afa5101189aaeeada9) fix(cli): duplicated short flag for `signer sign`, closes [#3483](https://www.github.com/tauri-apps/tauri/pull/3483) ([#3492](https://www.github.com/tauri-apps/tauri/pull/3492)) on 2022-02-17

## \[1.0.0-rc.5]

- Allow passing arguments to the `build` runner (`tauri build -- <ARGS>...`).
  - [679fe1fe](https://www.github.com/tauri-apps/tauri/commit/679fe1fedd6ed016ab1140c8087c2d1404504bfb) feat(cli.rs): allow passing arguments to the build runner, closes [#3398](https://www.github.com/tauri-apps/tauri/pull/3398) ([#3431](https://www.github.com/tauri-apps/tauri/pull/3431)) on 2022-02-13
- Improve error message when the dev runner command fails.
  - [759d1afb](https://www.github.com/tauri-apps/tauri/commit/759d1afb86f3657f6071a2ae39c9be21e20ed22c) feat(cli): improve error message when dev runner command fails ([#3447](https://www.github.com/tauri-apps/tauri/pull/3447)) on 2022-02-13
- Increase `tauri.conf.json` directory lookup depth to `3` and allow changing it with the `TAURI_PATH_DEPTH` environment variable.
  - [c6031c70](https://www.github.com/tauri-apps/tauri/commit/c6031c7070c6bb7539bbfdfe42cb73012829c910) feat(cli): increase lookup depth, add env var option ([#3451](https://www.github.com/tauri-apps/tauri/pull/3451)) on 2022-02-13
- Added `tauri-build`, `tao` and `wry` version to the `info` command output.
  - [16f1173f](https://www.github.com/tauri-apps/tauri/commit/16f1173f456b1db543d0160df2c9828708bfc68a) feat(cli): add tao and wry version to the `info` output ([#3443](https://www.github.com/tauri-apps/tauri/pull/3443)) on 2022-02-13
- **Breaking change:** The extra arguments passed to `tauri dev` using `-- <ARGS>...` are now propagated to the runner (defaults to cargo). To pass arguments to your binary using Cargo, you now need to run `tauri dev -- -- <ARGS-TO-YOUR-BINARY>...` (notice the double `--`).
  - [679fe1fe](https://www.github.com/tauri-apps/tauri/commit/679fe1fedd6ed016ab1140c8087c2d1404504bfb) feat(cli.rs): allow passing arguments to the build runner, closes [#3398](https://www.github.com/tauri-apps/tauri/pull/3398) ([#3431](https://www.github.com/tauri-apps/tauri/pull/3431)) on 2022-02-13
- Change the `init` template configuration to disable CSP for better usability for new users.
  - [102a5e9b](https://www.github.com/tauri-apps/tauri/commit/102a5e9bb83c5d8388dc9aedc7f03cc57bdae8cb) refactor(cli.rs): change template config CSP to null, closes [#3427](https://www.github.com/tauri-apps/tauri/pull/3427) ([#3429](https://www.github.com/tauri-apps/tauri/pull/3429)) on 2022-02-13

## \[1.0.0-rc.4]

- Change default value for the `freezePrototype` configuration to `false`.
  - Bumped due to a bump in tauri-utils.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

## \[1.0.0-rc.3]

- Fixes Tauri path resolution on projects without Git or a `.gitignore` file.
  - [d8acbe11](https://www.github.com/tauri-apps/tauri/commit/d8acbe11492bd990e6983c7e63e0f1a8f1ea5c7c) fix(cli.rs): app path resolution on projects without git, closes [#3409](https://www.github.com/tauri-apps/tauri/pull/3409) ([#3410](https://www.github.com/tauri-apps/tauri/pull/3410)) on 2022-02-11

## \[1.0.0-rc.2]

- Fix `init` command prompting for values even if the argument has been provided on the command line.
  - [def76840](https://www.github.com/tauri-apps/tauri/commit/def76840257a1447723ecda13c807cf0c881f083) fix(cli.rs): do not prompt for `init` values if arg set ([#3400](https://www.github.com/tauri-apps/tauri/pull/3400)) on 2022-02-11
  - [41052dee](https://www.github.com/tauri-apps/tauri/commit/41052deeda2a00ee2b8ec2041c9c87c11de82ab2) fix(covector): add cli.js to change files on 2022-02-11
- Fixes CLI freezing when running `light.exe` on Windows without the `--verbose` flag.
  - [8beab636](https://www.github.com/tauri-apps/tauri/commit/8beab6363491e2a8757cc9fc0fa1eccc98ece916) fix(cli): build freezing on Windows, closes [#3399](https://www.github.com/tauri-apps/tauri/pull/3399) ([#3402](https://www.github.com/tauri-apps/tauri/pull/3402)) on 2022-02-11
- Respect `.gitignore` configuration when looking for the folder with the `tauri.conf.json` file.
  - [9c6c5a8c](https://www.github.com/tauri-apps/tauri/commit/9c6c5a8c52c6460d0b0a1a55300e1828262994ba) perf(cli.rs): improve performance of tauri dir lookup reading .gitignore ([#3405](https://www.github.com/tauri-apps/tauri/pull/3405)) on 2022-02-11
  - [41052dee](https://www.github.com/tauri-apps/tauri/commit/41052deeda2a00ee2b8ec2041c9c87c11de82ab2) fix(covector): add cli.js to change files on 2022-02-11

## \[1.0.0-rc.1]

- Include `vswhere.exe` on the published package.
  - [3227502e](https://www.github.com/tauri-apps/tauri/commit/3227502e8c9f137e5783cba2e0c692473cc5456d) fix(cli.rs): package `vswhere.exe` on 2022-02-10

## \[1.0.0-rc.0]

- Do not force Tauri application code on `src-tauri` folder and use a glob pattern to look for a subfolder with a `tauri.conf.json` file.
  - [a8cff6b3](https://www.github.com/tauri-apps/tauri/commit/a8cff6b3bc3288a53d7cdc5b3cb95d371309d2d6) feat(cli): do not enforce `src-tauri` folder structure, closes [#2643](https://www.github.com/tauri-apps/tauri/pull/2643) ([#2654](https://www.github.com/tauri-apps/tauri/pull/2654)) on 2021-09-27
- Define `TAURI_PLATFORM`, `TAURI_ARCH`, `TAURI_FAMILY`, `TAURI_PLATFORM_TYPE`, `TAURI_PLATFORM_VERSION` and `TAURI_DEBUG` environment variables for the `beforeDevCommand` and `beforeBuildCommand` scripts.
  - [8599313a](https://www.github.com/tauri-apps/tauri/commit/8599313a0f56d9777d335426467e79ba687be1d4) feat(cli.rs): env vars for beforeDev/beforeBuild commands, closes [#2610](https://www.github.com/tauri-apps/tauri/pull/2610) ([#2655](https://www.github.com/tauri-apps/tauri/pull/2655)) on 2021-09-26
  - [b5ee03a1](https://www.github.com/tauri-apps/tauri/commit/b5ee03a13a1c0d0ff677cf9c8d7ef28516fffa5b) feat(cli.rs): expose debug flag to beforeDev/beforeBuild commands ([#2727](https://www.github.com/tauri-apps/tauri/pull/2727)) on 2021-10-08
  - [9bb68973](https://www.github.com/tauri-apps/tauri/commit/9bb68973dd10f3cb98d2a95e5432bfc765d77064) fix(cli.rs): prefix the "before script" env vars with `TAURI_` ([#3274](https://www.github.com/tauri-apps/tauri/pull/3274)) on 2022-01-24
- Allow `config` arg to be a path to a JSON file on the `dev` and `build` commands.
  - [7b81e5b8](https://www.github.com/tauri-apps/tauri/commit/7b81e5b82e665fe0562b91ac33b63a871af9e111) feat(cli.rs): allow config argument to be a path to a JSON file ([#2938](https://www.github.com/tauri-apps/tauri/pull/2938)) on 2021-11-22
- Add `rustup` version and active rust toolchain to the `info` command output.
  - [28aaec87](https://www.github.com/tauri-apps/tauri/commit/28aaec87e2f6445859e9dbaaf2231d02d1e1d4b5) feat(cli.rs): add active toolchain and rustup to `tauri info`, closes [#2730](https://www.github.com/tauri-apps/tauri/pull/2730) ([#2986](https://www.github.com/tauri-apps/tauri/pull/2986)) on 2021-12-09
- Add `Visual Studio Build Tools` installed versions to the `info` command output.
  - [d5f07d14](https://www.github.com/tauri-apps/tauri/commit/d5f07d14f31954fe89ff5045fd4ef72cfc2b9ac1) feat(cli.rs): build tools info ([#2618](https://www.github.com/tauri-apps/tauri/pull/2618)) on 2021-09-21
- The inferred development server port for Svelte is now `8080` (assumes latest Svelte with `sirv-cli >= 2.0.0`).
  - [de0543f3](https://www.github.com/tauri-apps/tauri/commit/de0543f3e052d2981a95ce3baa8470592740a1a2) feat(cli.rs): change inferred dev server port to 8080 for Svelte apps on 2022-02-05
- Detect if tauri is used from git in the `info` command.
  - [65ad5b5e](https://www.github.com/tauri-apps/tauri/commit/65ad5b5ef923bf1e6b6f078d794d071a04fcdf57) feat(cli.rs/info): detect if tauri is used from git ([#3309](https://www.github.com/tauri-apps/tauri/pull/3309)) on 2022-02-05
- Drop the `dialoguer` soft fork and use the published version instead.
  - [b1f5c6d7](https://www.github.com/tauri-apps/tauri/commit/b1f5c6d7ac48c7407f28402afef0d3e521314127) refactor(cli.rs): drop `dialoguer` and `console` soft fork ([#2790](https://www.github.com/tauri-apps/tauri/pull/2790)) on 2021-10-22
- Fix `build` command when executed on a 32-bit Windows machine when pulling from the `binary-releases` repo.
  - [35588b2e](https://www.github.com/tauri-apps/tauri/commit/35588b2e04d5be8e5708583bdc52a012341bc75e) fix(cli.rs): check default arch at runtime, closes [#3067](https://www.github.com/tauri-apps/tauri/pull/3067) ([#3078](https://www.github.com/tauri-apps/tauri/pull/3078)) on 2021-12-27
- The `generate` and `sign` commands are now available under a `signer` subcommand.
  - [1458ab3c](https://www.github.com/tauri-apps/tauri/commit/1458ab3c535637ada996ab0ff3494cd75fe40bf7) refactor(cli.rs): `signer` and `plugin` subcommands, use new clap derive syntax ([#2928](https://www.github.com/tauri-apps/tauri/pull/2928)) on 2021-12-09
- Use `tauri-utils` to get the `Config` types.
  - [4de285c3](https://www.github.com/tauri-apps/tauri/commit/4de285c3967d32250d73acdd5d171a6fd332d2b3) feat(core): validate Cargo features matching allowlist \[TRI-023] on 2022-01-09
- Print warning and exit if `distDir` contains `node_modules`, `src-tauri` or `target` folders.
  - [7ed3f3b7](https://www.github.com/tauri-apps/tauri/commit/7ed3f3b7e4268708bbe8f83c45653e5d6704824b) feat(cli.rs): validate `distDir`, closes [#2554](https://www.github.com/tauri-apps/tauri/pull/2554) ([#2701](https://www.github.com/tauri-apps/tauri/pull/2701)) on 2021-10-04
- Fix `tauri build` failing on Windows if `tauri.conf.json > tauri > bundle > Windows > wix > license` is used.
  - [17a1ad68](https://www.github.com/tauri-apps/tauri/commit/17a1ad682363e51365b57899c8d7557b1b65201c) fix(cli.rs): ensure `target/release/wix` exists, closes [#2927](https://www.github.com/tauri-apps/tauri/pull/2927) ([#2987](https://www.github.com/tauri-apps/tauri/pull/2987)) on 2021-12-07
- Added `dev_csp` to the `security` configuration object.
  - [cf54dcf9](https://www.github.com/tauri-apps/tauri/commit/cf54dcf9c81730e42c9171daa9c8aa474c95b522) feat: improve `CSP` security with nonces and hashes, add `devCsp` \[TRI-004] ([#8](https://www.github.com/tauri-apps/tauri/pull/8)) on 2022-01-09
- Kill process if `beforeDevCommand` exits with a non-zero status code.
  - [a2d5929a](https://www.github.com/tauri-apps/tauri/commit/a2d5929a8f1f6310d186199cc54246fcb0a01b46) feat(cli.rs): wait for dev URL to be reachable, exit if command fails ([#3358](https://www.github.com/tauri-apps/tauri/pull/3358)) on 2022-02-08
- Fixes output directory detection when using Cargo workspaces.
  - [8d630bc8](https://www.github.com/tauri-apps/tauri/commit/8d630bc8c494cba6ac1604b7777b89b763044471) fix(cli.rs): fix workspace detection, fixes [#2614](https://www.github.com/tauri-apps/tauri/pull/2614), closes [#2515](https://www.github.com/tauri-apps/tauri/pull/2515) ([#2644](https://www.github.com/tauri-apps/tauri/pull/2644)) on 2021-09-23
- Allow using a fixed version for the Webview2 runtime via the `tauri > bundle > windows > webviewFixedRuntimePath` config option.
  - [85df94f2](https://www.github.com/tauri-apps/tauri/commit/85df94f2b0d40255812b42c5e32a70c4b45392df) feat(core): config for fixed webview2 runtime version path ([#27](https://www.github.com/tauri-apps/tauri/pull/27)) on 2021-11-02
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
- Added `$ tauri plugin init` command, which initializes a Tauri plugin.
  - [ac8e69a9](https://www.github.com/tauri-apps/tauri/commit/ac8e69a98ca1d6f646344ffdef1876cc9274323a) feat(cli.rs): add `init plugin` command, bootstraps a plugin project ([#2669](https://www.github.com/tauri-apps/tauri/pull/2669)) on 2021-09-27
  - [db275f0b](https://www.github.com/tauri-apps/tauri/commit/db275f0b633f44fb2f85755d32929dfb7893b1e0) refactor(cli.rs): rename `init plugin` subcommand to `plugin init` ([#2885](https://www.github.com/tauri-apps/tauri/pull/2885)) on 2021-11-13
- **Breaking change:** Add `macos-private-api` feature flag, enabled via `tauri.conf.json > tauri > macOSPrivateApi`.
  - [6ac21b3c](https://www.github.com/tauri-apps/tauri/commit/6ac21b3cef7f14358df38cc69ea3d277011accaf) feat: add private api feature flag ([#7](https://www.github.com/tauri-apps/tauri/pull/7)) on 2022-01-09
- Move the copying of resources and sidecars from `cli.rs` to `tauri-build` so using the Cargo CLI directly processes the files for the application execution in development.
  - [5eb72c24](https://www.github.com/tauri-apps/tauri/commit/5eb72c24deddf5a01093bea96b90c0d8806afc3f) refactor: copy resources and sidecars on the Cargo build script ([#3357](https://www.github.com/tauri-apps/tauri/pull/3357)) on 2022-02-08
- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22
- Automatically `strip` the built binary on Linux and macOS if `--debug` is not specified.
  - [2f3a582c](https://www.github.com/tauri-apps/tauri/commit/2f3a582c69994d66f2035bbe62825eafc869d90f) feat(cli.rs): strip release binaries \[TRI-031] ([#22](https://www.github.com/tauri-apps/tauri/pull/22)) on 2022-01-09
- Fixes pnpm error when running `pnpm tauri info`.
  - [2026134f](https://www.github.com/tauri-apps/tauri/commit/2026134f471a212aea9b227a873ecc937dda1acc) fix(cli.rs): pnpm tauri info exits with error (fix [#2509](https://www.github.com/tauri-apps/tauri/pull/2509)) ([#2510](https://www.github.com/tauri-apps/tauri/pull/2510)) on 2021-08-24
- Add support to building Universal macOS Binaries through the virtual target `universal-apple-darwin` (run `tauri build --target universal-apple-darwin`).
  - [83f52fdb](https://www.github.com/tauri-apps/tauri/commit/83f52fdbe3a9ffd98dffc752e5f9e14322b56e60) feat: Add `universal-darwin-macos` build target, closes [#3317](https://www.github.com/tauri-apps/tauri/pull/3317) ([#3318](https://www.github.com/tauri-apps/tauri/pull/3318)) on 2022-02-04
- Wait for `devPath` URL to be reachable before starting the application. Skipped if the `TAURI_SKIP_DEVSERVER_CHECK` environment variable is set to `true`.
  - [a2d5929a](https://www.github.com/tauri-apps/tauri/commit/a2d5929a8f1f6310d186199cc54246fcb0a01b46) feat(cli.rs): wait for dev URL to be reachable, exit if command fails ([#3358](https://www.github.com/tauri-apps/tauri/pull/3358)) on 2022-02-08
- On Windows, Fix `beforeDevCommand` and `beforeBuildCommand` not executing the expected command if it contains quotes. This is done by executing them with `CMD /S /C {command}` instead of `CMD /C {command}` on Windows.
  - [52e9a6d8](https://www.github.com/tauri-apps/tauri/commit/52e9a6d81a701a66a8cf6a67c2b37d135583543f) fix: Make CMD handle quotes `"` properly. ([#3334](https://www.github.com/tauri-apps/tauri/pull/3334)) on 2022-02-06
- Allow setting the localization file for WiX.
  - [af329f27](https://www.github.com/tauri-apps/tauri/commit/af329f2722d6194c6d70e976fc970dc2c9e4de2b) feat(bundler): wix localization, closes [#3174](https://www.github.com/tauri-apps/tauri/pull/3174) ([#3179](https://www.github.com/tauri-apps/tauri/pull/3179)) on 2022-02-05

## \[1.0.0-beta.7]

- Update cli.yml to pass clap ArgSettings::MultipleValues assertion.
  - [0391ac3d](https://www.github.com/tauri-apps/tauri/commit/0391ac3dc96d9c74c34a957e4cb70da88a0a85b7) fix: Update cli.yml to pass clap ArgSettings::MultipleValues assertion. ([#2506](https://www.github.com/tauri-apps/tauri/pull/2506)) ([#2507](https://www.github.com/tauri-apps/tauri/pull/2507)) on 2021-08-22

## \[1.0.0-beta.6]

- Added `APPLE_SIGNING_IDENTITY` as supported environment variable for the bundler.
  - [44f6ee4c](https://www.github.com/tauri-apps/tauri/commit/44f6ee4cfdfad5fb21d96e69f8776c0e68685682) chore(ci): add step to detect code signing ([#2245](https://www.github.com/tauri-apps/tauri/pull/2245)) on 2021-08-08
- Added configuration for the WiX banner icon under `tauri.conf.json > tauri > bundle > windows > wix > bannerPath`.
  - [13003ec7](https://www.github.com/tauri-apps/tauri/commit/13003ec761b1530705d6129519dc4e226eb992c7) feat(bundler): add config for WiX banner path, closes [#2175](https://www.github.com/tauri-apps/tauri/pull/2175) ([#2448](https://www.github.com/tauri-apps/tauri/pull/2448)) on 2021-08-16
- Added configuration for the WiX dialog background bitmap under `tauri.conf.json > tauri > bundle > windows > wix > dialogImagePath`.
  - [9bfdeb42](https://www.github.com/tauri-apps/tauri/commit/9bfdeb42effeeec27aa15bbc5b05040eadfda5ba) feat(bundler): add config for WiX dialog image path ([#2449](https://www.github.com/tauri-apps/tauri/pull/2449)) on 2021-08-16
- Only convert package name and binary name to kebab-case, keeping the `.desktop` `Name` field with the original configured value.
  - [3f039cb8](https://www.github.com/tauri-apps/tauri/commit/3f039cb8a308b0f18deaa37d7cfb1cc50d308d0e) fix: keep original `productName` for .desktop `Name` field, closes [#2295](https://www.github.com/tauri-apps/tauri/pull/2295) ([#2384](https://www.github.com/tauri-apps/tauri/pull/2384)) on 2021-08-10
- Merge platform-specific `tauri.linux.conf.json`, `tauri.windows.conf.json` and `tauri.macos.conf.json` into the config JSON from `tauri.conf.json`.
  - [71d687b7](https://www.github.com/tauri-apps/tauri/commit/71d687b787cd722c60879adda543897826bf42c9) feat(cli.rs): platform-specific conf.json ([#2309](https://www.github.com/tauri-apps/tauri/pull/2309)) on 2021-07-28
- Update minimum Rust version to 1.54.0.
  - [a5394716](https://www.github.com/tauri-apps/tauri/commit/a53947160985a4f5b0ad1fbb4aa6865d6f852c66) chore: update rust to 1.54.0 ([#2434](https://www.github.com/tauri-apps/tauri/pull/2434)) on 2021-08-15

## \[1.0.0-beta.5]

- Run powershell commands with `-NoProfile` flag
  - [3e6f3416](https://www.github.com/tauri-apps/tauri/commit/3e6f34160deab4f774d90aba28122e5b6b6f9db2) fix(cli.rs): run powershell kill command without profile ([#2130](https://www.github.com/tauri-apps/tauri/pull/2130)) on 2021-06-30
- Adds `release` argument to the `dev` command. Allowing to run the backend in release mode during development.
  - [7ee2dc8b](https://www.github.com/tauri-apps/tauri/commit/7ee2dc8b690703f509ab2d6ecdf9dafd6b72cd0b) feat(cli.rs): add release argument to the dev command ([#2192](https://www.github.com/tauri-apps/tauri/pull/2192)) on 2021-07-12
- Fixes `center` and `focus` not being allowed in `tauri.conf.json > tauri > windows` and ignored in `WindowBuilderWrapper`.
  - [bc2c331d](https://www.github.com/tauri-apps/tauri/commit/bc2c331dec3dec44c79e659b082b5fb6b65cc5ea) fix: center and focus not being allowed in config ([#2199](https://www.github.com/tauri-apps/tauri/pull/2199)) on 2021-07-12

## \[1.0.0-beta.4]

- Improve error message when the product name is invalid.
  - [1a41e9f0](https://www.github.com/tauri-apps/tauri/commit/1a41e9f040cfa18b6cc1380dfe21251d56e3f973) feat(cli.rs): improve error message on app rename, closes [#2101](https://www.github.com/tauri-apps/tauri/pull/2101) ([#2114](https://www.github.com/tauri-apps/tauri/pull/2114)) on 2021-06-28

## \[1.0.0-beta.3]

- Properly detect target platform's architecture.
  - [628a53eb](https://www.github.com/tauri-apps/tauri/commit/628a53eb6176f811d22d7730f08a99e5c370dbf4) fix(cli): properly detect target architecture, closes [#2040](https://www.github.com/tauri-apps/tauri/pull/2040) ([#2102](https://www.github.com/tauri-apps/tauri/pull/2102)) on 2021-06-28
- Fixes `build` command when the `target` arg is set.
  - [8e238701](https://www.github.com/tauri-apps/tauri/commit/8e2387018940e9e1421948d74a82156661ce2e4b) fix(cli.rs): fix out dir detection when target arg is set, closes [#2040](https://www.github.com/tauri-apps/tauri/pull/2040) ([#2098](https://www.github.com/tauri-apps/tauri/pull/2098)) on 2021-06-27

## \[1.0.0-beta.2]

- Support `cargo tauri build` on Apple M1 chip.
  - [3bf853d7](https://www.github.com/tauri-apps/tauri/commit/3bf853d782b491ad4965a1da25d19337eeac161f) feat(cli.rs): support tauri build on M1 chip ([#1915](https://www.github.com/tauri-apps/tauri/pull/1915)) on 2021-05-29
- Infer `app name` and `window title` from `package.json > productName` or `package.json > name`.
  Infer `distDir` and `devPath` by reading the package.json and trying to determine the UI framework (Vue.js, Angular, React, Svelte and some UI frameworks).
  - [21a971c3](https://www.github.com/tauri-apps/tauri/commit/21a971c3b76bf0c26d00b2520b4976fa526738f5) feat(cli.rs): infer devPath/distDir/appName from package.json ([#1930](https://www.github.com/tauri-apps/tauri/pull/1930)) on 2021-05-31
- Watch workspace crates on `dev` command.
  - [86a23ff3](https://www.github.com/tauri-apps/tauri/commit/86a23ff30b4f18effa39c87b7cae6b7e324d131c) added support for cargo workspaces for `dev` command ([#1827](https://www.github.com/tauri-apps/tauri/pull/1827)) on 2021-05-13
- Adds `features` argument to the `dev` and `build` commands.
  - [6ec8e84d](https://www.github.com/tauri-apps/tauri/commit/6ec8e84d9172c090ee1549db56c98c66f12436ff) feat(cli.rs): add `features` arg to dev/build ([#1828](https://www.github.com/tauri-apps/tauri/pull/1828)) on 2021-05-13
- Fixes the libwebkit2gtk package name.
  - [e08065d7](https://www.github.com/tauri-apps/tauri/commit/e08065d7fe8398b41180b3a64854ec8e71174d42) fix: deb installation error ([#1844](https://www.github.com/tauri-apps/tauri/pull/1844)) on 2021-05-18
- Properly keep all `tauri` features that are not managed by the CLI.
  - [17c7c439](https://www.github.com/tauri-apps/tauri/commit/17c7c4396ff2d5e13fc8726c2965b4e810fad6b9) refactor(core): use `attohttpc` by default ([#1861](https://www.github.com/tauri-apps/tauri/pull/1861)) on 2021-05-19
- Copy resources and binaries to `OUT_DIR` on `tauri dev` command.
  - [8f29a260](https://www.github.com/tauri-apps/tauri/commit/8f29a260e67aa111f6aeb262bd846a46d2858ce9) fix(cli.rs): copy resources and binaries on dev, closes [#1298](https://www.github.com/tauri-apps/tauri/pull/1298) ([#1946](https://www.github.com/tauri-apps/tauri/pull/1946)) on 2021-06-04
- Read cargo features from `tauri.conf.json > build > features` and propagate them on `dev` and `build`.
  - [2b814e9c](https://www.github.com/tauri-apps/tauri/commit/2b814e9c937489af0acb56051bd01c0d7fca2413) added cargo features to tauri config ([#1824](https://www.github.com/tauri-apps/tauri/pull/1824)) on 2021-05-13
- Fixes `tauri.conf.json > tauri > bundle > targets` not applying to the bundler.
  - [8be35ced](https://www.github.com/tauri-apps/tauri/commit/8be35ced78658de732360e3b20d7d70108c9b32d) fix(cli.rs): `tauri.conf.json > tauri > bundle > targets` being ignored ([#1945](https://www.github.com/tauri-apps/tauri/pull/1945)) on 2021-06-04
- Fixes `info` command not striping `\r` from child process version output.
  - [6a95d7ac](https://www.github.com/tauri-apps/tauri/commit/6a95d7acc378b40230bab18d00ea32de40a5818c) fix(cli.rs): `info` version checks not striping `\r` on Windows ([#1952](https://www.github.com/tauri-apps/tauri/pull/1952)) on 2021-06-05
- Allow setting a path to a license file for the Windows Installer (`tauri.conf.json > bundle > windows > wix > license`).
  - [b769c7f7](https://www.github.com/tauri-apps/tauri/commit/b769c7f7da4064b6133bf39a82127863d0d35531) feat(bundler): windows installer license, closes [#2009](https://www.github.com/tauri-apps/tauri/pull/2009) ([#2027](https://www.github.com/tauri-apps/tauri/pull/2027)) on 2021-06-21
- Change the `csp` value on the template to include `wss:` and `tauri:` to the `default-src` attribute.
  - [463fd00d](https://www.github.com/tauri-apps/tauri/commit/463fd00d06241c734994fe8e1882788dc30cc993) fix(csp): add wss and tauri to conf template ([#1974](https://www.github.com/tauri-apps/tauri/pull/1974)) on 2021-06-15
- Adds `tauri > bundle > windows > wix > language` config option. See https://docs.microsoft.com/en-us/windows/win32/msi/localizing-the-error-and-actiontext-tables.
  - [47919619](https://www.github.com/tauri-apps/tauri/commit/47919619815900fc3af47ec5873e31afb778b0ad) feat(bundler): allow setting wix language, closes [#1976](https://www.github.com/tauri-apps/tauri/pull/1976) ([#1988](https://www.github.com/tauri-apps/tauri/pull/1988)) on 2021-06-15

## \[1.0.0-beta.1]

- Add `'self'` to default CSP because otherwise no joy on macOS.
  - [12268e6](https://www.github.com/tauri-apps/tauri/commit/12268e6e69dc9a7034652f50316d3545cac687c7) fix(csp): add 'self' ([#1794](https://www.github.com/tauri-apps/tauri/pull/1794)) on 2021-05-12
- Fix a typo that would result in bundle arg being ignored.
  - [71f6a5e](https://www.github.com/tauri-apps/tauri/commit/71f6a5ed442a43bf1008043c95a1a90effdd2f81) fix(cli.rs/build): fix typo getting bundle arg ([#1783](https://www.github.com/tauri-apps/tauri/pull/1783)) on 2021-05-12

## \[1.0.0-beta.0]

- Fixes a cargo `target/` cache issue.
  - [79feb6a](https://www.github.com/tauri-apps/tauri/commit/79feb6a918c2b40af771b5dccc94c8f6f4176986) fix(cli.rs): cargo build failed due to cache issue, closes [#1543](https://www.github.com/tauri-apps/tauri/pull/1543) ([#1741](https://www.github.com/tauri-apps/tauri/pull/1741)) on 2021-05-07
- Improve error logging.
  - [5cc4b11](https://www.github.com/tauri-apps/tauri/commit/5cc4b11f5d00a1e7e580e31785b31c491c06d8d7) feat(cli.rs): add context to errors ([#1674](https://www.github.com/tauri-apps/tauri/pull/1674)) on 2021-05-01
- Adds Webview2 version on `info` command.
  - [2b4e2b7](https://www.github.com/tauri-apps/tauri/commit/2b4e2b7560515b76002d0c724bcca1f470ed106f) feat(cli.rs/info): get webview2 version on windows ([#1669](https://www.github.com/tauri-apps/tauri/pull/1669)) on 2021-05-04
- Adds `--runner [PROGRAM]` argument on the `dev` and `build` command, allowing using the specified program to run and build the application (example program: `cross`).
  - [5c1fe52](https://www.github.com/tauri-apps/tauri/commit/5c1fe52c2bd74e2a8f6c99c2870af967e6309e8d) feat(cli.rs): allow using cross instead of cargo, add target triple arg ([#1664](https://www.github.com/tauri-apps/tauri/pull/1664)) on 2021-04-30
- Adds `--target [TARGET_TRIPLE]` option to the `build` command (example: `--target arm-unknown-linux-gnueabihf`).
  - [5c1fe52](https://www.github.com/tauri-apps/tauri/commit/5c1fe52c2bd74e2a8f6c99c2870af967e6309e8d) feat(cli.rs): allow using cross instead of cargo, add target triple arg ([#1664](https://www.github.com/tauri-apps/tauri/pull/1664)) on 2021-04-30
- Rename `--target` option on the `build` command to `--bundle`.
  - [5c1fe52](https://www.github.com/tauri-apps/tauri/commit/5c1fe52c2bd74e2a8f6c99c2870af967e6309e8d) feat(cli.rs): allow using cross instead of cargo, add target triple arg ([#1664](https://www.github.com/tauri-apps/tauri/pull/1664)) on 2021-04-30
- Automatically add Tauri dependencies to the debian package `Depends` section.
  - [72b8048](https://www.github.com/tauri-apps/tauri/commit/72b8048b5ada7a18d71b0fd8a4a0177109b43db7) feat(cli.rs): fill debian `depends` with tauri dependencies ([#1767](https://www.github.com/tauri-apps/tauri/pull/1767)) on 2021-05-10
- Properly kill `beforeDevCommand` process.
  - [ac2cbcb](https://www.github.com/tauri-apps/tauri/commit/ac2cbcb131819e01074e1ed8fb6808260c56a027) fix(cli.rs): `before dev` process kill, closes [#1626](https://www.github.com/tauri-apps/tauri/pull/1626) ([#1700](https://www.github.com/tauri-apps/tauri/pull/1700)) on 2021-05-04
- Adds support to `tauri` dependency as string and table on `Cargo.toml`.
  - [df8bdcf](https://www.github.com/tauri-apps/tauri/commit/df8bdcf0631fd4e1e7035eb20a954574da96de66) feat(cli.rs): add support to string and table dependency, closes [#1653](https://www.github.com/tauri-apps/tauri/pull/1653) ([#1654](https://www.github.com/tauri-apps/tauri/pull/1654)) on 2021-04-29
- Show `framework` and `bundler` on the `info` command by reading the `package.json` file and matching known dependencies.
  - [152c755](https://www.github.com/tauri-apps/tauri/commit/152c755c4787b323ca3469c45934cc1e4d368cfa) feat(cli.rs): `framework` and `bundler` on info cmd, closes [#1681](https://www.github.com/tauri-apps/tauri/pull/1681) ([#1682](https://www.github.com/tauri-apps/tauri/pull/1682)) on 2021-05-02

## \[1.0.0-beta-rc.4]

- Fixes the Message `command` name value on plugin invoke handler.
  - Bumped due to a bump in tauri.
  - [422dd5e](https://www.github.com/tauri-apps/tauri/commit/422dd5e2a0a03bb1556915c78f110bfab092c874) fix(core): command name on plugin invoke handler ([#1577](https://www.github.com/tauri-apps/tauri/pull/1577)) on 2021-04-21
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25
- The package info APIs now checks the `package` object on `tauri.conf.json`.
  - Bumped due to a bump in tauri.
  - [8fd1baf](https://www.github.com/tauri-apps/tauri/commit/8fd1baf69b14bb81d7be9d31605ed7f02058b392) fix(core): pull package info from tauri.conf.json if set ([#1581](https://www.github.com/tauri-apps/tauri/pull/1581)) on 2021-04-22
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25

## \[1.0.0-beta-rc.3]

- Check if distDir assets are built after running `beforeDevCommand`.
  - [a670d3a](https://www.github.com/tauri-apps/tauri/commit/a670d3a457bc0c0135b879c746d26a5f121c87a7) fix(cli.rs): check if distDir exists after running `beforeDevCommand` ([#1586](https://www.github.com/tauri-apps/tauri/pull/1586)) on 2021-04-22
- Fixes `tauri info` display version for the `@tauri-apps/api` package.
  - [0012782](https://www.github.com/tauri-apps/tauri/commit/0012782e43bd4e7e49528853c226b8e0e24b8794) fix(cli.rs): `info` command `npm_package_version` parsing `beta-rc` ([#1587](https://www.github.com/tauri-apps/tauri/pull/1587)) on 2021-04-22
- Fixes crash on usage of modifier keys on Windows when running `tauri init`.
  - [d623d95](https://www.github.com/tauri-apps/tauri/commit/d623d95fcb67736bc0862866b347c7102cde66aa) fix(cli.rs): inliner dialoguer & console until they publish, fixes [#1492](https://www.github.com/tauri-apps/tauri/pull/1492) ([#1610](https://www.github.com/tauri-apps/tauri/pull/1610)) on 2021-04-25
- Enable `tauri` `updater` feature when `tauri.conf.json > tauri > updater > active` is set to `true`.
  - [9490b25](https://www.github.com/tauri-apps/tauri/commit/9490b257d2564840eb0c9167340bf444bca84699) fix(cli.rs): enable the `updater` feature on cli ([#1597](https://www.github.com/tauri-apps/tauri/pull/1597)) on 2021-04-23

## \[1.0.0-beta-rc.2]

- Add missing camelcase rename for config
  - [bdf7072](https://www.github.com/tauri-apps/tauri/commit/bdf707285e3d307ab083009c274ccb56d5053ff2) fix(cli.rs/info): add missing camelCase rename ([#1505](https://www.github.com/tauri-apps/tauri/pull/1505)) on 2021-04-14
- Fix `tauri info`
- Properly detect `yarn` and `npm` versions on windows.
- Fix a panic caused by a wrong field name in `metadata.json`
- [71666e9](https://www.github.com/tauri-apps/tauri/commit/71666e9f9cfb5499a727b3f95182e89073f67d7b) fix(cli.rs): fix panic & use `cmd` to run `yarn`&`npm` on windows ([#1511](https://www.github.com/tauri-apps/tauri/pull/1511)) on 2021-04-17
- Sync `metadata.json` via script to update version reference to cli.js, tauri (core) and tauri-build.
  - [1f64927](https://www.github.com/tauri-apps/tauri/commit/1f64927362ef20761d7cd3591281519eb292aa33) chore: sync cli.rs metadata.json file versions ([#1534](https://www.github.com/tauri-apps/tauri/pull/1534)) on 2021-04-19

## \[1.0.0-beta-rc.1]

- Missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
  - Bumped due to a bump in api.
  - [b2569a7](https://www.github.com/tauri-apps/tauri/commit/b2569a729a3caa88bdba62abc31f0665e1323aaa) fix(js-api): dist ([#1498](https://www.github.com/tauri-apps/tauri/pull/1498)) on 2021-04-15

## \[1.0.0-beta-rc.0]

- You can now run `cargo tauri build -t none` to speed up the build if you don't need executables.
  - [4d507f9](https://www.github.com/tauri-apps/tauri/commit/4d507f9adfb26819f9d6406b191fdaa6188145f4) feat(cli/core): add support for building without targets ([#1203](https://www.github.com/tauri-apps/tauri/pull/1203)) on 2021-02-10
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The `dev` and `build` pipeline is now written in Rust.
  - [3e8abe3](https://www.github.com/tauri-apps/tauri/commit/3e8abe376407bb0ca8893602590ed9edf7aa71a1) feat(cli) rewrite the core CLI in Rust ([#851](https://www.github.com/tauri-apps/tauri/pull/851)) on 2021-01-30
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Run `beforeDevCommand` and `beforeBuildCommand` in a shell.
  - [32eb0d5](https://www.github.com/tauri-apps/tauri/commit/32eb0d562b135d8df19c78ff22aa53c73f459c76) feat(cli): run beforeDev and beforeBuild in a shell, closes [#1295](https://www.github.com/tauri-apps/tauri/pull/1295) ([#1399](https://www.github.com/tauri-apps/tauri/pull/1399)) on 2021-03-28
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Fixes `<a target="_blank">` polyfill.
  - [4ee044a](https://www.github.com/tauri-apps/tauri/commit/4ee044a3e662a0ac2be98f7e1286088d721c3307) fix(cli): use correct arg in `_blanks` links polyfill ([#1362](https://www.github.com/tauri-apps/tauri/pull/1362)) on 2021-03-17
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Adds `productName` and `version` configs on `tauri.conf.json > package`.
  - [5b3d9b2](https://www.github.com/tauri-apps/tauri/commit/5b3d9b2c07da766f81981ba7c4961cd354d51340) feat(config): allow setting product name and version on tauri.conf.json ([#1358](https://www.github.com/tauri-apps/tauri/pull/1358)) on 2021-03-22
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The `info` command was rewritten in Rust.
  - [c3e06ee](https://www.github.com/tauri-apps/tauri/commit/c3e06ee9e88b3631da6eeb17d61ddd41cd5c6fe9) refactor(cli): rewrite info in Rust ([#1389](https://www.github.com/tauri-apps/tauri/pull/1389)) on 2021-03-25
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The `init` command was rewritten in Rust.
  - [f72b93b](https://www.github.com/tauri-apps/tauri/commit/f72b93b676ba8c48fd9273c187de3dbbc410fa0f) refactor(cli): rewrite init command in Rust ([#1382](https://www.github.com/tauri-apps/tauri/pull/1382)) on 2021-03-24
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- All the arguments passed after `tauri dev --` are now propagated to the binary.
  - [4e9d31c](https://www.github.com/tauri-apps/tauri/commit/4e9d31c70ba13f1cabe830c6519a1b5f4789fd7b) feat(cli): propagate args passed after `dev --`, closes [#1406](https://www.github.com/tauri-apps/tauri/pull/1406) ([#1407](https://www.github.com/tauri-apps/tauri/pull/1407)) on 2021-03-30
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Alpha version of tauri-updater. Please refer to the `README` for more details.
  - [6d70c8e](https://www.github.com/tauri-apps/tauri/commit/6d70c8e1e256fe839c4a947375bb529d7b4f7301) feat(updater): Alpha version ([#643](https://www.github.com/tauri-apps/tauri/pull/643)) on 2021-04-05
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
